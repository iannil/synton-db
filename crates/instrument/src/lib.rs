//! # SYNTON-DB Instrumentation System
//!
//! This crate provides comprehensive instrumentation and tracing capabilities
//! for the SYNTON-DB cognitive database. It offers:
//!
//! - Function-level tracing via `#[trace]` macro
//! - Checkpoint recording via `#[checkpoint]` macro
//! - In-memory trace collection and aggregation
//! - Lifecycle, timeline, and statistics views
//! - Export to JSON and Mermaid formats
//!
//! ## Quick Start
//!
//! ```rust
//! use synton_instrument::{trace, TraceCollector};
//!
//! #[trace]
//! async fn process_data(input: &str) -> Result<Vec<String>> {
//!     // Function is automatically traced
//!     Ok(vec![input.to_string()])
//! }
//!
//! // Get the global collector
//! let collector = TraceCollector::global();
//!
//! // Query trace data
//! let lifecycle = collector.get_lifecycle(trace_id);
//! let timeline = collector.get_timeline(trace_id);
//! ```
//!
//! ## Macros
//!
//! ### `#[trace]`
//!
//! Automatically instrument a function:
//!
//! ```rust
//! #[trace]
//! fn my_function(x: i32, y: i32) -> i32 {
//!     x + y
//! }
//! ```
//!
//! With options:
//!
//! ```rust
//! #[trace(name = "custom_name")]
//! #[trace(skip_args)]
//! #[trace(skip = ["sensitive_arg"])]
//! #[trace(kind = "database_query")]
//! async fn query_db(conn: &str, sensitive_arg: &str) -> Result<Vec<u8>> {
//!     // ...
//! }
//! ```
//!
//! ### `#[checkpoint]`
//!
//! Record execution checkpoints:
//!
//! ```rust
//! #[checkpoint]
//! fn checkpoint_name() {
//!     // Record checkpoint
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod collector;
pub mod span;
pub mod statistics;
pub mod views;

// Re-export commonly used types
pub use collector::{CollectorConfig, TraceCollector};
pub use span::{
    new_span_id, new_trace_id, SpanId, SpanKind, SpanMetadata, SpanStatus, TraceEvent, TraceId,
    TraceSpan,
};
pub use statistics::{SpanNameStats, StatisticsManager, TimeWindowStats, TraceMetadata};
pub use views::{DashboardStats, DurationRecord, ExportFormat, LifecycleView, Statistics, TimelineView, TraceSummary};

// Re-export macros from the macro crate
pub use synton_instrument_macros::{checkpoint, trace, TraceMetadata};

// Global collector accessor
static COLLECTOR: once_cell::sync::Lazy<TraceCollector> =
    once_cell::sync::Lazy::new(TraceCollector::new);

/// Get the global trace collector instance.
pub fn global_collector() -> &'static TraceCollector {
    &COLLECTOR
}

/// Initialize the instrumentation system with custom config.
pub fn init(config: CollectorConfig) {
    global_collector().update_config(config);
}

/// Reset the instrumentation system (clears all data).
pub fn reset() {
    global_collector().clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::SpanMetadata;

    fn create_test_span(name: &str) -> TraceSpan {
        TraceSpan::new(
            name,
            new_span_id(),
            None,
            SpanKind::Function,
            std::collections::HashMap::new(),
        )
    }

    #[test]
    fn test_global_collector() {
        let collector = global_collector();
        assert_eq!(collector.span_count(), 0);
    }

    #[test]
    fn test_reset() {
        let collector = global_collector();

        let metadata = SpanMetadata::new(
            "test".into(),
            "module".into(),
            "file.rs".into(),
            1,
            SpanKind::Function,
        );

        let span_id = collector.enter_span("test".into(), None, metadata);
        assert_eq!(collector.span_count(), 1);

        reset();
        assert_eq!(collector.span_count(), 0);
    }

    #[test]
    fn test_trace_export_json() {
        let collector = global_collector();

        let metadata = SpanMetadata::new(
            "test".into(),
            "module".into(),
            "file.rs".into(),
            1,
            SpanKind::Function,
        );

        let span_id = collector.enter_span("root".into(), None, metadata);
        collector.complete_span(span_id, None, 100.0);

        let json = collector.export_json(span_id);
        assert!(json.is_some());
    }

    #[test]
    fn test_trace_export_mermaid() {
        let collector = global_collector();

        let metadata = SpanMetadata::new(
            "test".into(),
            "module".into(),
            "file.rs".into(),
            1,
            SpanKind::Function,
        );

        let span_id = collector.enter_span("root".into(), None, metadata);
        collector.complete_span(span_id, None, 100.0);

        let mermaid = collector.export_mermaid(span_id);
        assert!(mermaid.is_some());
        assert!(mermaid.unwrap().starts_with("flowchart TD"));
    }

    #[test]
    fn test_statistics() {
        let collector = global_collector();
        let stats = collector.get_statistics();
        assert_eq!(stats.active_spans, 0);
    }

    #[test]
    fn test_export_format() {
        assert_eq!(ExportFormat::from_str("json"), Some(ExportFormat::Json));
        assert_eq!(ExportFormat::from_str("mermaid"), Some(ExportFormat::Mermaid));
        assert_eq!(ExportFormat::from_str("text"), Some(ExportFormat::Text));
        assert_eq!(ExportFormat::from_str("unknown"), None);

        assert_eq!(ExportFormat::Json.as_str(), "json");
        assert_eq!(ExportFormat::Mermaid.as_str(), "mermaid");
        assert_eq!(ExportFormat::Text.as_str(), "text");
    }

    #[test]
    fn test_span_kind_conversion() {
        assert!(matches!(SpanKind::from("function"), SpanKind::Function));
        assert!(matches!(SpanKind::from("async_task"), SpanKind::AsyncTask));
        assert!(matches!(SpanKind::from("database"), SpanKind::DatabaseQuery));
        assert!(matches!(SpanKind::from("external"), SpanKind::ExternalCall));
    }
}
