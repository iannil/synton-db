// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Hierarchical summary selection for Graph-RAG.
//!
//! Enables selecting the appropriate granularity of context based on
//! query complexity and available storage levels (document/paragraph/sentence).

use crate::retrieval::RetrievedNode;
use serde::{Deserialize, Serialize};
use synton_core::NodeType;
use uuid::Uuid;

/// Summary level for context selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SummaryLevel {
    /// Document-level summary (highest granularity, most compressed).
    Document = 0,

    /// Paragraph-level chunks.
    Paragraph = 1,

    /// Sentence-level chunks (lowest granularity, most detailed).
    Sentence = 2,
}

impl SummaryLevel {
    /// Get all levels from this level downward.
    pub fn inclusive_levels(&self) -> Vec<SummaryLevel> {
        match self {
            Self::Document => vec![Self::Document, Self::Paragraph, Self::Sentence],
            Self::Paragraph => vec![Self::Paragraph, Self::Sentence],
            Self::Sentence => vec![Self::Sentence],
        }
    }

    /// Get the name of this level.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Paragraph => "paragraph",
            Self::Sentence => "sentence",
        }
    }
}

/// Configuration for summary selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryConfig {
    /// Default summary level to use.
    pub default_level: SummaryLevel,

    /// Maximum context tokens (for compression decisions).
    pub max_tokens: usize,

    /// Whether to automatically adjust level based on context size.
    pub auto_adjust: bool,

    /// Preference for detail vs brevity (0.0 = brief, 1.0 = detailed).
    pub detail_preference: f32,
}

impl Default for SummaryConfig {
    fn default() -> Self {
        Self {
            default_level: SummaryLevel::Paragraph,
            max_tokens: 4096,
            auto_adjust: true,
            detail_preference: 0.5,
        }
    }
}

impl SummaryConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_level(mut self, level: SummaryLevel) -> Self {
        self.default_level = level;
        self
    }

    pub fn with_max_tokens(mut self, tokens: usize) -> Self {
        self.max_tokens = tokens;
        self
    }

    pub fn with_auto_adjust(mut self, enabled: bool) -> Self {
        self.auto_adjust = enabled;
        self
    }

    pub fn with_detail(mut self, preference: f32) -> Self {
        self.detail_preference = preference.clamp(0.0, 1.0);
        self
    }
}

/// Hierarchical selector for choosing appropriate summary level.
#[derive(Debug, Clone)]
pub struct HierarchicalSelector {
    config: SummaryConfig,
}

impl HierarchicalSelector {
    pub fn new(config: SummaryConfig) -> Self {
        Self { config }
    }

    pub fn with_level(level: SummaryLevel) -> Self {
        Self {
            config: SummaryConfig {
                default_level: level,
                ..Default::default()
            },
        }
    }

    /// Select the appropriate summary level based on context requirements.
    pub fn select_level(
        &self,
        nodes: &[RetrievedNode],
        estimated_query_complexity: f32,
    ) -> SummaryLevel {
        // Estimate total context size
        let total_tokens: usize = nodes
            .iter()
            .map(|n| n.node.content().len() / 4) // Rough token estimate
            .sum();

        // Auto-adjust if enabled
        if self.config.auto_adjust {
            if total_tokens > self.config.max_tokens {
                // Too much context, move to higher level (more compressed)
                return match self.config.default_level {
                    SummaryLevel::Sentence => SummaryLevel::Paragraph,
                    SummaryLevel::Paragraph => SummaryLevel::Document,
                    SummaryLevel::Document => SummaryLevel::Document,
                };
            } else if total_tokens < self.config.max_tokens / 4 {
                // Plenty of room, can use more detail
                return match self.config.default_level {
                    SummaryLevel::Sentence => SummaryLevel::Sentence,
                    SummaryLevel::Paragraph => SummaryLevel::Sentence,
                    SummaryLevel::Document => SummaryLevel::Paragraph,
                };
            }
        }

        // Adjust based on query complexity
        if estimated_query_complexity > 0.7 {
            // Complex query - need more detail
            return match self.config.default_level {
                SummaryLevel::Document => SummaryLevel::Paragraph,
                level => level,
            };
        } else if estimated_query_complexity < 0.3 {
            // Simple query - can use less detail
            return match self.config.default_level {
                SummaryLevel::Sentence => SummaryLevel::Paragraph,
                level => level,
            };
        }

        self.config.default_level
    }

