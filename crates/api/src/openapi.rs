// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! OpenAPI/Swagger documentation for the REST API.

use uuid::Uuid;

/// OpenAPI documentation for SYNTON-DB REST API.
#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "SYNTON-DB API",
        version = "0.1.0",
        description = "Cognitive Database API - A graph-vector database for LLM applications",
        contact(
            name = "SYNTON-DB Team",
            url = "https://github.com/synton-db/synton-db"
        ),
        license(
            name = "Apache-2.0",
            url = "https://www.apache.org/licenses/LICENSE-2.0.html"
        )
    ),
    paths(
        crate::rest::health_check,
        crate::rest::stats,
        crate::rest::add_node,
        crate::rest::get_node,
        crate::rest::get_all_nodes,
        crate::rest::delete_node,
        crate::rest::add_edge,
        crate::rest::query,
        crate::rest::traverse,
        crate::rest::hybrid_search,
        crate::rest::bulk_operation,
    ),
    components(
        schemas(
            HealthResponse,
            DatabaseStats,
            NodeInfo,
            AddNodeRequest,
            AddNodeResponse,
            GetNodeResponse,
            DeleteNodeRequest,
            DeleteNodeResponse,
            EdgeInfo,
            AddEdgeRequest,
            AddEdgeResponse,
            QueryRequest,
            QueryResponse,
            TraverseRequest,
            TraverseResponse,
            HybridSearchRequest,
            HybridSearchResponse,
            BulkOperationRequest,
            BulkOperationResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "nodes", description = "Node management endpoints"),
        (name = "edges", description = "Edge management endpoints"),
        (name = "query", description = "Query and search endpoints"),
        (name = "graph", description = "Graph traversal endpoints"),
    )
)]
pub struct ApiDoc;

/// Health check response schema.
#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    /// API version
    pub version: String,
}

/// Database statistics schema.
#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct DatabaseStats {
    /// Total number of nodes
    pub node_count: usize,
    /// Total number of edges
    pub edge_count: usize,
    /// Number of embedded nodes
    pub embedded_count: usize,
}

/// Node information schema.
#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct NodeInfo {
    /// Node ID
    pub id: Uuid,
    /// Node content
    pub content: String,
    /// Node type
    pub node_type: String,
    /// Creation timestamp
    pub created_at: String,
}

/// Edge information schema.
#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct EdgeInfo {
    /// Unique edge identifier (source-target-relation)
    pub id: String,
    /// Source node ID
    pub source: Uuid,
    /// Target node ID
    pub target: Uuid,
    /// Relation type
    pub relation: String,
    /// Edge weight
    pub weight: f32,
}

/// Add node request schema.
#[derive(utoipa::ToSchema, serde::Deserialize)]
pub struct AddNodeRequest {
    /// Node content/text
    #[schema(example = "Machine learning is a subset of AI")]
    pub content: String,
    /// Node type (entity, concept, fact, raw_chunk)
    #[schema(example = "concept")]
    pub node_type: String,
    /// Optional embedding vector (384 dims for default model)
    pub embedding: Option<Vec<f32>>,
    /// Optional attributes as JSON
    pub attributes: Option<serde_json::Value>,
}

/// Add node response schema.
#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct AddNodeResponse {
    /// The created node
    pub node: NodeInfo,
    /// Whether the node was newly created
    pub created: bool,
}

/// Get node response schema.
#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct GetNodeResponse {
    /// The requested node, if found
    pub node: Option<NodeInfo>,
}

/// Delete node request schema.
#[derive(utoipa::ToSchema, serde::Deserialize)]
pub struct DeleteNodeRequest {
    /// Node ID to delete
    pub id: Uuid,
}

/// Delete node response schema.
#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct DeleteNodeResponse {
    /// Whether the node was deleted
    pub deleted: bool,
    /// ID of the deleted node
    pub id: Uuid,
}

/// Add edge request schema.
#[derive(utoipa::ToSchema, serde::Deserialize)]
pub struct AddEdgeRequest {
    /// Source node ID
    pub source: Uuid,
    /// Target node ID
    pub target: Uuid,
    /// Relation type (causes, similar_to, is_part_of, is_a, has_property)
    #[schema(example = "similar_to")]
    pub relation: String,
    /// Edge weight (0.0 - 1.0)
    #[schema(example = 0.8, minimum = 0.0, maximum = 1.0)]
    pub weight: f32,
}

/// Add edge response schema.
#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct AddEdgeResponse {
    /// The created edge
    pub edge: EdgeInfo,
}

/// Query request schema.
#[derive(utoipa::ToSchema, serde::Deserialize)]
pub struct QueryRequest {
    /// Query string (supports text search)
    #[schema(example = "machine learning")]
    pub query: String,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Include metadata in results
    pub include_metadata: bool,
}

/// Query response schema.
#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct QueryResponse {
    /// Matching nodes
    pub nodes: Vec<NodeInfo>,
    /// Total count of results
    pub total_count: usize,
    /// Query execution time in milliseconds
    pub execution_time_ms: u64,
    /// Whether results were truncated
    pub truncated: bool,
}

/// Traverse request schema.
#[derive(utoipa::ToSchema, serde::Deserialize)]
pub struct TraverseRequest {
    /// Starting node ID
    pub start_id: Uuid,
    /// Maximum traversal depth
    #[schema(example = 2, minimum = 1, maximum = 10)]
    pub max_depth: usize,
    /// Maximum nodes to return
    #[schema(example = 100)]
    pub max_nodes: usize,
    /// Traversal direction (forward, backward, both)
    #[schema(example = "forward")]
    pub direction: String,
}

/// Traverse response schema.
#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct TraverseResponse {
    /// Nodes found during traversal
    pub nodes: Vec<NodeInfo>,
    /// Edges found during traversal
    pub edges: Vec<EdgeInfo>,
    /// Maximum depth reached
    pub depth: usize,
    /// Whether traversal was truncated
    pub truncated: bool,
}

/// Hybrid search request schema (GraphRAG).
#[derive(utoipa::ToSchema, serde::Deserialize)]
pub struct HybridSearchRequest {
    /// Query string for hybrid search
    #[schema(example = "neural network architecture")]
    pub query: String,
    /// Number of results to return (k)
    #[schema(example = 10, minimum = 1, maximum = 100)]
    pub k: usize,
}

/// Hybrid search response schema.
#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct HybridSearchResponse {
    /// Relevant nodes found
    pub nodes: Vec<NodeInfo>,
    /// Number of results
    pub count: usize,
}

/// Bulk operation request schema.
#[derive(utoipa::ToSchema, serde::Deserialize)]
pub struct BulkOperationRequest {
    /// Nodes to add
    pub nodes: Vec<AddNodeRequest>,
    /// Edges to add
    pub edges: Vec<AddEdgeRequest>,
}

/// Bulk operation response schema.
#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct BulkOperationResponse {
    /// IDs of created nodes
    pub node_ids: Vec<Uuid>,
    /// IDs of created edges
    pub edge_ids: Vec<String>,
    /// Number of successful operations
    pub success_count: usize,
    /// Number of failed operations
    pub failure_count: usize,
    /// Error messages
    pub errors: Vec<String>,
}

