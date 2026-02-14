//! Persistent trace data collector for SYNTON-DB.
//!
//! This crate provides persistent storage for trace data using RocksDB,
//! as well as query interfaces for retrieving stored traces.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub mod persistence;
pub mod query;

// Re-export commonly used types
pub use error::{CollectorError, CollectorResult};
pub use persistence::{PersistenceBackend, RocksDbPersistence};
pub use query::{QueryFilter, TraceQuery};

use synton_instrument::{SpanId, TraceId, TraceSpan};

/// Default path for trace storage.
pub const DEFAULT_TRACE_PATH: &str = "./data/traces";

/// Default column family name for spans.
pub const DEFAULT_SPAN_COLUMN: &str = "spans";

/// Default column family name for events.
pub const DEFAULT_EVENT_COLUMN: &str = "events";

/// Trace collector with persistent storage.
pub struct TraceCollector {
    /// Persistence backend.
    persistence: RocksDbPersistence,

    /// In-memory cache of recent traces.
    cache: once_cell::sync::Lazy<tokio::sync::RwLock<std::collections::HashMap<TraceId, TraceSpan>>>,
}

impl TraceCollector {
    /// Create a new trace collector with default storage path.
    pub fn new() -> anyhow::Result<Self> {
        Self::with_path(DEFAULT_TRACE_PATH)
    }

    /// Create a new trace collector with custom storage path.
    pub fn with_path(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let persistence = RocksDbPersistence::new(path)?;

        Ok(Self {
            persistence,
            cache: once_cell::sync::Lazy::new(Default::default),
        })
    }

    /// Store a trace span.
    pub fn store_span(&self, span: &TraceSpan) -> CollectorResult<()> {
        self.persistence.store_span(span)?;
        self.update_cache(span);
        Ok(())
    }

    /// Store a trace event.
    pub fn store_event(&self, event: &synton_instrument::TraceEvent) -> CollectorResult<()> {
        self.persistence.store_event(event)?;
        Ok(())
    }

    /// Retrieve a trace by ID.
    pub fn get_trace(&self, trace_id: TraceId) -> CollectorResult<Option<TraceSpan>> {
        // First check cache
        if let Ok(cache) = self.cache.read() {
            if let Some(span) = cache.get(&trace_id) {
                return Ok(Some(span.clone()));
            }
        }

        // Then check persistence
        if let Some(span) = self.persistence.get_span(&trace_id.to_string())? {
            self.update_cache(&span);
            Ok(Some(span))
        } else {
            Ok(None)
        }
    }

    /// Query traces with filters.
    pub fn query_traces(&self, filter: &QueryFilter) -> CollectorResult<Vec<TraceSpan>> {
        self.persistence.query_spans(filter)
    }

    /// Get recent traces.
    pub fn recent_traces(&self, limit: usize) -> CollectorResult<Vec<TraceSpan>> {
        let filter = QueryFilter::default().with_limit(limit);
        self.query_traces(&filter)
    }

    /// Clear all stored traces.
    pub fn clear(&self) -> CollectorResult<()> {
        self.persistence.clear()?;
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
        Ok(())
    }

    /// Flush pending writes to disk.
    pub fn flush(&self) -> CollectorResult<()> {
        self.persistence.flush()
    }

    /// Close the collector and release resources.
    pub fn close(self) -> CollectorResult<()> {
        self.persistence.close()
    }

    /// Update the in-memory cache with a span.
    fn update_cache(&self, span: &TraceSpan) {
        if let Ok(mut cache) = self.cache.write() {
            if span.parent_id.is_none() {
                // Only cache root spans
                cache.insert(span.id, span.clone());
            }

            // Limit cache size
            if cache.len() > 1000 {
                // Remove oldest entries
                let keys: Vec<_> = cache.keys().copied().collect();
                for key in keys.iter().take(100) {
                    cache.remove(key);
                }
            }
        }
    }

    /// Get the number of traces in storage.
    pub fn count(&self) -> CollectorResult<usize> {
        self.persistence.count()
    }

    /// Get storage statistics.
    pub fn stats(&self) -> CollectorResult<persistence::StorageStats> {
        self.persistence.stats()
    }
}

impl Default for TraceCollector {
    fn default() -> Self {
        Self::new().expect("Failed to create trace collector with default path")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collector_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let collector = TraceCollector::with_path(temp_dir.path());
        assert!(collector.is_ok());
    }

    #[test]
    fn test_store_and_retrieve() {
        let temp_dir = tempfile::tempdir().unwrap();
        let collector = TraceCollector::with_path(temp_dir.path()).unwrap();

        let span = TraceSpan::new(
            "test_span",
            uuid::Uuid::new_v4(),
            None,
            synton_instrument::SpanKind::Function,
            std::collections::HashMap::new(),
        );

        collector.store_span(&span).unwrap();

        let retrieved = collector.get_trace(&span.id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test_span");
    }
}
