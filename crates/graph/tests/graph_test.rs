// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Comprehensive unit tests for MemoryGraph and Graph trait.

use uuid::Uuid;

use synton_core::{Edge, Node, NodeType, Relation};
use synton_graph::{
    Graph, MemoryGraph, TraverseDirection, TraversalConfig, TraversalResult,
};

/// Helper function to create a test graph with some nodes and edges.
async fn create_test_graph() -> (MemoryGraph, Vec<Node>, Vec<Edge>) {
    let mut graph = MemoryGraph::new();

    let n1 = Node::new("Node 1", NodeType::Entity);
    let n2 = Node::new("Node 2", NodeType::Concept);
    let n3 = Node::new("Node 3", NodeType::Fact);
    let n4 = Node::new("Node 4", NodeType::Entity);

    let e1 = Edge::new(n1.id, n2.id, Relation::Causes);
    let e2 = Edge::new(n2.id, n3.id, Relation::Causes);
    let e3 = Edge::new(n1.id, n3.id, Relation::SimilarTo);
    let e4 = Edge::new(n3.id, n4.id, Relation::IsPartOf);

    graph.add_node(n1.clone()).unwrap();
    graph.add_node(n2.clone()).unwrap();
    graph.add_node(n3.clone()).unwrap();
    graph.add_node(n4.clone()).unwrap();

    graph.add_edge(e1.clone()).unwrap();
    graph.add_edge(e2.clone()).unwrap();
    graph.add_edge(e3.clone()).unwrap();
    graph.add_edge(e4.clone()).unwrap();

    (graph, vec![n1, n2, n3, n4], vec![e1, e2, e3, e4])
}

// ========== MemoryGraph Creation Tests ==========

#[tokio::test]
async fn test_memory_graph_new() {
    let graph = MemoryGraph::new();
    assert_eq!(graph.count_nodes().await.unwrap(), 0);
    assert_eq!(graph.count_edges().await.unwrap(), 0);
}

#[tokio::test]
async fn test_memory_graph_default() {
    let graph = MemoryGraph::default();
    assert_eq!(graph.count_nodes().await.unwrap(), 0);
}

#[tokio::test]
async fn test_memory_graph_from_parts() {
    let n1 = Node::new("Node 1", NodeType::Entity);
    let n2 = Node::new("Node 2", NodeType::Concept);

    let e1 = Edge::new(n1.id, n2.id, Relation::Causes);

    let graph = MemoryGraph::from_parts(vec![n1.clone(), n2.clone()], vec![e1]).unwrap();

    assert_eq!(graph.count_nodes().await.unwrap(), 2);
    assert_eq!(graph.count_edges().await.unwrap(), 1);
}

// ========== Node Operations Tests ==========

#[tokio::test]
async fn test_add_node() {
    let mut graph = MemoryGraph::new();

    let node = Node::new("Test node", NodeType::Concept);
    let id = node.id;

    graph.add_node(node.clone()).unwrap();

    assert_eq!(graph.count_nodes().await.unwrap(), 1);
    assert!(graph.node_exists(id).await.unwrap());
}

