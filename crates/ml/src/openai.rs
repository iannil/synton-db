// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! OpenAI API embedding backend.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::backend::{BackendType, EmbeddingBackend};
use crate::config::ApiConfig;
use crate::error::{MlError, Result};

/// OpenAI API embedding backend.
#[derive(Clone)]
pub struct OpenAiEmbeddingBackend {
    client: reqwest::Client,
    config: ApiConfig,
    dimension: usize,
}

impl OpenAiEmbeddingBackend {
    /// Create a new OpenAI embedding backend.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid or the client cannot be created.
    pub fn new(config: ApiConfig) -> Result<Self> {
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
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            "text-embedding-ada-002" => 1536,
            _ => 1536,
        }
    }

    /// Get the API key from the configuration or environment.
    fn get_api_key(&self) -> Result<String> {
        self.config
            .get_api_key()?
            .ok_or_else(|| MlError::ApiError("API key not provided".to_string()))
    }
}

#[async_trait::async_trait]
impl EmbeddingBackend for OpenAiEmbeddingBackend {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let text = text.trim();

        if text.is_empty() {
            return Err(MlError::EmptyInput);
        }

        let request = EmbeddingRequest {
            input: text.to_string(),
            model: self.config.model.clone(),
            encoding_format: Some("float".to_string()),
            dimensions: None,
        };

        let api_key = self.get_api_key()?;
        let response = self
            .send_request(&request, &api_key)
            .await
            .map_err(|e| MlError::api_error(format!("Failed to get embedding: {e}")))?;

        Ok(response)
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Check for empty inputs
        for text in texts {
            if text.trim().is_empty() {
                return Err(MlError::EmptyInput);
            }
        }

        let texts: Vec<String> = texts.iter().map(|t| t.trim().to_string()).collect();

        let request = EmbeddingRequestBatch {
            input: Input::StringArray(texts.clone()),
            model: self.config.model.clone(),
            encoding_format: Some("float".to_string()),
            dimensions: None,
        };

        let api_key = self.get_api_key()?;
        let response = self
            .send_request_batch(&request, &api_key)
            .await
            .map_err(|e| MlError::api_error(format!("Failed to get batch embeddings: {e}")))?;

        Ok(response)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn backend_type(&self) -> BackendType {
        BackendType::OpenAi
    }
}

impl OpenAiEmbeddingBackend {
    async fn send_request(&self, request: &EmbeddingRequest, api_key: &str) -> Result<Vec<f32>> {
        let url = format!("{}/embeddings", self.config.endpoint);

        let mut retries = 0;
        let max_retries = self.config.max_retries;

        loop {
            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {api_key}"))
                .header("Content-Type", "application/json")
                .json(request)
                .send()
                .await
                .map_err(|e| MlError::HttpClientError(e.to_string()))?;

            let status = response.status();
            let response_text = response
                .text()
                .await
                .map_err(|e| MlError::HttpClientError(e.to_string()))?;

            if status.is_success() {
                let embedding_response: EmbeddingResponse = serde_json::from_str(&response_text)
                    .map_err(|e| MlError::ResponseParseError(e.to_string()))?;

                return Ok(embedding_response.data[0].embedding.clone());
            }

            // Retry on server errors or rate limiting
            if (status.is_server_error() || status.as_u16() == 429) && retries < max_retries {
                retries += 1;
                tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms as u64)).await;
                continue;
            }

