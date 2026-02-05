// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License);

use async_trait::async_trait;
use uuid::Uuid;

use crate::{GraphError, GraphResult};
use synton_core::{Edge, Node, Relation};

/// Direction for graph traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TraverseDirection {
    /// Follow outgoing edges only.
    #[default]
    Forward,
    /// Follow incoming edges only.
    Backward,
    /// Follow both incoming and outgoing edges.
    Both,
}

impl TraverseDirection {
    /// Returns true if this direction includes forward traversal.
    pub fn includes_forward(&self) -> bool {
        matches!(self, Self::Forward | Self::Both)
    }

    /// Returns true if this direction includes backward traversal.
    pub fn includes_backward(&self) -> bool {
        matches!(self, Self::Backward | Self::Both)
    }
}

/// Configuration for graph traversal operations.
#[derive(Debug, Clone, PartialEq)]
pub struct TraversalConfig {
    /// Maximum traversal depth (number of hops)
    pub max_depth: usize,

    /// Maximum number of nodes to visit
    pub max_nodes: usize,

    /// Traversal direction
    pub direction: TraverseDirection,

    /// Filter relations to follow (empty = all relations)
    pub relation_filter: Vec<Relation>,

    /// Whether to detect and avoid cycles
    pub avoid_cycles: bool,

    /// Whether to include the start node in results
    pub include_start: bool,
}

impl Default for TraversalConfig {
    fn default() -> Self {
        Self {
            max_depth: 3,
            max_nodes: 100,
            direction: TraverseDirection::Forward,
            relation_filter: Vec::new(),
            avoid_cycles: true,
            include_start: false,
        }
    }
}

impl TraversalConfig {
    pub fn with_depth(max_depth: usize) -> Self {
        Self {
            max_depth,
            ..Default::default()
        }
    }

    pub fn with_max_nodes(mut self, max_nodes: usize) -> Self {
        self.max_nodes = max_nodes;
        self
    }

    pub fn with_direction(mut self, direction: TraverseDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_relation(mut self, relation: Relation) -> Self {
        self.relation_filter.push(relation);
        self
    }

    pub fn with_avoid_cycles(mut self, avoid: bool) -> Self {
        self.avoid_cycles = avoid;
        self
    }

    pub fn with_include_start(mut self, include: bool) -> Self {
        self.include_start = include;
        self
    }
}

/// Result of a graph traversal operation.
#[derive(Debug, Clone, PartialEq)]
pub struct TraversalResult {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub depth: usize,
    pub truncated: bool,
}

impl TraversalResult {
    pub fn new(nodes: Vec<Node>, edges: Vec<Edge>, depth: usize) -> Self {
        Self {
            nodes,
            edges,
            depth,
            truncated: false,
        }
    }

