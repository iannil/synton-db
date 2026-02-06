// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! End-to-end workflow integration tests.
//!
//! Tests complete knowledge graph workflows including:
//! - Node and edge creation
//! - Query operations
//! - Graph traversal
//! - Vector search
//! - Memory management

use std::sync::Arc;
use synton_api::{
    AddEdgeRequest, AddNodeRequest, QueryRequest, SyntonDbService, TraverseRequest,
    TraverseDirection,
};
use synton_core::NodeType;
use synton_storage::rocksdb::{RocksdbConfig, RocksdbStore};
use tempfile::tempdir;

/// Create a test service with optional persistence.
async fn create_test_service(with_persistence: bool) -> SyntonDbService {
    if with_persistence {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let config = RocksdbConfig {
            path: temp_dir.path().to_str().unwrap().to_string(),
            ..Default::default()
        };
        let store = RocksdbStore::open(config).expect("Failed to open RocksDB");
        SyntonDbService::with_store(Arc::new(store))
    } else {
        SyntonDbService::new()
    }
}

#[tokio::test]
async fn test_complete_knowledge_graph_workflow() {
    let service = create_test_service(false).await;

    // 1. Create a knowledge graph about machine learning
    let ml = service
        .add_node(AddNodeRequest::new(
            "Machine Learning is a subset of artificial intelligence".to_string(),
            NodeType::Concept,
        ))
        .await
        .unwrap()
        .node;

    let dl = service
        .add_node(AddNodeRequest::new(
            "Deep Learning uses neural networks with multiple layers".to_string(),
            NodeType::Concept,
        ))
        .await
        .unwrap()
        .node;

    let nn = service
        .add_node(AddNodeRequest::new(
            "Neural Networks are computing systems inspired by biological neurons".to_string(),
            NodeType::Concept,
        ))
        .await
        .unwrap()
        .node;

    let python = service
        .add_node(AddNodeRequest::new(
            "Python is a popular programming language for ML".to_string(),
            NodeType::Entity,
        ))
        .await
        .unwrap()
        .node;

    // 2. Connect them with edges
    service
        .add_edge(AddEdgeRequest {
            source: ml.id,
            target: dl.id,
            relation: synton_core::Relation::IsA,
            ..Default::default()
        })
        .await
        .unwrap();

    service
        .add_edge(AddEdgeRequest {
            source: dl.id,
            target: nn.id,
            relation: synton_core::Relation::IsPartOf,
            ..Default::default()
        })
        .await
        .unwrap();

    service
        .add_edge(AddEdgeRequest {
            source: python.id,
            target: ml.id,
            relation: synton_core::Relation::SimilarTo,
            ..Default::default()
        })
        .await
        .unwrap();

    // 3. Query for "learning"
    let query_result = service
        .query(QueryRequest {
            query: "learning".to_string(),
            limit: Some(10),
            include_metadata: false,
        })
        .await
        .unwrap();

    assert!(!query_result.nodes.is_empty());
    assert!(query_result.nodes.iter().any(|n| n.id == ml.id));

    // 4. Traverse from ML
    let traverse_result = service
        .traverse(TraverseRequest {
            start_id: ml.id,
            max_depth: 2,
            max_nodes: 10,
            direction: TraverseDirection::Forward,
        })
        .await
        .unwrap();

    assert!(traverse_result.nodes.len() >= 2); // Should find DL and NN

    // 5. Verify stats
    let stats = service.stats().await.unwrap();
    assert_eq!(stats.node_count, 4);
    assert_eq!(stats.edge_count, 3);
}

#[tokio::test]
async fn test_query_with_no_results() {
    let service = create_test_service(false).await;

    service
        .add_node(AddNodeRequest::new(
            "Machine Learning".to_string(),
            NodeType::Concept,
        ))
        .await
        .unwrap();

    let query_result = service
        .query(QueryRequest {
            query: "quantum physics".to_string(),
            limit: Some(10),
            include_metadata: false,
        })
        .await
        .unwrap();

    assert!(query_result.nodes.is_empty());
}