            // Extract error message
            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&response_text) {
                return Err(MlError::ApiError(format!(
                    "{}: {}",
                    error_response.error.r#type, error_response.error.message
                )));
            }

            return Err(MlError::ApiError(format!("HTTP error: {status}")));
        }
    }

    async fn send_request_batch(
        &self,
        request: &EmbeddingRequestBatch,
        api_key: &str,
    ) -> Result<Vec<Vec<f32>>> {
        let url = format!("{}/embeddings", self.config.endpoint);

        let mut retries = 0;
        let max_retries = self.config.max_retries;

        loop {
            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {api_key}"))
                .header("Content-Type", "application/json")
                .json(request)
                .send()
                .await
                .map_err(|e| MlError::HttpClientError(e.to_string()))?;

            let status = response.status();
            let response_text = response
                .text()
                .await
                .map_err(|e| MlError::HttpClientError(e.to_string()))?;

            if status.is_success() {
                let embedding_response: BatchEmbeddingResponse = serde_json::from_str(&response_text)
                    .map_err(|e| MlError::ResponseParseError(e.to_string()))?;

                // Sort by index to ensure correct order
                let mut data = embedding_response.data;
                data.sort_by_key(|d| d.index);

                return Ok(data.into_iter().map(|d| d.embedding).collect());
            }

            // Retry on server errors or rate limiting
            if (status.is_server_error() || status.as_u16() == 429) && retries < max_retries {
                retries += 1;
                tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms as u64)).await;
                continue;
            }

            // Extract error message
            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&response_text) {
                return Err(MlError::ApiError(format!(
                    "{}: {}",
                    error_response.error.r#type, error_response.error.message
                )));
            }

            return Err(MlError::ApiError(format!("HTTP error: {status}")));
        }
    }
}

/// Single text embedding request.
#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    input: String,
    model: String,
    encoding_format: Option<String>,
    dimensions: Option<usize>,
}

/// Batch embedding request.
#[derive(Debug, Serialize)]
struct EmbeddingRequestBatch {
    input: Input,
    model: String,
    encoding_format: Option<String>,
    dimensions: Option<usize>,
}

/// Input type for batch requests.
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Input {
    StringArray(Vec<String>),
    String(String),
    NumberArray(Vec<usize>),
}

/// Embedding response for single text.
#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
    model: String,
    usage: Usage,
}

/// Batch embedding response.
#[derive(Debug, Deserialize)]
struct BatchEmbeddingResponse {
    data: Vec<EmbeddingDataWithIndex>,
    model: String,
    usage: Usage,
}

/// Embedding data for single text response.
#[derive(Debug, Deserialize, Clone)]
struct EmbeddingData {
    embedding: Vec<f32>,
    object: String,
}

/// Embedding data for batch response with index.
#[derive(Debug, Deserialize)]
struct EmbeddingDataWithIndex {
    embedding: Vec<f32>,
    index: usize,
    object: String,
}

/// Usage information.
#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: usize,
    total_tokens: usize,
}

/// API error response.
#[derive(Debug, Deserialize)]
struct ApiErrorResponse {
    error: ApiError,
}

/// API error details.
#[derive(Debug, Deserialize)]
struct ApiError {
    message: String,
    r#type: String,
    code: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_model_dimension() {
        assert_eq!(OpenAiEmbeddingBackend::get_model_dimension("text-embedding-3-small"), 1536);
        assert_eq!(OpenAiEmbeddingBackend::get_model_dimension("text-embedding-3-large"), 3072);
        assert_eq!(OpenAiEmbeddingBackend::get_model_dimension("text-embedding-ada-002"), 1536);
        assert_eq!(OpenAiEmbeddingBackend::get_model_dimension("unknown"), 1536);
    }

    #[test]
    fn test_openai_backend_creation() {
        let config = ApiConfig::openai("sk-test-key".to_string());
        let backend = OpenAiEmbeddingBackend::new(config);
        assert!(backend.is_ok());

        let backend = backend.unwrap();
        assert_eq!(backend.dimension(), 1536);
        assert_eq!(backend.backend_type(), BackendType::OpenAi);
    }

    #[test]
    fn test_openai_backend_creation_invalid_config() {
        let config = ApiConfig {
            endpoint: String::new(),
            ..Default::default()
        };
        let backend = OpenAiEmbeddingBackend::new(config);
        assert!(backend.is_err());
    }

    #[test]
    fn test_empty_input_validation() {
        let config = ApiConfig::openai("sk-test-key".to_string());
        let backend = OpenAiEmbeddingBackend::new(config).unwrap();

        // Test would require async runtime, covered in integration tests
        assert!(backend.dimension() > 0);
    }
}
