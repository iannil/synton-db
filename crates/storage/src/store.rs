// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use async_trait::async_trait;
use futures::stream::BoxStream;
use uuid::Uuid;

use crate::{StorageError, StorageResult};
use synton_core::{Edge, Filter, Node};

/// Column family names for RocksDB storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColumnFamily {
    /// Node data
    Nodes,
    /// Edge data
    Edges,
    /// Outgoing edges index (source -> edges)
    EdgesOut,
    /// Incoming edges index (target -> edges)
    EdgesIn,
    /// Metadata
    Metadata,
    /// Access log for memory decay
    AccessLog,
}

impl ColumnFamily {
    /// All column families
    pub const ALL: &'static [Self] = &[
        Self::Nodes,
        Self::Edges,
        Self::EdgesOut,
        Self::EdgesIn,
        Self::Metadata,
        Self::AccessLog,
    ];

    /// Get the column family name as a string.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Nodes => "nodes",
            Self::Edges => "edges",
            Self::EdgesOut => "edges_out",
            Self::EdgesIn => "edges_in",
            Self::Metadata => "metadata",
            Self::AccessLog => "access_log",
        }
    }
}

impl AsRef<str> for ColumnFamily {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for ColumnFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ColumnFamily {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "nodes" => Ok(Self::Nodes),
            "edges" => Ok(Self::Edges),
            "edges_out" => Ok(Self::EdgesOut),
            "edges_in" => Ok(Self::EdgesIn),
            "metadata" => Ok(Self::Metadata),
            "access_log" => Ok(Self::AccessLog),
            _ => Err(format!("Unknown column family: {}", s)),
        }
    }
}

/// Write operation for batch writes.
#[derive(Debug, Clone)]
pub enum WriteOp {
    /// Put a node
    PutNode(Node),
    /// Delete a node
    DeleteNode(Uuid),
    /// Put an edge
    PutEdge(Edge),
    /// Delete an edge
    DeleteEdge(Uuid, Uuid, String),
    /// Put raw data to a column family
    Put { cf: ColumnFamily, key: Vec<u8>, value: Vec<u8> },
    /// Delete raw data from a column family
    Delete { cf: ColumnFamily, key: Vec<u8> },
}

/// Node filter for scanning operations.
#[derive(Debug, Clone, Default)]
pub struct NodeFilter {
    /// Filter by node type
    pub node_types: Option<Vec<synton_core::NodeType>>,
    /// Minimum confidence
    pub min_confidence: Option<f32>,
    /// Maximum confidence
    pub max_confidence: Option<f32>,
    /// Minimum access score
    pub min_access_score: Option<f32>,
    /// Created after timestamp
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    /// Custom filters
    pub custom: Vec<Filter>,
}

impl NodeFilter {
    /// Create a new empty filter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node type filter.
    pub fn with_node_type(mut self, node_type: synton_core::NodeType) -> Self {
        self.node_types.get_or_insert_with(Vec::new).push(node_type);
        self
    }

    /// Set minimum confidence.
    pub fn with_min_confidence(mut self, value: f32) -> Self {
        self.min_confidence = Some(value);
        self
    }

    /// Set maximum confidence.
    pub fn with_max_confidence(mut self, value: f32) -> Self {
        self.max_confidence = Some(value);
        self
    }

    /// Check if a node matches this filter.
    pub fn matches(&self, node: &Node) -> bool {
        if let Some(types) = &self.node_types {
            if !types.contains(&node.node_type) {
                return false;
            }
        }
        if let Some(min_conf) = self.min_confidence {
            if node.meta.confidence < min_conf {
                return false;
            }
        }
        if let Some(max_conf) = self.max_confidence {
            if node.meta.confidence > max_conf {
                return false;
            }
        }
        if let Some(min_score) = self.min_access_score {
            if node.meta.access_score < min_score {
                return false;
            }
        }
        if let Some(after) = self.created_after {
            if node.meta.created_at < after {
                return false;
            }
        }
        true
    }
}

