// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Comprehensive unit tests for ML module and local embedding backend.

use synton_ml::{
    backend::{BackendType, DeviceType},
    config::{ApiConfig, EmbeddingConfig, LocalModelConfig},
    error::MlError,
    service::EmbeddingService,
};

// ========== EmbeddingConfig Tests ==========

#[test]
fn test_config_default() {
    let config = EmbeddingConfig::default();

    assert_eq!(config.backend, BackendType::Local);
    assert_eq!(config.local.model_name, "sentence-transformers/all-MiniLM-L6-v2");
}

#[test]
fn test_config_local() {
    let config = EmbeddingConfig::local("BAAI/bge-base-en-v1.5".to_string());

    assert_eq!(config.backend, BackendType::Local);
    assert_eq!(config.local.model_name, "BAAI/bge-base-en-v1.5");
}

#[test]
fn test_config_openai() {
    let config = EmbeddingConfig::openai("sk-test-key".to_string());

    assert_eq!(config.backend, BackendType::OpenAi);
    assert_eq!(config.api.model, "text-embedding-3-small");
    assert_eq!(config.api.api_key, Some("sk-test-key".to_string()));
}

#[test]
fn test_config_ollama() {
    let config = EmbeddingConfig::ollama();

    assert_eq!(config.backend, BackendType::Ollama);
    assert_eq!(config.api.endpoint, "http://localhost:11434");
    assert_eq!(config.api.model, "nomic-embed-text");
}

// ========== LocalModelConfig Tests ==========

#[test]
fn test_local_model_config_default() {
    let config = LocalModelConfig::default();

    assert_eq!(config.model_name, "sentence-transformers/all-MiniLM-L6-v2");
    assert_eq!(config.device, DeviceType::Cpu);
    assert!(config.max_length > 0);
}

#[test]
fn test_local_model_config_custom() {
    let config = LocalModelConfig {
        model_name: "BAAI/bge-base-en-v1.5".to_string(),
        device: DeviceType::Metal,
        max_length: 256,
        ..Default::default()
    };

    assert_eq!(config.model_name, "BAAI/bge-base-en-v1.5");
    assert_eq!(config.device, DeviceType::Metal);
}

// ========== ApiConfig Tests ==========

#[test]
fn test_api_config_default() {
    let config = ApiConfig::default();

    assert_eq!(config.endpoint, "https://api.openai.com/v1");
    assert_eq!(config.model, "text-embedding-3-small");
    assert_eq!(config.timeout_secs, 30);
    assert!(config.api_key.is_none());
}

#[test]
fn test_api_config_openai() {
    let config = ApiConfig::openai("sk-12345".to_string());

    assert_eq!(config.endpoint, "https://api.openai.com/v1");
    assert_eq!(config.api_key, Some("sk-12345".to_string()));
    assert_eq!(config.model, "text-embedding-3-small");
}

#[test]
fn test_api_config_ollama() {
    let config = ApiConfig::ollama();

    assert_eq!(config.endpoint, "http://localhost:11434");
    assert_eq!(config.model, "nomic-embed-text");
    assert!(config.api_key.is_none());
}

// ========== BackendType Tests ==========

#[test]
fn test_backend_type_display() {
    assert_eq!(format!("{}", BackendType::Local), "local");
    assert_eq!(format!("{}", BackendType::OpenAi), "openai");
    assert_eq!(format!("{}", BackendType::Ollama), "ollama");
}

#[test]
fn test_backend_type_from_str() {
    assert_eq!("local".parse::<BackendType>().unwrap(), BackendType::Local);
    assert_eq!("openai".parse::<BackendType>().unwrap(), BackendType::OpenAi);
    assert_eq!("ollama".parse::<BackendType>().unwrap(), BackendType::Ollama);
    assert!("unknown".parse::<BackendType>().is_err());
}

// ========== DeviceType Tests ==========

#[test]
fn test_device_type_display() {
    assert_eq!(format!("{}", DeviceType::Cpu), "cpu");
    assert_eq!(format!("{}", DeviceType::Cuda), "cuda");
    assert_eq!(format!("{}", DeviceType::Metal), "metal");
    assert_eq!(format!("{}", DeviceType::Rocm), "rocm");
}

