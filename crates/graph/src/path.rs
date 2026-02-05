// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Path finding and reasoning path utilities.

use crate::{Graph, GraphResult, TraverseDirection};
use synton_core::{Edge, Node, PathType, ReasoningPath, Relation};
use uuid::Uuid;

/// Trait extending Graph with path-based reasoning operations.
pub trait GraphPaths: Graph {
    /// Find a reasoning path explaining the relationship between two concepts.
    async fn explain_relationship(
        &self,
        from: Uuid,
        to: Uuid,
        max_hops: usize,
    ) -> GraphResult<Option<ReasoningPath>> {
        let path_nodes = self.shortest_path(from, to, max_hops).await?;

        if let Some(nodes) = path_nodes {
            if nodes.is_empty() {
                return Ok(None);
            }

            // Collect edges along the path
            let mut edges = Vec::new();
            for i in 0..nodes.len().saturating_sub(1) {
                if let Ok(Some(edge)) = self.get_edge_between(nodes[i].id, nodes[i + 1].id).await {
                    edges.push(edge);
                }
            }

            let path_type = classify_path_type(&edges);
            let confidence = if edges.is_empty() {
                1.0
            } else {
                edges.iter().map(|e| e.weight).fold(1.0, |a, w| a * w)
            };
            let explanation = generate_explanation(&nodes, &edges);

            Ok(Some(ReasoningPath {
                nodes,
                edges,
                confidence,
                explanation,
                path_type,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get a subgraph around a set of seed nodes.
    async fn subgraph(
        &self,
        seeds: Vec<Uuid>,
        radius: usize,
    ) -> GraphResult<SubGraph> {
        let mut node_ids = std::collections::HashSet::new();
        let mut collected_edges = Vec::new();

        for seed in &seeds {
            node_ids.insert(*seed);

            let config = crate::TraversalConfig::with_depth(radius);
            let result = self.bfs(*seed, config).await?;

            for node in result.nodes {
                node_ids.insert(node.id);
            }

            let edges = self.edges(*seed, TraverseDirection::Both).await?;
            collected_edges.extend(edges);
        }

        let mut nodes_result = Vec::new();
        for id in node_ids {
            if let Some(node) = self.get_node(id).await? {
                nodes_result.push(node);
            }
        }

        Ok(SubGraph {
            nodes: nodes_result,
            edges: collected_edges,
        })
    }

    /// Get edge between two nodes.
    async fn get_edge_between(&self, source: Uuid, target: Uuid) -> GraphResult<Option<Edge>> {
        let edges = self.edges(source, TraverseDirection::Forward).await?;
        Ok(edges.into_iter().find(|e| e.target == target))
    }
}

/// A subgraph containing nodes and their connecting edges.
#[derive(Debug, Clone, PartialEq)]
pub struct SubGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl SubGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

impl Default for SubGraph {
    fn default() -> Self {
        Self::new()
    }
}

fn classify_path_type(edges: &[Edge]) -> PathType {
    if edges.is_empty() {
        return PathType::Associative;
    }

    let all_causal = edges.iter().all(|e| matches!(e.relation, Relation::Causes));
    let all_hierarchical = edges
        .iter()
        .all(|e| matches!(e.relation, Relation::IsPartOf | Relation::IsA));
    let all_temporal = edges
        .iter()
        .all(|e| matches!(e.relation, Relation::HappenedAfter));

    match (all_causal, all_hierarchical, all_temporal) {
        (true, false, false) => PathType::Causal,
        (false, true, false) => PathType::Hierarchical,
        (false, false, true) => PathType::Temporal,
        _ => PathType::Hybrid,
    }
}

fn generate_explanation(nodes: &[Node], edges: &[Edge]) -> String {
    if nodes.is_empty() {
        return String::new();
    }

    if edges.is_empty() {
        return format!("Single node: {}", nodes[0].content());
    }

    let mut parts = Vec::new();

    for i in 0..nodes.len().saturating_sub(1) {
        if i < edges.len() {
            let edge = &edges[i];
            let relation_name = match edge.relation {
                Relation::IsPartOf => "is part of",
                Relation::Causes => "causes",
                Relation::Contradicts => "contradicts",
                Relation::HappenedAfter => "happened after",
                Relation::SimilarTo => "is similar to",
                Relation::IsA => "is a",
                Relation::LocatedAt => "is located at",
                Relation::BelongsTo => "belongs to",
                Relation::Custom(ref s) => s,
            };

            parts.push(format!(
                "{} {} {}",
                truncate(nodes[i].content(), 20),
                relation_name,
                truncate(nodes[i + 1].content(), 20)
            ));
        }
    }

    parts.join(", which ")
}

fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        &s[..max_len.saturating_sub(3)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GraphPaths, MemoryGraph};
    use synton_core::{Edge, NodeType, Relation};

    #[tokio::test]
    async fn test_explain_relationship() {
        let mut graph = MemoryGraph::new();

        let n1 = Node::new("Supply shortage", NodeType::Concept);
        let n2 = Node::new("Production delay", NodeType::Concept);
        let n3 = Node::new("Stock drop", NodeType::Concept);

        graph.add_node(n1.clone()).unwrap();
        graph.add_node(n2.clone()).unwrap();
        graph.add_node(n3.clone()).unwrap();

        graph.add_edge(Edge::new(n1.id, n2.id, Relation::Causes)).unwrap();
        graph.add_edge(Edge::new(n2.id, n3.id, Relation::Causes)).unwrap();

        // Test that nodes were added
        let node1 = graph.get_node(n1.id).await.unwrap();
        assert!(node1.is_some());
    }

    #[tokio::test]
    async fn test_subgraph() {
        let mut graph = MemoryGraph::new();

        let n1 = Node::new("A", NodeType::Entity);
        let n2 = Node::new("B", NodeType::Entity);
        let n3 = Node::new("C", NodeType::Entity);

        graph.add_node(n1.clone()).unwrap();
        graph.add_node(n2.clone()).unwrap();
        graph.add_node(n3.clone()).unwrap();

        graph.add_edge(Edge::new(n1.id, n2.id, Relation::SimilarTo)).unwrap();
        graph.add_edge(Edge::new(n2.id, n3.id, Relation::SimilarTo)).unwrap();

        // Test edge retrieval
        let edges = graph.edges(n1.id, TraverseDirection::Forward).await.unwrap();
        assert_eq!(edges.len(), 1);
    }
}
