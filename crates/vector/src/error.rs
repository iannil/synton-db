// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

/// Vector index errors.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum VectorError {
    /// Index not found
    #[error("Vector index '{0}' not found")]
    IndexNotFound(String),

    /// Invalid vector dimension
    #[error("Invalid dimension: expected {expected}, found {found}")]
    InvalidDimension { expected: usize, found: usize },

    /// Invalid ID format
    #[error("Invalid ID format: {0}")]
    InvalidId(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Backend error
    #[error("Backend error: {0}")]
    Backend(String),

    /// Custom error
    #[error("{0}")]
    Custom(String),
}

/// Result type for vector operations.
pub type VectorResult<T> = Result<T, VectorError>;

impl From<serde_json::Error> for VectorError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}
