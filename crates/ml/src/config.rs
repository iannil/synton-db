// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Configuration structures for the ML module.

use serde::{Deserialize, Serialize};

use crate::backend::{BackendType, DeviceType};
use crate::error::{MlError, Result as MlResult};

/// Local model configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LocalModelConfig {
    /// Model name or path.
    /// Can be:
    /// - A HuggingFace model ID (e.g., "sentence-transformers/all-MiniLM-L6-v2")
    /// - A local path to a model directory
    pub model_name: String,

    /// Device to run inference on.
    pub device: DeviceType,

    /// Maximum sequence length (in tokens).
    pub max_length: usize,

    /// Batch size for batch inference.
    pub batch_size: usize,

    /// Use quantized model if available.
    pub use_quantized: bool,

    /// Number of threads for CPU inference.
    pub num_threads: usize,

    /// Model cache directory.
    pub cache_dir: Option<String>,

    /// Retry model download on failure.
    pub retry_download: bool,

    /// Maximum number of download retries.
    pub max_retries: usize,
}

impl Default for LocalModelConfig {
    fn default() -> Self {
        Self {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            device: DeviceType::default(),
            max_length: 512,
            batch_size: 32,
            use_quantized: true,
            num_threads: 4,
            cache_dir: None,
            retry_download: true,
            max_retries: 3,
        }
    }
}

/// API-based embedding configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ApiConfig {
    /// API endpoint URL.
    /// For OpenAI: "https://api.openai.com/v1"
    /// For Ollama: "http://localhost:11434"
    pub endpoint: String,

    /// API key (optional for local APIs like Ollama).
    pub api_key: Option<String>,

    /// Model name to use.
    /// For OpenAI: "text-embedding-3-small" or "text-embedding-3-large"
    /// For Ollama: "nomic-embed-text", "mxbai-embed-large", etc.
    pub model: String,

    /// Request timeout in seconds.
    pub timeout_secs: u64,

    /// Maximum retries for failed requests.
    pub max_retries: usize,

    /// Retry delay in milliseconds.
    pub retry_delay_ms: usize,

    /// Batch size for batch requests (if supported).
    pub batch_size: usize,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://api.openai.com/v1".to_string(),
            api_key: None,
            model: "text-embedding-3-small".to_string(),
            timeout_secs: 30,
            max_retries: 3,
            retry_delay_ms: 1000,
            batch_size: 100,
        }
    }
}

impl ApiConfig {
    /// Create an OpenAI configuration.
    pub fn openai(api_key: String) -> Self {
        Self {
            endpoint: "https://api.openai.com/v1".to_string(),
            api_key: Some(api_key),
            model: "text-embedding-3-small".to_string(),
            ..Default::default()
        }
    }

    /// Create an Ollama configuration.
    pub fn ollama() -> Self {
        Self {
            endpoint: "http://localhost:11434".to_string(),
            api_key: None,
            model: "nomic-embed-text".to_string(),
            timeout_secs: 60,
            batch_size: 10, // Ollama handles smaller batches better
            ..Default::default()
        }
    }

    /// Get the API key from the configuration or environment variable.
    pub fn get_api_key(&self) -> MlResult<Option<String>> {
        if let Some(key) = &self.api_key {
            // Check for environment variable placeholder
            if key.starts_with("${") && key.ends_with('}') {
                let var_name = &key[2..key.len() - 1];
                Ok(std::env::var(var_name).ok())
            } else {
                Ok(Some(key.clone()))
            }
        } else {
            Ok(None)
        }
    }

    /// Validate the API configuration.
    pub fn validate(&self) -> MlResult<()> {
        if self.endpoint.is_empty() {
            return Err(MlError::invalid_config("API endpoint cannot be empty"));
        }

        // Validate URL format
        if let Err(e) = url::Url::parse(&self.endpoint) {
            return Err(MlError::invalid_config(format!("Invalid API endpoint URL: {e}")));
        }

        if self.model.is_empty() {
            return Err(MlError::invalid_config("Model name cannot be empty"));
        }

        Ok(())
    }
}

/// Complete embedding configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EmbeddingConfig {
    /// Backend type to use.
    pub backend: BackendType,

    /// Local model configuration.
    pub local: LocalModelConfig,

    /// API configuration (for OpenAI/Ollama backends).
    pub api: ApiConfig,

    /// Embedding dimension (overrides model default if set).
    /// Set to 0 to use model's default dimension.
    #[serde(default)]
    pub dimension_override: usize,

    /// Enable embedding caching.
    pub cache_enabled: bool,

    /// Maximum cache size (number of embeddings).
    pub cache_size: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            backend: BackendType::default(),
            local: LocalModelConfig::default(),
            api: ApiConfig::default(),
            dimension_override: 0,
            cache_enabled: true,
            cache_size: 10000,
        }
    }
}

