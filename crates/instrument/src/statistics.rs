//! Statistics aggregation and analysis for trace data.

use std::collections::HashMap;
use std::sync::RwLock;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::span::{SpanId, TraceSpan, SpanKind};

/// Aggregated statistics for a time window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindowStats {
    /// Start of the window.
    pub window_start: DateTime<Utc>,

    /// End of the window.
    pub window_end: DateTime<Utc>,

    /// Number of spans completed in this window.
    pub spans_completed: u64,

    /// Number of spans failed in this window.
    pub spans_failed: u64,

    /// Average duration in milliseconds.
    pub avg_duration_ms: f64,

    /// Minimum duration in milliseconds.
    pub min_duration_ms: f64,

    /// Maximum duration in milliseconds.
    pub max_duration_ms: f64,

    /// P50 (median) duration in milliseconds.
    pub p50_duration_ms: f64,

    /// P95 duration in milliseconds.
    pub p95_duration_ms: f64,

    /// P99 duration in milliseconds.
    pub p99_duration_ms: f64,
}

impl TimeWindowStats {
    /// Create empty stats for a time window.
    pub fn new(window_start: DateTime<Utc>, window_end: DateTime<Utc>) -> Self {
        Self {
            window_start,
            window_end,
            spans_completed: 0,
            spans_failed: 0,
            avg_duration_ms: 0.0,
            min_duration_ms: f64::MAX,
            max_duration_ms: 0.0,
            p50_duration_ms: 0.0,
            p95_duration_ms: 0.0,
            p99_duration_ms: 0.0,
        }
    }

    /// Calculate percentiles from a list of durations.
    pub fn calculate_percentiles(durations: &mut Vec<f64>) -> (f64, f64, f64) {
        if durations.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        durations.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let len = durations.len();
        let p50_idx = (len as f64 * 0.5).floor() as usize;
        let p95_idx = (len as f64 * 0.95).floor() as usize;
        let p99_idx = (len as f64 * 0.99).floor() as usize;

        (
            durations[p50_idx.min(len - 1)],
            durations[p95_idx.min(len - 1)],
            durations[p99_idx.min(len - 1)],
        )
    }
}

/// Statistics by span name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanNameStats {
    /// Name of the span.
    pub name: String,

    /// Total invocations.
    pub invocations: u64,

    /// Successful invocations.
    pub successes: u64,

    /// Failed invocations.
    pub failures: u64,

    /// Average duration in milliseconds.
    pub avg_duration_ms: f64,

    /// Total duration in milliseconds.
    pub total_duration_ms: f64,
}

impl SpanNameStats {
    /// Create new stats for a span name.
    pub fn new(name: String) -> Self {
        Self {
            name,
            invocations: 0,
            successes: 0,
            failures: 0,
            avg_duration_ms: 0.0,
            total_duration_ms: 0.0,
        }
    }

    /// Record a span execution.
    pub fn record(&mut self, duration_ms: f64, success: bool) {
        self.invocations += 1;
        self.total_duration_ms += duration_ms;
        self.avg_duration_ms = self.total_duration_ms / self.invocations as f64;

        if success {
            self.successes += 1;
        } else {
            self.failures += 1;
        }
    }

    /// Get the success rate.
    pub fn success_rate(&self) -> f64 {
        if self.invocations == 0 {
            1.0
        } else {
            self.successes as f64 / self.invocations as f64
        }
    }
}

/// Aggregated statistics manager.
pub struct StatisticsManager {
    /// Statistics by span name.
    by_name: RwLock<HashMap<String, SpanNameStats>>,

    /// All recorded durations.
    durations: RwLock<Vec<f64>>,

    /// Error counts by error type.
    errors_by_type: RwLock<HashMap<String, u64>>,
}

impl StatisticsManager {
    /// Create a new statistics manager.
    pub fn new() -> Self {
        Self {
            by_name: RwLock::new(HashMap::new()),
            durations: RwLock::new(Vec::new()),
            errors_by_type: RwLock::new(HashMap::new()),
        }
    }

