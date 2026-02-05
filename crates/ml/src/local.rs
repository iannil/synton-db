// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Local embedding backend using Candle.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::backend::EmbeddingBackend;
use crate::config::LocalModelConfig;
use crate::error::{MlError, Result};

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
        });

        Ok(Self { inner })
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
            let text_key = &text.to_string();
            if let Some(cached) = cache.get(text_key) {
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

                    if let Some(cached) = cache.get(text) {
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
        // This is a placeholder implementation.
        // In a full implementation, this would:
        // 1. Load the Candle model and tokenizer
        // 2. Tokenize the input text
        // 3. Run the model forward pass
        // 4. Return mean-pooled embeddings

        // For now, return a mock embedding for testing the infrastructure
        tracing::warn!("Using mock embedding implementation. Candle integration pending.");
        Ok(vec![0.0f32; self.inner.dimension])
    }

    async fn embed_batch_with_candle(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // Placeholder implementation for batch embedding
        tracing::warn!("Using mock batch embedding implementation. Candle integration pending.");
        Ok(texts.iter().map(|_| vec![0.0f32; self.inner.dimension]).collect())
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
