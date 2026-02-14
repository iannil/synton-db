//! View types for trace data visualization.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::span::TraceEvent;

/// Lifecycle view of a trace execution.
///
/// Shows the hierarchical structure of spans with timing and status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleView {
    /// Unique identifier for this span.
    pub id: String,

    /// Span name (typically function name).
    pub name: String,

    /// Start timestamp.
    pub start_time: DateTime<Utc>,

    /// End timestamp, if completed.
    pub end_time: Option<DateTime<Utc>>,

    /// Duration in milliseconds.
    pub duration_ms: f64,

    /// Status of the span.
    pub status: String,

    /// Child spans (nested calls).
    pub children: Vec<LifecycleView>,

    /// Arguments passed to the function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<serde_json::Value>,

    /// Return value, if completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
}

impl LifecycleView {
    /// Count total spans in this lifecycle (including children).
    pub fn span_count(&self) -> usize {
        1 + self.children.iter().map(|c| c.span_count()).sum::<usize>()
    }

    /// Get the maximum depth of this lifecycle.
    pub fn max_depth(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self.children.iter().map(|c| c.max_depth()).max().unwrap_or(0)
        }
    }

    /// Get all spans as a flat list.
    pub fn flatten(&self) -> Vec<&LifecycleView> {
        let mut result = vec![self];
        for child in &self.children {
            result.extend(child.flatten());
        }
        result
    }

    /// Find a span by ID.
    pub fn find_by_id(&self, id: &str) -> Option<&LifecycleView> {
        if self.id == id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_by_id(id) {
                return Some(found);
            }
        }
        None
    }
}

/// Timeline view of trace events.
///
/// Shows events in chronological order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineView {
    /// Trace identifier.
    pub trace_id: String,

    /// Events in chronological order.
    pub events: Vec<TraceEvent>,

    /// Total duration of the trace in milliseconds.
    pub total_duration_ms: f64,
}

impl TimelineView {
    /// Filter events by type.
    pub fn filter_by_type(&self, event_type: &str) -> Vec<&TraceEvent> {
        self.events
            .iter()
            .filter(|e| {
                match e {
                    TraceEvent::Enter { .. } => event_type == "enter",
                    TraceEvent::Exit { .. } => event_type == "exit",
                    TraceEvent::Checkpoint { .. } => event_type == "checkpoint",
                    TraceEvent::Error { .. } => event_type == "error",
                    TraceEvent::Custom { .. } => event_type == "custom",
                    _ => false,
                }
            })
            .collect()
    }

    /// Get only checkpoint events.
    pub fn checkpoints(&self) -> Vec<&TraceEvent> {
        self.filter_by_type("checkpoint")
    }

    /// Get only error events.
    pub fn errors(&self) -> Vec<&TraceEvent> {
        self.filter_by_type("error")
    }

    /// Get events within a time range.
    pub fn in_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&TraceEvent> {
        self.events
            .iter()
            .filter(|e| {
                let ts = e.timestamp();
                ts >= start && ts <= end
            })
            .collect()
    }

    /// Count events by type.
    pub fn count_by_type(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        for event in &self.events {
            let type_name = match event {
                TraceEvent::Enter { .. } => "enter",
                TraceEvent::Exit { .. } => "exit",
                TraceEvent::Checkpoint { .. } => "checkpoint",
                TraceEvent::Error { .. } => "error",
                TraceEvent::Custom { .. } => "custom",
            };
            *counts.entry(type_name.to_string()).or_insert(0) += 1;
        }
        counts
    }
}

/// Statistics view for the collector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    /// Total spans created.
    pub spans_created: u64,

    /// Total spans completed successfully.
    pub spans_completed: u64,

    /// Total spans that failed.
    pub spans_failed: u64,

    /// Total checkpoints recorded.
    pub checkpoints: u64,

    /// Currently active (running) spans.
    pub active_spans: usize,

    /// Total events recorded.
    pub total_events: usize,
}

impl Statistics {
    /// Calculate the success rate (completed / total completed or failed).
    pub fn success_rate(&self) -> f64 {
        let total = self.spans_completed + self.spans_failed;
        if total == 0 {
            1.0
        } else {
            self.spans_completed as f64 / total as f64
        }
    }

