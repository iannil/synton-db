// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Local embedding backend using Candle.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::backend::EmbeddingBackend;
use crate::config::LocalModelConfig;
use crate::error::{MlError, Result};

#[cfg(feature = "candle")]
use candle::{Device as CandleDevice, DType as CandleDType, Tensor as CandleTensor};

/// Local embedding backend using Candle.
///
/// This backend requires the `candle` feature to be enabled.
#[derive(Clone)]
pub struct LocalEmbeddingBackend {
    inner: Arc<LocalEmbeddingBackendInner>,
}

/// Inner state of the local embedding backend.
struct LocalEmbeddingBackendInner {
    config: LocalModelConfig,
    cache: Arc<RwLock<lru::LruCache<String, Vec<f32>>>>,
    dimension: usize,
    #[cfg(feature = "candle")]
    model: Arc<RwLock<Option<LoadedModel>>>,
}

/// Loaded model with runtime components.
#[cfg(feature = "candle")]
struct LoadedModel {
    model: candle_transformers::models::bert::BertModel,
    tokenizer: tokenizers::Tokenizer,
    device: CandleDevice,
}

impl LocalEmbeddingBackend {
    /// Create a new local embedding backend.
    ///
    /// # Errors
    ///
    /// Returns an error if the model cannot be loaded.
    pub async fn new(config: LocalModelConfig) -> Result<Self> {
        // Validate configuration
        if config.model_name.is_empty() {
            return Err(MlError::invalid_config("Model name cannot be empty"));
        }

        // Determine expected dimension based on model
        let dimension = Self::get_model_dimension(&config.model_name);

        let cache_size = 1000; // Default cache size
        let cache = Arc::new(RwLock::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(cache_size).unwrap(),
        )));

        let inner = Arc::new(LocalEmbeddingBackendInner {
            config,
            cache,
            dimension,
            #[cfg(feature = "candle")]
            model: Arc::new(RwLock::new(None)),
        });

        Ok(Self { inner })
    }

    /// Ensure the model is loaded (lazy loading).
    #[cfg(feature = "candle")]
    async fn ensure_model_loaded(&self) -> Result<()> {
        let mut model_guard = self.inner.model.write().await;
        if model_guard.is_some() {
            return Ok(());
        }

        // Load the model
        let loader = crate::loader::ModelLoader::new();
        let loaded = loader.load_embedding_model(&self.inner.config.model_name).await?;

        *model_guard = Some(LoadedModel {
            model: match loaded.model {
                crate::loader::ModelWrapper::Bert(m) => m,
            },
            tokenizer: loaded.tokenizer,
            device: loaded.device,
        });

        Ok(())
    }

    /// Get the expected embedding dimension for a model.
    fn get_model_dimension(model_name: &str) -> usize {
        match model_name {
            "sentence-transformers/all-MiniLM-L6-v2" => 384,
            "sentence-transformers/all-mpnet-base-v2" => 768,
            "BAAI/bge-small-en-v1.5" => 384,
            "BAAI/bge-base-en-v1.5" => 768,
            "BAAI/bge-large-en-v1.5" => 1024,
            "sentence-t5-base" => 768,
            "sentence-t5-large" => 768,
            "sentence-t5-xl" => 768,
            "e5-base" | "intfloat/e5-base" => 768,
            "e5-large" | "intfloat/e5-large" => 1024,
            "e5-small" | "intfloat/e5-small" => 384,
            _ => 384, // Default fallback
        }
    }

    /// Check if the Candle feature is enabled.
    pub fn is_available() -> bool {
        cfg!(feature = "candle")
    }
}

#[async_trait::async_trait]
impl EmbeddingBackend for LocalEmbeddingBackend {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let text = text.trim();

        if text.is_empty() {
            return Err(MlError::EmptyInput);
        }

        // Check cache
        {
            let cache = self.inner.cache.read().await;
            let text_key = text.to_string();
            if let Some(cached) = cache.get(&text_key) {
                return Ok(cached.clone());
            }
        }

        // Generate embedding
        #[cfg(feature = "candle")]
        {
            let embedding = self.embed_with_candle(text).await?;

            // Update cache
            {
                let mut cache = self.inner.cache.write().await;
                cache.put(text.to_string(), embedding.clone());
            }

            Ok(embedding)
        }

