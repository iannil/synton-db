// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! HTTP client for SYNTON-DB REST API.
//!
//! This module provides a client for communicating with a running SYNTON-DB
//! instance via its REST API.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{McpError, McpResult};
use synton_core::{Edge, Node, NodeType, Relation};

/// Default endpoint for SYNTON-DB REST API.
pub const DEFAULT_ENDPOINT: &str = "http://localhost:8080";

/// HTTP client for SYNTON-DB.
#[derive(Clone)]
pub struct SyntonDbClient {
    /// HTTP client.
    client: reqwest::Client,
    /// API endpoint.
    endpoint: String,
    /// In-memory node cache.
    cache: Arc<RwLock<lru::LruCache<String, serde_json::Value>>>,
}

impl SyntonDbClient {
    /// Create a new client with the default endpoint.
    pub fn new() -> Self {
        Self::with_endpoint(DEFAULT_ENDPOINT)
    }

    /// Create a new client with a custom endpoint.
    pub fn with_endpoint(endpoint: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint: endpoint.into(),
            cache: Arc::new(RwLock::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(1024).unwrap(),
            ))),
        }
    }

    /// Get the API endpoint.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Health check.
    pub async fn health(&self) -> McpResult<HealthStatus> {
        let url = format!("{}/health", self.endpoint);
        let response = self.client.get(&url).send().await.map_err(|e| {
            McpError::Http(format!("Failed to connect to SYNTON-DB at {}: {}", self.endpoint, e))
        })?;

        if !response.status().is_success() {
            return Err(McpError::Api(format!(
                "Health check failed: {}",
                response.status()
            )));
        }

        let health: HealthStatus = response.json().await?;
        Ok(health)
    }

    /// Add a node to the database.
    pub async fn add_node(
        &self,
        content: String,
        node_type: NodeType,
    ) -> McpResult<AddNodeResponse> {
        let url = format!("{}/nodes", self.endpoint);
        let request = AddNodeRequest {
            content,
            node_type,
            embedding: None,
            attributes: None,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| McpError::Http(format!("Failed to add node: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(McpError::Api(format!(
                "Failed to add node: {} - {}",
                status, text
            )));
        }

        let result: AddNodeResponse = response.json().await?;
        Ok(result)
    }

    /// Get a node by ID.
    pub async fn get_node(&self, id: Uuid) -> McpResult<Option<Node>> {
        let url = format!("{}/nodes/{}", self.endpoint, id);

        // Check cache first
        let cache_key = format!("node:{}", id);
        {
            let mut cache = self.cache.write().await;
            if let Some(cached) = cache.get(&cache_key) {
                if let Ok(node) = serde_json::from_value::<Node>(cached.clone()) {
                    return Ok(Some(node));
                }
            }
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| McpError::Http(format!("Failed to get node: {}", e)))?;

        if response.status() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(McpError::Api(format!(
                "Failed to get node: {}",
                response.status()
            )));
        }

        let result: GetNodeResponse = response.json().await?;
        let node = result.node;

        // Cache the result
        if let Some(ref n) = node {
            let mut cache = self.cache.write().await;
            let _ = cache.put(
                cache_key,
                serde_json::to_value(n).unwrap_or_default(),
            );
        }

        Ok(node)
    }

    /// Query the database.
    pub async fn query(&self, query: String, limit: Option<usize>) -> McpResult<QueryResult> {
        let url = format!("{}/query", self.endpoint);
        let request = QueryRequest {
            query,
            limit,
            include_metadata: true,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| McpError::Http(format!("Failed to query: {}", e)))?;

        if !response.status().is_success() {
            return Err(McpError::Api(format!(
                "Query failed: {}",
                response.status()
            )));
        }

        let result: QueryResult = response.json().await?;
        Ok(result)
    }

    /// Hybrid search (Graph-RAG).
    pub async fn hybrid_search(&self, query: String, k: usize) -> McpResult<Vec<Node>> {
        let url = format!("{}/hybrid_search", self.endpoint);
        let request = HybridSearchRequest { query, k };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| McpError::Http(format!("Failed to hybrid search: {}", e)))?;

        if !response.status().is_success() {
            return Err(McpError::Api(format!(
                "Hybrid search failed: {}",
                response.status()
            )));
        }

        let result: HybridSearchResponse = response.json().await?;
        Ok(result.nodes)
    }

    /// Traverse the graph.
    pub async fn traverse(
        &self,
        start_id: Uuid,
        max_depth: usize,
        max_nodes: usize,
    ) -> McpResult<TraverseResult> {
        let url = format!("{}/traverse", self.endpoint);
        let request = TraverseRequest {
            start_id,
            max_depth,
            max_nodes,
            direction: TraverseDirection::Both,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| McpError::Http(format!("Failed to traverse: {}", e)))?;

        if !response.status().is_success() {
            return Err(McpError::Api(format!(
                "Traverse failed: {}",
                response.status()
            )));
        }

        let result: TraverseResult = response.json().await?;
        Ok(result)
    }

    /// Add an edge to the database.
    pub async fn add_edge(
        &self,
        source: Uuid,
        target: Uuid,
        relation: Relation,
        weight: f32,
    ) -> McpResult<Edge> {
        let url = format!("{}/edges", self.endpoint);
        let request = AddEdgeRequest {
            source,
            target,
            relation,
            weight,
            vector: None,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| McpError::Http(format!("Failed to add edge: {}", e)))?;

        if !response.status().is_success() {
            return Err(McpError::Api(format!(
                "Failed to add edge: {}",
                response.status()
            )));
        }

        let result: AddEdgeResponse = response.json().await?;
        Ok(result.edge)
    }

    /// Get database statistics.
    pub async fn stats(&self) -> McpResult<DatabaseStats> {
        let url = format!("{}/stats", self.endpoint);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| McpError::Http(format!("Failed to get stats: {}", e)))?;

        if !response.status().is_success() {
            return Err(McpError::Api(format!(
                "Failed to get stats: {}",
                response.status()
            )));
        }

        let result: DatabaseStats = response.json().await?;
        Ok(result)
    }

    /// Get all nodes.
    pub async fn get_all_nodes(&self) -> McpResult<Vec<Node>> {
        let url = format!("{}/nodes", self.endpoint);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| McpError::Http(format!("Failed to get nodes: {}", e)))?;

        if !response.status().is_success() {
            return Err(McpError::Api(format!(
                "Failed to get nodes: {}",
                response.status()
            )));
        }

        let nodes: Vec<Node> = response.json().await?;
        Ok(nodes)
    }
}

