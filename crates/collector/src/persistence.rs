//! Persistent storage backend for trace data.

use rocksdb::{Options, DB, ColumnFamilyDescriptor, SingleThreaded};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

use crate::{DEFAULT_EVENT_COLUMN, DEFAULT_SPAN_COLUMN};
use synton_instrument::{TraceEvent, TraceSpan};

/// Storage statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    /// Total number of spans stored.
    pub span_count: usize,

    /// Total number of events stored.
    pub event_count: usize,

    /// Storage size in bytes (if available).
    pub storage_size_bytes: Option<u64>,
}

/// Trait for persistent storage backends.
pub trait PersistenceBackend: Send + Sync {
    /// Store a trace span.
    fn store_span(&self, span: &TraceSpan) -> crate::CollectorResult<()>;

    /// Get a span by ID.
    fn get_span(&self, id: &str) -> crate::CollectorResult<Option<TraceSpan>>;

    /// Store a trace event.
    fn store_event(&self, event: &TraceEvent) -> crate::CollectorResult<()>;

    /// Query spans with filters.
    fn query_spans(&self, filter: &crate::QueryFilter) -> crate::CollectorResult<Vec<TraceSpan>>;

    /// Clear all stored data.
    fn clear(&self) -> crate::CollectorResult<()>;

    /// Flush pending writes.
    fn flush(&self) -> crate::CollectorResult<()>;

    /// Get storage statistics.
    fn stats(&self) -> crate::CollectorResult<StorageStats>;

    /// Close the storage backend.
    fn close(self) -> crate::CollectorResult<()>;
}

/// RocksDB-based persistence implementation.
pub struct RocksDbPersistence {
    /// The RocksDB instance.
    db: Arc<DB>,

    /// Column family for spans.
    _span_cf: Arc<ColumnFamilyDescriptor>,
}

impl RocksDbPersistence {
    /// Create a new RocksDB persistence backend.
    pub fn new(path: impl AsRef<Path>) -> crate::CollectorResult<Self> {
        let path = path.as_ref();

        // Create column families
        let span_cf = ColumnFamilyDescriptor::new(DEFAULT_SPAN_COLUMN, rocksdb::Options::default());
        let event_cf = ColumnFamilyDescriptor::new(DEFAULT_EVENT_COLUMN, rocksdb::Options::default());

        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);

        let db = DB::open_cf(
            &db_opts,
            path,
            &[span_cf.clone(), event_cf.clone()],
        )
        .map_err(|e| crate::CollectorError::RocksDb(e))?;

        Ok(Self {
            db: Arc::new(db),
            _span_cf: Arc::new(span_cf),
        })
    }

    /// Serialize a span for storage.
    fn serialize_span(&self, span: &TraceSpan) -> crate::CollectorResult<Vec<u8>> {
        serde_json::to_vec(span).map_err(Into::into)
    }

    /// Deserialize a span from storage.
    fn deserialize_span(&self, data: &[u8]) -> crate::CollectorResult<TraceSpan> {
        serde_json::from_slice(data).map_err(Into::into)
    }
}

impl PersistenceBackend for RocksDbPersistence {
    fn store_span(&self, span: &TraceSpan) -> crate::CollectorResult<()> {
        let key = span.id.to_string();
        let value = self.serialize_span(span)?;

        self.db
            .put_cf(self._span_cf.as_ref(), key.as_bytes(), value)
            .map_err(Into::into)
    }

    fn get_span(&self, id: &str) -> crate::CollectorResult<Option<TraceSpan>> {
        match self.db.get_cf(self._span_cf.as_ref(), id.as_bytes()) {
            Ok(Some(data)) => {
                let span = self.deserialize_span(&data)?;
                Ok(Some(span))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(crate::CollectorError::RocksDb(e)),
        }
    }

    fn store_event(&self, event: &TraceEvent) -> crate::CollectorResult<()> {
        let key = format!("event_{}", chrono::Utc::now().timestamp_micros());
        let value = serde_json::to_vec(event).map_err(Into::into)?;

        self.db
            .put_cf(self._span_cf.as_ref(), key.as_bytes(), value)
            .map_err(Into::into)
    }

    fn query_spans(&self, _filter: &crate::QueryFilter) -> crate::CollectorResult<Vec<TraceSpan>> {
        // Simple implementation: return all spans
        // A full implementation would use RocksDB iterator with filters
        let mut spans = Vec::new();

        let iter = self.db.iterator_cf(self._span_cf.as_ref(), rocksdb::IteratorMode::Start);

        for item in iter {
            match item {
                Ok((_key, value)) => {
                    if let Ok(span) = self.deserialize_span(&value) {
                        spans.push(span);
                    }
                }
                Err(_) => break,
            }
        }

        Ok(spans)
    }

    fn clear(&self) -> crate::CollectorResult<()> {
        // Clear all column families
        self.db
            .drop_cf(self._span_cf.as_ref())
            .map_err(Into::into)?;

        // Recreate the column family
        let span_cf = ColumnFamilyDescriptor::new(DEFAULT_SPAN_COLUMN, rocksdb::Options::default());
        let _ = self.db.create_cf(&span_cf);

        self._span_cf = Arc::new(span_cf);

        Ok(())
    }

    fn flush(&self) -> crate::CollectorResult<()> {
        self.db
            .flush_wal(true)
            .map_err(Into::into)
    }

    fn stats(&self) -> crate::CollectorResult<StorageStats> {
        let mut span_count = 0;
        let mut event_count = 0;

        let iter = self.db.iterator_cf(self._span_cf.as_ref(), rocksdb::IteratorMode::Start);
        for item in iter {
            match item {
                Ok((_key, _value)) => span_count += 1,
                Err(_) => break,
            }
        }

        Ok(StorageStats {
            span_count,
            event_count,
            storage_size_bytes: None, // Would need live files to calculate
        })
    }

    fn close(self) -> crate::CollectorResult<()> {
        // DB will be closed when dropped
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_persistence() -> (RocksDbPersistence, TempDir) {
        let temp_dir = tempfile::tempdir().unwrap();
        let persistence = RocksDbPersistence::new(temp_dir.path()).unwrap();
        (persistence, temp_dir)
    }

    #[test]
    fn test_store_and_retrieve() {
        let (persistence, _temp_dir) = create_test_persistence();

        let span = TraceSpan::new(
            "test",
            uuid::Uuid::new_v4(),
            None,
            synton_instrument::SpanKind::Function,
            std::collections::HashMap::new(),
        );

        persistence.store_span(&span).unwrap();

        let retrieved = persistence.get_span(&span.id.to_string()).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test");
    }

    #[test]
    fn test_query_empty() {
        let (persistence, _temp_dir) = create_test_persistence();

        let results = persistence.query_spans(&crate::QueryFilter::default()).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_stats() {
        let (persistence, _temp_dir) = create_test_persistence();

        let stats = persistence.stats().unwrap();
        assert_eq!(stats.span_count, 0);
    }
}