/// Abstract storage interface for SYNTON-DB.
///
/// This trait defines the core storage operations that can be
/// implemented by different backends (RocksDB, in-memory, etc.).
#[async_trait]
pub trait Store: Send + Sync {
    // ========== Node Operations ==========

    /// Get a node by ID.
    async fn get_node(&self, id: Uuid) -> StorageResult<Option<Node>>;

    /// Put a node (insert or update).
    async fn put_node(&self, node: &Node) -> StorageResult<()>;

    /// Delete a node.
    async fn delete_node(&self, id: Uuid) -> StorageResult<bool>;

    /// Check if a node exists.
    async fn node_exists(&self, id: Uuid) -> StorageResult<bool>;

    // ========== Edge Operations ==========

    /// Get an edge by source and target IDs.
    async fn get_edge(
        &self,
        source: Uuid,
        target: Uuid,
        relation: &str,
    ) -> StorageResult<Option<Edge>>;

    /// Put an edge (insert or update).
    async fn put_edge(&self, edge: &Edge) -> StorageResult<()>;

    /// Delete an edge.
    async fn delete_edge(&self, source: Uuid, target: Uuid, relation: &str) -> StorageResult<bool>;

    /// Get all outgoing edges from a node.
    async fn get_outgoing_edges(&self, source: Uuid) -> StorageResult<Vec<Edge>>;

    /// Get all incoming edges to a node.
    async fn get_incoming_edges(&self, target: Uuid) -> StorageResult<Vec<Edge>>;

    // ========== Batch Operations ==========

    /// Execute multiple write operations atomically.
    async fn batch_write(&self, ops: Vec<WriteOp>) -> StorageResult<()>;

    // ========== Scan Operations ==========

    /// Scan nodes with optional filter.
    async fn scan_nodes(
        &self,
        filter: Option<NodeFilter>,
    ) -> StorageResult<BoxStream<'_, StorageResult<Node>>>;

    /// Count total nodes.
    async fn count_nodes(&self) -> StorageResult<usize>;

    /// Count total edges.
    async fn count_edges(&self) -> StorageResult<usize>;

    // ========== Metadata Operations ==========

    /// Get metadata value by key.
    async fn get_metadata(&self, key: &str) -> StorageResult<Option<Vec<u8>>>;

    /// Put metadata value.
    async fn put_metadata(&self, key: &str, value: &[u8]) -> StorageResult<()>;

    // ========== Utility ==========

    /// Flush all pending writes to disk.
    async fn flush(&self) -> StorageResult<()>;

    /// Check if the store is closed.
    fn is_closed(&self) -> bool;
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_family_names() {
        assert_eq!(ColumnFamily::Nodes.as_str(), "nodes");
        assert_eq!(ColumnFamily::Edges.as_str(), "edges");
        assert_eq!(ColumnFamily::Metadata.as_str(), "metadata");
    }

    #[test]
    fn test_column_family_from_str() {
        assert_eq!("nodes".parse::<ColumnFamily>().unwrap(), ColumnFamily::Nodes);
        assert_eq!("edges".parse::<ColumnFamily>().unwrap(), ColumnFamily::Edges);
        assert!("unknown".parse::<ColumnFamily>().is_err());
    }

    #[test]
    fn test_node_filter() {
        let filter = NodeFilter::new()
            .with_min_confidence(0.5)
            .with_node_type(synton_core::NodeType::Entity);

        let matching_node = Node::new("Test", synton_core::NodeType::Entity)
            .with_confidence(0.8);
        assert!(filter.matches(&matching_node));

        let non_matching_type = Node::new("Test", synton_core::NodeType::Concept)
            .with_confidence(0.8);
        assert!(!filter.matches(&non_matching_type));

        let non_matching_conf = Node::new("Test", synton_core::NodeType::Entity)
            .with_confidence(0.3);
        assert!(!filter.matches(&non_matching_conf));
    }
}
