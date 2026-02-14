// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use std::fmt;

/// API errors.
#[derive(Debug, Clone)]
pub enum ApiError {
    /// Node not found.
    NodeNotFound(uuid::Uuid),

    /// Invalid trace ID.
    InvalidTraceId(String),

    /// Trace not found.
    TraceNotFound(String),

    /// Invalid request.
    InvalidRequest(String),

    /// Internal service error.
    Internal(String),

    /// Storage error.
    Storage(String),

    /// Serialization error.
    Serialization(String),

    /// Not implemented.
    NotImplemented(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NodeNotFound(id) => write!(f, "Node not found: {}", id),
            Self::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
            Self::Storage(msg) => write!(f, "Storage error: {}", msg),
            Self::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            Self::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            Self::InvalidTraceId(id) => write!(f, "Invalid trace ID: {}", id),
            Self::TraceNotFound(id) => write!(f, "Trace not found: {}", id),
        }
    }
}

impl std::error::Error for ApiError {}

/// Result type for API operations.
pub type ApiResult<T> = Result<T, ApiError>;

impl From<synton_graph::GraphError> for ApiError {
    fn from(e: synton_graph::GraphError) -> Self {
        Self::Storage(e.to_string())
    }
}

impl From<synton_graphrag::GraphRagError> for ApiError {
    fn from(e: synton_graphrag::GraphRagError) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<synton_storage::StorageError> for ApiError {
    fn from(e: synton_storage::StorageError) -> Self {
        Self::Storage(e.to_string())
    }
}

impl From<synton_paql::ParseError> for ApiError {
    fn from(e: synton_paql::ParseError) -> Self {
        Self::InvalidRequest(e.to_string())
    }
}

impl From<synton_memory::MemoryError> for ApiError {
    fn from(e: synton_memory::MemoryError) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<synton_core::CoreError> for ApiError {
    fn from(e: synton_core::CoreError) -> Self {
        Self::Internal(e.to_string())
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::NodeNotFound(_) => (axum::http::StatusCode::NOT_FOUND, self.to_string()),
            ApiError::InvalidRequest(_) => (axum::http::StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::Internal(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ApiError::Storage(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ApiError::Serialization(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ApiError::NotImplemented(_) => (axum::http::StatusCode::NOT_IMPLEMENTED, self.to_string()),
            ApiError::InvalidTraceId(_) => (axum::http::StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::TraceNotFound(_) => (axum::http::StatusCode::NOT_FOUND, self.to_string()),
        };

        let body = axum::Json(serde_json::json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    #[test]
    fn test_error_display() {
        let id = uuid::Uuid::new_v4();
        let error = ApiError::NodeNotFound(id);
        assert!(error.to_string().contains("not found"));
    }

    #[test]
    fn test_error_response() {
        let error = ApiError::InvalidRequest("test error".to_string());
        let response = error.into_response();

        // Should return BAD_REQUEST status
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }
}