        #[cfg(not(feature = "candle"))]
        {
            let _ = text;
            Err(MlError::ModelLoadFailed(
                "Candle feature is not enabled. Please enable the 'candle' feature to use local models.".to_string(),
            ))
        }
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        #[cfg(feature = "candle")]
        {
            let mut results = Vec::with_capacity(texts.len());
            let mut uncached_texts = Vec::new();
            let mut uncached_indices = Vec::new();

            // Check cache
            {
                let cache = self.inner.cache.read().await;
                for (i, text) in texts.iter().enumerate() {
                    let text = text.trim();
                    if text.is_empty() {
                        return Err(MlError::EmptyInput);
                    }

                    if let Some(cached) = cache.get(&text.to_string()) {
                        results.push(Some(cached.clone()));
                    } else {
                        results.push(None);
                        uncached_texts.push(text.to_string());
                        uncached_indices.push(i);
                    }
                }
            }

            // Generate embeddings for uncached texts
            if !uncached_texts.is_empty() {
                let new_embeddings = self.embed_batch_with_candle(&uncached_texts).await?;

                // Update cache and fill results
                {
                    let mut cache = self.inner.cache.write().await;
                    for (text, embedding) in uncached_texts.iter().zip(new_embeddings.iter()) {
                        cache.put(text.clone(), embedding.clone());
                    }
                }

                for (idx, embedding) in uncached_indices.iter().zip(new_embeddings.into_iter()) {
                    results[*idx] = Some(embedding);
                }
            }

            // Extract results
            let final_results: Result<Vec<_>> = results.into_iter().map(|r| r.ok_or(MlError::EmptyInput)).collect();
            final_results
        }

        #[cfg(not(feature = "candle"))]
        {
            let _ = texts;
            Err(MlError::ModelLoadFailed(
                "Candle feature is not enabled. Please enable the 'candle' feature to use local models.".to_string(),
            ))
        }
    }

    fn dimension(&self) -> usize {
        self.inner.dimension
    }

    fn backend_type(&self) -> crate::backend::BackendType {
        crate::backend::BackendType::Local
    }
}

#[cfg(feature = "candle")]
impl LocalEmbeddingBackend {
    async fn embed_with_candle(&self, text: &str) -> Result<Vec<f32>> {
        // Ensure model is loaded
        self.ensure_model_loaded().await?;

        let model_guard = self.inner.model.read().await;
        let model = model_guard.as_ref().unwrap();

        // Tokenize input
        let tokens = model
            .tokenizer
            .encode(text, true)
            .map_err(|e| MlError::EmbeddingFailed(format!("Tokenization failed: {}", e)))?;

        let ids = tokens.get_ids();

        if ids.is_empty() {
            return Err(MlError::EmptyInput);
        }

        // Create input tensor
        let input_ids = CandleTensor::from_vec(ids.to_vec(), (1, ids.len()), &model.device)
            .map_err(|e| MlError::EmbeddingFailed(format!("Failed to create input tensor: {}", e)))?;

        // Create attention mask
        let attention_mask = CandleTensor::ones(
            (1, ids.len()),
            CandleDType::U8,
            &model.device,
        )
        .map_err(|e| MlError::EmbeddingFailed(format!("Failed to create attention mask: {}", e)))?;

        // Run the model - Candle 0.9.2 forward takes optional token_type_ids as 3rd arg
        let embeddings = model
            .model
            .forward(&input_ids, &attention_mask, None)
            .map_err(|e| MlError::EmbeddingFailed(format!("Model forward pass failed: {}", e)))?;

        // Mean pooling
        let pooled = self.mean_pool(&embeddings, &attention_mask)
            .map_err(|e| MlError::EmbeddingFailed(format!("Mean pooling failed: {}", e)))?;

        // Convert to Vec<f32>
        let result = pooled
            .to_vec1()
            .map_err(|e| MlError::EmbeddingFailed(format!("Failed to convert embedding: {}", e)))?;

        Ok(result)
    }

