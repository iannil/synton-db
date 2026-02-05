// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Graph traversal algorithms and utilities.

use crate::{Graph, GraphResult, TraverseDirection};
use synton_core::Node;
use uuid::Uuid;

/// Path finding utility functions.
impl dyn Graph {
    /// Find all paths from start to end within max_depth.
    pub async fn find_all_paths(
        &self,
        start: Uuid,
        end: Uuid,
        max_depth: usize,
    ) -> GraphResult<Vec<Vec<Node>>> {
        let mut all_paths = Vec::new();

        // Iterative DFS with explicit stack
        // Stack elements: (current_node, path_so_far, visited_set)
        let mut stack = vec![(start, vec![start], std::collections::HashSet::from([start]))];

        while let Some((current, path, visited)) = stack.pop() {
            if current == end {
                // Convert path to nodes
                let mut nodes = Vec::new();
                for id in &path {
                    if let Some(node) = self.get_node(*id).await? {
                        nodes.push(node);
                    }
                }
                all_paths.push(nodes);
                continue;
            }

            if path.len() >= max_depth {
                continue;
            }

            // Get outgoing neighbors
            let neighbors = self.neighbors(current, TraverseDirection::Forward).await?;

            for neighbor in neighbors {
                if !visited.contains(&neighbor.id) {
                    let mut new_path = path.clone();
                    new_path.push(neighbor.id);
                    let mut new_visited = visited.clone();
                    new_visited.insert(neighbor.id);
                    stack.push((neighbor.id, new_path, new_visited));
                }
            }
        }

        Ok(all_paths)
    }

    /// Find all simple paths (no repeated nodes) between two nodes.
    pub async fn find_simple_paths(
        &self,
        start: Uuid,
        end: Uuid,
        max_depth: usize,
    ) -> GraphResult<Vec<Vec<Node>>> {
        let mut all_paths = Vec::new();

        // Iterative DFS with explicit stack
        let mut stack = vec![(start, vec![start], std::collections::HashSet::from([start]))];

        while let Some((current, path, visited)) = stack.pop() {
            if current == end {
                // Convert path to nodes
                let mut nodes = Vec::new();
                for id in &path {
                    if let Some(node) = self.get_node(*id).await? {
                        nodes.push(node);
                    }
                }
                all_paths.push(nodes);
                continue;
            }

            if path.len() >= max_depth {
                continue;
            }

            // Get outgoing neighbors
            let neighbors = self.neighbors(current, TraverseDirection::Forward).await?;

            for neighbor in neighbors {
                if !visited.contains(&neighbor.id) {
                    let mut new_path = path.clone();
                    new_path.push(neighbor.id);
                    let mut new_visited = visited.clone();
                    new_visited.insert(neighbor.id);
                    stack.push((neighbor.id, new_path, new_visited));
                }
            }
        }

        Ok(all_paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MemoryGraph, TraversalConfig};
    use synton_core::{Edge, NodeType, Relation};

    #[tokio::test]
    async fn test_graph_extensions() {
        let mut graph = MemoryGraph::new();

        let n1 = Node::new("A", NodeType::Entity);
        let n2 = Node::new("B", NodeType::Entity);
        let n3 = Node::new("C", NodeType::Entity);

        graph.add_node(n1.clone()).unwrap();
        graph.add_node(n2.clone()).unwrap();
        graph.add_node(n3.clone()).unwrap();

        // Create a path: A -> B -> C
        graph.add_edge(Edge::new(n1.id, n2.id, Relation::Causes)).unwrap();
        graph.add_edge(Edge::new(n2.id, n3.id, Relation::Causes)).unwrap();

        // Test that node exists
        let retrieved = graph.get_node(n1.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content(), "A");
    }
}
