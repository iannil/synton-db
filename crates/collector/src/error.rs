//! Error types for the trace collector.

use thiserror::Error;

/// Result type for collector operations.
pub type CollectorResult<T> = Result<T, CollectorError>;

/// Errors that can occur in the trace collector.
#[derive(Debug, Error)]
pub enum CollectorError {
    /// Error from RocksDB.
    #[error("RocksDB error: {0}")]
    RocksDb(#[from] rocksdb::Error),

    /// Error from serialization.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Trace not found.
    #[error("Trace not found: {0}")]
    NotFound(String),

    /// Invalid query parameter.
    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
