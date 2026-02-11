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
mod formatter;
mod summary;
mod expansion;

pub use error::{GraphRagError, GraphRagResult};
pub use rag::{GraphRag, GraphRagConfig};
pub use retrieval::{RetrievalConfig, RetrievalMode, RetrievalResult, RetrievedContext};
pub use scorer::{RelevanceScore, Scorer};

// Formatter exports
pub use formatter::{
    ContextFormatter, FormatConfig, FormatStyle,
    FlatFormatter, StructuredFormatter, MarkdownFormatter,
    JsonFormatter, CompactFormatter, get_formatter,
};

// Summary exports
pub use summary::{
    SummaryLevel, SummaryConfig, HierarchicalSelector,
    ContextCompressor, CompressionStrategy, HierarchicalNode,
};

// Expansion exports
pub use expansion::{
    ExpansionConfig, ExpansionStrategy, ExpansionResult,
    ExpansionStats, NeighborExpander, RelationExpander,
    ExpansionScorer,
};

/// Re-exports commonly used types
pub mod prelude {
    pub use crate::{GraphRag, GraphRagConfig, GraphRagError, GraphRagResult};
    pub use crate::{RetrievalConfig, RetrievalMode, RetrievalResult, RetrievedContext};
    pub use crate::{RelevanceScore, Scorer};
}
