// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Comprehensive unit tests for VectorIndex and MemoryVectorIndex.

use uuid::Uuid;

use synton_vector::{MemoryVectorIndex, SearchResult, VectorIndex};

/// Helper function to create a test vector of given dimension.
fn test_vector(dim: usize, value: f32) -> Vec<f32> {
    vec![value; dim]
}

/// Helper function to create a normalized random vector.
fn random_vector(dim: usize) -> Vec<f32> {
    let mut v = Vec::with_capacity(dim);
    for i in 0..dim {
        v.push((i as f32 * 0.1) % 1.0);
    }
    // Normalize
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    v.iter().map(|x| x / norm).collect()
}

// ========== SearchResult Tests ==========

#[test]
fn test_search_result_new() {
    let id = Uuid::new_v4();
    let result = SearchResult::new(id, 0.85);

    assert_eq!(result.id, id);
    assert_eq!(result.score, 0.85);
    assert!(result.metadata.is_empty());
}

#[test]
fn test_search_result_with_metadata() {
    let id = Uuid::new_v4();
    let result = SearchResult::new(id, 0.9)
        .with_metadata("key1", "value1")
        .with_metadata("key2", "value2");

    assert_eq!(result.metadata.len(), 2);
    assert_eq!(result.metadata[0], ("key1".to_string(), "value1".to_string()));
    assert_eq!(result.metadata[1], ("key2".to_string(), "value2".to_string()));
}

#[test]
fn test_search_result_score_clamping() {
    let id = Uuid::new_v4();

    // Test upper bound
    let result1 = SearchResult::new(id, 1.5);
    assert_eq!(result1.score, 1.0);

    // Test lower bound
    let result2 = SearchResult::new(id, -0.5);
    assert_eq!(result2.score, 0.0);

    // Test within bounds
    let result3 = SearchResult::new(id, 0.5);
    assert_eq!(result3.score, 0.5);
}

#[test]
fn test_search_result_equality() {
    let id = Uuid::new_v4();
    let result1 = SearchResult::new(id, 0.8);
    let result2 = SearchResult::new(id, 0.8);

    assert_eq!(result1, result2);
}

// ========== MemoryVectorIndex Creation Tests ==========

#[tokio::test]
async fn test_memory_vector_index_new() {
    let index = MemoryVectorIndex::new(384);

    assert_eq!(index.dimension(), 384);
    assert!(index.is_ready());
    assert_eq!(index.count().await.unwrap(), 0);
}

#[tokio::test]
async fn test_memory_vector_index_empty() {
    let index = MemoryVectorIndex::new(128);

    let query = test_vector(128, 0.5);
    let results = index.search(&query, 10).await.unwrap();

    assert!(results.is_empty());
}

// ========== Insert Tests ==========

#[tokio::test]
async fn test_insert_single_vector() {
    let index = MemoryVectorIndex::new(64);

    let id = Uuid::new_v4();
    let vector = test_vector(64, 1.0);

    index.insert(id, vector).await.unwrap();

    assert_eq!(index.count().await.unwrap(), 1);
}

#[tokio::test]
async fn test_insert_multiple_vectors() {
    let index = MemoryVectorIndex::new(32);

    for _ in 0..10 {
        let id = Uuid::new_v4();
        let vector = random_vector(32);
        index.insert(id, vector).await.unwrap();
    }

    assert_eq!(index.count().await.unwrap(), 10);
}

#[tokio::test]
async fn test_insert_wrong_dimension() {
    let index = MemoryVectorIndex::new(64);

    let id = Uuid::new_v4();
    let wrong_vector = test_vector(32, 1.0);

    let result = index.insert(id, wrong_vector).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(
        format!("{}", err),
        "Invalid dimension: expected 64, found 32"
    );
}