    /// Filter nodes to only include those at the specified level.
    pub fn filter_by_level(&self, nodes: &[RetrievedNode], level: SummaryLevel) -> Vec<RetrievedNode> {
        nodes.iter()
            .filter(|n| self.node_matches_level(n, level))
            .cloned()
            .collect()
    }

    /// Expand nodes to include children (move to lower level).
    pub fn expand_to_children(&self, nodes: &[RetrievedNode], _level: SummaryLevel) -> Vec<RetrievedNode> {
        // In production, this would query the graph for child nodes
        // For now, return the original nodes
        nodes.to_vec()
    }

    /// Collapse nodes to include parents (move to higher level).
    pub fn collapse_to_parents(&self, nodes: &[RetrievedNode], _level: SummaryLevel) -> Vec<RetrievedNode> {
        // In production, this would query the graph for parent nodes
        // For now, return the original nodes
        nodes.to_vec()
    }

    /// Check if a node matches the specified summary level.
    fn node_matches_level(&self, node: &RetrievedNode, level: SummaryLevel) -> bool {
        match level {
            SummaryLevel::Document => {
                // Document level only includes entities and concepts
                matches!(node.node.node_type, NodeType::Entity | NodeType::Concept)
            }
            SummaryLevel::Paragraph => {
                // Paragraph level includes everything except raw chunks
                matches!(node.node.node_type,
                    NodeType::Entity | NodeType::Concept | NodeType::Fact)
            }
            SummaryLevel::Sentence => {
                // Everything is valid at sentence level
                true
            }
        }
    }

    /// Select and format context at the appropriate level.
    pub fn select_context(
        &self,
        nodes: &[RetrievedNode],
        query_complexity: f32,
    ) -> Vec<RetrievedNode> {
        let level = self.select_level(nodes, query_complexity);
        self.filter_by_level(nodes, level)
    }
}

impl Default for HierarchicalSelector {
    fn default() -> Self {
        Self::new(SummaryConfig::default())
    }
}

/// Compression strategy for large contexts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionStrategy {
    /// No compression - use all retrieved content.
    None,

    /// Remove redundant information.
    Deduplicate,

    /// Extract key sentences only.
    KeySentences,

    /// Summarize clusters of related information.
    ClusterSummary,

    /// Aggressive compression - keep only top matches.
    TopOnly,
}

/// Compress context to fit within token limits.
pub struct ContextCompressor {
    max_tokens: usize,
    strategy: CompressionStrategy,
}

impl ContextCompressor {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            strategy: CompressionStrategy::Deduplicate,
        }
    }

    pub fn with_strategy(mut self, strategy: CompressionStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Compress nodes to fit within token limits.
    pub fn compress(&self, nodes: Vec<RetrievedNode>) -> Vec<RetrievedNode> {
        let current_tokens: usize = nodes
            .iter()
            .map(|n| n.node.content().len() / 4)
            .sum();

        if current_tokens <= self.max_tokens {
            return nodes;
        }

        match self.strategy {
            CompressionStrategy::None => nodes,
            CompressionStrategy::Deduplicate => self.deduplicate(nodes),
            CompressionStrategy::KeySentences => self.extract_key_sentences(nodes),
            CompressionStrategy::ClusterSummary => self.cluster_summary(nodes),
            CompressionStrategy::TopOnly => self.top_only(nodes),
        }
    }

    /// Remove near-duplicate content.
    fn deduplicate(&self, nodes: Vec<RetrievedNode>) -> Vec<RetrievedNode> {
        let mut result = Vec::new();
        let mut seen_content = std::collections::HashSet::new();

        for node in nodes {
            // Simple content hash for deduplication
            let content_hash = {
                let content = node.node.content();
                if content.len() > 50 {
                    &content[..50]
                } else {
                    content
                }
            };

            if seen_content.insert(content_hash.to_string()) {
                result.push(node);
            }
        }

        result
    }

    /// Extract key sentences from content.
    fn extract_key_sentences(&self, nodes: Vec<RetrievedNode>) -> Vec<RetrievedNode> {
        // In production, this would use NLP to extract important sentences
        // For now, keep the first half of each content
        nodes
            .into_iter()
            .map(|n| {
                let content = n.node.content();
                if content.len() > 200 {
                    // Would truncate here in production
                }
                n
            })
            .collect()
    }

    /// Create summaries of content clusters.
    fn cluster_summary(&self, nodes: Vec<RetrievedNode>) -> Vec<RetrievedNode> {
        // In production, this would cluster similar content and summarize
        // For now, return top nodes by score
        self.top_only(nodes)
    }

    /// Keep only top scoring nodes.
    fn top_only(&self, mut nodes: Vec<RetrievedNode>) -> Vec<RetrievedNode> {
        nodes.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        nodes.truncate(nodes.len() / 2);
        nodes
    }
}

