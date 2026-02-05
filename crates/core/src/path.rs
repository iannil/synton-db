// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{Edge, Node};

/// Type of reasoning path in graph traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathType {
    /// Causal chain (A causes B causes C)
    Causal,

    /// Hierarchical structure (A is part of B is part of C)
    Hierarchical,

    /// Temporal sequence (A happened before B happened before C)
    Temporal,

    /// Associative chain (A is related to B is related to C)
    Associative,

    /// Mixed path with multiple relation types
    Hybrid,
}

impl fmt::Display for PathType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Causal => write!(f, "causal"),
            Self::Hierarchical => write!(f, "hierarchical"),
            Self::Temporal => write!(f, "temporal"),
            Self::Associative => write!(f, "associative"),
            Self::Hybrid => write!(f, "hybrid"),
        }
    }
}

/// A reasoning path resulting from graph traversal.
///
/// Represents a chain of nodes and edges that form a logical
/// reasoning trace, supporting Graph-RAG operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningPath {
    /// Nodes in the path, in order
    pub nodes: Vec<Node>,

    /// Edges connecting the nodes
    pub edges: Vec<Edge>,

    /// Overall confidence score for this path
    pub confidence: f32,

    /// Natural language explanation of the path
    pub explanation: String,

    /// Type of reasoning this path represents
    pub path_type: PathType,
}

impl ReasoningPath {
    /// Create a new reasoning path with default values.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            confidence: 0.0,
            explanation: String::new(),
            path_type: PathType::Associative,
        }
    }

    /// Create a reasoning path from nodes and edges.
    pub fn from_components(nodes: Vec<Node>, edges: Vec<Edge>) -> Self {
        Self {
            nodes,
            edges,
            confidence: 0.0,
            explanation: String::new(),
            path_type: PathType::Hybrid,
        }
    }

    /// Get the starting node of the path.
    #[must_use]
    pub fn start(&self) -> Option<&Node> {
        self.nodes.first()
    }

    /// Get the ending node of the path.
    #[must_use]
    pub fn end(&self) -> Option<&Node> {
        self.nodes.last()
    }

    /// Get the number of nodes in the path.
    #[inline]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the path is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get the number of hops (edges) in the path.
    #[inline]
    pub fn hops(&self) -> usize {
        self.edges.len()
    }

    /// Set the path type.
    pub fn with_path_type(mut self, path_type: PathType) -> Self {
        self.path_type = path_type;
        self
    }

    /// Set the confidence score.
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set the explanation.
    pub fn with_explanation(mut self, explanation: impl Into<String>) -> Self {
        self.explanation = explanation.into();
        self
    }

    /// Calculate the minimum confidence along the path (from edge weights).
    pub fn calculate_min_confidence(&self) -> f32 {
        self.edges
            .iter()
            .map(|e| e.weight)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(1.0)
    }
}

impl Default for ReasoningPath {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ReasoningPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Path ({} nodes, {} hops, confidence={})",
            self.nodes.len(),
            self.hops(),
            self.confidence
        )?;
        if !self.explanation.is_empty() {
            write!(f, ": {}", self.explanation)?;
        }
        Ok(())
    }
}

/// Builder for constructing reasoning paths.
#[derive(Debug, Clone)]
#[allow(dead_code)]  // Reserved for future use
pub struct PathBuilder {
    path: ReasoningPath,
}

impl PathBuilder {
    /// Create a new path builder.
    pub fn new() -> Self {
        Self {
            path: ReasoningPath::new(),
        }
    }

    /// Add a node to the path.
    pub fn add_node(mut self, node: Node) -> Self {
        self.path.nodes.push(node);
        self
    }

    /// Add multiple nodes to the path.
    pub fn add_nodes(mut self, nodes: impl IntoIterator<Item = Node>) -> Self {
        self.path.nodes.extend(nodes);
        self
    }

    /// Add an edge to the path.
    pub fn add_edge(mut self, edge: Edge) -> Self {
        self.path.edges.push(edge);
        self
    }

    /// Add multiple edges to the path.
    pub fn add_edges(mut self, edges: impl IntoIterator<Item = Edge>) -> Self {
        self.path.edges.extend(edges);
        self
    }

