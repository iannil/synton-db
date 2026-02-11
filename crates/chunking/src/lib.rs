// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Adaptive document chunking for SYNTON-DB.
//!
//! This crate provides semantic-aware document splitting strategies
//! that preserve contextual boundaries and support hierarchical storage.

mod chunk;
mod error;
mod strategy;
mod fixed;
mod semantic;
mod hierarchical;

pub use chunk::{Chunk, ChunkMetadata, ChunkType};
pub use error::{ChunkingError, Result};
pub use strategy::{ChunkingStrategy, ChunkingConfig};

// Re-export strategies
pub use fixed::{FixedChunker, FixedChunkConfig};
pub use semantic::{SemanticChunker, SemanticChunkConfig};
pub use hierarchical::{
    HierarchicalChunker,
    HierarchicalChunkConfig,
    HierarchicalChunks,
    HierarchicalChunking,
};

/// Default chunking configuration.
pub fn default_config() -> ChunkingConfig {
    ChunkingConfig::default()
}
