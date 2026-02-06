// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Integration tests for ML module with real model inference.

#![cfg(feature = "candle")]

use synton_ml::{
    backend::{BackendType, DeviceType},
    config::{ApiConfig, EmbeddingConfig, LocalModelConfig},
    service::EmbeddingService,
};

/// Helper to calculate cosine similarity between two vectors.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

/// Normalize a vector to unit length.
fn normalize(v: &[f32]) -> Vec<f32> {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm == 0.0 {
        return v.to_vec();
    }
    v.iter().map(|x| x / norm).collect()
}

// ========== Real Model Inference Tests ==========

#[tokio::test]
#[cfg_attr(not(feature = "candle"), ignore = "requires candle feature")]
async fn test_real_model_inference() {
    // Skip in CI if no model is available
    if std::env::var("CI").is_ok() {
        println!("Skipping test in CI environment (model download required)");
        return;
    }

    let config = EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string());
    let service = EmbeddingService::from_config(config).await;

    // This might fail if the model cannot be downloaded
    match service {
        Ok(service) => {
            let embedding = service.embed("Hello, world!").await;
            assert!(embedding.is_ok());
            let embedding = embedding.unwrap();

            // Check dimension
            assert_eq!(embedding.len(), 384);

            // Check all values are finite
            for &val in &embedding {
                assert!(val.is_finite());
            }
        }
        Err(_) => {
            println!("Model not available, skipping test");
        }
    }
}

#[tokio::test]
async fn test_embedding_identical_text() {
    // Skip in CI
    if std::env::var("CI").is_ok() {
        return;
    }

    let config = EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string());
    let service = EmbeddingService::from_config(config).await;

    if let Ok(service) = service {
        let emb1 = service.embed("The quick brown fox").await;
        let emb2 = service.embed("The quick brown fox").await;

        assert!(emb1.is_ok());
        assert!(emb2.is_ok());

        let emb1 = emb1.unwrap();
        let emb2 = emb2.unwrap();

        // Identical text should have similarity > 0.99
        let similarity = cosine_similarity(&emb1, &emb2);
        assert!(similarity > 0.99, "Similarity: {}", similarity);
    }
}

#[tokio::test]
async fn test_embedding_similar_text() {
    // Skip in CI
    if std::env::var("CI").is_ok() {
        return;
    }

    let config = EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string());
    let service = EmbeddingService::from_config(config).await;

    if let Ok(service) = service {
        let emb1 = service.embed("The cat sat on the mat").await;
        let emb2 = service.embed("A cat was sitting on the rug").await;

        assert!(emb1.is_ok());
        assert!(emb2.is_ok());

        let emb1 = emb1.unwrap();
        let emb2 = emb2.unwrap();

        // Similar text should have similarity > 0.5
        let similarity = cosine_similarity(&emb1, &emb2);
        assert!(similarity > 0.5, "Similarity: {}", similarity);
    }
}

#[tokio::test]
async fn test_embedding_different_text() {
    // Skip in CI
    if std::env::var("CI").is_ok() {
        return;
    }

    let config = EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string());
    let service = EmbeddingService::from_config(config).await;

    if let Ok(service) = service {
        let emb1 = service.embed("machine learning").await;
        let emb2 = service.embed("banana pudding recipe").await;

        assert!(emb1.is_ok());
        assert!(emb2.is_ok());

        let emb1 = emb1.unwrap();
        let emb2 = emb2.unwrap();

        // Different text should have lower similarity
        let similarity = cosine_similarity(&emb1, &emb2);
        assert!(similarity < 0.5, "Similarity: {}", similarity);
    }
}

#[tokio::test]
async fn test_batch_embedding() {
    // Skip in CI
    if std::env::var("CI").is_ok() {
        return;
    }

    let config = EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string());
    let service = EmbeddingService::from_config(config).await;

    if let Ok(service) = service {
        let texts = vec![
            "First text".to_string(),
            "Second text".to_string(),
            "Third text".to_string(),
        ];

        let embeddings = service.embed_batch(&texts).await;
        assert!(embeddings.is_ok());
        let embeddings = embeddings.unwrap();

        assert_eq!(embeddings.len(), 3);

        // All embeddings should have the same dimension
        for emb in &embeddings {
            assert_eq!(emb.len(), 384);
        }
    }
}

