//! Core span and trace data structures for instrumentation.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Kind is defined in this module, not imported

/// Unique identifier for a trace execution.
pub type TraceId = Uuid;

/// Unique identifier for a span within a trace.
pub type SpanId = Uuid;

/// Status of a span execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpanStatus {
    /// Span is currently running.
    Running,
    /// Span completed successfully.
    Completed,
    /// Span failed with an error.
    Failed(String),
    /// Span was cancelled.
    Cancelled,
}

impl SpanStatus {
    /// Returns true if the span represents a successful completion.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Completed)
    }

    /// Returns true if the span represents a failure.
    pub fn is_failure(&self) -> bool {
        matches!(self, Self::Failed(_))
    }

    /// Returns true if the span is still running.
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }
}

impl Default for SpanStatus {
    fn default() -> Self {
        Self::Running
    }
}

/// Kind/classification of a span.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpanKind {
    /// Regular function call.
    Function,
    /// Asynchronous task.
    AsyncTask,
    /// Database query.
    DatabaseQuery,
    /// External API call.
    ExternalCall,
    /// Internal component.
    Internal,
    /// Custom span kind.
    Custom(String),
}

impl Default for SpanKind {
    fn default() -> Self {
        Self::Function
    }
}

impl From<&str> for SpanKind {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "function" => Self::Function,
            "async_task" | "async-task" | "asynctask" => Self::AsyncTask,
            "database_query" | "database-query" | "database" => Self::DatabaseQuery,
            "external_call" | "external-call" | "external" => Self::ExternalCall,
            "internal" => Self::Internal,
            other => Self::Custom(other.to_string()),
        }
    }
}

/// Metadata associated with a span.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanMetadata {
    /// Function name.
    pub function_name: String,

    /// Module path.
    pub module_path: String,

    /// Source file.
    pub file: String,

    /// Line number.
    pub line: u32,

    /// Arguments passed to the function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<HashMap<String, serde_json::Value>>,

    /// Span kind.
    pub kind: SpanKind,
}

impl SpanMetadata {
    /// Create new span metadata.
    pub fn new(
        function_name: String,
        module_path: String,
        file: String,
        line: u32,
        kind: SpanKind,
    ) -> Self {
        Self {
            function_name,
            module_path,
            file,
            line,
            args: None,
            kind,
        }
    }

    /// Set the arguments for this span.
    pub fn with_args(mut self, args: HashMap<String, serde_json::Value>) -> Self {
        self.args = Some(args);
        self
    }
}

/// A single span in a trace tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSpan {
    /// Unique span identifier.
    pub id: SpanId,

    /// Parent span identifier, if any.
    pub parent_id: Option<SpanId>,

    /// Span name (typically function name).
    pub name: String,

    /// Start time of the span.
    pub start_time: DateTime<Utc>,

    /// End time of the span, if completed.
    pub end_time: Option<DateTime<Utc>>,

    /// Duration in milliseconds, if completed.
    pub duration_ms: Option<f64>,

    /// Span status.
    pub status: SpanStatus,

    /// Additional metadata.
    pub metadata: SpanMetadata,

    /// Result value (if any), serialized.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,

    /// Custom attributes.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, serde_json::Value>,

    /// Child span IDs.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<SpanId>,
}

impl TraceSpan {
    /// Create a new trace span.
    pub fn new(
        name: impl Into<String>,
        id: SpanId,
        parent_id: Option<SpanId>,
        kind: SpanKind,
        args: HashMap<String, serde_json::Value>,
    ) -> Self {
        let name = name.into();

        // Try to get caller info from std::any::type_name or use defaults
        let (function_name, module_path, file, line) = Self::caller_info();

        Self {
            id,
            parent_id,
            name,
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
            status: SpanStatus::Running,
            metadata: SpanMetadata {
                function_name,
                module_path,
                file,
                line,
                args: Some(args),
                kind,
            },
            result: None,
            attributes: HashMap::new(),
            children: Vec::new(),
        }
    }

    /// Get caller information using the std::any::type_name fallback.
    fn caller_info() -> (String, String, String, u32) {
        // This is a fallback - the macro should inject actual caller info
        let function_name = std::any::type_name::<()>()
            .split("::")
            .last()
            .unwrap_or("unknown")
            .to_string();
        let module_path = std::any::type_name::<()>().to_string();
        let file = "unknown".to_string();
        let line = 0;

        (function_name, module_path, file, line)
    }