    /// Calculate the failure rate.
    pub fn failure_rate(&self) -> f64 {
        1.0 - self.success_rate()
    }

    /// Get total processed spans.
    pub fn total_processed(&self) -> u64 {
        self.spans_completed + self.spans_failed
    }
}

/// Dashboard statistics for visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    /// Basic statistics.
    pub stats: Statistics,

    /// Top spans by duration.
    pub top_duration_spans: Vec<DurationRecord>,

    /// Recent traces.
    pub recent_traces: Vec<TraceSummary>,
}

/// A span with its duration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationRecord {
    /// Span ID.
    pub span_id: String,

    /// Span name.
    pub name: String,

    /// Duration in milliseconds.
    pub duration_ms: f64,

    /// Status.
    pub status: String,
}

/// Summary of a trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSummary {
    /// Trace ID.
    pub trace_id: String,

    /// Root span name.
    pub name: String,

    /// Total duration in milliseconds.
    pub duration_ms: f64,

    /// Number of spans in the trace.
    pub span_count: usize,

    /// Start time.
    pub start_time: DateTime<Utc>,

    /// Overall status (success if no errors).
    pub has_errors: bool,
}

/// Export format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// JSON format.
    Json,
    /// Mermaid flowchart.
    Mermaid,
    /// Plain text.
    Text,
}

impl ExportFormat {
    /// Parse from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "mermaid" => Some(Self::Mermaid),
            "text" | "txt" => Some(Self::Text),
            _ => None,
        }
    }

    /// Convert to string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Mermaid => "mermaid",
            Self::Text => "text",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_span_count() {
        let view = LifecycleView {
            id: "1".to_string(),
            name: "root".to_string(),
            start_time: Utc::now(),
            end_time: None,
            duration_ms: 100.0,
            status: "running".to_string(),
            children: vec![
                LifecycleView {
                    id: "2".to_string(),
                    name: "child1".to_string(),
                    start_time: Utc::now(),
                    end_time: None,
                    duration_ms: 50.0,
                    status: "running".to_string(),
                    children: vec![],
                    args: None,
                    result: None,
                },
                LifecycleView {
                    id: "3".to_string(),
                    name: "child2".to_string(),
                    start_time: Utc::now(),
                    end_time: None,
                    duration_ms: 30.0,
                    status: "running".to_string(),
                    children: vec![],
                    args: None,
                    result: None,
                },
            ],
            args: None,
            result: None,
        };

        assert_eq!(view.span_count(), 3);
    }

    #[test]
    fn test_lifecycle_max_depth() {
        let view = LifecycleView {
            id: "1".to_string(),
            name: "root".to_string(),
            start_time: Utc::now(),
            end_time: None,
            duration_ms: 100.0,
            status: "running".to_string(),
            children: vec![
                LifecycleView {
                    id: "2".to_string(),
                    name: "child1".to_string(),
                    start_time: Utc::now(),
                    end_time: None,
                    duration_ms: 50.0,
                    status: "running".to_string(),
                    children: vec![
                        LifecycleView {
                            id: "4".to_string(),
                            name: "grandchild".to_string(),
                            start_time: Utc::now(),
                            end_time: None,
                            duration_ms: 20.0,
                            status: "running".to_string(),
                            children: vec![],
                            args: None,
                            result: None,
                        },
                    ],
                    args: None,
                    result: None,
                },
                LifecycleView {
                    id: "3".to_string(),
                    name: "child2".to_string(),
                    start_time: Utc::now(),
                    end_time: None,
                    duration_ms: 30.0,
                    status: "running".to_string(),
                    children: vec![],
                    args: None,
                    result: None,
                },
            ],
            args: None,
            result: None,
        };

        assert_eq!(view.max_depth(), 3);
    }

    #[test]
    fn test_statistics_success_rate() {
        let stats = Statistics {
            spans_created: 100,
            spans_completed: 80,
            spans_failed: 20,
            checkpoints: 50,
            active_spans: 0,
            total_events: 150,
        };

        assert_eq!(stats.success_rate(), 0.8);
        assert_eq!(stats.failure_rate(), 0.2);
        assert_eq!(stats.total_processed(), 100);
    }
}
