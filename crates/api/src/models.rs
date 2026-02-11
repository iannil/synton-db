// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use synton_core::{Edge, Node, NodeType, Relation};

/// Request to add a node to the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddNodeRequest {
    /// Node content.
    pub content: String,

    /// Node type.
    pub node_type: NodeType,

    /// Optional embedding vector.
    pub embedding: Option<Vec<f32>>,

    /// Optional attributes.
    pub attributes: Option<serde_json::Value>,
}

impl AddNodeRequest {
    /// Create a new request.
    pub fn new(content: String, node_type: NodeType) -> Self {
        Self {
            content,
            node_type,
            embedding: None,
            attributes: None,
        }
    }

    /// Set the embedding.
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }

    /// Set attributes.
    pub fn with_attributes(mut self, attributes: serde_json::Value) -> Self {
        self.attributes = Some(attributes);
        self
    }
}

/// Response from adding a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddNodeResponse {
    /// The created node.
    pub node: Node,

    /// Whether the node was newly created or already existed.
    pub created: bool,
}

/// Request to add an edge to the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddEdgeRequest {
    /// Source node ID.
    pub source: Uuid,

    /// Target node ID.
    pub target: Uuid,

    /// Relation type.
    pub relation: Relation,

    /// Edge weight (0.0 - 1.0).
    #[serde(default = "default_weight")]
    pub weight: f32,

    /// Optional relation vector.
    pub vector: Option<Vec<f32>>,
}

impl Default for AddEdgeRequest {
    fn default() -> Self {
        Self {
            source: Uuid::default(),
            target: Uuid::default(),
            relation: Relation::SimilarTo,
            weight: default_weight(),
            vector: None,
        }
    }
}

fn default_weight() -> f32 {
    1.0
}

/// Response from adding an edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddEdgeResponse {
    /// The created edge.
    pub edge: Edge,
}

/// Request to query the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    /// Query string (PaQL or natural language).
    pub query: String,

    /// Maximum number of results.
    pub limit: Option<usize>,

    /// Whether to include metadata in results.
    pub include_metadata: bool,
}

/// Response from a database query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    /// Retrieved nodes.
    pub nodes: Vec<Node>,

    /// Total number of matching nodes (may exceed returned nodes).
    pub total_count: usize,

    /// Query execution time in milliseconds.
    pub execution_time_ms: u64,

    /// Whether results were truncated.
    pub truncated: bool,
}

/// Request for graph traversal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraverseRequest {
    /// Starting node ID.
    pub start_id: Uuid,

    /// Maximum traversal depth.
    pub max_depth: usize,

    /// Maximum number of nodes to return.
    pub max_nodes: usize,

    /// Traversal direction.
    pub direction: TraverseDirection,
}

/// Direction for graph traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraverseDirection {
    /// Forward (outgoing edges).
    Forward,

    /// Backward (incoming edges).
    Backward,

    /// Both directions.
    Both,
}

impl From<synton_graph::TraverseDirection> for TraverseDirection {
    fn from(dir: synton_graph::TraverseDirection) -> Self {
        match dir {
            synton_graph::TraverseDirection::Forward => Self::Forward,
            synton_graph::TraverseDirection::Backward => Self::Backward,
            synton_graph::TraverseDirection::Both => Self::Both,
        }
    }
}

impl From<TraverseDirection> for synton_graph::TraverseDirection {
    fn from(dir: TraverseDirection) -> Self {
        match dir {
            TraverseDirection::Forward => synton_graph::TraverseDirection::Forward,
            TraverseDirection::Backward => synton_graph::TraverseDirection::Backward,
            TraverseDirection::Both => synton_graph::TraverseDirection::Both,
        }
    }
}

/// Response from graph traversal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraverseResponse {
    /// Traversed nodes.
    pub nodes: Vec<Node>,

    /// Traversed edges.
    pub edges: Vec<Edge>,

    /// Maximum depth reached.
    pub depth: usize,

    /// Whether traversal was truncated due to limits.
    pub truncated: bool,
}

/// Request to get a node by ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetNodeRequest {
    /// Node ID.
    pub id: Uuid,
}

/// Response with a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetNodeResponse {
    /// The node if found.
    pub node: Option<Node>,
}

/// Request to delete a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteNodeRequest {
    /// Node ID.
    pub id: Uuid,
}