#[test]
fn test_device_type_from_str() {
    assert_eq!("cpu".parse::<DeviceType>().unwrap(), DeviceType::Cpu);
    assert_eq!("cuda".parse::<DeviceType>().unwrap(), DeviceType::Cuda);
    assert_eq!("metal".parse::<DeviceType>().unwrap(), DeviceType::Metal);
    assert_eq!("mps".parse::<DeviceType>().unwrap(), DeviceType::Metal);
    assert_eq!("rocm".parse::<DeviceType>().unwrap(), DeviceType::Rocm);
    assert!("unknown".parse::<DeviceType>().is_err());
}

#[test]
fn test_device_type_equality() {
    assert_eq!(DeviceType::Cpu, DeviceType::Cpu);
    assert_eq!(DeviceType::Cuda, DeviceType::Cuda);
    assert_ne!(DeviceType::Cpu, DeviceType::Cuda);
}

// ========== MlError Tests ==========

#[test]
fn test_error_display() {
    let error = MlError::EmptyInput;
    assert_eq!(error.to_string(), "Empty input provided");

    let error = MlError::ModelLoadFailed("model not found".to_string());
    assert!(error.to_string().contains("model not found"));

    let error = MlError::ApiError("API call failed".to_string());
    assert!(error.to_string().contains("API call failed"));
}

#[test]
fn test_error_constructors() {
    let error = MlError::model_load_failed("test message");
    assert!(matches!(error, MlError::ModelLoadFailed(_)));

    let error = MlError::inference_failed("test message");
    assert!(matches!(error, MlError::InferenceFailed(_)));

    let error = MlError::api_error("test message");
    assert!(matches!(error, MlError::ApiError(_)));

    let error = MlError::invalid_config("test message");
    assert!(matches!(error, MlError::InvalidConfig(_)));

    let error = MlError::tokenization_error("test message");
    assert!(matches!(error, MlError::TokenizationError(_)));

    let error = MlError::download_error("test message");
    assert!(matches!(error, MlError::DownloadError(_)));
}

#[test]
fn test_error_debug() {
    let error = MlError::EmptyInput;
    let debug_str = format!("{:?}", error);
    assert!(!debug_str.is_empty());
}

// ========== EmbeddingConfig Dimension Tests ==========

#[test]
fn test_config_dimension_local() {
    let config = EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string());
    assert_eq!(config.dimension(), 384);

    let config = EmbeddingConfig::local("sentence-transformers/all-mpnet-base-v2".to_string());
    assert_eq!(config.dimension(), 768);

    let config = EmbeddingConfig::local("BAAI/bge-small-en-v1.5".to_string());
    assert_eq!(config.dimension(), 384);

    let config = EmbeddingConfig::local("BAAI/bge-large-en-v1.5".to_string());
    assert_eq!(config.dimension(), 1024);

    // Unknown model defaults to 384
    let config = EmbeddingConfig::local("unknown/model".to_string());
    assert_eq!(config.dimension(), 384);
}

#[test]
fn test_config_dimension_openai() {
    let config = EmbeddingConfig::openai("sk-test".to_string());
    assert_eq!(config.dimension(), 1536); // text-embedding-3-small

    let mut config = EmbeddingConfig::openai("sk-test".to_string());
    config.api.model = "text-embedding-3-large".to_string();
    assert_eq!(config.dimension(), 3072);

    config.api.model = "text-embedding-ada-002".to_string();
    assert_eq!(config.dimension(), 1536);
}

#[test]
fn test_config_dimension_ollama() {
    let config = EmbeddingConfig::ollama();
    assert_eq!(config.dimension(), 768); // nomic-embed-text

    let mut config = EmbeddingConfig::ollama();
    config.api.model = "mxbai-embed-large".to_string();
    assert_eq!(config.dimension(), 1024);
}

#[test]
fn test_config_dimension_override() {
    let mut config = EmbeddingConfig::local("test-model".to_string());
    assert_eq!(config.dimension(), 384);

    config.dimension_override = 512;
    assert_eq!(config.dimension(), 512);
}

// ========== EmbeddingConfig Validation Tests ==========

