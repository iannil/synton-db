// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use std::fmt;

/// Graph-RAG specific errors.
#[derive(Debug, Clone, PartialEq)]
pub enum GraphRagError {
    /// Node not found in vector index
    NodeNotFound(uuid::Uuid),

    /// Vector search failed
    VectorSearchFailed(String),

    /// Graph traversal failed
    GraphTraversalFailed(String),

    /// Invalid configuration
    InvalidConfig(String),

    /// Context too large
    ContextTooLarge { size: usize, max: usize },

    /// Scoring failed
    ScoringFailed(String),

    /// Custom error
    Custom(String),
}

impl fmt::Display for GraphRagError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NodeNotFound(id) => write!(f, "Node not found: {}", id),
            Self::VectorSearchFailed(e) => write!(f, "Vector search failed: {}", e),
            Self::GraphTraversalFailed(e) => write!(f, "Graph traversal failed: {}", e),
            Self::InvalidConfig(e) => write!(f, "Invalid configuration: {}", e),
            Self::ContextTooLarge { size, max } => {
                write!(f, "Context too large: {} > {}", size, max)
            }
            Self::ScoringFailed(e) => write!(f, "Scoring failed: {}", e),
            Self::Custom(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for GraphRagError {}

/// Result type for Graph-RAG operations.
pub type GraphRagResult<T> = Result<T, GraphRagError>;

impl From<synton_core::CoreError> for GraphRagError {
    fn from(e: synton_core::CoreError) -> Self {
        Self::Custom(e.to_string())
    }
}

impl From<synton_graph::GraphError> for GraphRagError {
    fn from(e: synton_graph::GraphError) -> Self {
        Self::GraphTraversalFailed(e.to_string())
    }
}

impl From<synton_vector::VectorError> for GraphRagError {
    fn from(e: synton_vector::VectorError) -> Self {
        Self::VectorSearchFailed(e.to_string())
    }
}
