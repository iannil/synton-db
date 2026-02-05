// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    models::{
        AddEdgeRequest, AddEdgeResponse, AddNodeRequest, AddNodeResponse, DatabaseStats,
        DeleteNodeRequest, DeleteNodeResponse, GetNodeRequest, GetNodeResponse, HealthResponse,
        MemoryStats, QueryRequest, QueryResponse, TraverseRequest, TraverseResponse,
    },
    ApiError, ApiResult,
};
use synton_core::{Edge, Node};
use synton_graph::{Graph, MemoryGraph, TraverseDirection, TraversalConfig};
use synton_memory::MemoryManager;

#[cfg(feature = "ml")]
use synton_ml::EmbeddingService;

/// Main SYNTON-DB service.
///
/// Combines all database components into a unified service.
pub struct SyntonDbService {
    /// In-memory graph for traversal.
    graph: Arc<RwLock<MemoryGraph>>,

    /// Memory manager for access score tracking.
    memory: Arc<RwLock<MemoryManager>>,

    /// Node lookup (for quick access by ID).
    nodes: Arc<RwLock<HashMap<Uuid, Node>>>,

    /// Embedding service (optional, requires ML feature).
    #[cfg(feature = "ml")]
    embedding: Option<Arc<EmbeddingService>>,
}

impl SyntonDbService {
    /// Create a new service instance.
    pub fn new() -> Self {
        let graph = Arc::new(RwLock::new(MemoryGraph::new()));
        let memory = Arc::new(RwLock::new(MemoryManager::new()));
        let nodes = Arc::new(RwLock::new(HashMap::new()));

        Self {
            graph,
            memory,
            nodes,
            #[cfg(feature = "ml")]
            embedding: None,
        }
    }

    /// Create a new service instance with embedding support.
    #[cfg(feature = "ml")]
    pub fn with_embedding(embedding: Arc<EmbeddingService>) -> Self {
        let graph = Arc::new(RwLock::new(MemoryGraph::new()));
        let memory = Arc::new(RwLock::new(MemoryManager::new()));
        let nodes = Arc::new(RwLock::new(HashMap::new()));

        Self {
            graph,
            memory,
            nodes,
            embedding: Some(embedding),
        }
    }

    /// Set the embedding service.
    #[cfg(feature = "ml")]
    pub fn set_embedding(&mut self, embedding: Arc<EmbeddingService>) {
        self.embedding = Some(embedding);
    }

    /// Get a reference to the embedding service.
    #[cfg(feature = "ml")]
    pub fn embedding(&self) -> Option<&Arc<EmbeddingService>> {
        self.embedding.as_ref()
    }

    /// Initialize the service with existing data.
    pub async fn initialize(&self, init_nodes: Vec<Node>, init_edges: Vec<Edge>) -> ApiResult<()> {
        let mut graph = self.graph.write().await;
        let mut nodes_map = self.nodes.write().await;

        for node in &init_nodes {
            graph.add_node(node.clone())?;
            nodes_map.insert(node.id, node.clone());
        }

        for edge in init_edges {
            graph.add_edge(edge)?;
        }

        // Initialize memory manager with nodes
        let mut memory = self.memory.write().await;
        for node in &init_nodes {
            memory.register(node.clone())?;
        }

        Ok(())
    }

    /// Add a node to the database.
    pub async fn add_node(&self, request: AddNodeRequest) -> ApiResult<AddNodeResponse> {
        // Generate embedding if ML feature is enabled
        #[cfg(feature = "ml")]
        let embedding = if let Some(embedding_service) = &self.embedding {
            match embedding_service.embed(&request.content).await {
                Ok(emb) => Some(emb),
                Err(e) => {
                    tracing::warn!("Failed to generate embedding: {}", e);
                    None
                }
            }
        } else {
            None
        };

        #[cfg(not(feature = "ml"))]
        let embedding = None;

        let node = Node::new(request.content.clone(), request.node_type);
        let node = if let Some(emb) = embedding {
            node.with_embedding(emb)
        } else {
            node
        };

        // Check if node already exists
        let exists = {
            let nodes = self.nodes.read().await;
            nodes.contains_key(&node.id)
        };

        if exists {
            // Return existing node
            let nodes = self.nodes.read().await;
            let existing = nodes.get(&node.id).cloned();
            return Ok(AddNodeResponse {
                node: existing.unwrap(),
                created: false,
            });
        }

        // Add to graph
        {
            let mut graph = self.graph.write().await;
            graph.add_node(node.clone())?;
        }

        // Add to nodes map
        {
            let mut nodes = self.nodes.write().await;
            nodes.insert(node.id, node.clone());
        }

        // Add to memory manager
        {
            let mut memory = self.memory.write().await;
            memory.register(node.clone())?;
        }

        Ok(AddNodeResponse {
            node,
            created: true,
        })
    }