    /// Create with actual caller info (used by the macro).
    pub fn with_caller_info(
        name: impl Into<String>,
        id: SpanId,
        parent_id: Option<SpanId>,
        kind: SpanKind,
        args: HashMap<String, serde_json::Value>,
        function_name: String,
        module_path: String,
        file: String,
        line: u32,
    ) -> Self {
        Self {
            id,
            parent_id,
            name: name.into(),
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
            status: SpanStatus::Running,
            metadata: SpanMetadata {
                function_name,
                module_path,
                file,
                line,
                args: Some(args),
                kind,
            },
            result: None,
            attributes: HashMap::new(),
            children: Vec::new(),
        }
    }

    /// Mark the span as completed with a result.
    pub fn complete(mut self, result: Option<serde_json::Value>) -> Self {
        self.end_time = Some(Utc::now());
        self.duration_ms = Some(
            self.end_time
                .unwrap()
                .signed_duration_since(self.start_time)
                .num_milliseconds() as f64,
        );
        self.status = SpanStatus::Completed;
        self.result = result;
        self
    }

    /// Mark the span as failed with an error.
    pub fn failed(mut self, error: impl Into<String>) -> Self {
        self.end_time = Some(Utc::now());
        self.duration_ms = Some(
            self.end_time
                .unwrap()
                .signed_duration_since(self.start_time)
                .num_milliseconds() as f64,
        );
        self.status = SpanStatus::Failed(error.into());
        self
    }

    /// Cancel the span.
    pub fn cancel(mut self) -> Self {
        self.end_time = Some(Utc::now());
        self.duration_ms = Some(
            self.end_time
                .unwrap()
                .signed_duration_since(self.start_time)
                .num_milliseconds() as f64,
        );
        self.status = SpanStatus::Cancelled;
        self
    }

    /// Add a custom attribute.
    pub fn with_attribute(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.attributes.insert(key.into(), value);
        self
    }

    /// Add a child span ID.
    pub fn add_child(&mut self, child_id: SpanId) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
    }

    /// Get the span duration in milliseconds.
    pub fn duration(&self) -> Option<f64> {
        if let Some(end) = self.end_time {
            Some(end.signed_duration_since(self.start_time).num_milliseconds() as f64)
        } else {
            None
        }
    }

    /// Check if the span is complete.
    pub fn is_complete(&self) -> bool {
        self.end_time.is_some()
    }

    /// Check if the span is a root span (no parent).
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    /// Get the depth of this span in the trace tree.
    pub fn depth(&self, collector: &crate::TraceCollector) -> usize {
        let mut depth = 0;
        let mut current_id = self.parent_id;

        while let Some(parent_id) = current_id {
            depth += 1;
            current_id = collector.get_span(parent_id).and_then(|s| s.parent_id);
        }

        depth
    }
}

/// Trace event captured during execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum TraceEvent {
    /// Span entered.
    Enter {
        span_id: SpanId,
        parent_id: Option<SpanId>,
        name: String,
        timestamp: DateTime<Utc>,
        metadata: SpanMetadata,
    },

    /// Span exited.
    Exit {
        span_id: SpanId,
        timestamp: DateTime<Utc>,
        result: Option<serde_json::Value>,
        duration_ms: f64,
    },

    /// Checkpoint reached.
    Checkpoint {
        span_id: SpanId,
        checkpoint_name: String,
        timestamp: DateTime<Utc>,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<serde_json::Value>,
    },

    /// Error occurred.
    Error {
        span_id: SpanId,
        error: String,
        timestamp: DateTime<Utc>,
        #[serde(skip_serializing_if = "Option::is_none")]
        backtrace: Option<String>,
    },

    /// Custom metric/event.
    Custom {
        span_id: SpanId,
        event_name: String,
        timestamp: DateTime<Utc>,
        data: HashMap<String, serde_json::Value>,
    },
}

impl TraceEvent {
    /// Get the span ID associated with this event.
    pub fn span_id(&self) -> SpanId {
        match self {
            Self::Enter { span_id, .. }
            | Self::Exit { span_id, .. }
            | Self::Checkpoint { span_id, .. }
            | Self::Error { span_id, .. }
            | Self::Custom { span_id, .. } => *span_id,
        }
    }

