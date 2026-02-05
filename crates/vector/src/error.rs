// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use std::fmt;

/// Vector index errors.
#[derive(Debug, Clone, PartialEq)]
pub enum VectorError {
    /// Index not found
    IndexNotFound(String),

    /// Invalid vector dimension
    InvalidDimension { expected: usize, found: usize },

    /// Invalid ID format
    InvalidId(String),

    /// Serialization error
    Serialization(String),

    /// Deserialization error
    Deserialization(String),

    /// Backend error
    Backend(String),

    /// Custom error
    Custom(String),
}

impl fmt::Display for VectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IndexNotFound(name) => write!(f, "Vector index '{}' not found", name),
            Self::InvalidDimension { expected, found } => {
                write!(f, "Invalid dimension: expected {}, found {}", expected, found)
            }
            Self::InvalidId(id) => write!(f, "Invalid ID format: {}", id),
            Self::Serialization(e) => write!(f, "Serialization error: {}", e),
            Self::Deserialization(e) => write!(f, "Deserialization error: {}", e),
            Self::Backend(e) => write!(f, "Backend error: {}", e),
            Self::Custom(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for VectorError {}

/// Result type for vector operations.
pub type VectorResult<T> = Result<T, VectorError>;

impl From<serde_json::Error> for VectorError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}