    /// Set the confidence score.
    pub fn confidence(mut self, confidence: f32) -> Self {
        self.path.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set the explanation.
    pub fn explanation(mut self, explanation: impl Into<String>) -> Self {
        self.path.explanation = explanation.into();
        self
    }

    /// Set the path type.
    pub fn path_type(mut self, path_type: PathType) -> Self {
        self.path.path_type = path_type;
        self
    }

    /// Build the path, validating nodes and edges match.
    pub fn build(self) -> Result<ReasoningPath, PathBuildError> {
        // Validate that edges count matches nodes - 1
        if !self.path.nodes.is_empty() && self.path.edges.len() != self.path.nodes.len().saturating_sub(1) {
            return Err(PathBuildError::EdgeNodeMismatch {
                nodes: self.path.nodes.len(),
                edges: self.path.edges.len(),
            });
        }

        Ok(self.path)
    }
}

impl Default for PathBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Error when building a path.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]  // Reserved for future use
pub enum PathBuildError {
    /// Edge count doesn't match node count
    EdgeNodeMismatch { nodes: usize, edges: usize },
}

impl fmt::Display for PathBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EdgeNodeMismatch { nodes, edges } => write!(
                f,
                "Path has {} nodes but {} edges (expected {} edges)",
                nodes,
                edges,
                nodes.saturating_sub(1)
            ),
        }
    }
}

impl std::error::Error for PathBuildError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Edge, Node, NodeType, Relation};

    #[test]
    fn test_empty_path() {
        let path = ReasoningPath::new();
        assert!(path.is_empty());
        assert_eq!(path.len(), 0);
        assert_eq!(path.hops(), 0);
        assert!(path.start().is_none());
        assert!(path.end().is_none());
    }

    #[test]
    fn test_path_builder() {
        let node1 = Node::new("A", NodeType::Entity);
        let node2 = Node::new("B", NodeType::Entity);
        let edge = Edge::new(node1.id, node2.id, Relation::Causes);

        let path = PathBuilder::new()
            .add_node(node1.clone())
            .add_node(node2.clone())
            .add_edge(edge)
            .confidence(0.8)
            .explanation("A to B")
            .build()
            .unwrap();

        assert_eq!(path.len(), 2);
        assert_eq!(path.hops(), 1);
        assert_eq!(path.start().unwrap().content(), "A");
        assert_eq!(path.end().unwrap().content(), "B");
    }

    #[test]
    fn test_path_builder_with_edges() {
        let node1 = Node::new("A", NodeType::Entity);
        let node2 = Node::new("B", NodeType::Entity);
        let node3 = Node::new("C", NodeType::Entity);

        let edge1 = Edge::new(node1.id, node2.id, Relation::Causes);
        let edge2 = Edge::new(node2.id, node3.id, Relation::Causes);

        let path = PathBuilder::new()
            .add_nodes(vec![node1, node2, node3])
            .add_edges(vec![edge1, edge2])
            .path_type(PathType::Causal)
            .build()
            .unwrap();

        assert_eq!(path.len(), 3);
        assert_eq!(path.hops(), 2);
        assert_eq!(path.path_type, PathType::Causal);
    }

    #[test]
    fn test_path_builder_validation() {
        let node1 = Node::new("A", NodeType::Entity);
        let node2 = Node::new("B", NodeType::Entity);
        let edge1 = Edge::new(node1.id, node2.id, Relation::Causes);

        // 2 nodes with 2 edges should fail (need 1 edge for 2 nodes)
        let result = PathBuilder::new()
            .add_nodes(vec![node1, node2])
            .add_edges(vec![edge1.clone(), edge1])
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_path_display() {
        let path = ReasoningPath {
            nodes: vec![Node::new("A", NodeType::Entity)],
            edges: vec![],
            confidence: 0.85,
            explanation: "Test path".to_string(),
            path_type: PathType::Associative,
        };

        let s = path.to_string();
        assert!(s.contains("1 nodes"));
        assert!(s.contains("confidence=0.85"));
        assert!(s.contains("Test path"));
    }
}
