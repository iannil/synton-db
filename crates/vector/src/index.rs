// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License)");

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{VectorError, VectorResult};
use synton_core::Filter;

/// Result of a vector search operation.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    /// Node ID
    pub id: Uuid,
    /// Similarity score (0.0 - 1.0, higher is better)
    pub score: f32,
    /// Additional metadata
    pub metadata: Vec<(String, String)>,
}

impl SearchResult {
    /// Create a new search result.
    pub fn new(id: Uuid, score: f32) -> Self {
        Self {
            id,
            score: score.clamp(0.0, 1.0),
            metadata: Vec::new(),
        }
    }

    /// Add metadata to the result.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }
}

/// Abstract vector index interface.
///
/// This trait defines the core vector operations for SYNTON-DB.
/// Implementations can use Lance, Faiss, or other vector stores.
#[async_trait]
pub trait VectorIndex: Send + Sync {
    /// Insert a single vector.
    async fn insert(&self, id: Uuid, vector: Vec<f32>) -> VectorResult<()>;

    /// Insert multiple vectors.
    async fn insert_batch(&self, vectors: Vec<(Uuid, Vec<f32>)>) -> VectorResult<()>;

    /// Search for k nearest neighbors.
    async fn search(&self, query: &[f32], k: usize) -> VectorResult<Vec<SearchResult>>;

    /// Hybrid search with metadata filters.
    async fn search_with_filter(
        &self,
        query: &[f32],
        _filter: Filter,
        k: usize,
    ) -> VectorResult<Vec<SearchResult>> {
        // Default implementation: ignores filter
        self.search(query, k).await
    }

    /// Delete a vector by ID.
    async fn remove(&self, id: Uuid) -> VectorResult<()>;

    /// Update a vector.
    async fn update(&self, id: Uuid, vector: Vec<f32>) -> VectorResult<()>;

    /// Get the total number of vectors.
    async fn count(&self) -> VectorResult<usize>;

    /// Get the embedding dimension.
    fn dimension(&self) -> usize;

    /// Check if the index is ready for queries.
    fn is_ready(&self) -> bool {
        true
    }
}

/// In-memory vector index for testing and simple use cases.
#[derive(Clone, Debug)]
pub struct MemoryVectorIndex {
    dimension: usize,
    vectors: Arc<RwLock<HashMap<Uuid, Vec<f32>>>>,
}

impl MemoryVectorIndex {
    /// Create a new in-memory vector index.
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            vectors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Calculate cosine similarity between two vectors.
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let mut dot_product = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for i in 0..a.len() {
            dot_product += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
        }

        let denom = norm_a.sqrt() * norm_b.sqrt();
        if denom > 0.0 {
            dot_product / denom
        } else {
            0.0
        }
    }
}

#[async_trait]
impl VectorIndex for MemoryVectorIndex {
    async fn insert(&self, id: Uuid, vector: Vec<f32>) -> VectorResult<()> {
        if vector.len() != self.dimension {
            return Err(VectorError::InvalidDimension {
                expected: self.dimension,
                found: vector.len(),
            });
        }
        let mut vectors = self.vectors.write().await;
        vectors.insert(id, vector);
        Ok(())
    }

    async fn insert_batch(&self, vectors: Vec<(Uuid, Vec<f32>)>) -> VectorResult<()> {
        let mut store = self.vectors.write().await;
        for (id, vector) in vectors {
            if vector.len() != self.dimension {
                return Err(VectorError::InvalidDimension {
                    expected: self.dimension,
                    found: vector.len(),
                });
            }
            store.insert(id, vector);
        }
        Ok(())
    }

    async fn search(&self, query: &[f32], k: usize) -> VectorResult<Vec<SearchResult>> {
        if query.len() != self.dimension {
            return Err(VectorError::InvalidDimension {
                expected: self.dimension,
                found: query.len(),
            });
        }

        let vectors = self.vectors.read().await;
        let mut results: Vec<SearchResult> = vectors
            .iter()
            .map(|(&id, vec)| {
                let score = Self::cosine_similarity(query, vec);
                SearchResult::new(id, score)
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(k);
        Ok(results)
    }

    async fn remove(&self, id: Uuid) -> VectorResult<()> {
        let mut vectors = self.vectors.write().await;
        vectors.remove(&id);
        Ok(())
    }

    async fn update(&self, id: Uuid, vector: Vec<f32>) -> VectorResult<()> {
        if vector.len() != self.dimension {
            return Err(VectorError::InvalidDimension {
                expected: self.dimension,
                found: vector.len(),
            });
        }
        let mut vectors = self.vectors.write().await;
        vectors.insert(id, vector);
        Ok(())
    }

    async fn count(&self) -> VectorResult<usize> {
        let vectors = self.vectors.read().await;
        Ok(vectors.len())
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

/// Helper function to dump all vectors from a memory index.
/// Used for migration and testing.
pub async fn memory_index_dump(index: &MemoryVectorIndex) -> VectorResult<Vec<(Uuid, Vec<f32>)>> {
    let vectors = index.vectors.read().await;
    Ok(vectors.iter().map(|(&id, vec)| (id, vec.clone())).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((MemoryVectorIndex::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((MemoryVectorIndex::cosine_similarity(&a, &c) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_search_result() {
        let id = Uuid::new_v4();
        let result = SearchResult::new(id, 0.85)
            .with_metadata("key", "value");

        assert_eq!(result.id, id);
        assert_eq!(result.score, 0.85);
        assert_eq!(result.metadata.len(), 1);
    }

    #[tokio::test]
    async fn test_memory_index() {
        let index = MemoryVectorIndex::new(3);
        assert_eq!(index.dimension(), 3);
        assert_eq!(index.count().await.unwrap(), 0);

        let query = vec![1.0, 0.0, 0.0];
        let results = index.search(&query, 5).await.unwrap();
        assert_eq!(results.len(), 0); // Empty index
    }

    /// Helper function to dump all vectors from a memory index.
    /// Used for migration and testing.
    #[cfg(any(test, feature = "lance"))]
    pub async fn memory_index_dump(index: &MemoryVectorIndex) -> VectorResult<Vec<(Uuid, Vec<f32>)>> {
        let vectors = index.vectors.read().await;
        Ok(vectors.iter().map(|(&id, vec)| (id, vec.clone())).collect())
    }
}
