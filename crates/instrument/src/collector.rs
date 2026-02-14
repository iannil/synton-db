//! Trace collector for aggregating and managing span data.

use std::collections::HashMap;
use std::sync::RwLock;

use dashmap::DashMap;
use uuid::Uuid;

use crate::span::{SpanId, SpanStatus, TraceEvent, TraceId, TraceSpan};
use crate::statistics::StatisticsManager;
use crate::views::{LifecycleView, Statistics, TimelineView};

/// Global trace collector instance.
static GLOBAL_COLLECTOR: once_cell::sync::Lazy<TraceCollector> =
    once_cell::sync::Lazy::new(TraceCollector::new);

/// Maximum number of spans to keep in memory.
const DEFAULT_MAX_SPANS: usize = 10_000;

/// Maximum number of events to keep in memory.
const DEFAULT_MAX_EVENTS: usize = 100_000;

/// Configuration for the trace collector.
#[derive(Debug, Clone)]
pub struct CollectorConfig {
    /// Maximum spans to keep in memory.
    pub max_spans: usize,

    /// Maximum events to keep in memory.
    pub max_events: usize,

    /// Whether to persist traces.
    pub persist_enabled: bool,

    /// Sampling rate (0.0 to 1.0).
    pub sample_rate: f64,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            max_spans: DEFAULT_MAX_SPANS,
            max_events: DEFAULT_MAX_EVENTS,
            persist_enabled: false,
            sample_rate: 1.0,
        }
    }
}

/// Main trace collector for aggregating span data.
///
/// ## Thread Safety
///
/// The collector is thread-safe and can be safely accessed from multiple threads.
///
/// ## Usage
///
/// ```rust,ignore
/// // Get the global collector instance
/// let collector = TraceCollector::global();
///
/// // Create a new span
/// let span_id = collector.enter_span("operation", None, Default::default());
///
/// // Record a checkpoint
/// collector.checkpoint(span_id, "checkpoint_name");
///
/// // Complete the span
/// collector.complete_span(span_id, None, 100.0);
/// ```
pub struct TraceCollector {
    /// Configuration.
    config: CollectorConfig,

    /// Collected spans indexed by ID.
    spans: DashMap<SpanId, TraceSpan>,

    /// Collected events in chronological order.
    events: RwLock<Vec<TraceEvent>>,

    /// Thread-local span stack.
    local_spans: thread_local::ThreadLocal<RwLock<Vec<SpanId>>>,

    /// Statistics manager.
    statistics: RwLock<StatisticsManager>,
}

impl TraceCollector {
    /// Create a new trace collector.
    pub fn new() -> Self {
        Self::with_config(CollectorConfig::default())
    }

    /// Create a new trace collector with custom configuration.
    pub fn with_config(config: CollectorConfig) -> Self {
        Self {
            config,
            spans: DashMap::new(),
            events: RwLock::new(Vec::new()),
            local_spans: thread_local::ThreadLocal::new(),
            statistics: RwLock::new(StatisticsManager::new()),
        }
    }