    pub fn with_truncated(mut self, truncated: bool) -> Self {
        self.truncated = truncated;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

/// Abstract graph interface.
#[async_trait]
pub trait Graph: Send + Sync {
    async fn neighbors(&self, id: Uuid, direction: TraverseDirection) -> GraphResult<Vec<Node>>;

    async fn edges(&self, id: Uuid, direction: TraverseDirection) -> GraphResult<Vec<Edge>>;

    async fn bfs(&self, start: Uuid, config: TraversalConfig) -> GraphResult<TraversalResult>;

    async fn dfs(&self, start: Uuid, config: TraversalConfig) -> GraphResult<TraversalResult>;

    async fn shortest_path(&self, from: Uuid, to: Uuid, max_depth: usize) -> GraphResult<Option<Vec<Node>>>;

    async fn node_exists(&self, id: Uuid) -> GraphResult<bool>;

    async fn get_node(&self, id: Uuid) -> GraphResult<Option<Node>>;

    async fn count_nodes(&self) -> GraphResult<usize>;

    async fn count_edges(&self) -> GraphResult<usize>;
}

/// In-memory graph implementation.
pub struct MemoryGraph {
    nodes: std::collections::HashMap<Uuid, Node>,
    edges: std::collections::HashMap<Uuid, Vec<Edge>>,
    incoming: std::collections::HashMap<Uuid, Vec<Edge>>,
}

impl MemoryGraph {
    pub fn new() -> Self {
        Self {
            nodes: Default::default(),
            edges: Default::default(),
            incoming: Default::default(),
        }
    }

    pub fn add_node(&mut self, node: Node) -> GraphResult<()> {
        if self.nodes.contains_key(&node.id) {
            return Err(GraphError::Custom(format!("Node {} already exists", node.id)));
        }
        self.nodes.insert(node.id, node);
        Ok(())
    }

    pub fn add_edge(&mut self, edge: Edge) -> GraphResult<()> {
        self.edges.entry(edge.source).or_default().push(edge.clone());
        self.incoming.entry(edge.target).or_default().push(edge);
        Ok(())
    }

    pub fn from_parts(nodes: Vec<Node>, edges: Vec<Edge>) -> GraphResult<Self> {
        let mut graph = Self::new();

        for node in nodes {
            graph.nodes.insert(node.id, node);
        }

        for edge in edges {
            graph.edges.entry(edge.source).or_default().push(edge.clone());
            graph.incoming.entry(edge.target).or_default().push(edge);
        }

        Ok(graph)
    }
}

impl Default for MemoryGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Graph for MemoryGraph {
    async fn edges(&self, id: Uuid, direction: TraverseDirection) -> GraphResult<Vec<Edge>> {
        let mut result = Vec::new();

        if direction.includes_forward() {
            if let Some(edges) = self.edges.get(&id) {
                result.extend(edges.clone());
            }
        }

        if direction.includes_backward() {
            if let Some(edges) = self.incoming.get(&id) {
                result.extend(edges.clone());
            }
        }

        Ok(result)
    }

    async fn neighbors(&self, id: Uuid, direction: TraverseDirection) -> GraphResult<Vec<Node>> {
        let edge_ids = self.edges(id, direction).await?;
        let mut neighbors = Vec::new();
        for edge in edge_ids {
            let target_id = if direction.includes_forward() {
                edge.target
            } else {
                edge.source
            };
            if let Some(node) = self.nodes.get(&target_id) {
                neighbors.push(node.clone());
            }
        }
        Ok(neighbors)
    }

    async fn bfs(&self, start: Uuid, config: TraversalConfig) -> GraphResult<TraversalResult> {
        if !self.nodes.contains_key(&start) {
            return Err(GraphError::NodeNotFound(start));
        }

        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        let mut result_nodes = Vec::new();
        let mut depth = 0;
        let mut next_depth_nodes = 0usize;

        if config.include_start {
            if let Some(node) = self.nodes.get(&start) {
                result_nodes.push(node.clone());
            }
            visited.insert(start);
        }

        queue.push_back((start, 0usize));

        while let Some((current_id, current_depth)) = queue.pop_front() {
            if current_depth > depth {
                depth = current_depth;
                next_depth_nodes = 0;
            }

            if current_depth >= config.max_depth || result_nodes.len() >= config.max_nodes {
                break;
            }

            let neighbors = self.neighbors(current_id, config.direction).await?;

            for neighbor in neighbors {
                let id = neighbor.id;
                if !visited.contains(&id) {
                    visited.insert(id);
                    result_nodes.push(neighbor);
                    queue.push_back((id, current_depth + 1));
                    next_depth_nodes += 1;
                }
            }
        }

        Ok(TraversalResult {
            nodes: result_nodes,
            edges: Vec::new(),
            depth,
            truncated: false,
        })
    }

    async fn dfs(&self, start: Uuid, config: TraversalConfig) -> GraphResult<TraversalResult> {
        if !self.nodes.contains_key(&start) {
            return Err(GraphError::NodeNotFound(start));
        }

        let mut visited = std::collections::HashSet::new();
        let mut result_nodes = Vec::new();

        if config.include_start {
            if let Some(node) = self.nodes.get(&start) {
                result_nodes.push(node.clone());
            }
            visited.insert(start);
        }

        // Iterative DFS using explicit stack
        // Stack elements: (node_id, depth)
        let mut stack = vec![(start, 0usize)];

        while let Some((current_id, depth)) = stack.pop() {
            if depth >= config.max_depth || result_nodes.len() >= config.max_nodes {
                continue;
            }

            let neighbors = self.neighbors(current_id, config.direction).await?;

            // Push neighbors in reverse order to process them in order
            for neighbor in neighbors.into_iter().rev() {
                let id = neighbor.id;
                if visited.insert(id) {
                    result_nodes.push(neighbor);
                    stack.push((id, depth + 1));
                }
            }
        }

        let calc_depth = result_nodes.len().saturating_sub(1);

        Ok(TraversalResult {
            nodes: result_nodes,
            edges: Vec::new(),
            depth: calc_depth,
            truncated: false,
        })
    }

    async fn shortest_path(&self, from: Uuid, to: Uuid, max_depth: usize) -> GraphResult<Option<Vec<Node>>> {
        if !self.nodes.contains_key(&from) || !self.nodes.contains_key(&to) {
            return Ok(None);
        }

        let mut visited: std::collections::HashMap<Uuid, (Uuid, Option<Node>)> = std::collections::HashMap::new();
        let mut queue = std::collections::VecDeque::new();

        visited.insert(from, (Uuid::nil(), self.nodes.get(&from).cloned()));
        queue.push_back(from);

        while let Some(current) = queue.pop_front() {
            let depth = Self::depth_of(&visited, current);

            if depth >= max_depth {
                continue;
            }

            if current == to {
                let mut path = Vec::new();
                let mut curr = current;
                while curr != from {
                    if let Some((prev, node)) = visited.get(&curr) {
                        if let Some(n) = node {
                            path.push(n.clone());
                        }
                        curr = *prev;
                    } else {
                        break;
                    }
                }
                if let Some(start_node) = self.nodes.get(&from) {
                    path.push(start_node.clone());
                }
                path.reverse();
                return Ok(Some(path));
            }

            if let Some(edges) = self.edges.get(&current) {
                for edge in edges {
                    if !visited.contains_key(&edge.target) {
                        visited.insert(edge.target, (current, self.nodes.get(&edge.target).cloned()));
                        queue.push_back(edge.target);
                    }
                }
            }
        }

        Ok(None)
    }

    async fn node_exists(&self, id: Uuid) -> GraphResult<bool> {
        Ok(self.nodes.contains_key(&id))
    }

    async fn get_node(&self, id: Uuid) -> GraphResult<Option<Node>> {
        Ok(self.nodes.get(&id).cloned())
    }

    async fn count_nodes(&self) -> GraphResult<usize> {
        Ok(self.nodes.len())
    }

    async fn count_edges(&self) -> GraphResult<usize> {
        Ok(self.edges.values().map(|v| v.len()).sum())
    }
}

impl MemoryGraph {
    fn depth_of(
        visited: &std::collections::HashMap<Uuid, (Uuid, Option<Node>)>,
        id: Uuid,
    ) -> usize {
        let mut depth = 0;
        let mut current = id;
        while current != Uuid::nil() {
            if let Some((prev, _)) = visited.get(&current) {
                current = *prev;
                depth += 1;
            } else {
                break;
            }
        }
        depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use synton_core::{NodeType, Source};

    #[tokio::test]
    async fn test_memory_graph_basic() {
        let mut graph = MemoryGraph::new();

        let node1 = Node::new("Node 1", NodeType::Entity);
        let node2 = Node::new("Node 2", NodeType::Entity);

        graph.add_node(node1).unwrap();
        graph.add_node(node2).unwrap();

        assert_eq!(graph.count_nodes().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_shortest_path() {
        let mut graph = MemoryGraph::new();

        let n1 = Node::new("A", NodeType::Entity);
        let n2 = Node::new("B", NodeType::Entity);
        let n3 = Node::new("C", NodeType::Entity);

        graph.add_node(n1.clone()).unwrap();
        graph.add_node(n2.clone()).unwrap();
        graph.add_node(n3.clone()).unwrap();

        graph.add_edge(Edge::new(n1.id, n2.id, Relation::Causes)).unwrap();
        graph.add_edge(Edge::new(n2.id, n3.id, Relation::Causes)).unwrap();

        let path = graph.shortest_path(n1.id, n3.id, 10).await.unwrap();
        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 3);
    }
}