/// A node with hierarchical metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalNode {
    /// Node ID.
    pub id: Uuid,

    /// Content at this level.
    pub content: String,

    /// Summary level.
    pub level: SummaryLevel,

    /// Parent node ID (for moving up the hierarchy).
    pub parent_id: Option<Uuid>,

    /// Child node IDs (for moving down the hierarchy).
    pub child_ids: Vec<Uuid>,

    /// Relevance score.
    pub score: f32,
}

impl HierarchicalNode {
    pub fn new(
        id: Uuid,
        content: String,
        level: SummaryLevel,
        score: f32,
    ) -> Self {
        Self {
            id,
            content,
            level,
            parent_id: None,
            child_ids: Vec::new(),
            score,
        }
    }

    pub fn with_parent(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn with_children(mut self, child_ids: Vec<Uuid>) -> Self {
        self.child_ids = child_ids;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_node(content: &str) -> crate::retrieval::RetrievedNode {
        let node = crate::retrieval::test_node(content);
        RetrievedNode::new(node, 0.8, 0, 0.8, true)
    }

    #[test]
    fn test_summary_level_names() {
        assert_eq!(SummaryLevel::Document.name(), "document");
        assert_eq!(SummaryLevel::Paragraph.name(), "paragraph");
        assert_eq!(SummaryLevel::Sentence.name(), "sentence");
    }

    #[test]
    fn test_summary_level_ordering() {
        assert!(SummaryLevel::Document < SummaryLevel::Paragraph);
        assert!(SummaryLevel::Paragraph < SummaryLevel::Sentence);
    }

    #[test]
    fn test_hierarchical_selector() {
        let selector = HierarchicalSelector::with_level(SummaryLevel::Paragraph);
        let nodes = vec![
            test_node("First paragraph content"),
            test_node("Second paragraph content"),
        ];

        let selected = selector.select_context(&nodes, 0.5);
        assert!(!selected.is_empty());
    }

    #[test]
    fn test_context_compressor() {
        let compressor = ContextCompressor::new(100);
        let nodes = (0..10)
            .map(|i| test_node(&format!("Content {}", i)))
            .collect();

        let compressed = compressor.compress(nodes);
        // Should return some nodes
        assert!(!compressed.is_empty());
    }

    #[test]
    fn test_format_config() {
        let config = SummaryConfig::new()
            .with_level(SummaryLevel::Document)
            .with_max_tokens(2048)
            .with_detail(0.8);

        assert_eq!(config.default_level, SummaryLevel::Document);
        assert_eq!(config.max_tokens, 2048);
        assert_eq!(config.detail_preference, 0.8);
    }

    #[test]
    fn test_hierarchical_node() {
        let id = Uuid::new_v4();
        let parent_id = Uuid::new_v4();
        let child_ids = vec![Uuid::new_v4(), Uuid::new_v4()];

        let node = HierarchicalNode::new(id, "Test content".to_string(), SummaryLevel::Paragraph, 0.9)
            .with_parent(parent_id)
            .with_children(child_ids.clone());

        assert_eq!(node.parent_id, Some(parent_id));
        assert_eq!(node.child_ids, child_ids);
    }
}