    async fn embed_batch_with_candle(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // For simplicity, process individually for now
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed_with_candle(text).await?);
        }
        Ok(results)
    }

    /// Mean pooling with attention mask.
    fn mean_pool(
        &self,
        embeddings: &CandleTensor,
        attention_mask: &CandleTensor,
    ) -> Result<CandleTensor> {
        use candle::DType;

        // Expand attention mask to match embedding dimensions
        let mask = attention_mask
            .to_dtype(DType::F32)
            .map_err(|e| MlError::EmbeddingFailed(format!("Failed to convert dtype: {}", e)))?
            .unsqueeze(2)
            .map_err(|e| MlError::EmbeddingFailed(format!("Failed to unsqueeze: {}", e)))?;

        // Multiply embeddings by mask and sum
        let product = (embeddings * &mask)
            .map_err(|e| MlError::EmbeddingFailed(format!("Failed to multiply: {}", e)))?;
        let sum = product
            .sum(1)
            .map_err(|e| MlError::EmbeddingFailed(format!("Failed to sum: {}", e)))?;

        // Count non-padding tokens
        let count = mask
            .sum(1)
            .map_err(|e| MlError::EmbeddingFailed(format!("Failed to sum mask: {}", e)))?
            .clamp(1.0, f32::MAX)
            .map_err(|e| MlError::EmbeddingFailed(format!("Failed to clamp: {}", e)))?;

        // Mean pooling
        let count_unsqueezed = count
            .unsqueeze(1)
            .map_err(|e| MlError::EmbeddingFailed(format!("Failed to unsqueeze count: {}", e)))?;
        let pooled = (sum / count_unsqueezed)
            .map_err(|e| MlError::EmbeddingFailed(format!("Failed to divide: {}", e)))?;
        Ok(pooled)
    }
}

// Stub lru module for when lru crate is not available
mod lru {
    use std::collections::HashMap;

    pub struct LruCache<K, V> {
        capacity: usize,
        map: HashMap<K, V>,
        keys: Vec<K>,
    }

    impl<K, V> LruCache<K, V>
    where
        K: std::hash::Hash + Eq + Clone,
    {
        pub fn new(capacity: std::num::NonZeroUsize) -> Self {
            Self {
                capacity: capacity.get(),
                map: HashMap::new(),
                keys: Vec::new(),
            }
        }

        pub fn get(&self, key: &K) -> Option<&V> {
            self.map.get(key)
        }

        pub fn put(&mut self, key: K, value: V) {
            if self.map.len() >= self.capacity && !self.map.contains_key(&key) {
                if let Some(old_key) = self.keys.first() {
                    self.map.remove(old_key);
                    self.keys.remove(0);
                }
            }
            self.map.insert(key.clone(), value);
            if !self.keys.contains(&key) {
                self.keys.push(key);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_model_dimension() {
        assert_eq!(
            LocalEmbeddingBackend::get_model_dimension("sentence-transformers/all-MiniLM-L6-v2"),
            384
        );
        assert_eq!(
            LocalEmbeddingBackend::get_model_dimension("BAAI/bge-base-en-v1.5"),
            768
        );
        assert_eq!(LocalEmbeddingBackend::get_model_dimension("unknown"), 384);
    }

    #[tokio::test]
    async fn test_local_backend_creation() {
        let config = LocalModelConfig {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            ..Default::default()
        };

        let backend = LocalEmbeddingBackend::new(config).await;
        assert!(backend.is_ok());
        let backend = backend.unwrap();
        assert_eq!(backend.dimension(), 384);
        assert_eq!(backend.backend_type(), crate::backend::BackendType::Local);
    }

    #[tokio::test]
    async fn test_local_backend_empty_config() {
        let config = LocalModelConfig {
            model_name: String::new(),
            ..Default::default()
        };

        let backend = LocalEmbeddingBackend::new(config).await;
        assert!(backend.is_err());
    }

    #[tokio::test]
    async fn test_embed_empty_input() {
        let config = LocalModelConfig {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            ..Default::default()
        };

        let backend = LocalEmbeddingBackend::new(config).await.unwrap();
        let result = backend.embed("").await;
        assert!(matches!(result, Err(MlError::EmptyInput)));

        let result = backend.embed("   ").await;
        assert!(matches!(result, Err(MlError::EmptyInput)));
    }

    #[tokio::test]
    #[cfg(feature = "candle")]
    async fn test_embed_cache() {
        let config = LocalModelConfig {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            ..Default::default()
        };

        let backend = LocalEmbeddingBackend::new(config).await.unwrap();

        // First call - should hit the backend
        let result1 = backend.embed("test text").await;
        assert!(result1.is_ok());

        // Second call - should hit cache
        let result2 = backend.embed("test text").await;
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());
    }
}