#[tokio::test]
async fn test_traverse_nonexistent_start() {
    let service = create_test_service(false).await;

    let result = service
        .traverse(TraverseRequest {
            start_id: uuid::Uuid::new_v4(),
            max_depth: 1,
            max_nodes: 10,
            direction: TraverseDirection::Forward,
        })
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_bulk_operations() {
    let service = create_test_service(false).await;

    let nodes = vec![
        AddNodeRequest::new("Node 1".to_string(), NodeType::Entity),
        AddNodeRequest::new("Node 2".to_string(), NodeType::Concept),
        AddNodeRequest::new("Node 3".to_string(), NodeType::Fact),
    ];

    // Add nodes
    let mut node_ids = Vec::new();
    for node_req in nodes {
        let resp = service.add_node(node_req).await.unwrap();
        node_ids.push(resp.node.id);
    }

    // Connect them
    service
        .add_edge(AddEdgeRequest {
            source: node_ids[0],
            target: node_ids[1],
            relation: synton_core::Relation::Causes,
            ..Default::default()
        })
        .await
        .unwrap();

    service
        .add_edge(AddEdgeRequest {
            source: node_ids[1],
            target: node_ids[2],
            relation: synton_core::Relation::Causes,
            ..Default::default()
        })
        .await
        .unwrap();

    // Traverse from first
    let traverse_result = service
        .traverse(TraverseRequest {
            start_id: node_ids[0],
            max_depth: 2,
            max_nodes: 10,
            direction: TraverseDirection::Forward,
        })
        .await
        .unwrap();

    assert_eq!(traverse_result.nodes.len(), 2); // 2 downstream nodes
}

#[tokio::test]
async fn test_case_insensitive_query() {
    let service = create_test_service(false).await;

    service
        .add_node(AddNodeRequest::new(
            "Machine Learning Algorithms".to_string(),
            NodeType::Concept,
        ))
        .await
        .unwrap();

    // Query with different cases
    let queries = vec!["MACHINE", "machine", "MaChInE"];

    for query in queries {
        let result = service
            .query(QueryRequest {
                query: query.to_string(),
                limit: Some(10),
                include_metadata: false,
            })
            .await
            .unwrap();

        assert!(!result.nodes.is_empty(), "Failed for query: {}", query);
    }
}

#[tokio::test]
async fn test_delete_and_query() {
    let service = create_test_service(false).await;

    let node = service
        .add_node(AddNodeRequest::new(
            "Temporary node".to_string(),
            NodeType::Concept,
        ))
        .await
        .unwrap()
        .node;

    // Verify it exists
    let query1 = service
        .query(QueryRequest {
            query: "temporary".to_string(),
            limit: Some(10),
            include_metadata: false,
        })
        .await
        .unwrap();
    assert!(!query1.nodes.is_empty());

    // Delete it
    service
        .delete_node(synton_api::DeleteNodeRequest { id: node.id })
        .await
        .unwrap();

    // Verify it's gone
    let query2 = service
        .query(QueryRequest {
            query: "temporary".to_string(),
            limit: Some(10),
            include_metadata: false,
        })
        .await
        .unwrap();
    assert!(query2.nodes.is_empty());
}

#[tokio::test]
async fn test_circular_relationships() {
    let service = create_test_service(false).await;

    let a = service
        .add_node(AddNodeRequest::new("Node A".to_string(), NodeType::Entity))
        .await
        .unwrap()
        .node;

    let b = service
        .add_node(AddNodeRequest::new("Node B".to_string(), NodeType::Entity))
        .await
        .unwrap()
        .node;

    let c = service
        .add_node(AddNodeRequest::new("Node C".to_string(), NodeType::Entity))
        .await
        .unwrap()
        .node;

    // Create circular relationships: A -> B -> C -> A
    service
        .add_edge(AddEdgeRequest {
            source: a.id,
            target: b.id,
            relation: synton_core::Relation::Causes,
            ..Default::default()
        })
        .await
        .unwrap();

    service
        .add_edge(AddEdgeRequest {
            source: b.id,
            target: c.id,
            relation: synton_core::Relation::Causes,
            ..Default::default()
        })
        .await
        .unwrap();

    service
        .add_edge(AddEdgeRequest {
            source: c.id,
            target: a.id,
            relation: synton_core::Relation::Causes,
            ..Default::default()
        })
        .await
        .unwrap();

    // Traverse from A with max_depth=3 should find B and C
    let result = service
        .traverse(TraverseRequest {
            start_id: a.id,
            max_depth: 3,
            max_nodes: 10,
            direction: TraverseDirection::Forward,
        })
        .await
        .unwrap();

    // Should find B and C (A is not included as start)
    assert!(result.nodes.iter().any(|n| n.id == b.id));
    assert!(result.nodes.iter().any(|n| n.id == c.id));
}

#[tokio::test]
async fn test_empty_service_stats() {
    let service = create_test_service(false).await;

    let stats = service.stats().await.unwrap();

    assert_eq!(stats.node_count, 0);
    assert_eq!(stats.edge_count, 0);
    assert_eq!(stats.embedded_count, 0);
}

#[tokio::test]
async fn test_query_limit() {
    let service = create_test_service(false).await;

    // Add 10 nodes with similar content
    for i in 0..10 {
        service
            .add_node(AddNodeRequest::new(
                format!("Test Node {}", i),
                NodeType::Entity,
            ))
            .await
            .unwrap();
    }

    // Query with limit 5
    let result = service
        .query(QueryRequest {
            query: "Test".to_string(),
            limit: Some(5),
            include_metadata: false,
        })
        .await
        .unwrap();

    assert_eq!(result.nodes.len(), 5);
}
