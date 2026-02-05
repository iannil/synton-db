// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use std::fmt;

use synton_core::CoreError;

/// Graph-specific errors.
#[derive(Debug, Clone, PartialEq)]
pub enum GraphError {
    /// Node not found
    NodeNotFound(uuid::Uuid),

    /// Edge not found
    EdgeNotFound(String),

    /// Cycle detected in traversal
    CycleDetected(Vec<uuid::Uuid>),

    /// Invalid traversal depth
    InvalidDepth(String),

    /// Storage error
    Storage(String),

    /// Custom error
    Custom(String),
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NodeNotFound(id) => write!(f, "Node not found: {}", id),
            Self::EdgeNotFound(e) => write!(f, "Edge not found: {}", e),
            Self::CycleDetected(path) => write!(f, "Cycle detected: {:?}", path),
            Self::InvalidDepth(e) => write!(f, "Invalid depth: {}", e),
            Self::Storage(e) => write!(f, "Storage error: {}", e),
            Self::Custom(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for GraphError {}

/// Result type for graph operations.
pub type GraphResult<T> = Result<T, GraphError>;

impl From<CoreError> for GraphError {
    fn from(e: CoreError) -> Self {
        Self::Custom(e.to_string())
    }
}
