// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Unified embedding service.

use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::RwLock;

use crate::backend::{BackendType, EmbeddingBackend};
use crate::config::EmbeddingConfig;
use crate::error::{MlError, Result};

/// Simple LRU cache implementation.
#[derive(Clone)]
struct LruCache<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    capacity: usize,
    map: HashMap<K, V>,
    keys: Vec<K>,
}

impl<K, V> LruCache<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    fn new(capacity: NonZeroUsize) -> Self {
        Self {
            capacity: capacity.get(),
            map: HashMap::new(),
            keys: Vec::new(),
        }
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    fn put(&mut self, key: K, value: V) {
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

    fn len(&self) -> usize {
        self.map.len()
    }

    fn clear(&mut self) {
        let cap = self.capacity;
        *self = Self::new(NonZeroUsize::new(cap).unwrap());
    }
}

mod backend_impl {
    use super::*;

    /// Enum representing all available backend implementations.
    #[derive(Clone)]
    pub enum AnyBackend {
        Local(super::super::local::LocalEmbeddingBackend),
        OpenAi(super::super::openai::OpenAiEmbeddingBackend),
        Ollama(super::super::ollama::OllamaEmbeddingBackend),
    }

    impl AnyBackend {
        pub fn backend_type(&self) -> BackendType {
            match self {
                Self::Local(_) => BackendType::Local,
                Self::OpenAi(_) => BackendType::OpenAi,
                Self::Ollama(_) => BackendType::Ollama,
            }
        }
    }

    #[async_trait::async_trait]
    impl EmbeddingBackend for AnyBackend {
        async fn embed(&self, text: &str) -> Result<Vec<f32>> {
            match self {
                Self::Local(b) => b.embed(text).await,
                Self::OpenAi(b) => b.embed(text).await,
                Self::Ollama(b) => b.embed(text).await,
            }
        }

        async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
            match self {
                Self::Local(b) => b.embed_batch(texts).await,
                Self::OpenAi(b) => b.embed_batch(texts).await,
                Self::Ollama(b) => b.embed_batch(texts).await,
            }
        }

        fn dimension(&self) -> usize {
            match self {
                Self::Local(b) => b.dimension(),
                Self::OpenAi(b) => b.dimension(),
                Self::Ollama(b) => b.dimension(),
            }
        }

        fn backend_type(&self) -> BackendType {
            self.backend_type()
        }
    }
}

use backend_impl::AnyBackend;

/// Statistics for the embedding service.
#[derive(Debug, Clone, Default)]
pub struct EmbeddingStats {
    /// Total number of embeddings generated.
    pub total_embeddings: usize,

    /// Total number of batch requests.
    pub total_batch_requests: usize,

    /// Total tokens processed (approximate).
    pub total_tokens: usize,

    /// Average embedding generation time (in milliseconds).
    pub avg_time_ms: f64,

    /// Number of cache hits.
    pub cache_hits: usize,

    /// Cache hit rate (0.0-1.0).
    pub cache_hit_rate: f64,
}

/// Unified embedding service.
///
/// Provides a single interface for generating embeddings using different backends.
#[derive(Clone)]
pub struct EmbeddingService {
    backend: Arc<AnyBackend>,
    config: EmbeddingConfig,
    cache: Arc<RwLock<LruCache<String, Vec<f32>>>>,
    stats: Arc<RwLock<EmbeddingStats>>,
}

impl EmbeddingService {
    /// Create a new embedding service from configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The configuration is invalid
    /// - The backend cannot be initialized
    pub async fn from_config(config: EmbeddingConfig) -> Result<Self> {
        config.validate()?;

        let backend = match config.backend {
            BackendType::Local => {
                let local_backend =
                    crate::local::LocalEmbeddingBackend::new(config.local.clone()).await?;
                AnyBackend::Local(local_backend)
            }
            BackendType::OpenAi => {
                let openai_backend = crate::openai::OpenAiEmbeddingBackend::new(config.api.clone())?;
                AnyBackend::OpenAi(openai_backend)
            }
            BackendType::Ollama => {
                let ollama_backend = crate::ollama::OllamaEmbeddingBackend::new(config.api.clone())?;
                AnyBackend::Ollama(ollama_backend)
            }
        };

        let cache_size = if config.cache_enabled {
            config.cache_size
        } else {
            1 // Minimum cache size
        };

        let cache = Arc::new(RwLock::new(LruCache::new(
            NonZeroUsize::new(cache_size).unwrap(),
        )));

        let stats = Arc::new(RwLock::new(EmbeddingStats::default()));

        Ok(Self {
            backend: Arc::new(backend),
            config,
            cache,
            stats,
        })
    }

