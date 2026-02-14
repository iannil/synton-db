//! Query interface for trace data.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use synton_instrument::{SpanKind, TraceSpan};

/// Filter for querying traces.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryFilter {
    /// Filter by trace ID.
    pub trace_id: Option<String>,

    /// Filter by span name.
    pub name: Option<String>,

    /// Filter by span kind.
    pub kind: Option<SpanKind>,

    /// Filter by minimum start time.
    pub start_after: Option<DateTime<Utc>>,

    /// Filter by maximum end time.
    pub end_before: Option<DateTime<Utc>>,

    /// Filter by minimum duration (milliseconds).
    pub min_duration_ms: Option<f64>,

    /// Filter by maximum duration (milliseconds).
    pub max_duration_ms: Option<f64>,

    /// Maximum number of results.
    pub limit: Option<usize>,

    /// Offset for pagination.
    pub offset: Option<usize>,
}

impl QueryFilter {
    /// Create a new filter builder.
    pub fn builder() -> QueryFilterBuilder {
        QueryFilterBuilder::default()
    }

    /// Add a trace ID filter.
    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }

    /// Add a name filter.
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Add a kind filter.
    pub fn with_kind(mut self, kind: SpanKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Add a start time filter.
    pub fn with_start_after(mut self, start: DateTime<Utc>) -> Self {
        self.start_after = Some(start);
        self
    }

    /// Add an end time filter.
    pub fn with_end_before(mut self, end: DateTime<Utc>) -> Self {
        self.end_before = Some(end);
        self
    }

    /// Add a duration range filter.
    pub fn with_duration_range(mut self, min: f64, max: f64) -> Self {
        self.min_duration_ms = Some(min);
        self.max_duration_ms = Some(max);
        self
    }

    /// Add a limit.
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Add an offset.
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Check if a span matches this filter.
    pub fn matches(&self, span: &TraceSpan) -> bool {
        // Check trace ID
        if let Some(ref trace_id) = self.trace_id {
            if span.id.to_string() != *trace_id {
                return false;
            }
        }

        // Check name
        if let Some(ref name) = self.name {
            if !span.name.contains(name) {
                return false;
            }
        }

        // Check kind
        if let Some(ref kind) = self.kind {
            if span.metadata.kind != *kind {
                return false;
            }
        }

        // Check start time
        if let Some(ref start) = self.start_after {
            if span.start_time < *start {
                return false;
            }
        }

        // Check end time
        if let Some(ref end) = self.end_before {
            if span.end_time.map_or(Utc::now(), |t| t) > *end {
                return false;
            }
        }

        // Check duration
        if let (Some(min), Some(max)) = (self.min_duration_ms, self.max_duration_ms) {
            if let Some(duration) = span.duration_ms {
                if duration < min || duration > max {
                    return false;
                }
            }
        }

        true
    }
}

/// Builder for creating query filters.
#[derive(Debug, Clone, Default)]
pub struct QueryFilterBuilder {
    filter: QueryFilter,
}

impl QueryFilterBuilder {
    /// Build the filter.
    pub fn build(self) -> QueryFilter {
        self.filter
    }

    /// Set trace ID filter.
    pub fn trace_id(mut self, trace_id: String) -> Self {
        self.filter.trace_id = Some(trace_id);
        self
    }

    /// Set name filter.
    pub fn name(mut self, name: String) -> Self {
        self.filter.name = Some(name);
        self
    }

    /// Set kind filter.
    pub fn kind(mut self, kind: SpanKind) -> Self {
        self.filter.kind = Some(kind);
        self
    }

    /// Set time range filter.
    pub fn time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.filter.start_after = Some(start);
        self.filter.end_before = Some(end);
        self
    }

    /// Set duration range filter.
    pub fn duration_range(mut self, min: f64, max: f64) -> Self {
        self.filter.min_duration_ms = Some(min);
        self.filter.max_duration_ms = Some(max);
        self
    }

    /// Set limit.
    pub fn limit(mut self, limit: usize) -> Self {
        self.filter.limit = Some(limit);
        self
    }

    /// Set offset.
    pub fn offset(mut self, offset: usize) -> Self {
        self.filter.offset = Some(offset);
        self
    }
}

/// Trait for querying trace data.
pub trait TraceQuery: Send + Sync {
    /// Execute a query and return matching spans.
    fn query(&self, filter: &QueryFilter) -> crate::CollectorResult<Vec<TraceSpan>>;

    /// Get a single trace by ID.
    fn get(&self, trace_id: &str) -> crate::CollectorResult<Option<TraceSpan>>;

    /// Count matching traces.
    fn count(&self, filter: &QueryFilter) -> crate::CollectorResult<usize>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_builder() {
        let filter = QueryFilter::builder()
            .name("test".to_string())
            .limit(10)
            .build();

        assert_eq!(filter.name, Some("test".to_string()));
        assert_eq!(filter.limit, Some(10));
    }

    #[test]
    fn test_filter_matches() {
        let span = TraceSpan::new(
            "test_function",
            Uuid::new_v4(),
            None,
            SpanKind::Function,
            std::collections::HashMap::new(),
        );

        let filter = QueryFilter {
            name: Some("test".to_string()),
            ..Default::default()
        };

        assert!(filter.matches(&span));

        let wrong_kind_filter = QueryFilter {
            kind: Some(SpanKind::AsyncTask),
            ..Default::default()
        };

        assert!(!wrong_kind_filter.matches(&span));
    }
}
