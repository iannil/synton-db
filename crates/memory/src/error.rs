// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use std::fmt;

/// Memory management errors.
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryError {
    /// Node not found.
    NodeNotFound(uuid::Uuid),

    /// Invalid decay rate.
    InvalidDecayRate(f32),

    /// Invalid access score.
    InvalidAccessScore(f32),

    /// Decay configuration error.
    ConfigError(String),

    /// Pruning failed.
    PruneFailed(String),

    /// Custom error.
    Custom(String),
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NodeNotFound(id) => write!(f, "Node not found: {}", id),
            Self::InvalidDecayRate(rate) => write!(f, "Invalid decay rate: {}", rate),
            Self::InvalidAccessScore(score) => write!(f, "Invalid access score: {}", score),
            Self::ConfigError(e) => write!(f, "Config error: {}", e),
            Self::PruneFailed(e) => write!(f, "Prune failed: {}", e),
            Self::Custom(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for MemoryError {}

/// Result type for memory operations.
pub type MemoryResult<T> = Result<T, MemoryError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        assert!(MemoryError::InvalidAccessScore(11.0).to_string().contains("Invalid"));
    }
}
