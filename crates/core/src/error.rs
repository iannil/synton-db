// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use std::fmt;

/// Core error type for SYNTON-DB.
#[derive(Debug, Clone, PartialEq)]
pub enum CoreError {
    /// Invalid node ID format
    InvalidNodeId(String),

    /// Invalid edge format
    InvalidEdgeFormat(String),

    /// Invalid filter condition
    InvalidFilter(String),

    /// Node not found
    NodeNotFound(uuid::Uuid),

    /// Edge not found
    EdgeNotFound(String),

    /// Invalid confidence value (must be 0.0-1.0)
    InvalidConfidence(f32),

    /// Invalid weight value (must be 0.0-1.0)
    InvalidWeight(f32),

    /// Invalid access score
    InvalidAccessScore(f32),

    /// Content too large
    ContentTooLarge {
        /// The actual content size in bytes
        size: usize,
        /// Maximum allowed content size in bytes
        max: usize
    },

    /// Content is empty
    EmptyContent,

    /// Self-referential edge (source == target)
    SelfReferentialEdge,

    /// Serialization error
    SerializationError(String),

    /// Deserialization error
    DeserializationError(String),

    /// Custom error with message
    Custom(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNodeId(id) => write!(f, "Invalid node ID format: {}", id),
            Self::InvalidEdgeFormat(e) => write!(f, "Invalid edge format: {}", e),
            Self::InvalidFilter(e) => write!(f, "Invalid filter condition: {}", e),
            Self::NodeNotFound(id) => write!(f, "Node not found: {}", id),
            Self::EdgeNotFound(e) => write!(f, "Edge not found: {}", e),
            Self::InvalidConfidence(c) => write!(f, "Invalid confidence value {}: must be 0.0-1.0", c),
            Self::InvalidWeight(w) => write!(f, "Invalid weight value {}: must be 0.0-1.0", w),
            Self::InvalidAccessScore(s) => write!(f, "Invalid access score {}: must be >= 0.0", s),
            Self::ContentTooLarge { size, max } => {
                write!(f, "Content too large: {} bytes (max: {} bytes)", size, max)
            }
            Self::EmptyContent => write!(f, "Content cannot be empty"),
            Self::SelfReferentialEdge => write!(f, "Self-referential edges are not allowed"),
            Self::SerializationError(e) => write!(f, "Serialization error: {}", e),
            Self::DeserializationError(e) => write!(f, "Deserialization error: {}", e),
            Self::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CoreError {}

/// Result type for core operations
pub type CoreResult<T> = Result<T, CoreError>;

impl From<serde_json::Error> for CoreError {
    fn from(e: serde_json::Error) -> Self {
        Self::SerializationError(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        assert!(CoreError::EmptyContent.to_string().contains("empty"));
        assert!(CoreError::InvalidConfidence(1.5).to_string().contains("0.0-1.0"));
        assert!(CoreError::SelfReferentialEdge.to_string().to_lowercase().contains("self-referential"));
    }

    #[test]
    fn test_error_equality() {
        assert_eq!(CoreError::EmptyContent, CoreError::EmptyContent);
        assert_eq!(CoreError::InvalidConfidence(0.5), CoreError::InvalidConfidence(0.5));
    }
}