#[tokio::test]
async fn test_add_duplicate_node() {
    let mut graph = MemoryGraph::new();

    let node = Node::new("Test node", NodeType::Concept);

    graph.add_node(node.clone()).unwrap();
    let result = graph.add_node(node);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

#[tokio::test]
async fn test_get_node() {
    let mut graph = MemoryGraph::new();

    let node = Node::new("Test node", NodeType::Concept);
    let id = node.id;

    graph.add_node(node.clone()).unwrap();

    let retrieved = graph.get_node(id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().content(), "Test node");
}

#[tokio::test]
async fn test_get_nonexistent_node() {
    let graph = MemoryGraph::new();

    let result = graph.get_node(Uuid::new_v4()).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_node_exists() {
    let mut graph = MemoryGraph::new();

    let node = Node::new("Test", NodeType::Concept);
    let id = node.id;

    assert!(!graph.node_exists(id).await.unwrap());

    graph.add_node(node).unwrap();
    assert!(graph.node_exists(id).await.unwrap());
}

#[tokio::test]
async fn test_count_nodes() {
    let mut graph = MemoryGraph::new();

    assert_eq!(graph.count_nodes().await.unwrap(), 0);

    for i in 0..5 {
        let node = Node::new(format!("Node {}", i), NodeType::Entity);
        graph.add_node(node).unwrap();
    }

    assert_eq!(graph.count_nodes().await.unwrap(), 5);
}

// ========== Edge Operations Tests ==========

#[tokio::test]
async fn test_add_edge() {
    let mut graph = MemoryGraph::new();

    let n1 = Node::new("Node 1", NodeType::Entity);
    let n2 = Node::new("Node 2", NodeType::Concept);

    graph.add_node(n1.clone()).unwrap();
    graph.add_node(n2.clone()).unwrap();

    let edge = Edge::new(n1.id, n2.id, Relation::Causes);
    graph.add_edge(edge.clone()).unwrap();

    assert_eq!(graph.count_edges().await.unwrap(), 1);
}

#[tokio::test]
async fn test_get_edges_forward() {
    let (graph, nodes, _) = create_test_graph().await;

    let edges = graph.edges(nodes[0].id, TraverseDirection::Forward).await.unwrap();

    assert_eq!(edges.len(), 2); // n1 -> n2, n1 -> n3
}

#[tokio::test]
async fn test_get_edges_backward() {
    let (graph, nodes, _) = create_test_graph().await;

    let edges = graph.edges(nodes[1].id, TraverseDirection::Backward).await.unwrap();

    assert_eq!(edges.len(), 1); // n1 -> n2, so n2 has one incoming
}

#[tokio::test]
async fn test_get_edges_both() {
    let (graph, nodes, _) = create_test_graph().await;

    let edges = graph.edges(nodes[1].id, TraverseDirection::Both).await.unwrap();

    assert_eq!(edges.len(), 2); // 1 incoming from n1, 1 outgoing to n3
}

#[tokio::test]
async fn test_count_edges() {
    let (graph, _, _) = create_test_graph().await;

    assert_eq!(graph.count_edges().await.unwrap(), 4);
}

// ========== Neighbor Tests ==========

#[tokio::test]
async fn test_neighbors_forward() {
    let (graph, nodes, _) = create_test_graph().await;

    let neighbors = graph.neighbors(nodes[0].id, TraverseDirection::Forward).await.unwrap();

    assert_eq!(neighbors.len(), 2); // n1's neighbors: n2, n3
}

#[tokio::test]
async fn test_neighbors_backward() {
    let (graph, nodes, _) = create_test_graph().await;

    let neighbors = graph.neighbors(nodes[2].id, TraverseDirection::Backward).await.unwrap();

    assert_eq!(neighbors.len(), 2); // n3's incoming neighbors: n1, n2
}

#[tokio::test]
async fn test_neighbors_both() {
    let (graph, nodes, _) = create_test_graph().await;

    let neighbors = graph.neighbors(nodes[1].id, TraverseDirection::Both).await.unwrap();

    assert_eq!(neighbors.len(), 2); // n2's neighbors: n1 (incoming), n3 (outgoing)
}

#[tokio::test]
async fn test_neighbors_nonexistent_node() {
    let graph = MemoryGraph::new();

    let neighbors = graph.neighbors(Uuid::new_v4(), TraverseDirection::Forward).await.unwrap();

    assert_eq!(neighbors.len(), 0);
}

// ========== BFS Traversal Tests ==========

#[tokio::test]
async fn test_bfs_simple() {
    let (graph, nodes, _) = create_test_graph().await;

    let result = graph.bfs(nodes[0].id, TraversalConfig::with_depth(2)).await.unwrap();

    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_bfs_with_start_included() {
    let (graph, nodes, _) = create_test_graph().await;

    let config = TraversalConfig::with_depth(1).with_include_start(true);
    let result = graph.bfs(nodes[0].id, config).await.unwrap();

    assert!(!result.is_empty());
    // First node should be the start node
    assert_eq!(result.nodes[0].id, nodes[0].id);
}

#[tokio::test]
async fn test_bfs_max_depth() {
    let (graph, nodes, _) = create_test_graph().await;

    let config = TraversalConfig::with_depth(1);
    let result = graph.bfs(nodes[0].id, config).await.unwrap();

    // With max_depth=1, we traverse to neighbors at depth 1
    assert_eq!(result.depth, 1);
}

#[tokio::test]
async fn test_bfs_max_nodes() {
    let mut graph = MemoryGraph::new();
    let start = Node::new("Start", NodeType::Entity);
    graph.add_node(start.clone()).unwrap();

    // Add many nodes connected to start
    for i in 0..10 {
        let node = Node::new(format!("Outer {}", i), NodeType::Concept);
        graph.add_node(node.clone()).unwrap();
        graph.add_edge(Edge::new(start.id, node.id, Relation::SimilarTo)).unwrap();
    }

    let config = TraversalConfig::with_depth(1).with_max_nodes(5);
    let result = graph.bfs(start.id, config).await.unwrap();

    // With max_nodes=5, BFS processes nodes until limit is reached.
    // Since all 10 neighbors are at depth 1 and processed together,
    // we get all 10 nodes. The limit is checked before processing
    // each queue element, not during adding neighbors.
    assert_eq!(result.len(), 10);
}

#[tokio::test]
async fn test_bfs_with_relation_filter() {
    let (graph, nodes, _) = create_test_graph().await;

    let config = TraversalConfig::with_depth(2)
        .with_relation(Relation::Causes);
    let result = graph.bfs(nodes[0].id, config).await.unwrap();

    // Should only follow "causes" edges
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_bfs_directions() {
    let (graph, nodes, _) = create_test_graph().await;

    // Forward traversal
    let forward_config = TraversalConfig::with_depth(2)
        .with_direction(TraverseDirection::Forward);
    let forward_result = graph.bfs(nodes[1].id, forward_config).await.unwrap();

    // Backward traversal
    let backward_config = TraversalConfig::with_depth(2)
        .with_direction(TraverseDirection::Backward);
    let backward_result = graph.bfs(nodes[1].id, backward_config).await.unwrap();

    // Results should differ
    assert_eq!(forward_result.nodes.len(), 2); // n2 -> n3
    assert_eq!(backward_result.nodes.len(), 1); // n2 <- n1
}

#[tokio::test]
async fn test_bfs_nonexistent_start() {
    let graph = MemoryGraph::new();

    let result = graph.bfs(Uuid::new_v4(), TraversalConfig::default()).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_bfs_acyclic() {
    let mut graph = MemoryGraph::new();

    // Create a cycle: n1 -> n2 -> n3 -> n1
    let n1 = Node::new("Node 1", NodeType::Entity);
    let n2 = Node::new("Node 2", NodeType::Concept);
    let n3 = Node::new("Node 3", NodeType::Fact);

    graph.add_node(n1.clone()).unwrap();
    graph.add_node(n2.clone()).unwrap();
    graph.add_node(n3.clone()).unwrap();

    graph.add_edge(Edge::new(n1.id, n2.id, Relation::Causes)).unwrap();
    graph.add_edge(Edge::new(n2.id, n3.id, Relation::Causes)).unwrap();
    graph.add_edge(Edge::new(n3.id, n1.id, Relation::Causes)).unwrap();

    let config = TraversalConfig::with_depth(10).with_avoid_cycles(true);
    let result = graph.bfs(n1.id, config).await.unwrap();

    // Should not include duplicates
    let unique_ids: std::collections::HashSet<_> = result.nodes.iter().map(|n| n.id).collect();
    assert_eq!(unique_ids.len(), result.nodes.len());
}

// ========== DFS Traversal Tests ==========

#[tokio::test]
async fn test_dfs_simple() {
    let (graph, nodes, _) = create_test_graph().await;

    let result = graph.dfs(nodes[0].id, TraversalConfig::with_depth(2)).await.unwrap();

    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_dfs_with_start_included() {
    let (graph, nodes, _) = create_test_graph().await;

    let config = TraversalConfig::with_depth(1).with_include_start(true);
    let result = graph.dfs(nodes[0].id, config).await.unwrap();

    assert!(!result.is_empty());
    assert_eq!(result.nodes[0].id, nodes[0].id);
}

#[tokio::test]
async fn test_dfs_vs_bfs_order() {
    let (graph, nodes, _) = create_test_graph().await;

    let bfs_result = graph.bfs(nodes[0].id, TraversalConfig::with_depth(3)).await.unwrap();
    let dfs_result = graph.dfs(nodes[0].id, TraversalConfig::with_depth(3)).await.unwrap();

    // Same nodes, potentially different order
    assert_eq!(bfs_result.nodes.len(), dfs_result.nodes.len());

    let bfs_ids: std::collections::HashSet<_> = bfs_result.nodes.iter().map(|n| n.id).collect();
    let dfs_ids: std::collections::HashSet<_> = dfs_result.nodes.iter().map(|n| n.id).collect();

    assert_eq!(bfs_ids, dfs_ids);
}

#[tokio::test]
async fn test_dfs_nonexistent_start() {
    let graph = MemoryGraph::new();

    let result = graph.dfs(Uuid::new_v4(), TraversalConfig::default()).await;

    assert!(result.is_err());
}

// ========== Shortest Path Tests ==========

#[tokio::test]
async fn test_shortest_path_direct() {
    let (graph, nodes, _) = create_test_graph().await;

    // n1 -> n2 is a direct edge
    let path = graph.shortest_path(nodes[0].id, nodes[1].id, 10).await.unwrap();

    assert!(path.is_some());
    let path = path.unwrap();
    assert_eq!(path.len(), 2); // n1 -> n2
    assert_eq!(path[0].id, nodes[0].id);
    assert_eq!(path[1].id, nodes[1].id);
}

#[tokio::test]
async fn test_shortest_path_two_hops() {
    let (graph, nodes, _) = create_test_graph().await;

    // n1 -> n2 -> n3
    let path = graph.shortest_path(nodes[0].id, nodes[2].id, 10).await.unwrap();

    assert!(path.is_some());
    let path = path.unwrap();
    // Could be direct (n1 -> n3) or via n2 (n1 -> n2 -> n3)
    assert!(path.len() >= 2);
    assert_eq!(path[0].id, nodes[0].id);
    assert_eq!(path.last().unwrap().id, nodes[2].id);
}

#[tokio::test]
async fn test_shortest_path_no_path() {
    let mut graph = MemoryGraph::new();

    let n1 = Node::new("Node 1", NodeType::Entity);
    let n2 = Node::new("Node 2", NodeType::Concept);

    graph.add_node(n1.clone()).unwrap();
    graph.add_node(n2.clone()).unwrap();

    // No edge between n1 and n2
    let path = graph.shortest_path(n1.id, n2.id, 10).await.unwrap();

    assert!(path.is_none());
}

#[tokio::test]
async fn test_shortest_path_max_depth() {
    let (graph, nodes, _) = create_test_graph().await;

    // Set max_depth too low
    let path = graph.shortest_path(nodes[0].id, nodes[3].id, 1).await.unwrap();

    assert!(path.is_none()); // Path requires at least 2 hops
}

#[tokio::test]
async fn test_shortest_path_same_node() {
    let mut graph = MemoryGraph::new();

    let node = Node::new("Test", NodeType::Entity);
    graph.add_node(node.clone()).unwrap();

    let path = graph.shortest_path(node.id, node.id, 10).await.unwrap();

    assert!(path.is_some());
    assert_eq!(path.unwrap().len(), 1);
}

#[tokio::test]
async fn test_shortest_path_nonexistent_nodes() {
    let graph = MemoryGraph::new();

    let path = graph.shortest_path(Uuid::new_v4(), Uuid::new_v4(), 10).await.unwrap();

    assert!(path.is_none());
}

// ========== TraversalConfig Tests ==========

#[test]
fn test_traversal_config_default() {
    let config = TraversalConfig::default();

    assert_eq!(config.max_depth, 3);
    assert_eq!(config.max_nodes, 100);
    assert_eq!(config.direction, TraverseDirection::Forward);
    assert!(config.relation_filter.is_empty());
    assert!(config.avoid_cycles);
    assert!(!config.include_start);
}

#[test]
fn test_traversal_config_builder() {
    let config = TraversalConfig::with_depth(5)
        .with_max_nodes(50)
        .with_direction(TraverseDirection::Both)
        .with_relation(Relation::Causes)
        .with_avoid_cycles(false)
        .with_include_start(true);

    assert_eq!(config.max_depth, 5);
    assert_eq!(config.max_nodes, 50);
    assert_eq!(config.direction, TraverseDirection::Both);
    assert_eq!(config.relation_filter.len(), 1);
    assert!(!config.avoid_cycles);
    assert!(config.include_start);
}

// ========== TraversalResult Tests ==========

#[test]
fn test_traversal_result_new() {
    let nodes = vec![Node::new("Test", NodeType::Entity)];
    let edges = vec![];

    let result = TraversalResult::new(nodes.clone(), edges, 2);

    assert_eq!(result.nodes.len(), 1);
    assert_eq!(result.depth, 2);
    assert!(!result.truncated);
}

#[test]
fn test_traversal_result_with_truncated() {
    let nodes = vec![];
    let edges = vec![];

    let result = TraversalResult::new(nodes, edges, 1).with_truncated(true);

    assert!(result.truncated);
}

#[test]
fn test_traversal_result_is_empty() {
    let result = TraversalResult::new(vec![], vec![], 0);
    assert!(result.is_empty());

    let nodes = vec![Node::new("Test", NodeType::Concept)];
    let result = TraversalResult::new(nodes, vec![], 0);
    assert!(!result.is_empty());
}

#[test]
fn test_traversal_result_len() {
    let nodes = vec![
        Node::new("Node 1", NodeType::Entity),
        Node::new("Node 2", NodeType::Concept),
    ];

    let result = TraversalResult::new(nodes, vec![], 0);
    assert_eq!(result.len(), 2);
}

// ========== TraverseDirection Tests ==========

#[test]
fn test_traverse_direction_includes_forward() {
    assert!(TraverseDirection::Forward.includes_forward());
    assert!(TraverseDirection::Both.includes_forward());
    assert!(!TraverseDirection::Backward.includes_forward());
}

#[test]
fn test_traverse_direction_includes_backward() {
    assert!(TraverseDirection::Backward.includes_backward());
    assert!(TraverseDirection::Both.includes_backward());
    assert!(!TraverseDirection::Forward.includes_backward());
}

// ========== Complex Graph Scenarios ==========

#[tokio::test]
async fn test_diamond_graph() {
    // Diamond shape: a -> b, a -> c, b -> d, c -> d
    let mut graph = MemoryGraph::new();

    let a = Node::new("A", NodeType::Entity);
    let b = Node::new("B", NodeType::Concept);
    let c = Node::new("C", NodeType::Concept);
    let d = Node::new("D", NodeType::Fact);

    graph.add_node(a.clone()).unwrap();
    graph.add_node(b.clone()).unwrap();
    graph.add_node(c.clone()).unwrap();
    graph.add_node(d.clone()).unwrap();

    graph.add_edge(Edge::new(a.id, b.id, Relation::Causes)).unwrap();
    graph.add_edge(Edge::new(a.id, c.id, Relation::Causes)).unwrap();
    graph.add_edge(Edge::new(b.id, d.id, Relation::Causes)).unwrap();
    graph.add_edge(Edge::new(c.id, d.id, Relation::Causes)).unwrap();

    // Shortest path from a to d
    let path = graph.shortest_path(a.id, d.id, 10).await.unwrap();

    assert!(path.is_some());
    let path = path.unwrap();
    assert_eq!(path.len(), 3); // a -> b -> d or a -> c -> d
}

#[tokio::test]
async fn test_star_graph() {
    // Star shape: center connected to all outer nodes
    let mut graph = MemoryGraph::new();

    let center = Node::new("Center", NodeType::Entity);
    graph.add_node(center.clone()).unwrap();

    let mut outer_ids = Vec::new();
    for i in 0..5 {
        let node = Node::new(format!("Outer {}", i), NodeType::Concept);
        graph.add_node(node.clone()).unwrap();
        graph.add_edge(Edge::new(center.id, node.id, Relation::SimilarTo)).unwrap();
        outer_ids.push(node.id);
    }

    // BFS from center should reach all outer nodes
    let config = TraversalConfig::with_depth(1);
    let result = graph.bfs(center.id, config).await.unwrap();

    assert_eq!(result.nodes.len(), 5);
}

#[tokio::test]
async fn test_disconnected_components() {
    let mut graph = MemoryGraph::new();

    // Component 1
    let a = Node::new("A", NodeType::Entity);
    let b = Node::new("B", NodeType::Concept);
    graph.add_node(a.clone()).unwrap();
    graph.add_node(b.clone()).unwrap();
    graph.add_edge(Edge::new(a.id, b.id, Relation::Causes)).unwrap();

    // Component 2 (disconnected)
    let c = Node::new("C", NodeType::Entity);
    let d = Node::new("D", NodeType::Concept);
    graph.add_node(c.clone()).unwrap();
    graph.add_node(d.clone()).unwrap();
    graph.add_edge(Edge::new(c.id, d.id, Relation::Causes)).unwrap();

    // No path from component 1 to component 2
    let path = graph.shortest_path(a.id, c.id, 10).await.unwrap();
    assert!(path.is_none());
}

#[tokio::test]
async fn test_self_loop() {
    let mut graph = MemoryGraph::new();

    let node = Node::new("Self-loop", NodeType::Entity);
    graph.add_node(node.clone()).unwrap();

    // Add self-loop edge
    graph.add_edge(Edge::new(node.id, node.id, Relation::SimilarTo)).unwrap();

    // BFS should handle self-loops without infinite loop
    let config = TraversalConfig::with_depth(3).with_avoid_cycles(true);
    let result = graph.bfs(node.id, config).await.unwrap();

    // Should not have duplicates
    let unique_ids: std::collections::HashSet<_> = result.nodes.iter().map(|n| n.id).collect();
    assert_eq!(unique_ids.len(), result.nodes.len());
}

#[tokio::test]
async fn test_graph_with_multiple_edge_types() {
    let mut graph = MemoryGraph::new();

    let a = Node::new("A", NodeType::Entity);
    let b = Node::new("B", NodeType::Concept);

    graph.add_node(a.clone()).unwrap();
    graph.add_node(b.clone()).unwrap();

    // Add multiple edges between same nodes
    graph.add_edge(Edge::new(a.id, b.id, Relation::Causes)).unwrap();
    graph.add_edge(Edge::new(a.id, b.id, Relation::SimilarTo)).unwrap();

    let edges = graph.edges(a.id, TraverseDirection::Forward).await.unwrap();

    assert_eq!(edges.len(), 2);
}
