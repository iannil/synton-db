// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use futures::{stream::BoxStream, StreamExt};
use uuid::Uuid;

use crate::{ColumnFamily, StorageError, StorageResult, Store, WriteOp};
use synton_core::{Edge, Node};

/// RocksDB configuration.
#[derive(Debug, Clone)]
pub struct RocksdbConfig {
    pub path: String,
    pub write_buffer_size: usize,
    pub max_write_buffers: usize,
    pub max_background_jobs: i32,
    pub create_if_missing: bool,
    pub create_missing_column_families: bool,
    pub compression: RocksdbCompression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RocksdbCompression {
    NoCompression,
    Snappy,
    Zlib,
    Bz2,
    Lz4,
    Lz4hc,
    Zstd,
}

impl Default for RocksdbConfig {
    fn default() -> Self {
        Self {
            path: "./data/synton-db".to_string(),
            write_buffer_size: 512 * 1024 * 1024,
            max_write_buffers: 4,
            max_background_jobs: 4,
            create_if_missing: true,
            create_missing_column_families: true,
            compression: RocksdbCompression::Lz4,
        }
    }
}

/// RocksDB implementation of the Store trait.
///
/// Note: This implementation uses `unsafe` to extend ColumnFamily lifetimes.
/// The ColumnFamily handles are owned by the DB and live as long as the DB lives.
pub struct RocksdbStore {
    db: Arc<rocksdb::DB>,
    config: RocksdbConfig,
}

impl RocksdbStore {
    pub fn open(config: RocksdbConfig) -> StorageResult<Self> {
        let cf_names: Vec<_> = ColumnFamily::ALL
            .iter()
            .map(|cf| cf.as_str().to_string())
            .collect();

        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(config.create_if_missing);
        opts.create_missing_column_families(config.create_missing_column_families);
        opts.set_write_buffer_size(config.write_buffer_size);
        opts.set_max_write_buffer_number(config.max_write_buffers as i32);
        opts.set_max_background_jobs(config.max_background_jobs);
        opts.set_max_open_files(5000);

        match config.compression {
            RocksdbCompression::NoCompression => {
                opts.set_compression_type(rocksdb::DBCompressionType::None);
            }
            RocksdbCompression::Snappy => {
                opts.set_compression_type(rocksdb::DBCompressionType::Snappy);
            }
            RocksdbCompression::Zlib => {
                opts.set_compression_type(rocksdb::DBCompressionType::Zlib);
            }
            RocksdbCompression::Bz2 => {
                opts.set_compression_type(rocksdb::DBCompressionType::Bz2);
            }
            RocksdbCompression::Lz4 => {
                opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
            }
            RocksdbCompression::Lz4hc => {
                opts.set_compression_type(rocksdb::DBCompressionType::Lz4hc);
            }
            RocksdbCompression::Zstd => {
                opts.set_compression_type(rocksdb::DBCompressionType::Zstd);
            }
        }

        let db = rocksdb::DB::open_cf_with_opts(
            &opts,
            &config.path,
            cf_names.iter().map(|name| {
                let mut cf_opts = rocksdb::Options::default();
                cf_opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
                (name.clone(), cf_opts)
            }),
        )
        .map_err(|e| StorageError::Rocksdb(e.to_string()))?;

        Ok(Self { db: Arc::new(db), config })
    }

    pub fn open_path<P: AsRef<Path>>(path: P) -> StorageResult<Self> {
        let config = RocksdbConfig {
            path: path.as_ref().to_str().unwrap().to_string(),
            ..Default::default()
        };
        Self::open(config)
    }

    /// Get a column family handle.
    /// 
    /// # Safety
    /// The returned reference is valid for as long as the DB lives.
    /// Since self.db is Arc<DB>, the DB won't be dropped while self is alive.
    fn cf(&self, cf: ColumnFamily) -> StorageResult<&rocksdb::ColumnFamily> {
        self.db
            .cf_handle(cf.as_str())
            .ok_or_else(|| {
                StorageError::InvalidOperation(format!("Column family {} not found", cf))
            })
    }

    fn serialize_node(node: &Node) -> StorageResult<Vec<u8>> {
        serde_json::to_vec(node).map_err(|e| StorageError::Serialization(e.to_string()))
    }

    fn deserialize_node(bytes: &[u8]) -> StorageResult<Node> {
        serde_json::from_slice(bytes).map_err(|e| StorageError::Deserialization(e.to_string()))
    }

    fn serialize_edge(edge: &Edge) -> StorageResult<Vec<u8>> {
        serde_json::to_vec(edge).map_err(|e| StorageError::Serialization(e.to_string()))
    }

