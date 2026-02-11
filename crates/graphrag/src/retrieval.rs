// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use crate::scorer::RelevanceScore;
use serde::{Deserialize, Serialize};
use synton_core::{Node, NodeType};
use uuid::Uuid;

/// Retrieval mode for Graph-RAG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RetrievalMode {
    /// Vector-only search (no graph expansion).
    VectorOnly,
    /// Graph-only search (traversal from seed nodes).
    GraphOnly,
    /// Combined: vector search + graph expansion.
    #[default]
    Hybrid,
}

/// Configuration for retrieval operations.
#[derive(Debug, Clone, PartialEq)]
pub struct RetrievalConfig {
    /// Maximum number of nodes to retrieve from vector search.
    pub max_vector_results: usize,

    /// Maximum number of nodes to retrieve from graph traversal.
    pub max_graph_results: usize,

    /// Maximum hop distance for graph traversal.
    pub max_hops: usize,

    /// Minimum relevance score threshold (0.0 - 1.0).
    pub min_relevance: f32,

    /// Whether to deduplicate results.
    pub deduplicate: bool,

    /// Maximum context size in tokens (approximate).
    pub max_context_size: usize,

    /// Retrieval mode.
    pub mode: RetrievalMode,
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            max_vector_results: 10,
            max_graph_results: 20,
            max_hops: 2,
            min_relevance: 0.5,
            deduplicate: true,
            max_context_size: 4096,
            mode: RetrievalMode::Hybrid,
        }
    }
}

impl RetrievalConfig {
    /// Create a new config with vector-only mode.
    pub fn vector_only() -> Self {
        Self {
            mode: RetrievalMode::VectorOnly,
            ..Default::default()
        }
    }

    /// Create a new config with graph-only mode.
    pub fn graph_only() -> Self {
        Self {
            mode: RetrievalMode::GraphOnly,
            ..Default::default()
        }
    }

    /// Create a new config with hybrid mode.
    pub fn hybrid() -> Self {
        Self {
            mode: RetrievalMode::Hybrid,
            ..Default::default()
        }
    }

    /// Set the maximum vector results.
    pub fn with_max_vector_results(mut self, max: usize) -> Self {
        self.max_vector_results = max;
        self
    }

    /// Set the maximum graph results.
    pub fn with_max_graph_results(mut self, max: usize) -> Self {
        self.max_graph_results = max;
        self
    }

    /// Set the maximum hop distance.
    pub fn with_max_hops(mut self, hops: usize) -> Self {
        self.max_hops = hops;
        self
    }

    /// Set the minimum relevance threshold.
    pub fn with_min_relevance(mut self, threshold: f32) -> Self {
        self.min_relevance = threshold.clamp(0.0, 1.0);
        self
    }

    /// Set whether to deduplicate results.
    pub fn with_deduplication(mut self, enabled: bool) -> Self {
        self.deduplicate = enabled;
        self
    }
}

/// A single retrieved node with its metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetrievedNode {
    /// The node itself.
    pub node: Node,

    /// Relevance score.
    pub score: f32,

    /// Hop distance from query.
    pub hop_distance: usize,

    /// Vector similarity score.
    pub vector_similarity: f32,

    /// Whether this is a direct vector match or graph-traversed.
    pub is_direct_match: bool,
}

impl RetrievedNode {
    /// Create a new retrieved node.
    pub fn new(
        node: Node,
        score: f32,
        hop_distance: usize,
        vector_similarity: f32,
        is_direct_match: bool,
    ) -> Self {
        Self {
            node,
            score,
            hop_distance,
            vector_similarity,
            is_direct_match,
        }
    }

    /// Get the node ID.
    pub fn id(&self) -> Uuid {
        self.node.id
    }

    /// Get the node content.
    pub fn content(&self) -> &str {
        self.node.content()
    }
}

/// The result of a retrieval operation.
#[derive(Debug, Clone, PartialEq)]
pub struct RetrievalResult {
    /// Retrieved nodes, sorted by relevance.
    pub nodes: Vec<RetrievedNode>,

    /// All relevance scores.
    pub scores: Vec<RelevanceScore>,

    /// Total context size in characters (approximate).
    pub context_size: usize,

    /// Whether results were truncated due to size limits.
    pub truncated: bool,

    /// Number of direct vector matches.
    pub direct_match_count: usize,

    /// Number of graph-traversed nodes.
    pub graph_traversed_count: usize,
}

