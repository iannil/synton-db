//! Instrumentation API for trace data visualization.
//!
//! Provides REST endpoints for:
//! - Trace lifecycle views
//! - Timeline views
//! - Statistics
//! - Export (JSON/Mermaid)

use axum::{
    extract::{Path, State, Query},
    routing::{get, Router},
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::rest::AppState;
use crate::ApiError;
use synton_instrument::{ExportFormat, LifecycleView, Statistics, TimelineView};

/// Path parameters for trace endpoints.
#[derive(Debug, Deserialize)]
pub struct TracePath {
    /// Trace or span ID.
    trace_id: String,
}

/// Lifecycle response.
#[derive(Debug, Clone, Serialize)]
pub struct LifecycleResponse {
    /// Trace lifecycle data.
    pub lifecycle: LifecycleView,
}

/// Timeline response.
#[derive(Debug, Clone, Serialize)]
pub struct TimelineResponse {
    /// Trace timeline data.
    pub timeline: TimelineView,
}

/// Statistics response.
#[derive(Debug, Clone, Serialize)]
pub struct StatisticsResponse {
    /// Instrumentation statistics.
    pub statistics: Statistics,
}

/// Export response.
#[derive(Debug, Clone, Serialize)]
pub struct ExportResponse {
    /// Export format.
    pub format: String,

    /// Export data.
    pub data: serde_json::Value,
}

// Implement IntoResponse for all response types
impl axum::response::IntoResponse for LifecycleResponse {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
}

impl axum::response::IntoResponse for TimelineResponse {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
}

impl axum::response::IntoResponse for StatisticsResponse {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
}

impl axum::response::IntoResponse for ExportResponse {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
}

/// Export query parameters.
#[derive(Debug, Deserialize)]
pub struct ExportParams {
    /// Export format: json, mermaid, text.
    format: Option<String>,
}

/// Create instrumentation router.
pub fn create_instrument_router() -> Router<AppState> {
    Router::new()
        // Get lifecycle view for a trace
        .route("/lifecycle/:trace_id", get(lifecycle_view))
        // Get timeline view for a trace
        .route("/timeline/:trace_id", get(timeline_view))
        // Get statistics
        .route("/stats", get(statistics))
        // Export trace data
        .route("/export/:trace_id", get(export_trace))
}

/// Get lifecycle view for a trace.
async fn lifecycle_view(
    State(state): State<AppState>,
    Path(trace_id): Path<String>,
) -> impl IntoResponse {
    let collector = &state.service.collector;
    let uuid = match Uuid::parse_str(&trace_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return ApiError::InvalidTraceId(trace_id).into_response();
        }
    };

    match collector.get_lifecycle(uuid) {
        Some(lifecycle) => LifecycleResponse { lifecycle }.into_response(),
        None => ApiError::TraceNotFound(trace_id.to_string()).into_response(),
    }
}

/// Get timeline view for a trace.
async fn timeline_view(
    State(state): State<AppState>,
    Path(trace_id): Path<String>,
) -> impl IntoResponse {
    let collector = &state.service.collector;
    let uuid = match Uuid::parse_str(&trace_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return ApiError::InvalidTraceId(trace_id).into_response();
        }
    };

    match collector.get_timeline(uuid) {
        Some(timeline) => TimelineResponse { timeline }.into_response(),
        None => ApiError::TraceNotFound(trace_id.to_string()).into_response(),
    }
}

/// Get instrumentation statistics.
async fn statistics(State(state): State<AppState>) -> impl IntoResponse {
    let collector = &state.service.collector;
    let stats = collector.get_statistics();
    StatisticsResponse { statistics: stats }.into_response()
}

/// Export trace data in specified format.
async fn export_trace(
    State(state): State<AppState>,
    Path(trace_id): Path<String>,
    Query(params): Query<ExportParams>,
) -> impl IntoResponse {
    let collector = &state.service.collector;
    let uuid = match Uuid::parse_str(&trace_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return ApiError::InvalidTraceId(trace_id).into_response();
        }
    };

    let format = params.format.as_deref().and_then(|f| {
        ExportFormat::from_str(f)
    });

    let data = match format {
        Some(ExportFormat::Json) => {
            match collector.export_json(uuid) {
                Some(data) => serde_json::to_value(data).unwrap_or(serde_json::json!({})),
                None => {
                    return ApiError::TraceNotFound(trace_id.to_string()).into_response();
                }
            }
        }
        Some(ExportFormat::Mermaid) => {
            match collector.export_mermaid(uuid) {
                Some(data) => serde_json::to_value(data).unwrap_or(serde_json::json!({})),
                None => {
                    return ApiError::TraceNotFound(trace_id.to_string()).into_response();
                }
            }
        }
        Some(ExportFormat::Text) => {
            match collector.export_mermaid(uuid) {
                Some(data) => serde_json::to_value(data).unwrap_or(serde_json::json!({})),
                None => {
                    return ApiError::TraceNotFound(trace_id.to_string()).into_response();
                }
            }
        }
        None => {
            // Default to JSON
            match collector.export_json(uuid) {
                Some(data) => serde_json::to_value(data).unwrap_or(serde_json::json!({})),
                None => {
                    return ApiError::TraceNotFound(trace_id.to_string()).into_response();
                }
            }
        }
    };

    let format_name = match format {
        Some(ExportFormat::Json) => "json",
        Some(ExportFormat::Mermaid) => "mermaid",
        Some(ExportFormat::Text) => "text",
        None => "json",
    };

    ExportResponse {
        format: format_name.to_string(),
        data,
    }.into_response()
}
