// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Ollama API embedding backend.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::backend::{BackendType, EmbeddingBackend};
use crate::config::ApiConfig;
use crate::error::{MlError, Result};

/// Ollama API embedding backend.
///
/// Ollama is a local LLM runner that also supports embedding models.
/// Default endpoint: http://localhost:11434
#[derive(Clone)]
pub struct OllamaEmbeddingBackend {
    client: reqwest::Client,
    config: ApiConfig,
    dimension: usize,
}

impl OllamaEmbeddingBackend {
    /// Create a new Ollama embedding backend.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid or the client cannot be created.
    pub fn new(config: ApiConfig) -> Result<Self> {
        // Set default Ollama endpoint if not specified
        let config = if config.endpoint.is_empty() || config.endpoint == ApiConfig::default().endpoint {
            ApiConfig::ollama()
        } else {
            config
        };

        config.validate()?;

        let timeout = Duration::from_secs(config.timeout_secs);
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| MlError::HttpClientError(e.to_string()))?;

        let dimension = Self::get_model_dimension(&config.model);

        Ok(Self {
            client,
            config,
            dimension,
        })
    }

    /// Get the expected embedding dimension for a model.
    fn get_model_dimension(model: &str) -> usize {
        match model {
            "nomic-embed-text" | "nomic-embed-text-v1.5" => 768,
            "mxbai-embed-large" => 1024,
            "all-minilm" | "all-minilm:l33-v2" => 384,
            "llama3" => 4096, // Not an embedding model, but has embeddings
            "mistral" => 4096,
            "gemma2" => 4096,
            _ => 768, // Default fallback
        }
    }

    /// Check if Ollama is available.
    pub async fn check_available(endpoint: &str) -> bool {
        let url = format!("{}/api/tags", endpoint);

        let client = match reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
        {
            Ok(c) => c,
            Err(_) => return false,
        };

        let response = client.get(&url).send().await;
        response.is_ok() && response.unwrap().status().is_success()
    }
}

#[async_trait::async_trait]
impl EmbeddingBackend for OllamaEmbeddingBackend {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let text = text.trim();

        if text.is_empty() {
            return Err(MlError::EmptyInput);
        }

        let request = OllamaEmbedRequest {
            model: self.config.model.clone(),
            prompt: text.to_string(),
        };

        let response = self
            .send_request(&request)
            .await
            .map_err(|e| MlError::api_error(format!("Failed to get embedding: {e}")))?;

        Ok(response)
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Ollama doesn't natively support batch embeddings
        // So we make parallel requests
        let mut results = Vec::with_capacity(texts.len());

        let mut tasks = Vec::new();
        for text in texts {
            let backend = self.clone();
            let text = text.clone();
            tasks.push(tokio::spawn(async move {
                backend.embed(&text).await
            }));
        }

        for task in tasks {
            let result = task
                .await
                .map_err(|e| MlError::InferenceFailed(e.to_string()))?;
            results.push(result?);
        }

        Ok(results)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn backend_type(&self) -> BackendType {
        BackendType::Ollama
    }
}

impl OllamaEmbeddingBackend {
    async fn send_request(&self, request: &OllamaEmbedRequest) -> Result<Vec<f32>> {
        let url = format!("{}/api/embed", self.config.endpoint);

        let mut retries = 0;
        let max_retries = self.config.max_retries;

        loop {
            let response = self
                .client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(request)
                .send()
                .await
                .map_err(|e| MlError::HttpClientError(e.to_string()))?;

            let status = response.status();

            if status.is_success() {
                let embedding_response: OllamaEmbedResponse = response
                    .json()
                    .await
                    .map_err(|e| MlError::ResponseParseError(e.to_string()))?;

                return Ok(embedding_response.embedding);
            }

            // Retry on server errors
            if status.is_server_error() && retries < max_retries {
                retries += 1;
                tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms as u64)).await;
                continue;
            }

            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            return Err(MlError::ApiError(format!("HTTP error: {status} - {error_text}")));
        }
    }
}

/// Ollama embedding request.
#[derive(Debug, Serialize)]
struct OllamaEmbedRequest {
    model: String,
    prompt: String,
}

/// Ollama embedding response.
#[derive(Debug, Deserialize)]
struct OllamaEmbedResponse {
    embedding: Vec<f32>,
}

/// Ollama models list response.
#[derive(Debug, Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModel>,
}

/// Ollama model information.
#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
    modified_at: String,
    size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_model_dimension() {
        assert_eq!(OllamaEmbeddingBackend::get_model_dimension("nomic-embed-text"), 768);
        assert_eq!(OllamaEmbeddingBackend::get_model_dimension("mxbai-embed-large"), 1024);
        assert_eq!(OllamaEmbeddingBackend::get_model_dimension("all-minilm"), 384);
        assert_eq!(OllamaEmbeddingBackend::get_model_dimension("unknown"), 768);
    }

    #[test]
    fn test_ollama_backend_creation() {
        let config = ApiConfig::ollama();
        let backend = OllamaEmbeddingBackend::new(config);
        assert!(backend.is_ok());

        let backend = backend.unwrap();
        assert_eq!(backend.dimension(), 768); // nomic-embed-text default
        assert_eq!(backend.backend_type(), BackendType::Ollama);
    }

    #[test]
    fn test_ollama_backend_with_custom_model() {
        let config = ApiConfig {
            endpoint: "http://localhost:11434".to_string(),
            api_key: None,
            model: "mxbai-embed-large".to_string(),
            ..Default::default()
        };

        let backend = OllamaEmbeddingBackend::new(config);
        assert!(backend.is_ok());

        let backend = backend.unwrap();
        assert_eq!(backend.dimension(), 1024);
    }

    #[test]
    fn test_ollama_backend_with_custom_endpoint() {
        let config = ApiConfig {
            endpoint: "http://192.168.1.100:11434".to_string(),
            api_key: None,
            model: "nomic-embed-text".to_string(),
            ..Default::default()
        };

        let backend = OllamaEmbeddingBackend::new(config);
        assert!(backend.is_ok());
    }

    #[tokio::test]
    async fn test_check_available_ollama_not_running() {
        // This will fail if Ollama is not running, which is expected in CI
        let available = OllamaEmbeddingBackend::check_available("http://localhost:11434").await;
        // We don't assert the result since it depends on whether Ollama is running
        // Just ensure it doesn't panic
        let _ = available;
    }
}