    /// Record a span completion.
    pub fn record_span(&self, span: &TraceSpan) {
        // Update by-name stats
        if let Ok(mut by_name) = self.by_name.write() {
            let stats = by_name
                .entry(span.name.clone())
                .or_insert_with(|| SpanNameStats::new(span.name.clone()));

            if let Some(duration) = span.duration_ms {
                stats.record(duration, span.status.is_success());
            }
        }

        // Record duration
        if let Some(duration) = span.duration_ms {
            if let Ok(mut durations) = self.durations.write() {
                durations.push(duration);
            }
        }

        // Record error if applicable
        if let Ok(mut errors) = self.errors_by_type.write() {
            if let crate::span::SpanStatus::Failed(error) = &span.status {
                // Simplify error type (first word or common pattern)
                let error_type = error
                    .split_whitespace()
                    .next()
                    .unwrap_or("unknown")
                    .to_string();
                *errors.entry(error_type).or_insert(0) += 1;
            }
        }
    }

    /// Record a checkpoint.
    pub fn record_checkpoint(&self) {
        // Checkpoints are just counted, not aggregated
    }

    /// Get statistics for a specific span name.
    pub fn get_span_stats(&self, name: &str) -> Option<SpanNameStats> {
        self.by_name
            .read()
            .ok()
            .and_then(|by_name| by_name.get(name).cloned())
    }

    /// Get all span name statistics.
    pub fn all_span_stats(&self) -> Vec<SpanNameStats> {
        self.by_name
            .read()
            .map(|by_name| by_name.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Get top spans by average duration.
    pub fn top_by_duration(&self, limit: usize) -> Vec<SpanNameStats> {
        let mut stats = self.all_span_stats();
        stats.sort_by(|a, b| {
            b.avg_duration_ms
                .partial_cmp(&a.avg_duration_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        stats.truncate(limit);
        stats
    }

    /// Get top spans by invocation count.
    pub fn top_by_invocations(&self, limit: usize) -> Vec<SpanNameStats> {
        let mut stats = self.all_span_stats();
        stats.sort_by(|a, b| b.invocations.cmp(&a.invocations));
        stats.truncate(limit);
        stats
    }

    /// Get percentiles for all durations.
    pub fn duration_percentiles(&self) -> (f64, f64, f64) {
        if let Ok(mut durations) = self.durations.write() {
            TimeWindowStats::calculate_percentiles(&mut durations)
        } else {
            (0.0, 0.0, 0.0)
        }
    }

    /// Get error counts by type.
    pub fn errors_by_type(&self) -> HashMap<String, u64> {
        self.errors_by_type
            .read()
            .map(|e| e.clone())
            .unwrap_or_default()
    }

    /// Clear all statistics.
    pub fn clear(&self) {
        if let Ok(mut by_name) = self.by_name.write() {
            by_name.clear();
        }
        if let Ok(mut durations) = self.durations.write() {
            durations.clear();
        }
        if let Ok(mut errors) = self.errors_by_type.write() {
            errors.clear();
        }
    }
}

impl Default for StatisticsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Trace metadata trait for custom types.
pub trait TraceMetadata {
    /// Get metadata for this trace.
    fn trace_metadata(&self) -> HashMap<String, serde_json::Value>;
}

// Blanket implementation for types that can be converted to JSON.
impl<T: serde::Serialize> TraceMetadata for T {
    fn trace_metadata(&self) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();
        if let Ok(value) = serde_json::to_value(self) {
            map.insert("value".to_string(), value);
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_name_stats() {
        let mut stats = SpanNameStats::new("test_function".to_string());

        stats.record(100.0, true);
        stats.record(200.0, true);
        stats.record(50.0, false);

        assert_eq!(stats.invocations, 3);
        assert_eq!(stats.successes, 2);
        assert_eq!(stats.failures, 1);
    }
}