/// Response from deleting a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteNodeResponse {
    /// Whether the node was found and deleted.
    pub deleted: bool,

    /// ID of the deleted node.
    pub id: Uuid,
}

/// Database statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    /// Total number of nodes.
    pub node_count: usize,

    /// Total number of edges.
    pub edge_count: usize,

    /// Number of nodes with embeddings.
    pub embedded_count: usize,

    /// Memory statistics.
    pub memory_stats: MemoryStats,
}

/// Memory statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total nodes being tracked.
    pub total_nodes: usize,

    /// Active nodes (above threshold).
    pub active_nodes: usize,

    /// Decayed nodes (below threshold).
    pub decayed_nodes: usize,

    /// Average access score.
    pub average_score: f32,

    /// Memory load factor.
    pub load_factor: f32,
}

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status.
    pub status: String,

    /// Version.
    pub version: String,

    /// Uptime in seconds.
    pub uptime_secs: u64,
}

/// Bulk operation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationRequest {
    /// Nodes to add.
    pub nodes: Vec<AddNodeRequest>,

    /// Edges to add.
    pub edges: Vec<AddEdgeRequest>,
}

/// Bulk operation response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationResponse {
    /// IDs of created nodes.
    pub node_ids: Vec<Uuid>,

    /// IDs of created edges.
    pub edge_ids: Vec<String>,

    /// Number of successful operations.
    pub success_count: usize,

    /// Number of failed operations.
    pub failure_count: usize,

    /// Any errors that occurred.
    pub errors: Vec<String>,
}

/// Hybrid search request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchRequest {
    /// Query string.
    pub query: String,

    /// Maximum number of results.
    #[serde(default = "default_k")]
    pub k: usize,
}

fn default_k() -> usize {
    10
}

impl HybridSearchRequest {
    /// Create a new hybrid search request.
    pub fn new(query: String, k: usize) -> Self {
        Self { query, k }
    }
}

/// Hybrid search response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchResponse {
    /// Retrieved nodes.
    pub nodes: Vec<Node>,

    /// Number of results.
    pub count: usize,
}

/// Chunking strategy for document ingestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkingStrategy {
    /// Fixed-size chunking by character count.
    Fixed { chunk_size: usize, overlap: usize },

    /// Semantic chunking based on sentence boundaries.
    Semantic {
        min_chunk_size: usize,
        max_chunk_size: usize,
        boundary_threshold: f32,
    },

    /// Hierarchical chunking with multiple levels.
    Hierarchical {
        include_sentences: bool,
        include_paragraphs: bool,
    },
}

impl Default for ChunkingStrategy {
    fn default() -> Self {
        Self::Semantic {
            min_chunk_size: 100,
            max_chunk_size: 500,
            boundary_threshold: 0.5,
        }
    }
}

/// Document ingestion request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestDocumentRequest {
    /// Document title.
    #[serde(default)]
    pub title: Option<String>,

    /// Document content.
    pub content: String,

    /// Chunking strategy.
    #[serde(default)]
    pub chunking: Option<ChunkingStrategy>,

    /// Whether to generate embeddings for chunks.
    #[serde(default = "default_embed")]
    pub embed: bool,

    /// Optional metadata to attach to all chunks.
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

fn default_embed() -> bool {
    true
}

/// Information about a single chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
    /// Chunk ID.
    pub id: Uuid,

    /// Chunk content.
    pub content: String,

    /// Chunk index.
    pub index: usize,

    /// Character range in original document.
    pub range: (usize, usize),

    /// Chunk type.
    pub chunk_type: String,

    /// Parent chunk ID (for hierarchical chunks).
    pub parent_id: Option<Uuid>,

    /// Child chunk IDs.
    pub child_ids: Vec<Uuid>,
}

/// Document ingestion response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestDocumentResponse {
    /// Document ID (root node).
    pub document_id: Uuid,

    /// Number of chunks created.
    pub chunk_count: usize,

    /// Chunk information.
    pub chunks: Vec<ChunkInfo>,

    /// Whether embeddings were generated.
    pub embedded: bool,

    /// Processing time in milliseconds.
    pub processing_time_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node_request() {
        let req = AddNodeRequest::new("test".to_string(), NodeType::Concept);
        assert_eq!(req.content, "test");
    }

    #[test]
    fn test_query_request() {
        let req = QueryRequest {
            query: "find AI".to_string(),
            limit: Some(10),
            include_metadata: false,
        };
        assert_eq!(req.query, "find AI");
    }
}
