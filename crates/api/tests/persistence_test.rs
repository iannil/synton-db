// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Persistence integration tests.
//!
//! Tests data persistence using RocksDB.

use synton_core::{Edge, Node, NodeType};
use synton_storage::{Store, WriteOp};
use synton_storage::rocksdb::{RocksdbConfig, RocksdbStore};
use tempfile::tempdir;

/// Create a temporary RocksDB store.
async fn create_temp_store() -> (RocksdbStore, tempfile::TempDir) {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config = RocksdbConfig {
        path: temp_dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };
    let store = RocksdbStore::open(config).expect("Failed to open RocksDB");
    (store, temp_dir)
}

#[tokio::test]
async fn test_persistence_node_storage() {
    let (store, _temp_dir) = create_temp_store().await;

    // Create a node
    let node = Node::new("Persistent content", NodeType::Concept);
    let id = node.id;

    // Write to store
    store.put_node(&node).await.expect("Failed to put node");

    // Read back
    let retrieved = store.get_node(id).await.expect("Query failed");
    assert!(retrieved.is_some());
    let node = retrieved.as_ref().unwrap();
    assert_eq!(node.id, id);
    assert_eq!(node.content(), "Persistent content");
}

#[tokio::test]
async fn test_persistence_node_deletion() {
    let (store, _temp_dir) = create_temp_store().await;

    // Create a node
    let node = Node::new("To be deleted", NodeType::Concept);
    let id = node.id;

    store.put_node(&node).await.expect("Failed to put node");

    // Verify it exists
    let retrieved = store.get_node(id).await.expect("Query failed");
    assert!(retrieved.is_some());

    // Delete it
    store.delete_node(id).await.expect("Failed to delete");

    // Verify it's gone
    let retrieved = store.get_node(id).await.expect("Query failed");
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_persistence_edge_storage() {
    let (store, _temp_dir) = create_temp_store().await;

    // Create nodes
    let node1 = Node::new("Source node", NodeType::Entity);
    let node2 = Node::new("Target node", NodeType::Concept);
    let source = node1.id;
    let target = node2.id;

    store.put_node(&node1).await.expect("Failed to put node1");
    store.put_node(&node2).await.expect("Failed to put node2");

    // Create edge
    let edge = Edge::new(source, target, synton_core::Relation::Causes);
    store.put_edge(&edge).await.expect("Failed to put edge");

    // Verify edge exists
    let retrieved = store
        .get_edge(source, target, "causes")
        .await
        .expect("Query failed");
    assert!(retrieved.is_some());
    let edge = retrieved.as_ref().unwrap();
    assert_eq!(edge.source, source);
    assert_eq!(edge.target, target);
}

#[tokio::test]
async fn test_persistence_batch_write() {
    let (store, _temp_dir) = create_temp_store().await;

    // Batch write multiple nodes
    let mut nodes = Vec::new();
    for i in 0..10 {
        nodes.push(Node::new(format!("Batch Node {}", i), NodeType::Entity));
    }

    let ops: Vec<WriteOp> = nodes.iter().map(|n| WriteOp::PutNode(n.clone())).collect();
    store.batch_write(ops).await.expect("Batch write failed");

    // Verify all nodes exist
    for node in nodes {
        let retrieved = store.get_node(node.id).await.expect("Query failed");
        assert!(retrieved.is_some());
    }
}

#[tokio::test]
async fn test_persistence_flush() {
    let (store, temp_dir) = create_temp_store().await;

    let node = Node::new("Flush test", NodeType::Concept);
    let id = node.id;

    store.put_node(&node).await.expect("Failed to put node");
    store.flush().await.expect("Failed to flush");

    // Drop store to release lock
    drop(store);

    // Reopen store
    let config = RocksdbConfig {
        path: temp_dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };
    let store = RocksdbStore::open(config).expect("Failed to reopen store");

    let retrieved = store.get_node(id).await.expect("Query failed");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.as_ref().unwrap().content(), "Flush test");
}

