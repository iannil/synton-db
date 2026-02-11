// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Error types for the ML module.


/// ML module error type.
#[derive(Debug, thiserror::Error)]
pub enum MlError {
    /// Failed to load the model.
    #[error("Failed to load model: {0}")]
    ModelLoadFailed(String),

    /// Failed during inference.
    #[error("Inference failed: {0}")]
    InferenceFailed(String),

    /// API request error.
    #[error("API error: {0}")]
    ApiError(String),

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Tokenization error.
    #[error("Tokenization failed: {0}")]
    TokenizationError(String),

    /// Model download error.
    #[error("Model download failed: {0}")]
    DownloadError(String),

    /// HTTP client error.
    #[error("HTTP client error: {0}")]
    HttpClientError(String),

    /// Response parsing error.
    #[error("Failed to parse response: {0}")]
    ResponseParseError(String),

    /// Empty input provided.
    #[error("Empty input provided")]
    EmptyInput,

    /// Input too long.
    #[error("Input exceeds maximum length of {max} tokens (got {actual} tokens)")]
    InputTooLong { max: usize, actual: usize },

    /// Embedding generation failed.
    #[error("Embedding generation failed: {0}")]
    EmbeddingFailed(String),
}

impl MlError {
    /// Create a model load failed error.
    pub fn model_load_failed(msg: impl Into<String>) -> Self {
        Self::ModelLoadFailed(msg.into())
    }

    /// Create an inference failed error.
    pub fn inference_failed(msg: impl Into<String>) -> Self {
        Self::InferenceFailed(msg.into())
    }

    /// Create an API error.
    pub fn api_error(msg: impl Into<String>) -> Self {
        Self::ApiError(msg.into())
    }

    /// Create an invalid config error.
    pub fn invalid_config(msg: impl Into<String>) -> Self {
        Self::InvalidConfig(msg.into())
    }

    /// Create a tokenization error.
    pub fn tokenization_error(msg: impl Into<String>) -> Self {
        Self::TokenizationError(msg.into())
    }

    /// Create a download error.
    pub fn download_error(msg: impl Into<String>) -> Self {
        Self::DownloadError(msg.into())
    }
}

/// Result type alias for ML operations.
pub type Result<T> = std::result::Result<T, MlError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = MlError::ModelLoadFailed("model not found".to_string());
        assert_eq!(err.to_string(), "Failed to load model: model not found");
    }

    #[test]
    fn test_error_constructors() {
        let err = MlError::model_load_failed("test");
        assert!(matches!(err, MlError::ModelLoadFailed(_)));

        let err = MlError::inference_failed("test");
        assert!(matches!(err, MlError::InferenceFailed(_)));

        let err = MlError::api_error("test");
        assert!(matches!(err, MlError::ApiError(_)));

        let err = MlError::invalid_config("test");
        assert!(matches!(err, MlError::InvalidConfig(_)));
    }
}