    /// Generate an embedding for a single text.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The input is empty
    /// - The backend fails to generate the embedding
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let text = text.trim();

        if text.is_empty() {
            return Err(MlError::EmptyInput);
        }

        // Check cache
        if self.config.cache_enabled {
            {
                let cache = self.cache.read().await;
                let text_key = &text.to_string();
                if let Some(cached) = cache.get(text_key) {
                    // Update stats
                    let mut stats = self.stats.write().await;
                    stats.total_embeddings += 1;
                    stats.cache_hits += 1;
                    stats.cache_hit_rate =
                        stats.cache_hits as f64 / stats.total_embeddings as f64;

                    return Ok(cached.clone());
                }
            }
        }

        // Generate embedding
        let start = Instant::now();
        let embedding = self.backend.embed(text).await?;
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        // Update cache
        if self.config.cache_enabled {
            let mut cache = self.cache.write().await;
            cache.put(text.to_string(), embedding.clone());
        }

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_embeddings += 1;
        stats.total_tokens += text.split_whitespace().count();
        let n = stats.total_embeddings as f64;
        stats.avg_time_ms = (stats.avg_time_ms * (n - 1.0) + elapsed) / n;

        Ok(embedding)
    }

    /// Generate embeddings for multiple texts in batch.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any input is empty
    /// - The backend fails to generate embeddings
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Check for empty inputs
        for text in texts {
            if text.trim().is_empty() {
                return Err(MlError::EmptyInput);
            }
        }

        let mut results = Vec::with_capacity(texts.len());
        let mut uncached_texts = Vec::new();
        let mut uncached_indices = Vec::new();

        // Check cache
        if self.config.cache_enabled {
            {
                let cache = self.cache.read().await;
                for (i, text) in texts.iter().enumerate() {
                    let text = text.trim();
                    let text_key = &text.to_string();
                    if let Some(cached) = cache.get(text_key) {
                        results.push(Some(cached.clone()));

                        // Update stats
                        let mut stats = self.stats.write().await;
                        stats.total_embeddings += 1;
                        stats.cache_hits += 1;
                    } else {
                        results.push(None);
                        uncached_texts.push(text.to_string());
                        uncached_indices.push(i);
                    }
                }
            }
        } else {
            uncached_texts = texts.iter().map(|t| t.trim().to_string()).collect();
            uncached_indices = (0..texts.len()).collect();
        }

        // Generate embeddings for uncached texts
        if !uncached_texts.is_empty() {
            let start = Instant::now();
            let new_embeddings = self.backend.embed_batch(&uncached_texts).await?;
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;

            // Update cache and fill results
            if self.config.cache_enabled {
                let mut cache = self.cache.write().await;
                for (text, embedding) in uncached_texts.iter().zip(new_embeddings.iter()) {
                    cache.put(text.clone(), embedding.clone());
                }
            }

            for (idx, embedding) in uncached_indices.into_iter().zip(new_embeddings.into_iter()) {
                results[idx] = Some(embedding);
            }

            // Update stats
            let mut stats = self.stats.write().await;
            stats.total_batch_requests += 1;
            stats.total_embeddings += uncached_texts.len();
            stats.total_tokens += uncached_texts.iter().map(|t| t.split_whitespace().count()).sum::<usize>();
            let n = stats.total_embeddings as f64;
            stats.avg_time_ms =
                (stats.avg_time_ms * (n - uncached_texts.len() as f64) + elapsed) / n;
            stats.cache_hit_rate = stats.cache_hits as f64 / stats.total_embeddings as f64;
        }

        // Extract results
        let final_results: Result<Vec<_>> =
            results.into_iter().map(|r| r.ok_or(MlError::EmptyInput)).collect();
        final_results
    }

    /// Get the dimension of the embeddings.
    pub fn dimension(&self) -> usize {
        self.backend.dimension()
    }

    /// Get the backend type being used.
    pub fn backend_type(&self) -> BackendType {
        self.backend.backend_type()
    }

    /// Get the configuration.
    pub fn config(&self) -> &EmbeddingConfig {
        &self.config
    }

    /// Get the current statistics.
    pub async fn stats(&self) -> EmbeddingStats {
        self.stats.read().await.clone()
    }

    /// Clear the embedding cache.
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get the cache size.
    pub async fn cache_size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }

    /// Check if the service is healthy.
    pub async fn health_check(&self) -> Result<()> {
        // Try to generate a test embedding
        let test_text = "health check";
        let _ = self.embed(test_text).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_creation_local() {
        let config = EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string());
        let service = EmbeddingService::from_config(config).await;
        assert!(service.is_ok());

        let service = service.unwrap();
        assert_eq!(service.backend_type(), BackendType::Local);
        assert_eq!(service.dimension(), 384);
    }

    #[tokio::test]
    async fn test_service_creation_openai() {
        let config = EmbeddingConfig::openai("sk-test".to_string());
        let service = EmbeddingService::from_config(config).await;
        assert!(service.is_ok());

        let service = service.unwrap();
        assert_eq!(service.backend_type(), BackendType::OpenAi);
        assert_eq!(service.dimension(), 1536);
    }

    #[tokio::test]
    async fn test_service_creation_ollama() {
        let config = EmbeddingConfig::ollama();
        let service = EmbeddingService::from_config(config).await;
        assert!(service.is_ok());

        let service = service.unwrap();
        assert_eq!(service.backend_type(), BackendType::Ollama);
        assert_eq!(service.dimension(), 768);
    }

    #[tokio::test]
    async fn test_embed_empty_input() {
        let config = EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string());
        let service = EmbeddingService::from_config(config).await.unwrap();

        let result = service.embed("").await;
        assert!(matches!(result, Err(MlError::EmptyInput)));

        let result = service.embed("   ").await;
        assert!(matches!(result, Err(MlError::EmptyInput)));
    }

    #[tokio::test]
    async fn test_embed_batch_empty() {
        let config = EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string());
        let service = EmbeddingService::from_config(config).await.unwrap();

        let result = service.embed_batch(&[]).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    #[cfg(feature = "candle")]
    async fn test_embed_cache() {
        let config = EmbeddingConfig {
            cache_enabled: true,
            cache_size: 10,
            ..EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string())
        };

        let service = EmbeddingService::from_config(config).await.unwrap();

        // First call
        let result1 = service.embed("test text").await;
        assert!(result1.is_ok());

        // Second call should hit cache
        let result2 = service.embed("test text").await;
        assert!(result2.is_ok());

        let stats = service.stats().await;
        assert_eq!(stats.total_embeddings, 2);
        assert_eq!(stats.cache_hits, 1);
    }

    #[tokio::test]
    #[cfg(feature = "candle")]
    async fn test_clear_cache() {
        let config = EmbeddingConfig {
            cache_enabled: true,
            cache_size: 10,
            ..EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string())
        };

        let service = EmbeddingService::from_config(config).await.unwrap();

        // Add to cache
        service.embed("test").await.unwrap();

        // Check cache size
        let size_before = service.cache_size().await;
        assert!(size_before > 0);

        // Clear cache
        service.clear_cache().await;

        // Check cache size
        let size_after = service.cache_size().await;
        assert_eq!(size_after, 0);
    }

    #[tokio::test]
    async fn test_embed_cache_api_backend() {
        // Test service creation with API backend (doesn't require candle)
        let config = EmbeddingConfig::openai("sk-test".to_string());

        let service = EmbeddingService::from_config(config).await.unwrap();

        // Verify service properties
        assert_eq!(service.backend_type(), BackendType::OpenAi);
        assert_eq!(service.dimension(), 1536);
    }
}
