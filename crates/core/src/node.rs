// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use std::borrow::Cow;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{CoreError, CoreResult, NodeType, Source};

/// Maximum content size in bytes (10MB)
pub const MAX_CONTENT_SIZE: usize = 10 * 1024 * 1024;

/// Decay constant for memory fading (Ebbinghaus curve approximation)
/// Default value approximately corresponds to 20% retention after 24 hours
pub const DEFAULT_DECAY_LAMBDA: f32 = 0.0015;

/// Metadata associated with a node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeMeta {
    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Last access timestamp (for memory decay calculation)
    pub accessed_at: Option<DateTime<Utc>>,

    /// Access score (0.0 - 10.0, used for memory decay)
    pub access_score: f32,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,

    /// Data source
    pub source: Source,

    /// Original document ID (if this is a chunk)
    pub document_id: Option<Uuid>,

    /// Chunk index (if this is a chunk)
    pub chunk_index: Option<usize>,
}

impl NodeMeta {
    /// Create new metadata with current timestamp.
    pub fn new(source: Source) -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            accessed_at: None,
            access_score: 1.0,
            confidence: 1.0,
            source,
            document_id: None,
            chunk_index: None,
        }
    }

    /// Validate the metadata values.
    pub fn validate(&self) -> CoreResult<()> {
        if !(0.0..=1.0).contains(&self.confidence) {
            return Err(CoreError::InvalidConfidence(self.confidence));
        }
        if self.access_score < 0.0 {
            return Err(CoreError::InvalidAccessScore(self.access_score));
        }
        Ok(())
    }

    /// Calculate decayed access score based on time passed.
    ///
    /// Uses exponential decay: score = initial * e^(-λ * time)
    pub fn decayed_score(&self, lambda: f32) -> f32 {
        if let Some(accessed) = self.accessed_at {
            let duration = Utc::now().signed_duration_since(accessed);
            let hours = duration.num_hours().max(0) as f32;
            self.access_score * (-lambda * hours).exp()
        } else {
            self.access_score
        }
    }
}

impl Default for NodeMeta {
    fn default() -> Self {
        Self::new(Source::default())
    }
}

/// A node in the Tensor-Graph.
///
/// Nodes represent semantic units - the atoms of knowledge in SYNTON-DB.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Node {
    /// Unique identifier
    pub id: Uuid,

    /// Content (using Cow for efficient borrowing)
    pub content: Cow<'static, str>,

    /// Optional embedding vector
    pub embedding: Option<Vec<f32>>,

    /// Node metadata
    pub meta: NodeMeta,

    /// Node type
    pub node_type: NodeType,

    /// Additional flexible attributes
    pub attributes: serde_json::Value,
}

impl Node {
    /// Create a new node with default metadata.
    pub fn new(content: impl Into<Cow<'static, str>>, node_type: NodeType) -> Self {
        Self::with_source(content, node_type, Source::default())
    }

    /// Create a new node with a specific source.
    pub fn with_source(
        content: impl Into<Cow<'static, str>>,
        node_type: NodeType,
        source: Source,
    ) -> Self {
        let content = content.into();
        Self {
            id: Uuid::new_v4(),
            content,
            embedding: None,
            meta: NodeMeta::new(source),
            node_type,
            attributes: serde_json::json!({}),
        }
    }

    /// Get the content as a string slice.
    #[inline]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Get the embedding vector if available.
    #[inline]
    pub fn embedding(&self) -> Option<&[f32]> {
        self.embedding.as_deref()
    }

    /// Get the embedding dimension if available.
    #[inline]
    pub fn embedding_dim(&self) -> Option<usize> {
        self.embedding.as_ref().map(|v| v.len())
    }

    /// Check if this node has an embedding.
    #[inline]
    pub fn has_embedding(&self) -> bool {
        self.embedding.is_some()
    }

    /// Validate the node's data.
    pub fn validate(&self) -> CoreResult<()> {
        if self.content.is_empty() {
            return Err(CoreError::EmptyContent);
        }
        if self.content.len() > MAX_CONTENT_SIZE {
            return Err(CoreError::ContentTooLarge {
                size: self.content.len(),
                max: MAX_CONTENT_SIZE,
            });
        }
        self.meta.validate()?;
        Ok(())
    }

    /// Update the access timestamp and optionally boost the score.
    pub fn access(&mut self, boost: f32) {
        self.meta.accessed_at = Some(Utc::now());
        self.meta.access_score = (self.meta.access_score + boost).min(10.0).max(0.0);
    }

    /// Apply memory decay based on time passed since last access.
    pub fn decay(&mut self, lambda: f32) {
        self.meta.access_score = self.meta.decayed_score(lambda);
    }

    /// Reinforce the node (increase access score significantly).
    pub fn reinforce(&mut self, delta: f32) {
        self.meta.access_score = (self.meta.access_score + delta).min(10.0);
        self.meta.accessed_at = Some(Utc::now());
    }

    /// Check if this node should be archived based on access score.
    #[inline]
    pub fn should_archive(&self, threshold: f32) -> bool {
        self.meta.decayed_score(DEFAULT_DECAY_LAMBDA) < threshold
    }