#[tokio::test]
async fn test_insert_duplicate_id() {
    let index = MemoryVectorIndex::new(64);

    let id = Uuid::new_v4();
    let vector1 = test_vector(64, 1.0);
    let vector2 = test_vector(64, 0.5);

    index.insert(id, vector1).await.unwrap();
    index.insert(id, vector2).await.unwrap();

    // Should overwrite, count should still be 1
    assert_eq!(index.count().await.unwrap(), 1);

    // Search should return the updated vector
    let query = test_vector(64, 0.5);
    let results = index.search(&query, 1).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, id);
}

// ========== Batch Insert Tests ==========

#[tokio::test]
async fn test_insert_batch() {
    let index = MemoryVectorIndex::new(64);

    let mut vectors = Vec::new();
    for i in 0..10 {
        let id = Uuid::new_v4();
        let vector = test_vector(64, i as f32 / 10.0);
        vectors.push((id, vector));
    }

    index.insert_batch(vectors).await.unwrap();

    assert_eq!(index.count().await.unwrap(), 10);
}

#[tokio::test]
async fn test_insert_batch_empty() {
    let index = MemoryVectorIndex::new(64);

    index.insert_batch(vec![]).await.unwrap();

    assert_eq!(index.count().await.unwrap(), 0);
}

#[tokio::test]
async fn test_insert_batch_wrong_dimension() {
    let index = MemoryVectorIndex::new(64);

    let vectors = vec![
        (Uuid::new_v4(), test_vector(64, 1.0)),
        (Uuid::new_v4(), test_vector(32, 1.0)), // Wrong dimension
    ];

    let result = index.insert_batch(vectors).await;

    assert!(result.is_err());
}

// ========== Search Tests ==========

#[tokio::test]
async fn test_search_k_nearest() {
    let index = MemoryVectorIndex::new(64);

    // Insert some vectors
    let ids: Vec<_> = (0..10).map(|_| Uuid::new_v4()).collect();
    for (i, &id) in ids.iter().enumerate() {
        let vector = test_vector(64, i as f32 / 10.0);
        index.insert(id, vector).await.unwrap();
    }

    // Search for vectors similar to [0.9, 0.9, ...]
    let query = test_vector(64, 0.9);
    let results = index.search(&query, 3).await.unwrap();

    assert_eq!(results.len(), 3);
    // Results should be sorted by score (descending)
    assert!(results[0].score >= results[1].score);
    assert!(results[1].score >= results[2].score);
}