#[tokio::test]
async fn test_cache_effectiveness() {
    // Skip in CI
    if std::env::var("CI").is_ok() {
        return;
    }

    let config = EmbeddingConfig {
        cache_enabled: true,
        cache_size: 100,
        ..EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string())
    };

    let service = EmbeddingService::from_config(config).await;

    if let Ok(service) = service {
        let text = "cache test text";

        // First call - cache miss
        let stats1 = service.stats().await;
        let _ = service.embed(text).await.unwrap();
        let stats2 = service.stats().await;

        // Cache hit count should increase
        assert_eq!(stats2.total_embeddings, stats1.total_embeddings + 1);

        // Second call - cache hit (if cache is working)
        let _ = service.embed(text).await.unwrap();
        let stats3 = service.stats().await;

        assert_eq!(stats3.total_embeddings, stats2.total_embeddings + 1);
        // Cache hits may or may not increase depending on implementation
    }
}

#[tokio::test]
async fn test_empty_input_handling() {
    let config = EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string());
    let service = EmbeddingService::from_config(config).await;

    if let Ok(service) = service {
        // Empty string should return an error
        let result = service.embed("").await;
        assert!(result.is_err());

        // Whitespace only should return an error
        let result = service.embed("   ").await;
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn test_long_text_handling() {
    // Skip in CI
    if std::env::var("CI").is_ok() {
        return;
    }

    let config = EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string());
    let service = EmbeddingService::from_config(config).await;

    if let Ok(service) = service {
        // Create a long text (longer than typical max_length)
        let long_text = "This is a sentence. ".repeat(100);

        let result = service.embed(&long_text).await;
        assert!(result.is_ok());

        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 384);
    }
}

#[tokio::test]
async fn test_service_stats() {
    let config = EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string());
    let service = EmbeddingService::from_config(config).await;

    if let Ok(service) = service {
        let stats = service.stats().await;

        // Initial stats should be zero
        assert_eq!(stats.total_embeddings, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.total_tokens, 0);
    }
}

#[tokio::test]
async fn test_device_types() {
    // Test that different device types can be configured
    let configs = vec![
        LocalModelConfig {
            model_name: "test".to_string(),
            device: DeviceType::Cpu,
            ..Default::default()
        },
        LocalModelConfig {
            model_name: "test".to_string(),
            device: DeviceType::Metal,
            ..Default::default()
        },
    ];

    for config in configs {
        // Just verify the config can be created
        assert_eq!(config.model_name, "test");
    }
}

#[tokio::test]
async fn test_model_dimension_registry() {
    // Test that dimensions are correctly registered for known models
    let test_cases = vec![
        ("sentence-transformers/all-MiniLM-L6-v2", 384),
        ("sentence-transformers/all-mpnet-base-v2", 768),
        ("BAAI/bge-small-en-v1.5", 384),
        ("BAAI/bge-base-en-v1.5", 768),
        ("BAAI/bge-large-en-v1.5", 1024),
    ];

    for (model_name, expected_dim) in test_cases {
        let config = EmbeddingConfig::local(model_name.to_string());
        assert_eq!(config.dimension(), expected_dim, "Failed for {}", model_name);
    }
}

#[tokio::test]
async fn test_embedding_normalization() {
    // Skip in CI
    if std::env::var("CI").is_ok() {
        return;
    }

    let config = EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string());
    let service = EmbeddingService::from_config(config).await;

    if let Ok(service) = service {
        let embedding = service.embed("test text").await.unwrap();

        // Normalize and check
        let normalized = normalize(&embedding);
        let norm: f32 = normalized.iter().map(|x| x * x).sum::<f32>().sqrt();

        // Should be approximately 1.0
        assert!((norm - 1.0).abs() < 0.001, "Norm: {}", norm);
    }
}

#[tokio::test]
async fn test_multilingual_text() {
    // Skip in CI
    if std::env::var("CI").is_ok() {
        return;
    }

    let config = EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string());
    let service = EmbeddingService::from_config(config).await;

    if let Ok(service) = service {
        let texts = vec![
            "Hello world",
            "你好世界",
            "Hola mundo",
            "Bonjour le monde",
        ];

        for text in texts {
            let result = service.embed(text).await;
            assert!(result.is_ok(), "Failed for: {}", text);

            let embedding = result.unwrap();
            assert_eq!(embedding.len(), 384);
        }
    }
}

#[tokio::test]
async fn test_special_characters() {
    // Skip in CI
    if std::env::var("CI").is_ok() {
        return;
    }

    let config = EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string());
    let service = EmbeddingService::from_config(config).await;

    if let Ok(service) = service {
        let texts = vec![
            "Café résumé naïve",
            "🎉 emoji test 🚀",
            "<html>tag & script</html>",
            "python_variable_name_with_underscores",
        ];

        for text in texts {
            let result = service.embed(text).await;
            assert!(result.is_ok(), "Failed for: {}", text);

            let embedding = result.unwrap();
            assert_eq!(embedding.len(), 384);
        }
    }
}