    /// Add an edge to the database.
    pub async fn add_edge(&self, request: AddEdgeRequest) -> ApiResult<AddEdgeResponse> {
        let edge = Edge::with_weight(request.source, request.target, request.relation, request.weight);

        // Validate nodes exist
        {
            let nodes = self.nodes.read().await;
            if !nodes.contains_key(&request.source) {
                return Err(ApiError::NodeNotFound(request.source));
            }
            if !nodes.contains_key(&request.target) {
                return Err(ApiError::NodeNotFound(request.target));
            }
        }

        // Add to graph
        {
            let mut graph = self.graph.write().await;
            graph.add_edge(edge.clone())?;
        }

        Ok(AddEdgeResponse { edge })
    }

    /// Get a node by ID.
    pub async fn get_node(&self, request: GetNodeRequest) -> ApiResult<GetNodeResponse> {
        let nodes = self.nodes.read().await;
        Ok(GetNodeResponse {
            node: nodes.get(&request.id).cloned(),
        })
    }

    /// Delete a node by ID.
    pub async fn delete_node(&self, request: DeleteNodeRequest) -> ApiResult<DeleteNodeResponse> {
        let removed = {
            let mut nodes = self.nodes.write().await;
            nodes.remove(&request.id)
        };

        if removed.is_some() {
            // Also remove from memory
            let mut memory = self.memory.write().await;
            memory.unregister(request.id);
            Ok(DeleteNodeResponse {
                deleted: true,
                id: request.id,
            })
        } else {
            Ok(DeleteNodeResponse {
                deleted: false,
                id: request.id,
            })
        }
    }

    /// Query the database.
    pub async fn query(&self, request: QueryRequest) -> ApiResult<QueryResponse> {
        let start = std::time::Instant::now();

        // Parse query using PaQL
        let parser = synton_paql::Parser::new();
        let parsed_query = parser.parse(&request.query)?;

        // Execute query (simplified MVP implementation)
        let nodes = self.text_search(&parsed_query.root, request.limit).await?;

        let elapsed = start.elapsed().as_millis() as u64;
        let total_count = nodes.len();
        let truncated = request.limit.map_or(false, |l| nodes.len() > l);

        Ok(QueryResponse {
            nodes,
            total_count,
            execution_time_ms: elapsed,
            truncated,
        })
    }

    /// Traverse the graph.
    pub async fn traverse(&self, request: TraverseRequest) -> ApiResult<TraverseResponse> {
        let graph = self.graph.read().await;

        let config = TraversalConfig::with_depth(request.max_depth)
            .with_max_nodes(request.max_nodes)
            .with_direction(request.direction.into());

        let result = graph.bfs(request.start_id, config).await?;

        // Get edges for the nodes
        let mut edges = Vec::new();
        for node in &result.nodes {
            let node_edges = graph.edges(node.id, TraverseDirection::Forward).await?;
            edges.extend(node_edges);
        }

        Ok(TraverseResponse {
            nodes: result.nodes,
            edges,
            depth: result.depth,
            truncated: result.truncated,
        })
    }

    /// Get database statistics.
    pub async fn stats(&self) -> ApiResult<DatabaseStats> {
        let graph = self.graph.read().await;
        let memory = self.memory.read().await;

        let node_count = graph.count_nodes().await?;
        let edge_count = graph.count_edges().await?;

        // Count nodes with embeddings
        let nodes = self.nodes.read().await;
        let embedded_count = nodes.values().filter(|n| n.embedding.is_some()).count();

        let memory_stats = memory.stats();

        Ok(DatabaseStats {
            node_count,
            edge_count,
            embedded_count,
            memory_stats: MemoryStats {
                total_nodes: memory_stats.total_nodes,
                active_nodes: memory_stats.active_nodes,
                decayed_nodes: memory_stats.decayed_nodes,
                average_score: memory_stats.average_score,
                load_factor: memory_stats.load_factor,
            },
        })
    }