#[tokio::test]
async fn test_config_validation_valid() {
    let config = EmbeddingConfig::default();
    assert!(config.validate().is_ok());

    let config = EmbeddingConfig::local("test-model".to_string());
    assert!(config.validate().is_ok());

    let config = EmbeddingConfig::openai("sk-test".to_string());
    assert!(config.validate().is_ok());

    let config = EmbeddingConfig::ollama();
    assert!(config.validate().is_ok());
}

#[test]
fn test_config_validation_invalid_local() {
    let mut config = EmbeddingConfig::local("".to_string());
    assert!(config.validate().is_err());
}

#[test]
fn test_config_validation_invalid_api() {
    let mut config = EmbeddingConfig::openai("sk-test".to_string());
    config.api.endpoint = "".to_string();
    assert!(config.validate().is_err());

    let mut config = EmbeddingConfig::openai("sk-test".to_string());
    config.api.model = "".to_string();
    assert!(config.validate().is_err());
}

// ========== EmbeddingService Tests ==========

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
async fn test_service_invalid_config() {
    let config = EmbeddingConfig {
        backend: BackendType::Local,
        local: LocalModelConfig {
            model_name: "".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    let service = EmbeddingService::from_config(config).await;
    assert!(service.is_err());
}

#[tokio::test]
async fn test_service_embed_empty_input() {
    let config = EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string());
    let service = EmbeddingService::from_config(config).await.unwrap();

    let result = service.embed("").await;
    assert!(matches!(result, Err(MlError::EmptyInput)));

    let result = service.embed("   ").await;
    assert!(matches!(result, Err(MlError::EmptyInput)));
}

#[tokio::test]
async fn test_service_embed_batch_empty() {
    let config = EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string());
    let service = EmbeddingService::from_config(config).await.unwrap();

    let result = service.embed_batch(&[]).await.unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn test_service_embed_batch_with_empty_string() {
    let config = EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string());
    let service = EmbeddingService::from_config(config).await.unwrap();

    let result = service.embed_batch(&["".to_string()]).await;
    assert!(matches!(result, Err(MlError::EmptyInput)));
}

#[tokio::test]
async fn test_service_stats() {
    let config = EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string());
    let service = EmbeddingService::from_config(config).await.unwrap();

    let stats = service.stats().await;

    assert_eq!(stats.total_embeddings, 0);
    assert_eq!(stats.cache_hits, 0);
    assert_eq!(stats.total_tokens, 0);
}

#[tokio::test]
async fn test_service_cache_size() {
    let config = EmbeddingConfig {
        cache_enabled: true,
        cache_size: 100,
        ..EmbeddingConfig::local("sentence-transformers/all-MiniLM-L6-v2".to_string())
    };

    let service = EmbeddingService::from_config(config).await.unwrap();

    let size = service.cache_size().await;
    assert_eq!(size, 0);
}

#[tokio::test]
async fn test_service_config() {
    let config = EmbeddingConfig::local("test-model".to_string());
    let service = EmbeddingService::from_config(config).await.unwrap();

    let service_config = service.config();
    assert_eq!(service_config.local.model_name, "test-model");
}

// ========== Config Serialization Tests ==========

#[test]
fn test_config_serialization() {
    let config = EmbeddingConfig::default();

    let toml = toml::to_string(&config).unwrap();
    assert!(toml.contains("backend = \"local\""));
    assert!(toml.contains("model_name"));

    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("\"local\""));
}

#[test]
fn test_config_deserialization() {
    let toml_str = r#"
        backend = "local"
        cache_enabled = true
        cache_size = 1000

        [local]
        model_name = "BAAI/bge-base-en-v1.5"
        max_length = 512
    "#;

    let config: EmbeddingConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.backend, BackendType::Local);
    assert_eq!(config.local.model_name, "BAAI/bge-base-en-v1.5");
    assert!(config.cache_enabled);
    assert_eq!(config.cache_size, 1000);
}

// ========== Backend Availability Tests ==========

#[test]
fn test_local_backend_is_available() {
    let is_available = synton_ml::local::LocalEmbeddingBackend::is_available();

    // Should return true based on feature flag
    #[cfg(feature = "candle")]
    assert!(is_available);

    #[cfg(not(feature = "candle"))]
    assert!(!is_available);
}