    /// Get the event timestamp.
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::Enter { timestamp, .. }
            | Self::Exit { timestamp, .. }
            | Self::Checkpoint { timestamp, .. }
            | Self::Error { timestamp, .. }
            | Self::Custom { timestamp, .. } => *timestamp,
        }
    }

    /// Create an enter event.
    pub fn enter(
        span_id: SpanId,
        parent_id: Option<SpanId>,
        name: String,
        metadata: SpanMetadata,
    ) -> Self {
        Self::Enter {
            span_id,
            parent_id,
            name,
            timestamp: Utc::now(),
            metadata,
        }
    }

    /// Create an exit event.
    pub fn exit(span_id: SpanId, result: Option<serde_json::Value>, duration_ms: f64) -> Self {
        Self::Exit {
            span_id,
            timestamp: Utc::now(),
            result,
            duration_ms,
        }
    }

    /// Create a checkpoint event.
    pub fn checkpoint(
        span_id: SpanId,
        checkpoint_name: String,
        data: Option<serde_json::Value>,
    ) -> Self {
        Self::Checkpoint {
            span_id,
            checkpoint_name,
            timestamp: Utc::now(),
            data,
        }
    }

    /// Create an error event.
    pub fn error(span_id: SpanId, error: String, backtrace: Option<String>) -> Self {
        Self::Error {
            span_id,
            timestamp: Utc::now(),
            error,
            backtrace,
        }
    }

    /// Create a custom event.
    pub fn custom(
        span_id: SpanId,
        event_name: String,
        data: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self::Custom {
            span_id,
            event_name,
            timestamp: Utc::now(),
            data,
        }
    }
}

/// Global span ID counter.
static NEXT_SPAN_ID: AtomicU64 = AtomicU64::new(1);

/// Generate a new unique span ID.
pub fn new_span_id() -> SpanId {
    // Use atomic counter for better performance than Uuid::new_v4()
    let counter = NEXT_SPAN_ID.fetch_add(1, Ordering::Relaxed);
    Uuid::from_u64_pair(0, counter)
}

/// Generate a new trace ID.
pub fn new_trace_id() -> TraceId {
    Uuid::new_v4()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_status() {
        assert!(SpanStatus::Running.is_running());
        assert!(SpanStatus::Completed.is_success());
        assert!(SpanStatus::Failed("error".into()).is_failure());
    }

    #[test]
    fn test_span_creation() {
        let span = TraceSpan::new(
            "test_span",
            new_span_id(),
            None,
            SpanKind::Function,
            HashMap::new(),
        );

        assert_eq!(span.name, "test_span");
        assert!(span.parent_id.is_none());
        assert!(span.is_root());
        assert!(span.status.is_running());
    }

    #[test]
    fn test_span_completion() {
        let span = TraceSpan::new(
            "test_span",
            new_span_id(),
            None,
            SpanKind::Function,
            HashMap::new(),
        );

        let completed = span.complete(Some(serde_json::json!("success")));
        assert!(completed.is_complete());
        assert!(completed.status.is_success());
        assert_eq!(completed.result, Some(serde_json::json!("success")));
    }

    #[test]
    fn test_span_with_attribute() {
        let span = TraceSpan::new(
            "test_span",
            new_span_id(),
            None,
            SpanKind::Function,
            HashMap::new(),
        )
        .with_attribute("key", serde_json::json!("value"));

        assert_eq!(
            span.attributes.get("key"),
            Some(&serde_json::json!("value"))
        );
    }

    #[test]
    fn test_span_kind_from_str() {
        assert!(matches!(
            SpanKind::from("function"),
            SpanKind::Function
        ));
        assert!(matches!(
            SpanKind::from("async_task"),
            SpanKind::AsyncTask
        ));
        assert!(matches!(
            SpanKind::from("database"),
            SpanKind::DatabaseQuery
        ));
        assert!(matches!(
            SpanKind::from("custom_type"),
            SpanKind::Custom(s) if s == "custom_type"
        ));
    }

    #[test]
    fn test_new_span_id() {
        let id1 = new_span_id();
        let id2 = new_span_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_new_trace_id() {
        let id1 = new_trace_id();
        let id2 = new_trace_id();
        assert_ne!(id1, id2);
    }
}