    /// Health check.
    pub fn health(&self) -> HealthResponse {
        HealthResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_secs: 0,
        }
    }

    /// Get all nodes.
    pub async fn all_nodes(&self) -> Vec<Node> {
        let nodes = self.nodes.read().await;
        nodes.values().cloned().collect()
    }

    /// Get the graph reference.
    pub async fn graph(&self) -> Arc<RwLock<MemoryGraph>> {
        self.graph.clone()
    }

    /// Get the memory manager reference.
    pub async fn memory(&self) -> Arc<RwLock<MemoryManager>> {
        self.memory.clone()
    }

    /// Text search implementation (non-recursive to avoid boxing).
    async fn text_search(
        &self,
        query_node: &synton_paql::QueryNode,
        limit: Option<usize>,
    ) -> ApiResult<Vec<Node>> {
        use std::collections::HashSet;
        use synton_paql::QueryNode;

        // Use an explicit stack for iterative processing instead of recursion
        let mut stack = vec![(query_node, false)];
        let mut results_map: HashMap<Uuid, Node> = HashMap::new();
        let exclude_set: HashSet<Uuid> = HashSet::new();
        let mut operation_count = 0;

        while let Some((current_node, processed)) = stack.pop() {
            if processed {
                // Second pass - combine results
                continue;
            }

            match current_node {
                QueryNode::TextSearch { query } => {
                    let query_lower = query.to_lowercase();
                    let nodes = self.nodes.read().await;

                    let mut found: Vec<_> = nodes
                        .values()
                        .filter(|n| {
                            !exclude_set.contains(&n.id)
                                && n.content().to_lowercase().contains(&query_lower)
                        })
                        .cloned()
                        .collect();

                    // Sort by access score (descending)
                    found.sort_by(|a, b| {
                        b.meta
                            .access_score
                            .partial_cmp(&a.meta.access_score)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });

                    // Merge into results, keeping higher access scores
                    for node in found {
                        results_map
                            .entry(node.id)
                            .and_modify(|existing| {
                                if node.meta.access_score > existing.meta.access_score {
                                    *existing = node.clone();
                                }
                            })
                            .or_insert(node);
                    }
                }
                QueryNode::And { left, right } => {
                    // Process both sides first
                    stack.push((left, false));
                    stack.push((right, false));
                }
                QueryNode::Or { left, right } => {
                    stack.push((left, false));
                    stack.push((right, false));
                }
                QueryNode::Not { input: _ } => {
                    // For MVP, skip Not queries in text search
                }
                QueryNode::GraphTraversal { .. } => {
                    // For MVP, skip graph traversal in text search
                }
                QueryNode::HybridSearch { .. } => {
                    // For MVP, treat as text search
                }
                QueryNode::Filter { input, .. } => {
                    stack.push((input, false));
                }
                _ => {}
            }

            operation_count += 1;
            if operation_count > 100 {
                break; // Safety limit
            }
        }

        let mut results: Vec<_> = results_map.into_values().collect();

        // Sort by access score (descending)
        results.sort_by(|a, b| {
            b.meta
                .access_score
                .partial_cmp(&a.meta.access_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply limit
        if let Some(limit) = limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// Simple text search helper.
    async fn simple_text_search(&self, query: &str, limit: Option<usize>) -> ApiResult<Vec<Node>> {
        let query_lower = query.to_lowercase();
        let nodes = self.nodes.read().await;

        let mut results: Vec<_> = nodes
            .values()
            .filter(|n| n.content().to_lowercase().contains(&query_lower))
            .cloned()
            .collect();

        // Sort by access score (descending)
        results.sort_by(|a, b| {
            b.meta
                .access_score
                .partial_cmp(&a.meta.access_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply limit
        if let Some(limit) = limit {
            results.truncate(limit);
        }

        Ok(results)
    }
}

impl Default for SyntonDbService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use synton_core::{NodeType, Relation};

    #[tokio::test]
    async fn test_service_creation() {
        let service = SyntonDbService::new();
        let health = service.health();
        assert_eq!(health.status, "healthy");
    }

    #[tokio::test]
    async fn test_add_node() {
        let service = SyntonDbService::new();
        let request = AddNodeRequest::new("Test concept".to_string(), NodeType::Concept);

        let response = service.add_node(request).await.unwrap();
        assert!(response.created);
        assert_eq!(response.node.content(), "Test concept");
    }

    #[tokio::test]
    async fn test_query() {
        let service = SyntonDbService::new();

        // Add a test node
        let node = Node::new("Machine learning concept", NodeType::Concept);
        service
            .add_node(AddNodeRequest::new(node.content().to_string(), node.node_type))
            .await
            .unwrap();

        let query = QueryRequest {
            query: "machine".to_string(),
            limit: Some(10),
            include_metadata: false,
        };

        let response = service.query(query).await.unwrap();
        assert!(!response.nodes.is_empty());
    }

    #[tokio::test]
    async fn test_stats() {
        let service = SyntonDbService::new();
        let stats = service.stats().await.unwrap();

        assert_eq!(stats.node_count, 0);
        assert_eq!(stats.edge_count, 0);
    }

    #[tokio::test]
    async fn test_add_edge() {
        let service = SyntonDbService::new();

        let n1_resp = service
            .add_node(AddNodeRequest::new("Node 1".to_string(), NodeType::Entity))
            .await
            .unwrap();
        let n2_resp = service
            .add_node(AddNodeRequest::new("Node 2".to_string(), NodeType::Entity))
            .await
            .unwrap();

        let edge_request = AddEdgeRequest {
            source: n1_resp.node.id,
            target: n2_resp.node.id,
            relation: Relation::Causes,
            ..Default::default()
        };

        let response = service.add_edge(edge_request).await.unwrap();
        assert_eq!(response.edge.source, n1_resp.node.id);
        assert_eq!(response.edge.target, n2_resp.node.id);
    }
}
