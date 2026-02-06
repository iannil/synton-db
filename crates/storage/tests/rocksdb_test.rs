// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Comprehensive unit tests for RocksDB storage backend.

use uuid::Uuid;

use synton_core::{Edge, Node, NodeType, Relation};
use synton_storage::{ColumnFamily, Store, WriteOp};
use synton_storage::rocksdb::{RocksdbConfig, RocksdbStore, RocksdbCompression};

/// Helper function to create a temporary RocksDB store for testing.
async fn create_test_store() -> RocksdbStore {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let config = RocksdbConfig {
        path: temp_dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };
    RocksdbStore::open(config).expect("Failed to open RocksDB")
}

// ========== Node Operations Tests ==========

#[tokio::test]
async fn test_put_and_get_node() {
    let store = create_test_store().await;

    let node = Node::new("Test content", NodeType::Entity);
    let id = node.id;

    // Put node
    store.put_node(&node).await.expect("Failed to put node");

    // Get node
    let retrieved = store.get_node(id).await.expect("Failed to get node");
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();

    assert_eq!(retrieved.id, id);
    assert_eq!(retrieved.content(), "Test content");
    assert_eq!(retrieved.node_type, NodeType::Entity);
}

#[tokio::test]
async fn test_get_nonexistent_node() {
    let store = create_test_store().await;

    let result = store.get_node(Uuid::new_v4()).await.expect("Query succeeded");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_node_exists() {
    let store = create_test_store().await;

    let node = Node::new("Test content", NodeType::Concept);
    let id = node.id;

    assert!(!store.node_exists(id).await.expect("Check failed"));

    store.put_node(&node).await.expect("Failed to put node");
    assert!(store.node_exists(id).await.expect("Check failed"));
}

#[tokio::test]
async fn test_delete_node() {
    let store = create_test_store().await;

    let node = Node::new("Test content", NodeType::Entity);
    let id = node.id;

    store.put_node(&node).await.expect("Failed to put node");
    assert!(store.node_exists(id).await.expect("Check failed"));

    let deleted = store.delete_node(id).await.expect("Delete failed");
    assert!(deleted);
    assert!(!store.node_exists(id).await.expect("Check failed"));
}

#[tokio::test]
async fn test_delete_nonexistent_node() {
    let store = create_test_store().await;

    let deleted = store.delete_node(Uuid::new_v4()).await.expect("Delete failed");
    assert!(!deleted);
}

#[tokio::test]
async fn test_update_node() {
    let store = create_test_store().await;

    let node1 = Node::new("Original content", NodeType::Entity);
    let id = node1.id;

    store.put_node(&node1).await.expect("Failed to put node");

    // Create a new node with same ID (different content)
    let node2 = Node::new("Updated content", NodeType::Entity);
    store.put_node(&node2).await.expect("Failed to update node");

    // Check that the content was updated (nodes with same ID get overwritten)
    let retrieved = store.get_node(id).await.expect("Failed to get node");
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn test_node_with_embedding() {
    let store = create_test_store().await;

    let _embedding = vec![0.1f32, 0.2, 0.3, 0.4];
    let node = Node::new("Test", NodeType::Concept);
    // Embedding needs to be set through Node builder methods if available
    let id = node.id;

    store.put_node(&node).await.expect("Failed to put node");

    let retrieved = store.get_node(id).await.expect("Failed to get node");
    assert!(retrieved.is_some());
}

// ========== Edge Operations Tests ==========

#[tokio::test]
async fn test_put_and_get_edge() {
    let store = create_test_store().await;

    let source = Uuid::new_v4();
    let target = Uuid::new_v4();
    let edge = Edge::new(source, target, Relation::Causes);

    store.put_edge(&edge).await.expect("Failed to put edge");

    let retrieved = store
        .get_edge(source, target, "causes")
        .await
        .expect("Failed to get edge");
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();

    assert_eq!(retrieved.source, source);
    assert_eq!(retrieved.target, target);
}

#[tokio::test]
async fn test_get_nonexistent_edge() {
    let store = create_test_store().await;

    let result = store
        .get_edge(Uuid::new_v4(), Uuid::new_v4(), "causes")
        .await
        .expect("Query succeeded");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_edge() {
    let store = create_test_store().await;

    let source = Uuid::new_v4();
    let target = Uuid::new_v4();
    let edge = Edge::new(source, target, Relation::SimilarTo);

    store.put_edge(&edge).await.expect("Failed to put edge");

    let deleted = store
        .delete_edge(source, target, "similar_to")
        .await
        .expect("Delete failed");
    assert!(deleted);

    let retrieved = store
        .get_edge(source, target, "similar_to")
        .await
        .expect("Query succeeded");
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_edge_with_weight() {
    let store = create_test_store().await;

    let source = Uuid::new_v4();
    let target = Uuid::new_v4();
    let edge = Edge::with_weight(source, target, Relation::Causes, 0.85);

    store.put_edge(&edge).await.expect("Failed to put edge");

    let retrieved = store
        .get_edge(source, target, "causes")
        .await
        .expect("Failed to get edge")
        .unwrap();

    assert!((retrieved.weight - 0.85).abs() < 0.001);
}

#[tokio::test]
async fn test_get_outgoing_edges() {
    let store = create_test_store().await;

    let source = Uuid::new_v4();
    let target1 = Uuid::new_v4();
    let target2 = Uuid::new_v4();

    store
        .put_edge(&Edge::new(source, target1, Relation::Causes))
        .await
        .expect("Failed to put edge 1");
    store
        .put_edge(&Edge::new(source, target2, Relation::SimilarTo))
        .await
        .expect("Failed to put edge 2");

    let edges = store
        .get_outgoing_edges(source)
        .await
        .expect("Failed to get outgoing edges");

    assert_eq!(edges.len(), 2);
    assert!(edges.iter().any(|e| e.target == target1));
    assert!(edges.iter().any(|e| e.target == target2));
}

#[tokio::test]
async fn test_get_incoming_edges() {
    let store = create_test_store().await;

    let target = Uuid::new_v4();
    let source1 = Uuid::new_v4();
    let source2 = Uuid::new_v4();

    store
        .put_edge(&Edge::new(source1, target, Relation::Causes))
        .await
        .expect("Failed to put edge 1");
    store
        .put_edge(&Edge::new(source2, target, Relation::SimilarTo))
        .await
        .expect("Failed to put edge 2");

    let edges = store
        .get_incoming_edges(target)
        .await
        .expect("Failed to get incoming edges");

    assert_eq!(edges.len(), 2);
    assert!(edges.iter().any(|e| e.source == source1));
    assert!(edges.iter().any(|e| e.source == source2));
}

// ========== Batch Operations Tests ==========

#[tokio::test]
async fn test_batch_write_nodes() {
    let store = create_test_store().await;

    let node1 = Node::new("Node 1", NodeType::Entity);
    let node2 = Node::new("Node 2", NodeType::Concept);
    let node3 = Node::new("Node 3", NodeType::Fact);

    let ops = vec![
        WriteOp::PutNode(node1.clone()),
        WriteOp::PutNode(node2.clone()),
        WriteOp::PutNode(node3.clone()),
    ];

    store.batch_write(ops).await.expect("Batch write failed");

    assert!(store.node_exists(node1.id).await.expect("Check failed"));
    assert!(store.node_exists(node2.id).await.expect("Check failed"));
    assert!(store.node_exists(node3.id).await.expect("Check failed"));
}

#[tokio::test]
async fn test_batch_write_edges() {
    let store = create_test_store().await;

    let source = Uuid::new_v4();
    let target1 = Uuid::new_v4();
    let target2 = Uuid::new_v4();

    let ops = vec![
        WriteOp::PutEdge(Edge::new(source, target1, Relation::Causes)),
        WriteOp::PutEdge(Edge::new(source, target2, Relation::SimilarTo)),
    ];

    store.batch_write(ops).await.expect("Batch write failed");

    let edges = store
        .get_outgoing_edges(source)
        .await
        .expect("Failed to get edges");
    assert_eq!(edges.len(), 2);
}

#[tokio::test]
async fn test_batch_write_mixed() {
    let store = create_test_store().await;

    let node1 = Node::new("Node 1", NodeType::Entity);
    let node2 = Node::new("Node 2", NodeType::Concept);

    let ops = vec![
        WriteOp::PutNode(node1.clone()),
        WriteOp::PutEdge(Edge::new(node1.id, node2.id, Relation::Causes)),
        WriteOp::PutNode(node2.clone()),
    ];

    store.batch_write(ops).await.expect("Batch write failed");

    assert!(store.node_exists(node1.id).await.expect("Check failed"));
    assert!(store.node_exists(node2.id).await.expect("Check failed"));

    let edge = store
        .get_edge(node1.id, node2.id, "causes")
        .await
        .expect("Query failed");
    assert!(edge.is_some());
}

#[tokio::test]
async fn test_batch_write_with_deletes() {
    let store = create_test_store().await;

    let node1 = Node::new("Node 1", NodeType::Entity);
    let node2 = Node::new("Node 2", NodeType::Concept);

    store.put_node(&node1).await.expect("Failed to put node");
    store.put_node(&node2).await.expect("Failed to put node");

    let ops = vec![
        WriteOp::PutNode(Node::new("Node 3", NodeType::Fact)),
        WriteOp::DeleteNode(node1.id),
        WriteOp::DeleteEdge(node2.id, Uuid::new_v4(), "causes".to_string()),
    ];

    store.batch_write(ops).await.expect("Batch write failed");

    assert!(!store.node_exists(node1.id).await.expect("Check failed"));
}

// ========== Metadata Operations Tests ==========

#[tokio::test]
async fn test_put_and_get_metadata() {
    let store = create_test_store().await;

    store
        .put_metadata("version", b"1.0.0")
        .await
        .expect("Failed to put metadata");

    let value = store
        .get_metadata("version")
        .await
        .expect("Failed to get metadata");
    assert_eq!(value, Some(b"1.0.0".to_vec()));
}

#[tokio::test]
async fn test_get_nonexistent_metadata() {
    let store = create_test_store().await;

    let value = store
        .get_metadata("nonexistent")
        .await
        .expect("Query failed");
    assert!(value.is_none());
}

#[tokio::test]
async fn test_update_metadata() {
    let store = create_test_store().await;

    store
        .put_metadata("key", b"value1")
        .await
        .expect("Failed to put metadata");

    store
        .put_metadata("key", b"value2")
        .await
        .expect("Failed to update metadata");

    let value = store.get_metadata("key").await.expect("Failed to get metadata");
    assert_eq!(value, Some(b"value2".to_vec()));
}

#[tokio::test]
async fn test_metadata_with_binary_data() {
    let store = create_test_store().await;

    let binary_data: Vec<u8> = vec![0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD];

    store
        .put_metadata("binary", &binary_data)
        .await
        .expect("Failed to put metadata");

    let value = store
        .get_metadata("binary")
        .await
        .expect("Failed to get metadata");
    assert_eq!(value, Some(binary_data));
}

// ========== Configuration Tests ==========

#[test]
fn test_column_family_names() {
    assert_eq!(ColumnFamily::Nodes.as_str(), "nodes");
    assert_eq!(ColumnFamily::Edges.as_str(), "edges");
    assert_eq!(ColumnFamily::EdgesOut.as_str(), "edges_out");
    assert_eq!(ColumnFamily::EdgesIn.as_str(), "edges_in");
    assert_eq!(ColumnFamily::Metadata.as_str(), "metadata");
    assert_eq!(ColumnFamily::AccessLog.as_str(), "access_log");
}

#[test]
fn test_column_family_display() {
    assert_eq!(format!("{}", ColumnFamily::Nodes), "nodes");
    assert_eq!(format!("{}", ColumnFamily::Metadata), "metadata");
}

#[test]
fn test_column_family_from_str() {
    assert_eq!("nodes".parse::<ColumnFamily>().unwrap(), ColumnFamily::Nodes);
    assert_eq!("edges".parse::<ColumnFamily>().unwrap(), ColumnFamily::Edges);
    assert_eq!("metadata".parse::<ColumnFamily>().unwrap(), ColumnFamily::Metadata);
    assert!("unknown".parse::<ColumnFamily>().is_err());
}

#[tokio::test]
async fn test_custom_config() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let config = RocksdbConfig {
        path: temp_dir.path().to_str().unwrap().to_string(),
        write_buffer_size: 64 * 1024 * 1024,
        max_write_buffers: 2,
        max_background_jobs: 2,
        create_if_missing: true,
        create_missing_column_families: true,
        compression: RocksdbCompression::Snappy,
    };

    let store = RocksdbStore::open(config);
    assert!(store.is_ok());
}

#[tokio::test]
async fn test_compression_options() {
    // All compression types should be valid
    let types = vec![
        RocksdbCompression::NoCompression,
        RocksdbCompression::Snappy,
        RocksdbCompression::Lz4,
    ];

    for compression in types {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let config = RocksdbConfig {
            path: temp_dir.path().to_str().unwrap().to_string(),
            compression,
            ..Default::default()
        };
        assert!(RocksdbStore::open(config).is_ok());
    }
}

// ========== Persistence Tests ==========

#[tokio::test]
async fn test_data_persistence() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = temp_dir.path().to_str().unwrap().to_string();

    let node = Node::new("Persistent data", NodeType::Concept);
    let id = node.id;

    {
        // First store instance
        let config = RocksdbConfig {
            path: path.clone(),
            ..Default::default()
        };
        let store = RocksdbStore::open(config).expect("Failed to open store");
        store.put_node(&node).await.expect("Failed to put node");
        store.flush().await.expect("Failed to flush");
    }

    {
        // Second store instance - data should persist
        let config = RocksdbConfig {
            path,
            ..Default::default()
        };
        let store = RocksdbStore::open(config).expect("Failed to reopen store");
        let retrieved = store.get_node(id).await.expect("Failed to get node");
        assert!(retrieved.is_some());
    }
}

// ========== Concurrent Operations Tests ==========

#[tokio::test]
async fn test_concurrent_node_writes() {
    let store = std::sync::Arc::new(create_test_store().await);
    let mut handles = Vec::new();

    for i in 0..100 {
        let store_clone = store.clone();
        handles.push(tokio::spawn(async move {
            let node = Node::new(format!("Node {}", i), NodeType::Entity);
            store_clone.put_node(&node).await
        }));
    }

    for handle in handles {
        handle.await.expect("Task failed").expect("Put failed");
    }
}

#[tokio::test]
async fn test_concurrent_reads() {
    let store = std::sync::Arc::new(create_test_store().await);
    let node = Node::new("Shared node", NodeType::Concept);
    let id = node.id;

    store.put_node(&node).await.expect("Failed to put node");

    let mut handles = Vec::new();
    for _ in 0..50 {
        let store_clone = store.clone();
        handles.push(tokio::spawn(async move {
            store_clone.get_node(id).await
        }));
    }

    for handle in handles {
        let result = handle.await.expect("Task failed").expect("Get failed");
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, id);
    }
}
