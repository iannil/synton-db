// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Error types for the chunking module.

use std::fmt;

/// Chunking error type.
#[derive(Debug, thiserror::Error)]
pub enum ChunkingError {
    /// Empty input provided.
    #[error("Empty input provided")]
    EmptyInput,

    /// Input too short for chunking.
    #[error("Input too short: minimum {min} characters, got {actual}")]
    InputTooShort { min: usize, actual: usize },

    /// Invalid chunk size.
    #[error("Invalid chunk size: {0}")]
    InvalidChunkSize(String),

    /// Invalid overlap size.
    #[error("Invalid overlap size: {0}")]
    InvalidOverlap(String),

    /// Embedding generation failed.
    #[error("Embedding generation failed: {0}")]
    EmbeddingFailed(String),

    /// Sentence boundary detection failed.
    #[error("Sentence boundary detection failed: {0}")]
    BoundaryDetectionFailed(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// IO error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Result type alias for chunking operations.
pub type Result<T> = std::result::Result<T, ChunkingError>;
