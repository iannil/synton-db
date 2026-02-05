// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! HTTP client for SYNTON-DB REST API.

use anyhow::Result;
use reqwest::Client;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use synton_core::{Edge, Node, NodeType, Relation};

/// API response wrapper
#[derive(Debug)]
pub struct ApiResponse<T> {
    pub data: T,
    pub status: u16,
}

/// HTTP client for SYNTON-DB REST API.
pub struct SyntonClient {
    base_url: String,
    client: Client,
}

impl SyntonClient {
    /// Create a new API client.
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap();

        Self { base_url, client }
    }

    /// Get the full URL for an endpoint.
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Send a GET request.
    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<ApiResponse<T>> {
        let url = self.url(path);
        let response = self.client.get(&url).send().await?;
        let status = response.status().as_u16();
        let data = response.json().await?;
        Ok(ApiResponse { data, status })
    }

    /// Send a POST request.
    async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<ApiResponse<T>> {
        let url = self.url(path);
        let response = self.client.post(&url).json(body).send().await?;
        let status = response.status().as_u16();
        let data = response.json().await?;
        Ok(ApiResponse { data, status })
    }

    /// Send a DELETE request.
    async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<ApiResponse<T>> {
        let url = self.url(path);
        let response = self.client.delete(&url).send().await?;
        let status = response.status().as_u16();
        let data = response.json().await?;
        Ok(ApiResponse { data, status })
    }

    /// Check health of the server.
    pub async fn health(&self) -> Result<HealthResponse> {
        let resp = self.get::<HealthResponse>("/health").await?;
        Ok(resp.data)
    }

    /// Get database statistics.
    pub async fn stats(&self) -> Result<StatsResponse> {
        let resp = self.get::<StatsResponse>("/stats").await?;
        Ok(resp.data)
    }

    /// Create a new node.
    pub async fn create_node(&self, content: String, node_type: NodeType) -> Result<Node> {
        #[derive(serde::Serialize)]
        struct AddNodeRequest {
            content: String,
            node_type: String,
        }

        let body = AddNodeRequest {
            content,
            node_type: format!("{:?}", node_type),
        };

        let resp = self.post::<serde_json::Value, _>("/nodes", &body).await?;

        // Parse the response
        if let Some(node) = resp.data["node"].as_object() {
            let id = node["id"].as_str().unwrap().to_string();
            let content = node["content"].as_str().unwrap().to_string();
            let node_type_str = node["node_type"].as_str().unwrap();
            let node_type = match node_type_str {
                "Entity" => NodeType::Entity,
                "Concept" => NodeType::Concept,
                "Fact" => NodeType::Fact,
                "RawChunk" => NodeType::RawChunk,
                _ => NodeType::Concept,
            };

            // Create node using NodeBuilder
            let built_node = synton_core::NodeBuilder::new(content, node_type)
                .id(Uuid::parse_str(&id)?)
                .build()?;
            Ok(built_node)
        } else {
            anyhow::bail!("Invalid response format");
        }
    }

    /// Get a node by ID.
    pub async fn get_node(&self, id: Uuid) -> Result<Option<Node>> {
        let resp = self.get::<serde_json::Value>(&format!("/nodes/{}", id)).await?;

        if resp.status == 404 {
            return Ok(None);
        }

        if let Some(node) = resp.data.get("node") {
            if node.is_null() {
                return Ok(None);
            }
            let id = node["id"].as_str().unwrap().to_string();
            let content = node["content"].as_str().unwrap().to_string();
            let node_type_str = node["node_type"].as_str().unwrap();
            let node_type = match node_type_str {
                "Entity" => NodeType::Entity,
                "Concept" => NodeType::Concept,
                "Fact" => NodeType::Fact,
                "RawChunk" => NodeType::RawChunk,
                _ => NodeType::Concept,
            };

            let built_node = synton_core::NodeBuilder::new(content, node_type)
                .id(Uuid::parse_str(&id)?)
                .build()?;
            return Ok(Some(built_node));
        }

        Ok(None)
    }

    /// Delete a node by ID.
    pub async fn delete_node(&self, id: Uuid) -> Result<bool> {
        #[derive(serde::Deserialize)]
        struct DeleteResponse {
            deleted: bool,
        }

        let resp = self.delete::<DeleteResponse>(&format!("/nodes/{}", id)).await?;
        Ok(resp.data.deleted)
    }

    /// List all nodes.
    pub async fn list_nodes(&self) -> Result<Vec<Node>> {
        let resp = self.get::<Vec<Node>>("/nodes").await?;
        Ok(resp.data)
    }

    /// Create a new edge.
    pub async fn create_edge(
        &self,
        source: Uuid,
        target: Uuid,
        relation: Relation,
        weight: f32,
    ) -> Result<Edge> {
        #[derive(serde::Serialize)]
        struct AddEdgeRequest {
            source: String,
            target: String,
            relation: String,
            weight: f32,
        }

        let body = AddEdgeRequest {
            source: source.to_string(),
            target: target.to_string(),
            relation: relation.to_string(),
            weight,
        };

        let resp = self.post::<serde_json::Value, _>("/edges", &body).await?;

        // Parse the response
        if let Some(edge) = resp.data["edge"].as_object() {
            let source = Uuid::parse_str(edge["source"].as_str().unwrap())?;
            let target = Uuid::parse_str(edge["target"].as_str().unwrap())?;
            let relation_str = edge["relation"].as_str().unwrap();
            let relation = match relation_str {
                "is_a" => Relation::IsA,
                "is_part_of" => Relation::IsPartOf,
                "causes" => Relation::Causes,
                "similar_to" => Relation::SimilarTo,
                "contradicts" => Relation::Contradicts,
                "happened_after" => Relation::HappenedAfter,
                "belongs_to" => Relation::BelongsTo,
                _ => Relation::SimilarTo,
            };
            let weight = edge["weight"].as_f64().unwrap_or(1.0) as f32;

            Ok(synton_core::EdgeBuilder::new(source, target, relation)
                .weight(weight)
                .build()?)
        } else {
            anyhow::bail!("Invalid response format");
        }
    }

    /// Execute a query.
    pub async fn query(&self, query: String, limit: Option<usize>) -> Result<QueryResponse> {
        #[derive(serde::Serialize)]
        struct QueryRequest {
            query: String,
            limit: Option<usize>,
        }

        let body = QueryRequest { query, limit };
        let resp = self.post::<QueryResponse, _>("/query", &body).await?;
        Ok(resp.data)
    }
}

/// Health check response.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Statistics response.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct StatsResponse {
    pub node_count: u64,
    pub edge_count: u64,
    pub embedded_count: u64,
}

/// Query response.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct QueryResponse {
    pub nodes: Vec<serde_json::Value>,
    pub total_count: usize,
    pub execution_time_ms: u64,
    pub truncated: bool,
}