impl Default for SyntonDbClient {
    fn default() -> Self {
        Self::new()
    }
}

// Request/Response types

/// Health check response.
#[derive(Debug, Deserialize)]
pub struct HealthStatus {
    /// Service status.
    pub status: String,
    /// Service version.
    pub version: String,
}

/// Add node request.
#[derive(Debug, Serialize)]
struct AddNodeRequest {
    content: String,
    node_type: NodeType,
    embedding: Option<Vec<f32>>,
    attributes: Option<serde_json::Value>,
}

/// Add node response.
#[derive(Debug, Deserialize)]
pub struct AddNodeResponse {
    /// Created node.
    pub node: Node,
    /// Whether it was newly created.
    pub created: bool,
}

/// Get node response.
#[derive(Debug, Deserialize)]
struct GetNodeResponse {
    node: Option<Node>,
}

/// Query request.
#[derive(Debug, Serialize)]
struct QueryRequest {
    query: String,
    limit: Option<usize>,
    include_metadata: bool,
}

/// Query result.
#[derive(Debug, Deserialize)]
pub struct QueryResult {
    /// Result nodes.
    pub nodes: Vec<Node>,
    /// Total count.
    pub total_count: usize,
    /// Execution time in ms.
    pub execution_time_ms: u64,
    /// Whether results were truncated.
    pub truncated: bool,
}

/// Hybrid search request.
#[derive(Debug, Serialize)]
struct HybridSearchRequest {
    query: String,
    k: usize,
}

/// Hybrid search response.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HybridSearchResponse {
    nodes: Vec<Node>,
    count: usize,
}

/// Traverse direction.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
enum TraverseDirection {
    Forward,
    Backward,
    Both,
}

/// Traverse request.
#[derive(Debug, Serialize)]
struct TraverseRequest {
    start_id: Uuid,
    max_depth: usize,
    max_nodes: usize,
    direction: TraverseDirection,
}

/// Traverse result.
#[derive(Debug, Deserialize)]
pub struct TraverseResult {
    /// Traversed nodes.
    pub nodes: Vec<Node>,
    /// Traversed edges.
    pub edges: Vec<Edge>,
    /// Maximum depth reached.
    pub depth: usize,
    /// Whether traversal was truncated.
    pub truncated: bool,
}

/// Add edge request.
#[derive(Debug, Serialize)]
struct AddEdgeRequest {
    source: Uuid,
    target: Uuid,
    relation: Relation,
    weight: f32,
    vector: Option<Vec<f32>>,
}

/// Add edge response.
#[derive(Debug, Deserialize)]
struct AddEdgeResponse {
    edge: Edge,
}

/// Database statistics.
#[derive(Debug, Deserialize)]
pub struct DatabaseStats {
    /// Node count.
    pub node_count: usize,
    /// Edge count.
    pub edge_count: usize,
    /// Nodes with embeddings.
    pub embedded_count: usize,
    /// Memory statistics.
    pub memory_stats: MemoryStats,
}

/// Memory statistics.
#[derive(Debug, Deserialize)]
pub struct MemoryStats {
    /// Total nodes.
    pub total_nodes: usize,
    /// Active nodes.
    pub active_nodes: usize,
    /// Decayed nodes.
    pub decayed_nodes: usize,
    /// Average access score.
    pub average_score: f32,
    /// Load factor.
    pub load_factor: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = SyntonDbClient::new();
        assert_eq!(client.endpoint(), DEFAULT_ENDPOINT);

        let custom_client = SyntonDbClient::with_endpoint("http://localhost:9090");
        assert_eq!(custom_client.endpoint(), "http://localhost:9090");
    }

    #[test]
    fn test_client_default() {
        let client = SyntonDbClient::default();
        assert_eq!(client.endpoint(), DEFAULT_ENDPOINT);
    }
}