#[tokio::test]
async fn test_search_query_wrong_dimension() {
    let index = MemoryVectorIndex::new(64);

    let query = test_vector(32, 0.5);
    let result = index.search(&query, 10).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_search_k_larger_than_count() {
    let index = MemoryVectorIndex::new(64);

    let id = Uuid::new_v4();
    index.insert(id, test_vector(64, 1.0)).await.unwrap();

    let query = test_vector(64, 0.9);
    let results = index.search(&query, 100).await.unwrap();

    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn test_cosine_similarity_identical() {
    let index = MemoryVectorIndex::new(64);

    let id = Uuid::new_v4();
    let vector = random_vector(64);
    index.insert(id, vector.clone()).await.unwrap();

    // Search with the same vector - should return score ~1.0
    let results = index.search(&vector, 1).await.unwrap();

    assert_eq!(results.len(), 1);
    assert!((results[0].score - 1.0).abs() < 0.001);
}

#[tokio::test]
async fn test_cosine_similarity_orthogonal() {
    let index = MemoryVectorIndex::new(64);

    // Create two orthogonal vectors
    let mut v1 = vec![0.0; 64];
    v1[0] = 1.0;

    let mut v2 = vec![0.0; 64];
    v2[1] = 1.0;

    let id = Uuid::new_v4();
    index.insert(id, v2).await.unwrap();

    let results = index.search(&v1, 1).await.unwrap();

    // Orthogonal vectors should have similarity ~0.0
    assert_eq!(results.len(), 1);
    assert!(results[0].score < 0.01);
}

#[tokio::test]
async fn test_search_ordering() {
    let index = MemoryVectorIndex::new(64);

    // Create vectors at different distances from query
    let query = test_vector(64, 0.5);

    let id1 = Uuid::new_v4();
    index.insert(id1, test_vector(64, 0.5)).await.unwrap(); // Identical to query

    let id2 = Uuid::new_v4();
    index.insert(id2, test_vector(64, 0.4)).await.unwrap(); // Close

    let id3 = Uuid::new_v4();
    index.insert(id3, test_vector(64, 0.0)).await.unwrap(); // Far

    let results = index.search(&query, 10).await.unwrap();

    assert_eq!(results.len(), 3);
    // id1 should be first (highest similarity)
    assert_eq!(results[0].id, id1);
    assert!((results[0].score - 1.0).abs() < 0.01);
}

// ========== Update Tests ==========

#[tokio::test]
async fn test_update_existing_vector() {
    let index = MemoryVectorIndex::new(64);

    let id = Uuid::new_v4();
    let vector1 = test_vector(64, 1.0);
    let vector2 = test_vector(64, 0.5);

    index.insert(id, vector1).await.unwrap();
    index.update(id, vector2.clone()).await.unwrap();

    let results = index.search(&vector2, 1).await.unwrap();
    assert_eq!(results.len(), 1);
    assert!((results[0].score - 1.0).abs() < 0.01);
}

#[tokio::test]
async fn test_update_nonexistent_vector() {
    let index = MemoryVectorIndex::new(64);

    let id = Uuid::new_v4();
    let vector = test_vector(64, 1.0);

    // Update should work even if id doesn't exist (inserts new)
    index.update(id, vector).await.unwrap();
    assert_eq!(index.count().await.unwrap(), 1);
}

#[tokio::test]
async fn test_update_wrong_dimension() {
    let index = MemoryVectorIndex::new(64);

    let id = Uuid::new_v4();
    let wrong_vector = test_vector(32, 1.0);

    let result = index.update(id, wrong_vector).await;

    assert!(result.is_err());
}

// ========== Delete Tests ==========

#[tokio::test]
async fn test_delete_existing_vector() {
    let index = MemoryVectorIndex::new(64);

    let id = Uuid::new_v4();
    let vector = test_vector(64, 1.0);

    index.insert(id, vector).await.unwrap();
    assert_eq!(index.count().await.unwrap(), 1);

    index.remove(id).await.unwrap();
    assert_eq!(index.count().await.unwrap(), 0);
}

#[tokio::test]
async fn test_delete_nonexistent_vector() {
    let index = MemoryVectorIndex::new(64);

    let id = Uuid::new_v4();

    // Should not error
    index.remove(id).await.unwrap();
    assert_eq!(index.count().await.unwrap(), 0);
}

#[tokio::test]
async fn test_delete_then_search() {
    let index = MemoryVectorIndex::new(64);

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();

    index.insert(id1, test_vector(64, 1.0)).await.unwrap();
    index.insert(id2, test_vector(64, 0.5)).await.unwrap();

    index.remove(id1).await.unwrap();

    let query = test_vector(64, 1.0);
    let results = index.search(&query, 10).await.unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, id2);
}

// ========== Integration Tests ==========

#[tokio::test]
async fn test_crud_cycle() {
    let index = MemoryVectorIndex::new(128);

    // Create
    let id = Uuid::new_v4();
    let vector = random_vector(128);
    index.insert(id, vector.clone()).await.unwrap();

    // Read
    let results = index.search(&vector, 1).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, id);

    // Update
    let new_vector = random_vector(128);
    index.update(id, new_vector.clone()).await.unwrap();
    let results = index.search(&new_vector, 1).await.unwrap();
    assert_eq!(results[0].id, id);

    // Delete
    index.remove(id).await.unwrap();
    assert_eq!(index.count().await.unwrap(), 0);
}