impl RetrievalResult {
    /// Create a new retrieval result.
    pub fn new(
        nodes: Vec<RetrievedNode>,
        scores: Vec<RelevanceScore>,
        context_size: usize,
    ) -> Self {
        let direct_match_count = nodes.iter().filter(|n| n.is_direct_match).count();
        let graph_traversed_count = nodes.len() - direct_match_count;

        Self {
            nodes,
            scores,
            context_size,
            truncated: false,
            direct_match_count,
            graph_traversed_count,
        }
    }

    /// Create an empty result.
    pub fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            scores: Vec::new(),
            context_size: 0,
            truncated: false,
            direct_match_count: 0,
            graph_traversed_count: 0,
        }
    }

    /// Check if the result is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get the number of retrieved nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Mark the result as truncated.
    pub fn with_truncated(mut self, truncated: bool) -> Self {
        self.truncated = truncated;
        self
    }
}

/// A context package ready for LLM consumption.
#[derive(Debug, Clone, PartialEq)]
pub struct RetrievedContext {
    /// The retrieval result.
    pub result: RetrievalResult,

    /// Formatted context string.
    pub context: String,

    /// Source metadata (for citations).
    pub sources: Vec<ContextSource>,
}

impl RetrievedContext {
    /// Create a new retrieved context.
    pub fn new(result: RetrievalResult, context: String, sources: Vec<ContextSource>) -> Self {
        Self {
            result,
            context,
            sources,
        }
    }

    /// Get the context string.
    pub fn as_str(&self) -> &str {
        &self.context
    }

    /// Get the context length in characters.
    pub fn len(&self) -> usize {
        self.context.len()
    }

    /// Check if the context is empty.
    pub fn is_empty(&self) -> bool {
        self.context.is_empty()
    }
}

/// Source metadata for context citations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextSource {
    /// Source ID.
    pub id: Uuid,

    /// Source content preview.
    pub preview: String,

    /// Relevance score.
    pub score: f32,

    /// Source type.
    pub source_type: NodeType,
}

impl ContextSource {
    /// Create a new context source.
    pub fn new(id: Uuid, preview: String, score: f32, source_type: NodeType) -> Self {
        Self {
            id,
            preview,
            score,
            source_type,
        }
    }
}

/// Helper function to create a test node.
pub fn test_node(content: &str) -> Node {
    Node::new(content.to_string(), NodeType::Concept)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retrieval_config_default() {
        let config = RetrievalConfig::default();

        assert_eq!(config.max_vector_results, 10);
        assert_eq!(config.max_graph_results, 20);
        assert_eq!(config.max_hops, 2);
        assert_eq!(config.min_relevance, 0.5);
        assert!(config.deduplicate);
        assert_eq!(config.mode, RetrievalMode::Hybrid);
    }

    #[test]
    fn test_retrieval_config_builder() {
        let config = RetrievalConfig::vector_only()
            .with_max_vector_results(20)
            .with_min_relevance(0.7)
            .with_max_hops(3);

        assert_eq!(config.mode, RetrievalMode::VectorOnly);
        assert_eq!(config.max_vector_results, 20);
        assert_eq!(config.min_relevance, 0.7);
        assert_eq!(config.max_hops, 3);
    }

    #[test]
    fn test_retrieved_node() {
        let node = Node::new("Test content", NodeType::Concept);
        let retrieved = RetrievedNode::new(node.clone(), 0.9, 0, 0.9, true);

        assert_eq!(retrieved.node, node);
        assert_eq!(retrieved.score, 0.9);
        assert_eq!(retrieved.hop_distance, 0);
        assert!(retrieved.is_direct_match);
        assert_eq!(retrieved.content(), "Test content");
    }

    #[test]
    fn test_retrieval_result() {
        let node = Node::new("Test", NodeType::Concept);
        let retrieved = RetrievedNode::new(node, 0.9, 0, 0.9, true);
        let scores = vec![RelevanceScore::direct_match(Uuid::new_v4(), 0.9)];

        let result = RetrievalResult::new(vec![retrieved], scores, 100);

        assert_eq!(result.len(), 1);
        assert_eq!(result.direct_match_count, 1);
        assert_eq!(result.graph_traversed_count, 0);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_retrieval_result_empty() {
        let result = RetrievalResult::empty();

        assert!(result.is_empty());
        assert_eq!(result.len(), 0);
    }
}
