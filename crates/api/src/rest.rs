// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use axum::extract::{Path as AxumPath, State};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    models::{
        AddEdgeRequest, AddEdgeResponse, AddNodeRequest, AddNodeResponse, DeleteNodeRequest,
        DeleteNodeResponse, GetNodeRequest, GetNodeResponse, HealthResponse, QueryRequest,
        QueryResponse, TraverseRequest, TraverseResponse,
    },
    ApiResult, SyntonDbService,
};

/// Application state for the REST API.
#[derive(Clone)]
pub struct AppState {
    /// The database service.
    pub service: Arc<SyntonDbService>,
}

impl AppState {
    /// Create a new application state.
    pub fn new(service: Arc<SyntonDbService>) -> Self {
        Self { service }
    }
}

/// Health check handler.
pub async fn health_check() -> axum::Json<HealthResponse> {
    // Get a dummy service for health check
    let service = SyntonDbService::new();
    axum::Json(service.health())
}

/// Get statistics handler.
pub async fn stats(
    State(state): State<AppState>,
) -> ApiResult<axum::Json<crate::models::DatabaseStats>> {
    let stats = state.service.stats().await?;
    Ok(axum::Json(stats))
}

/// Add a node handler.
pub async fn add_node(
    State(state): State<AppState>,
    axum::Json(request): axum::Json<AddNodeRequest>,
) -> ApiResult<axum::Json<AddNodeResponse>> {
    let response = state.service.add_node(request).await?;
    Ok(axum::Json(response))
}

/// Get a node by ID handler.
pub async fn get_node(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
) -> ApiResult<axum::Json<GetNodeResponse>> {
    let request = GetNodeRequest { id };
    let response = state.service.get_node(request).await?;
    Ok(axum::Json(response))
}

/// Delete a node handler.
pub async fn delete_node(
    State(state): State<AppState>,
    axum::Json(request): axum::Json<DeleteNodeRequest>,
) -> ApiResult<axum::Json<DeleteNodeResponse>> {
    let response = state.service.delete_node(request).await?;
    Ok(axum::Json(response))
}

/// Add an edge handler.
pub async fn add_edge(
    State(state): State<AppState>,
    axum::Json(request): axum::Json<AddEdgeRequest>,
) -> ApiResult<axum::Json<AddEdgeResponse>> {
    let response = state.service.add_edge(request).await?;
    Ok(axum::Json(response))
}

/// Query handler.
pub async fn query(
    State(state): State<AppState>,
    axum::Json(request): axum::Json<QueryRequest>,
) -> ApiResult<axum::Json<QueryResponse>> {
    let response = state.service.query(request).await?;
    Ok(axum::Json(response))
}

/// Traverse handler.
pub async fn traverse(
    State(state): State<AppState>,
    axum::Json(request): axum::Json<TraverseRequest>,
) -> ApiResult<axum::Json<TraverseResponse>> {
    let response = state.service.traverse(request).await?;
    Ok(axum::Json(response))
}

/// Get all nodes handler.
pub async fn get_all_nodes(
    State(state): State<AppState>,
) -> axum::Json<Vec<synton_core::Node>> {
    let nodes = state.service.all_nodes().await;
    axum::Json(nodes)
}

/// Bulk operation handler.
pub async fn bulk_operation(
    State(state): State<AppState>,
    axum::Json(request): axum::Json<crate::models::BulkOperationRequest>,
) -> ApiResult<axum::Json<crate::models::BulkOperationResponse>> {
    let mut node_ids = Vec::new();
    let mut edge_ids = Vec::new();
    let mut success_count = 0;
    let mut failure_count = 0;
    let mut errors = Vec::new();

    // Add nodes
    for node_req in request.nodes {
        match state.service.add_node(node_req).await {
            Ok(resp) => {
                node_ids.push(resp.node.id);
                success_count += 1;
            }
            Err(e) => {
                errors.push(format!("Node creation failed: {}", e));
                failure_count += 1;
            }
        }
    }

    // Add edges
    for edge_req in request.edges {
        match state.service.add_edge(edge_req).await {
            Ok(resp) => {
                edge_ids.push(resp.edge.id());
                success_count += 1;
            }
            Err(e) => {
                errors.push(format!("Edge creation failed: {}", e));
                failure_count += 1;
            }
        }
    }

    let response = crate::models::BulkOperationResponse {
        node_ids,
        edge_ids,
        success_count,
        failure_count,
        errors,
    };

    Ok(axum::Json(response))
}

/// Create the REST API router.
pub fn create_router() -> axum::Router {
    let service = Arc::new(SyntonDbService::new());
    let state = AppState::new(service);

    axum::Router::new()
        .route("/health", axum::routing::get(health_check))
        .route("/stats", axum::routing::get(stats))
        .route("/nodes", axum::routing::post(add_node))
        .route("/nodes", axum::routing::get(get_all_nodes))
        .route("/nodes/:id", axum::routing::get(get_node))
        .route("/nodes/:id", axum::routing::delete(delete_node))
        .route("/edges", axum::routing::post(add_edge))
        .route("/query", axum::routing::post(query))
        .route("/traverse", axum::routing::post(traverse))
        .route("/bulk", axum::routing::post(bulk_operation))
        .with_state(state)
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        )
        .layer(tower_http::trace::TraceLayer::new_for_http())
}

/// Start the REST API server.
pub async fn run_server(addr: impl tokio::net::ToSocketAddrs) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_router();

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("REST API server listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use synton_core::NodeType;

    #[tokio::test]
    async fn test_app_state_creation() {
        let service = Arc::new(SyntonDbService::new());
        let state = AppState::new(service);
        assert_eq!(state.service.stats().await.unwrap().node_count, 0);
    }

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        assert_eq!(response.status, "healthy");
    }

    #[tokio::test]
    async fn test_add_node_handler() {
        let service = Arc::new(SyntonDbService::new());
        let state = AppState::new(service);

        let request = AddNodeRequest::new("Test node".to_string(), NodeType::Concept);

        let result = add_node(State(state), axum::Json(request)).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.0.created);
    }
}