    /// Set the embedding vector.
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }

    /// Set attributes.
    pub fn with_attributes(mut self, attributes: serde_json::Value) -> Self {
        self.attributes = attributes;
        self
    }

    /// Set confidence score.
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.meta.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set the document ID (for chunks).
    pub fn with_document_id(mut self, id: Uuid) -> Self {
        self.meta.document_id = Some(id);
        self
    }

    /// Set the chunk index (for chunks).
    pub fn with_chunk_index(mut self, index: usize) -> Self {
        self.meta.chunk_index = Some(index);
        self
    }
}

/// Builder for constructing nodes.
#[derive(Debug)]
pub struct NodeBuilder {
    node: Node,
}

impl NodeBuilder {
    /// Create a new builder.
    pub fn new(content: impl Into<Cow<'static, str>>, node_type: NodeType) -> Self {
        Self {
            node: Node::new(content, node_type),
        }
    }

    /// Set a specific ID (useful for reconstruction).
    pub fn id(mut self, id: Uuid) -> Self {
        self.node.id = id;
        self
    }

    /// Set the source.
    pub fn source(mut self, source: Source) -> Self {
        self.node.meta.source = source;
        self
    }

    /// Set the embedding.
    pub fn embedding(mut self, embedding: Vec<f32>) -> Self {
        self.node.embedding = Some(embedding);
        self
    }

    /// Set the confidence.
    pub fn confidence(mut self, confidence: f32) -> Self {
        self.node.meta.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set the initial access score.
    pub fn access_score(mut self, score: f32) -> Self {
        self.node.meta.access_score = score.max(0.0);
        self
    }

    /// Set the document ID.
    pub fn document_id(mut self, id: Uuid) -> Self {
        self.node.meta.document_id = Some(id);
        self
    }

    /// Set the chunk index.
    pub fn chunk_index(mut self, index: usize) -> Self {
        self.node.meta.chunk_index = Some(index);
        self
    }

    /// Add an attribute.
    pub fn attribute(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        if let Some(obj) = self.node.attributes.as_object_mut() {
            obj.insert(key.into(), value.into());
        }
        self
    }

    /// Build the node, validating before returning.
    pub fn build(self) -> CoreResult<Node> {
        self.node.validate()?;
        Ok(self.node)
    }

    /// Build without validation.
    pub fn build_unvalidated(self) -> Node {
        self.node
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::new("Hello, world!", NodeType::Entity);
        assert_eq!(node.content(), "Hello, world!");
        assert_eq!(node.node_type, NodeType::Entity);
        assert!(!node.has_embedding());
        assert_eq!(node.meta.confidence, 1.0);
    }

    #[test]
    fn test_node_with_source() {
        let node = Node::with_source("Test", NodeType::Fact, Source::ApiImport);
        assert_eq!(node.meta.source, Source::ApiImport);
    }

    #[test]
    fn test_node_validation() {
        // Valid node
        let node = Node::new("Valid content", NodeType::Entity);
        assert!(node.validate().is_ok());

        // Empty content
        let node = Node::new("", NodeType::Entity);
        assert!(matches!(node.validate(), Err(CoreError::EmptyContent)));

        // Invalid confidence
        let mut node = Node::new("Test", NodeType::Entity);
        node.meta.confidence = 1.5;
        assert!(matches!(node.validate(), Err(CoreError::InvalidConfidence(_))));
    }

    #[test]
    fn test_node_access() {
        let mut node = Node::new("Test", NodeType::Entity);
        let initial_score = node.meta.access_score;

        node.access(0.5);
        assert!(node.meta.access_score > initial_score);
        assert!(node.meta.accessed_at.is_some());
    }

    #[test]
    fn test_node_reinforce() {
        let mut node = Node::new("Test", NodeType::Entity);
        node.reinforce(2.0);
        assert_eq!(node.meta.access_score, 3.0);
    }

    #[test]
    fn test_node_decay() {
        let mut node = Node::new("Test", NodeType::Entity);
        node.meta.access_score = 5.0;
        node.meta.accessed_at = Some(Utc::now() - chrono::Duration::hours(24));

        let before = node.meta.access_score;
        node.decay(DEFAULT_DECAY_LAMBDA);
        assert!(node.meta.access_score < before);
    }

    #[test]
    fn test_node_with_embedding() {
        let node = Node::new("Test", NodeType::Entity).with_embedding(vec![0.1, 0.2, 0.3]);
        assert!(node.has_embedding());
        assert_eq!(node.embedding_dim(), Some(3));
    }

    #[test]
    fn test_node_builder() {
        let node = NodeBuilder::new("Builder test", NodeType::Concept)
            .confidence(0.85)
            .access_score(2.0)
            .attribute("key", "value")
            .build()
            .unwrap();

        assert_eq!(node.content(), "Builder test");
        assert_eq!(node.meta.confidence, 0.85);
        assert_eq!(node.meta.access_score, 2.0);
        assert_eq!(
            node.attributes.get("key"),
            Some(&serde_json::json!("value"))
        );
    }
}
