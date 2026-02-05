// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Graph-RAG: Graph Retrieval-Augmented Generation
//!
//! This module combines vector similarity search with graph traversal
//! to provide enhanced context retrieval for LLM applications.

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod rag;
mod retrieval;
mod scorer;

pub use error::{GraphRagError, GraphRagResult};
pub use rag::{GraphRag, GraphRagConfig};
pub use retrieval::{RetrievalConfig, RetrievalMode, RetrievalResult, RetrievedContext};
pub use scorer::{RelevanceScore, Scorer};

/// Re-exports commonly used types
pub mod prelude {
    pub use crate::{GraphRag, GraphRagConfig, GraphRagError, GraphRagResult};
    pub use crate::{RetrievalConfig, RetrievalMode, RetrievalResult, RetrievedContext};
    pub use crate::{RelevanceScore, Scorer};
}
