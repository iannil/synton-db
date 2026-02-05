// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use std::fmt;

use synton_core::CoreError;
use uuid::Uuid;

/// Storage layer errors.
#[derive(Debug)]
pub enum StorageError {
    /// Underlying RocksDB error
    Rocksdb(String),

    /// Node not found
    NodeNotFound(Uuid),

    /// Edge not found
    EdgeNotFound(String),

    /// Serialization failed
    Serialization(String),

    /// Deserialization failed
    Deserialization(String),

    /// Invalid operation
    InvalidOperation(String),

    /// Database closed
    DatabaseClosed,

    /// I/O error
    Io(std::io::Error),

    /// Core error wrapped
    Core(CoreError),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rocksdb(e) => write!(f, "RocksDB error: {}", e),
            Self::NodeNotFound(id) => write!(f, "Node not found: {}", id),
            Self::EdgeNotFound(e) => write!(f, "Edge not found: {}", e),
            Self::Serialization(e) => write!(f, "Serialization error: {}", e),
            Self::Deserialization(e) => write!(f, "Deserialization error: {}", e),
            Self::InvalidOperation(e) => write!(f, "Invalid operation: {}", e),
            Self::DatabaseClosed => write!(f, "Database is closed"),
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Core(e) => write!(f, "Core error: {}", e),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<std::io::Error> for StorageError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<CoreError> for StorageError {
    fn from(e: CoreError) -> Self {
        Self::Core(e)
    }
}

/// Result type for storage operations.
pub type StorageResult<T> = Result<T, StorageError>;