impl EmbeddingConfig {
    /// Create a local model configuration.
    pub fn local(model_name: String) -> Self {
        Self {
            backend: BackendType::Local,
            local: LocalModelConfig {
                model_name,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Create an OpenAI configuration.
    pub fn openai(api_key: String) -> Self {
        Self {
            backend: BackendType::OpenAi,
            api: ApiConfig::openai(api_key),
            ..Default::default()
        }
    }

    /// Create an Ollama configuration.
    pub fn ollama() -> Self {
        Self {
            backend: BackendType::Ollama,
            api: ApiConfig::ollama(),
            ..Default::default()
        }
    }

    /// Validate the configuration.
    pub fn validate(&self) -> MlResult<()> {
        match self.backend {
            BackendType::Local => {
                if self.local.model_name.is_empty() {
                    return Err(MlError::invalid_config("Local model name cannot be empty"));
                }
            }
            BackendType::OpenAi | BackendType::Ollama => {
                self.api.validate()?;
            }
        }

        Ok(())
    }

    /// Get the expected embedding dimension.
    /// Returns the dimension override if set, otherwise returns the model default.
    pub fn dimension(&self) -> usize {
        if self.dimension_override > 0 {
            return self.dimension_override;
        }

        match self.backend {
            BackendType::Local => {
                // Default for common models
                match self.local.model_name.as_str() {
                    "sentence-transformers/all-MiniLM-L6-v2" => 384,
                    "sentence-transformers/all-mpnet-base-v2" => 768,
                    "BAAI/bge-small-en-v1.5" => 384,
                    "BAAI/bge-base-en-v1.5" => 768,
                    "BAAI/bge-large-en-v1.5" => 1024,
                    _ => 384, // Default fallback
                }
            }
            BackendType::OpenAi => {
                // OpenAI embedding dimensions
                match self.api.model.as_str() {
                    "text-embedding-3-small" => 1536,
                    "text-embedding-3-large" => 3072,
                    "text-embedding-ada-002" => 1536,
                    _ => 1536,
                }
            }
            BackendType::Ollama => {
                // Ollama embedding dimensions (model-dependent)
                match self.api.model.as_str() {
                    "nomic-embed-text" => 768,
                    "mxbai-embed-large" => 1024,
                    "all-minilm" => 384,
                    _ => 768, // Default for Ollama
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EmbeddingConfig::default();
        assert_eq!(config.backend, BackendType::Local);
        assert_eq!(config.local.model_name, "sentence-transformers/all-MiniLM-L6-v2");
        assert!(config.cache_enabled);
    }

    #[test]
    fn test_openai_config() {
        let config = EmbeddingConfig::openai("sk-test".to_string());
        assert_eq!(config.backend, BackendType::OpenAi);
        assert_eq!(config.api.model, "text-embedding-3-small");
        assert_eq!(config.api.api_key, Some("sk-test".to_string()));
    }

    #[test]
    fn test_ollama_config() {
        let config = EmbeddingConfig::ollama();
        assert_eq!(config.backend, BackendType::Ollama);
        assert_eq!(config.api.endpoint, "http://localhost:11434");
        assert_eq!(config.api.model, "nomic-embed-text");
    }

    #[test]
    fn test_api_config_env_var() {
        std::env::set_var("TEST_API_KEY", "env-key-123");
        let config = ApiConfig {
            api_key: Some("${TEST_API_KEY}".to_string()),
            ..Default::default()
        };

        assert_eq!(config.get_api_key().unwrap(), Some("env-key-123".to_string()));
        std::env::remove_var("TEST_API_KEY");
    }

    #[test]
    fn test_dimension() {
        let config = EmbeddingConfig::default();
        assert_eq!(config.dimension(), 384);

        let mut config = EmbeddingConfig::openai("test".to_string());
        assert_eq!(config.dimension(), 1536);

        config.api.model = "text-embedding-3-large".to_string();
        assert_eq!(config.dimension(), 3072);

        config.dimension_override = 512;
        assert_eq!(config.dimension(), 512);
    }

    #[test]
    fn test_config_validation() {
        let config = EmbeddingConfig::default();
        assert!(config.validate().is_ok());

        let mut config = EmbeddingConfig::default();
        config.local.model_name = String::new();
        assert!(config.validate().is_err());

        let mut config = EmbeddingConfig::openai("test".to_string());
        config.api.endpoint = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = EmbeddingConfig::default();
        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("backend = \"local\""));
        assert!(toml.contains("model_name"));
    }
}