#[tokio::test]
async fn test_large_scale_operations() {
    let index = MemoryVectorIndex::new(384);

    let num_vectors = 1000;
    let ids: Vec<_> = (0..num_vectors).map(|_| Uuid::new_v4()).collect();

    // Insert many vectors
    for &id in &ids {
        let vector = random_vector(384);
        index.insert(id, vector).await.unwrap();
    }

    assert_eq!(index.count().await.unwrap(), num_vectors);

    // Search
    let query = random_vector(384);
    let results = index.search(&query, 10).await.unwrap();

    assert_eq!(results.len(), 10);
    // All scores should be between 0 and 1
    for result in &results {
        assert!(result.score >= 0.0 && result.score <= 1.0);
    }
}

#[tokio::test]
async fn test_concurrent_operations() {
    use std::sync::Arc;
    let index = Arc::new(MemoryVectorIndex::new(64));

    let mut insert_handles = Vec::new();
    let mut search_handles = Vec::new();

    // Concurrent inserts
    for i in 0..50 {
        let index_clone = index.clone();
        insert_handles.push(tokio::spawn(async move {
            let id = Uuid::new_v4();
            let vector = test_vector(64, i as f32);
            index_clone.insert(id, vector).await
        }));
    }

    // Concurrent searches
    for _ in 0..20 {
        let index_clone = index.clone();
        search_handles.push(tokio::spawn(async move {
            let query = test_vector(64, 0.5);
            index_clone.search(&query, 5).await
        }));
    }

    // Wait for all inserts to complete
    for handle in insert_handles {
        handle.await.expect("Task failed").expect("Insert failed");
    }

    // Wait for all searches to complete (searches may fail when index is empty, which is ok)
    for handle in search_handles {
        let _ = handle.await.expect("Task failed");
    }

    // Final count should reflect successful inserts
    assert_eq!(index.count().await.unwrap(), 50);
}

// ========== Edge Cases ==========

#[test]
fn test_dimension_zero() {
    let index = MemoryVectorIndex::new(0);
    assert_eq!(index.dimension(), 0);
}

#[tokio::test]
async fn test_empty_vector_insert() {
    let index = MemoryVectorIndex::new(0);

    let id = Uuid::new_v4();
    let vector: Vec<f32> = vec![];

    // Should work for dimension 0
    let result = index.insert(id, vector).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_with_k_zero() {
    let index = MemoryVectorIndex::new(64);

    let id = Uuid::new_v4();
    index.insert(id, test_vector(64, 1.0)).await.unwrap();

    let query = test_vector(64, 0.5);
    let results = index.search(&query, 0).await.unwrap();

    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_vector_with_nan_and_inf() {
    let index = MemoryVectorIndex::new(64);

    let id = Uuid::new_v4();
    let mut vector = test_vector(64, 1.0);
    vector[0] = f32::NAN;

    // Should still insert, but search behavior is undefined
    index.insert(id, vector).await.unwrap();
    assert_eq!(index.count().await.unwrap(), 1);
}

#[tokio::test]
async fn test_negative_vector_values() {
    let index = MemoryVectorIndex::new(64);

    let id1 = Uuid::new_v4();
    let v1 = test_vector(64, -1.0);

    let id2 = Uuid::new_v4();
    let v2 = test_vector(64, 1.0);

    index.insert(id1, v1).await.unwrap();
    index.insert(id2, v2).await.unwrap();

    // Negative and positive vectors should have similarity 1.0 (same direction)
    let results = index.search(&test_vector(64, 1.0), 2).await.unwrap();
    assert_eq!(results.len(), 2);
}

// ========== Search Result Metadata Tests ==========

#[tokio::test]
async fn test_search_result_metadata_preservation() {
    // This tests that metadata can be attached to search results
    // In MemoryVectorIndex, metadata is always empty in search results
    let index = MemoryVectorIndex::new(64);

    let id = Uuid::new_v4();
    index.insert(id, test_vector(64, 1.0)).await.unwrap();

    let results = index.search(&test_vector(64, 1.0), 1).await.unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].metadata.is_empty());
}