    fn deserialize_edge(bytes: &[u8]) -> StorageResult<Edge> {
        serde_json::from_slice(bytes).map_err(|e| StorageError::Deserialization(e.to_string()))
    }
}

// Make the store Send + Sync
unsafe impl Send for RocksdbStore {}
unsafe impl Sync for RocksdbStore {}

#[async_trait]
impl Store for RocksdbStore {
    async fn get_node(&self, id: Uuid) -> StorageResult<Option<Node>> {
        let cf = self.cf(ColumnFamily::Nodes)?;
        match self.db.get_cf(cf, id.as_bytes()) {
            Ok(Some(bytes)) => Ok(Some(Self::deserialize_node(&bytes)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(StorageError::Rocksdb(e.to_string())),
        }
    }

    async fn put_node(&self, node: &Node) -> StorageResult<()> {
        let cf = self.cf(ColumnFamily::Nodes)?;
        let value = Self::serialize_node(node)?;
        self.db
            .put_cf(cf, node.id.as_bytes(), value)
            .map_err(|e| StorageError::Rocksdb(e.to_string()))
    }

    async fn delete_node(&self, id: Uuid) -> StorageResult<bool> {
        if !self.node_exists(id).await? {
            return Ok(false);
        }
        let cf = self.cf(ColumnFamily::Nodes)?;
        self.db
            .delete_cf(cf, id.as_bytes())
            .map_err(|e| StorageError::Rocksdb(e.to_string()))?;
        Ok(true)
    }

    async fn node_exists(&self, id: Uuid) -> StorageResult<bool> {
        let cf = self.cf(ColumnFamily::Nodes)?;
        self.db
            .get_cf(cf, id.as_bytes())
            .map(|v| v.is_some())
            .map_err(|e| StorageError::Rocksdb(e.to_string()))
    }

    async fn get_edge(
        &self,
        source: Uuid,
        target: Uuid,
        relation: &str,
    ) -> StorageResult<Option<Edge>> {
        let cf = self.cf(ColumnFamily::Edges)?;
        let key = format!("{}::{}::{}", source, target, relation);
        match self.db.get_cf(cf, key.as_bytes()) {
            Ok(Some(bytes)) => Ok(Some(Self::deserialize_edge(&bytes)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(StorageError::Rocksdb(e.to_string())),
        }
    }

    async fn put_edge(&self, edge: &Edge) -> StorageResult<()> {
        let cf = self.cf(ColumnFamily::Edges)?;
        let value = Self::serialize_edge(edge)?;
        let key = edge.id();
        self.db
            .put_cf(cf, key.as_bytes(), value)
            .map_err(|e| StorageError::Rocksdb(e.to_string()))
    }

    async fn delete_edge(
        &self,
        source: Uuid,
        target: Uuid,
        relation: &str,
    ) -> StorageResult<bool> {
        let cf = self.cf(ColumnFamily::Edges)?;
        let key = format!("{}::{}::{}", source, target, relation);
        self.db
            .delete_cf(cf, key.as_bytes())
            .map_err(|e| StorageError::Rocksdb(e.to_string()))?;
        Ok(true)
    }

    async fn get_outgoing_edges(&self, source: Uuid) -> StorageResult<Vec<Edge>> {
        let cf = self.cf(ColumnFamily::Edges)?;
        let prefix = format!("{}::", source);
        let mut edges = Vec::new();

        let iter = self.db.prefix_iterator_cf(cf, prefix.as_bytes());
        for item in iter {
            let (_, bytes) = item.map_err(|e| StorageError::Rocksdb(e.to_string()))?;
            if let Ok(edge) = Self::deserialize_edge(&bytes) {
                edges.push(edge);
            }
        }

        Ok(edges)
    }

    async fn get_incoming_edges(&self, target: Uuid) -> StorageResult<Vec<Edge>> {
        let cf = self.cf(ColumnFamily::Edges)?;
        let target_str = target.to_string();
        let mut edges = Vec::new();

        let iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::Start);
        for item in iter {
            let (key, bytes) = item.map_err(|e| StorageError::Rocksdb(e.to_string()))?;
            let key_str = String::from_utf8_lossy(&key);
            if key_str.split("::").nth(1) == Some(&target_str) {
                if let Ok(edge) = Self::deserialize_edge(&bytes) {
                    edges.push(edge);
                }
            }
        }

        Ok(edges)
    }

    async fn batch_write(&self, ops: Vec<WriteOp>) -> StorageResult<()> {
        let nodes_cf = self.cf(ColumnFamily::Nodes)?;
        let edges_cf = self.cf(ColumnFamily::Edges)?;
        
        let mut batch = rocksdb::WriteBatch::default();

        for op in ops {
            match op {
                WriteOp::PutNode(node) => {
                    let value = Self::serialize_node(&node)?;
                    batch.put_cf(nodes_cf, node.id.as_bytes(), value);
                }
                WriteOp::DeleteNode(id) => {
                    batch.delete_cf(nodes_cf, id.as_bytes());
                }
                WriteOp::PutEdge(edge) => {
                    let value = Self::serialize_edge(&edge)?;
                    let key = edge.id();
                    batch.put_cf(edges_cf, key.as_bytes(), value);
                }
                WriteOp::DeleteEdge(source, target, relation) => {
                    let key = format!("{}::{}::{}", source, target, relation);
                    batch.delete_cf(edges_cf, key.as_bytes());
                }
                WriteOp::Put { cf, key, value } => {
                    let cf_handle = self.cf(cf)?;
                    batch.put_cf(cf_handle, key, value);
                }
                WriteOp::Delete { cf, key } => {
                    let cf_handle = self.cf(cf)?;
                    batch.delete_cf(cf_handle, key);
                }
            }
        }

        self.db
            .write(batch)
            .map_err(|e| StorageError::Rocksdb(e.to_string()))
    }

    async fn scan_nodes(
        &self,
        filter: Option<crate::store::NodeFilter>,
    ) -> StorageResult<BoxStream<'_, StorageResult<Node>>> {
        let cf = self.cf(ColumnFamily::Nodes)?;
        let iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::Start);
        let mut nodes = Vec::new();

        for item in iter {
            let (_, bytes) = item.map_err(|e| StorageError::Rocksdb(e.to_string()))?;
            let node = Self::deserialize_node(&bytes)?;

            if let Some(ref f) = filter {
                if !f.matches(&node) {
                    continue;
                }
            }

            nodes.push(node);
        }

        let stream = futures::stream::iter(nodes.into_iter().map(Ok)).boxed();
        Ok(stream)
    }

    async fn count_nodes(&self) -> StorageResult<usize> {
        let _cf = self.cf(ColumnFamily::Nodes)?;
        Ok(0)
    }

    async fn count_edges(&self) -> StorageResult<usize> {
        let _cf = self.cf(ColumnFamily::Edges)?;
        Ok(0)
    }

    async fn get_metadata(&self, key: &str) -> StorageResult<Option<Vec<u8>>> {
        let cf = self.cf(ColumnFamily::Metadata)?;
        self.db
            .get_cf(cf, key.as_bytes())
            .map_err(|e| StorageError::Rocksdb(e.to_string()))
    }

    async fn put_metadata(&self, key: &str, value: &[u8]) -> StorageResult<()> {
        let cf = self.cf(ColumnFamily::Metadata)?;
        self.db
            .put_cf(cf, key.as_bytes(), value)
            .map_err(|e| StorageError::Rocksdb(e.to_string()))
    }

    async fn flush(&self) -> StorageResult<()> {
        self.db
            .flush()
            .map_err(|e| StorageError::Rocksdb(e.to_string()))
    }

    fn is_closed(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use synton_core::{NodeType, Relation};

    #[tokio::test]
    async fn test_rocksdb_store_basic() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = RocksdbConfig {
            path: temp_dir.path().to_str().unwrap().to_string(),
            ..Default::default()
        };

        let store = RocksdbStore::open(config).unwrap();

        let node = Node::new("Test content", NodeType::Entity);
        let id = node.id;

        store.put_node(&node).await.unwrap();
        let retrieved = store.get_node(id).await.unwrap().unwrap();

        assert_eq!(retrieved.content(), "Test content");
        assert_eq!(retrieved.node_type, NodeType::Entity);
    }

    #[tokio::test]
    async fn test_rocksdb_edge_operations() {
        let temp_dir = tempfile::tempdir().unwrap();
        let store = RocksdbStore::open_path(temp_dir.path()).unwrap();

        let source = Uuid::new_v4();
        let target = Uuid::new_v4();
        let edge = Edge::new(source, target, Relation::Causes);

        store.put_edge(&edge).await.unwrap();

        let retrieved = store
            .get_edge(source, target, "causes")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(retrieved.source, source);
        assert_eq!(retrieved.target, target);
    }

    #[tokio::test]
    async fn test_rocksdb_metadata() {
        let temp_dir = tempfile::tempdir().unwrap();
        let store = RocksdbStore::open_path(temp_dir.path()).unwrap();

        store.put_metadata("test_key", b"test_value").await.unwrap();

        let value = store.get_metadata("test_key").await.unwrap();
        assert_eq!(value, Some(b"test_value".to_vec()));
    }
}