    /// Get the global collector instance.
    pub fn global() -> &'static Self {
        &GLOBAL_COLLECTOR
    }

    /// Update the collector configuration.
    pub fn update_config(&self, config: CollectorConfig) {
        // Update max_spans by evicting if necessary
        if config.max_spans < self.spans.len() {
            let to_remove = self.spans.len() - config.max_spans;
            let mut count = 0;
            self.spans.retain(|_, span| {
                if count < to_remove && span.is_complete() {
                    count += 1;
                    false
                } else {
                    true
                }
            });
        }

        // Update max_events by evicting old events
        if let Ok(events) = self.events.read() {
            if config.max_events < events.len() {
                let to_remove = events.len() - config.max_events;
                if let Ok(mut events) = self.events.write() {
                    *events = events.split_off(to_remove);
                }
            }
        }
    }

    /// Get the current span ID for this thread.
    pub fn current_span_id() -> SpanId {
        GLOBAL_COLLECTOR
            .local_spans
            .get_or_default()
            .read()
            .ok()
            .and_then(|stack| stack.last().copied())
            .unwrap_or_else(Uuid::new_v4)
    }

    /// Get the parent span ID for this thread.
    pub fn parent_span_id() -> Option<SpanId> {
        let stack = GLOBAL_COLLECTOR.local_spans.get_or_default();
        if let Ok(stack) = stack.read() {
            if stack.len() >= 2 {
                Some(stack[stack.len() - 2])
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Enter a new span.
    pub fn enter_span(
        &self,
        name: String,
        parent_id: Option<SpanId>,
        metadata: crate::span::SpanMetadata,
    ) -> SpanId {
        let span_id = crate::span::new_span_id();

        // Update parent's children if applicable
        if let Some(parent) = parent_id {
            if let Some(mut parent_span) = self.spans.get_mut(&parent) {
                parent_span.add_child(span_id);
            }
        }

        // Create the span
        let span = TraceSpan {
            id: span_id,
            parent_id,
            name: name.clone(),
            start_time: chrono::Utc::now(),
            end_time: None,
            duration_ms: None,
            status: SpanStatus::Running,
            metadata,
            result: None,
            attributes: HashMap::new(),
            children: Vec::new(),
        };

        // Store the span
        self.spans.insert(span_id, span);

        // Record enter event
        if let Ok(mut events) = self.events.write() {
            let span_ref = self.spans.get(&span_id).unwrap();
            events.push(TraceEvent::enter(
                span_id,
                parent_id,
                name,
                span_ref.metadata.clone(),
            ));
        }

        // Push to local stack
        if let Some(stack) = self.local_spans.get() {
            if let Ok(mut stack) = stack.write() {
                stack.push(span_id);
            }
        }

        span_id
    }

    /// Complete a span with a result.
    pub fn complete_span(&self, span_id: SpanId, result: Option<serde_json::Value>, duration_ms: f64) {
        // Update span
        if let Some(mut span) = self.spans.get_mut(&span_id) {
            span.end_time = Some(chrono::Utc::now());
            span.duration_ms = Some(duration_ms);
            span.status = SpanStatus::Completed;
            span.result = result;
        }

        // Record exit event
        if let Ok(mut events) = self.events.write() {
            events.push(TraceEvent::exit(span_id, None, duration_ms));
        }

        // Pop from local stack
        if let Some(stack) = self.local_spans.get() {
            if let Ok(mut stack) = stack.write() {
                if stack.last() == Some(&span_id) {
                    stack.pop();
                }
            }
        }

        // Update statistics
        if let Ok(mut stats) = self.statistics.write() {
            stats.record_checkpoint();
        }
    }

    /// Mark a span as failed.
    pub fn fail_span(&self, span_id: SpanId, error: String, duration_ms: f64) {
        // Update span
        if let Some(mut span) = self.spans.get_mut(&span_id) {
            span.end_time = Some(chrono::Utc::now());
            span.duration_ms = Some(duration_ms);
            span.status = SpanStatus::Failed(error.clone());
        }

        // Record error event
        if let Ok(mut events) = self.events.write() {
            events.push(TraceEvent::error(span_id, error, None));
        }

        // Pop from local stack
        if let Some(stack) = self.local_spans.get() {
            if let Ok(mut stack) = stack.write() {
                if stack.last() == Some(&span_id) {
                    stack.pop();
                }
            }
        }
    }

    /// Record a checkpoint event.
    pub fn checkpoint(&self, span_id: SpanId, checkpoint_name: String) {
        if let Ok(mut events) = self.events.write() {
            events.push(TraceEvent::checkpoint(span_id, checkpoint_name, None));
        }
    }

    /// Record a checkpoint with data.
    pub fn checkpoint_with_data(
        &self,
        span_id: SpanId,
        checkpoint_name: String,
        data: serde_json::Value,
    ) {
        if let Ok(mut events) = self.events.write() {
            events.push(TraceEvent::checkpoint(span_id, checkpoint_name, Some(data)));
        }
    }

    /// Record a custom event.
    pub fn record_event(
        &self,
        span_id: SpanId,
        event_name: String,
        data: HashMap<String, serde_json::Value>,
    ) {
        if let Ok(mut events) = self.events.write() {
            events.push(TraceEvent::custom(span_id, event_name, data));
        }
    }

    /// Get a span by ID.
    pub fn get_span(&self, span_id: SpanId) -> Option<TraceSpan> {
        self.spans.get(&span_id).map(|s| s.clone())
    }

    /// Get all spans.
    pub fn all_spans(&self) -> Vec<TraceSpan> {
        self.spans.iter().map(|s| s.clone()).collect()
    }

    /// Get all root spans (spans without parents).
    pub fn root_spans(&self) -> Vec<TraceSpan> {
        self.spans
            .iter()
            .filter(|s| s.parent_id.is_none())
            .map(|s| s.clone())
            .collect()
    }

    /// Get all events.
    pub fn all_events(&self) -> Vec<TraceEvent> {
        self.events
            .read()
            .map(|e| e.clone())
            .unwrap_or_default()
    }

    /// Get events for a specific trace.
    pub fn trace_events(&self, root_id: SpanId) -> Vec<TraceEvent> {
        let all_spans = self.collect_trace_tree(root_id);
        let span_ids: std::collections::HashSet<SpanId> =
            all_spans.iter().map(|s| s.id).collect();

        self.events
            .read()
            .map(|e| {
                e.iter()
                    .filter(|ev| span_ids.contains(&ev.span_id()))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Recursively collect all spans in a trace tree.
    fn collect_trace_tree(&self, root_id: SpanId) -> Vec<TraceSpan> {
        let mut result = Vec::new();
        let mut to_visit = vec![root_id];
        let mut visited = std::collections::HashSet::new();

        while let Some(span_id) = to_visit.pop() {
            if visited.contains(&span_id) {
                continue;
            }

            visited.insert(span_id);

            if let Some(span) = self.get_span(span_id) {
                to_visit.extend(span.children.iter());
                result.push(span);
            }
        }

        result
    }

    /// Get lifecycle view for a trace.
    pub fn get_lifecycle(&self, root_id: SpanId) -> Option<LifecycleView> {
        let spans = self.collect_trace_tree(root_id);

        if spans.is_empty() {
            return None;
        }

        let mut span_map: HashMap<SpanId, LifecycleView> = HashMap::new();

        // First pass: create all nodes
        for span in &spans {
            span_map.insert(
                span.id,
                LifecycleView {
                    id: span.id.to_string(),
                    name: span.name.clone(),
                    start_time: span.start_time,
                    end_time: span.end_time,
                    duration_ms: span.duration_ms.unwrap_or(0.0),
                    status: format!("{:?}", span.status),
                    children: Vec::new(),
                    args: span
                        .metadata
                        .args
                        .as_ref()
                        .and_then(|a| serde_json::to_value(a).ok()),
                    result: span.result.clone(),
                },
            );
        }

        // Second pass: build tree structure - collect all parent-child relationships first
        let mut parent_child_pairs: Vec<(SpanId, SpanId)> = Vec::new();
        for span in &spans {
            if let Some(parent_id) = span.parent_id {
                parent_child_pairs.push((parent_id, span.id));
            }
        }

        // Then apply them without holding mutable borrows
        for (parent_id, child_id) in parent_child_pairs {
            if let Some(child_view) = span_map.get(&child_id).cloned() {
                if let Some(parent_view) = span_map.get_mut(&parent_id) {
                    parent_view.children.push(child_view);
                }
            }
        }

        span_map.remove(&root_id)
    }

    /// Get timeline view for a trace.
    pub fn get_timeline(&self, root_id: SpanId) -> Option<TimelineView> {
        let spans = self.collect_trace_tree(root_id);

        if spans.is_empty() {
            return None;
        }

        let events = self.trace_events(root_id);
        let mut sorted_events = events;
        sorted_events.sort_by_key(|e| e.timestamp());

        Some(TimelineView {
            trace_id: root_id.to_string(),
            events: sorted_events,
            total_duration_ms: spans
                .first()
                .and_then(|root| {
                    let root_end = root.end_time.unwrap_or(root.start_time);
                    Some(root_end.signed_duration_since(root.start_time).num_milliseconds() as f64)
                })
                .unwrap_or(0.0),
        })
    }

    /// Get statistics.
    pub fn get_statistics(&self) -> Statistics {
        let span_count = self.spans.iter().filter(|s| !s.is_complete()).count();
        let event_count = self.events.read().map(|e| e.len()).unwrap_or(0);

        Statistics {
            spans_created: 0, // Would need counter
            spans_completed: 0,
            spans_failed: 0,
            checkpoints: 0,
            active_spans: span_count,
            total_events: event_count,
        }
    }

    /// Clear all collected data.
    pub fn clear(&self) {
        self.spans.clear();
        if let Ok(mut events) = self.events.write() {
            events.clear();
        }
    }

    /// Export trace data as JSON.
    pub fn export_json(&self, root_id: SpanId) -> Option<serde_json::Value> {
        let lifecycle = self.get_lifecycle(root_id)?;
        serde_json::to_value(lifecycle).ok()
    }

    /// Export trace data as Mermaid flowchart.
    pub fn export_mermaid(&self, root_id: SpanId) -> Option<String> {
        let spans = self.collect_trace_tree(root_id);
        if spans.is_empty() {
            return None;
        }

        let mut mermaid = String::from("flowchart TD\n");

        for span in &spans {
            let node_id = format!("N{}", span.id.to_string().replace('-', ""));
            let label = format!("{}[\"{}\"]", node_id, span.name);

            if let Some(parent_id) = span.parent_id {
                let parent_node = format!("N{}", parent_id.to_string().replace('-', ""));
                mermaid.push_str(&format!("    {} --> {}\n", parent_node, label));
            } else {
                mermaid.push_str(&format!("    {}\n", label));
            }
        }

        Some(mermaid)
    }

    /// Get the number of spans currently in memory.
    pub fn span_count(&self) -> usize {
        self.spans.len()
    }

    /// Get the number of events currently in memory.
    pub fn event_count(&self) -> usize {
        self.events.read().map(|e| e.len()).unwrap_or(0)
    }
}

impl Default for TraceCollector {
    fn default() -> Self {
        Self::new()
    }
}