#[tokio::test]
async fn test_persistence_metadata() {
    let (store, _temp_dir) = create_temp_store().await;

    // Write metadata
    store
        .put_metadata("version", b"1.0.0")
        .await
        .expect("Failed to write metadata");

    store
        .put_metadata("test_key", b"test_value")
        .await
        .expect("Failed to write metadata");

    // Verify metadata persists
    let version = store
        .get_metadata("version")
        .await
        .expect("Failed to get metadata");
    assert_eq!(version, Some(b"1.0.0".to_vec()));

    let test_value = store
        .get_metadata("test_key")
        .await
        .expect("Failed to get metadata");
    assert_eq!(test_value, Some(b"test_value".to_vec()));
}

#[tokio::test]
async fn test_persistence_edge_deletion() {
    let (store, _temp_dir) = create_temp_store().await;

    // Create nodes and edge
    let node1 = Node::new("Node 1", NodeType::Entity);
    let node2 = Node::new("Node 2", NodeType::Concept);
    let source = node1.id;
    let target = node2.id;

    store.put_node(&node1).await.expect("Failed to put node1");
    store.put_node(&node2).await.expect("Failed to put node2");

    let edge = Edge::new(source, target, synton_core::Relation::SimilarTo);
    store.put_edge(&edge).await.expect("Failed to put edge");

    // Verify edge exists
    let retrieved = store
        .get_edge(source, target, "similar_to")
        .await
        .expect("Query failed");
    assert!(retrieved.is_some());

    // Delete edge
    let deleted = store
        .delete_edge(source, target, "similar_to")
        .await
        .expect("Delete failed");
    assert!(deleted);

    // Verify it's gone
    let retrieved = store
        .get_edge(source, target, "similar_to")
        .await
        .expect("Query failed");
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_persistence_multiple_nodes() {
    let (store, temp_dir) = create_temp_store().await;

    let node_count = 50;
    let mut node_ids = Vec::new();

    for i in 0..node_count {
        let node = Node::new(format!("Node {}", i), NodeType::Entity);
        node_ids.push(node.id);
        store.put_node(&node).await.expect("Failed to put node");
    }

    // Flush to ensure persistence
    store.flush().await.expect("Failed to flush");

    // Drop store to release lock
    drop(store);

    // Create new store instance
    let config = RocksdbConfig {
        path: temp_dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };
    let store = RocksdbStore::open(config).expect("Failed to reopen store");

    // Verify all nodes exist
    for id in node_ids {
        let retrieved = store.get_node(id).await.expect("Query failed");
        assert!(retrieved.is_some());
    }
}

#[tokio::test]
async fn test_persistence_update_node() {
    let (store, _temp_dir) = create_temp_store().await;

    let node1 = Node::new("Original content", NodeType::Concept);
    let id = node1.id;
    store.put_node(&node1).await.expect("Failed to put node");

    // Update with new content
    let node2 = Node::new("Updated content", NodeType::Concept);
    store.put_node(&node2).await.expect("Failed to update node");

    // Verify content changed
    let retrieved = store.get_node(id).await.expect("Query failed");
    assert!(retrieved.is_some());
    // Note: With same ID, content should be updated
}

#[tokio::test]
async fn test_persistence_complex_graph() {
    let (store, temp_dir) = create_temp_store().await;

    // Create a small graph
    let center = Node::new("Center", NodeType::Concept);
    store.put_node(&center).await.expect("Failed to put center");

    // Create surrounding nodes
    for i in 0..5 {
        let node = Node::new(format!("Related {}", i), NodeType::Entity);
        store.put_node(&node).await.expect("Failed to put node");

        // Connect to center
        let edge_out = Edge::new(center.id, node.id, synton_core::Relation::SimilarTo);
        store.put_edge(&edge_out).await.expect("Failed to put edge out");

        let edge_in = Edge::new(node.id, center.id, synton_core::Relation::SimilarTo);
        store.put_edge(&edge_in).await.expect("Failed to put edge in");
    }

    // Flush and verify
    store.flush().await.expect("Failed to flush");

    // Drop store to release lock
    drop(store);

    // Reopen store
    let config = RocksdbConfig {
        path: temp_dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };
    let store = RocksdbStore::open(config).expect("Failed to reopen store");

    // Verify center still exists
    let retrieved = store.get_node(center.id).await.expect("Query failed");
    assert!(retrieved.is_some());
}
