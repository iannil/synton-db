// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    models::{
        AddEdgeRequest, AddEdgeResponse, AddNodeRequest, AddNodeResponse, ChunkInfo,
        ChunkingStrategy as ApiChunkingStrategy, DatabaseStats, DeleteNodeRequest,
        DeleteNodeResponse, GetNodeRequest, GetNodeResponse, HealthResponse,
        IngestDocumentRequest, IngestDocumentResponse, MemoryStats, QueryRequest,
        QueryResponse, TraverseRequest, TraverseResponse,
    },
    ApiError, ApiResult,
};
use synton_core::{Edge, Node, NodeType};
use synton_graph::{Graph, MemoryGraph, TraverseDirection, TraversalConfig};
use synton_memory::MemoryManager;

#[cfg(feature = "ml")]
use synton_ml::EmbeddingService;

use synton_storage::Store;
use synton_vector::{VectorIndex, MemoryVectorIndex};
use synton_chunking::{
    ChunkMetadata, ChunkingStrategy as ChunkingStrategyTrait,
    FixedChunker, FixedChunkConfig, HierarchicalChunker,
    HierarchicalChunkConfig, SemanticChunker, SemanticChunkConfig,
};

use synton_instrument::TraceCollector;

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

    /// Persistent storage backend (optional).
    store: Option<Arc<dyn Store>>,

    /// Vector index for semantic search.
    vector_index: Option<Arc<dyn VectorIndex>>,

    /// Embedding service (optional, requires ML feature).
    #[cfg(feature = "ml")]
    embedding: Option<Arc<EmbeddingService>>,

    /// Whether persistence is enabled.
    persistence_enabled: bool,

    /// Instrumentation collector.
    pub collector: &'static TraceCollector,
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
            store: None,
            vector_index: None,
            persistence_enabled: false,
            #[cfg(feature = "ml")]
            embedding: None,
            collector: TraceCollector::global(),
        }
    }

    /// Create a new service instance with persistent storage.
    pub fn with_store(store: Arc<dyn Store>) -> Self {
        let graph = Arc::new(RwLock::new(MemoryGraph::new()));
        let memory = Arc::new(RwLock::new(MemoryManager::new()));
        let nodes = Arc::new(RwLock::new(HashMap::new()));

        Self {
            graph,
            memory,
            nodes,
            store: Some(store),
            vector_index: None,
            persistence_enabled: true,
            #[cfg(feature = "ml")]
            embedding: None,
            collector: TraceCollector::global(),
        }
    }

    /// Create a new service instance with embedding support.
    #[cfg(feature = "ml")]
    pub fn with_embedding(embedding: Arc<EmbeddingService>) -> Self {
        let graph = Arc::new(RwLock::new(MemoryGraph::new()));
        let memory = Arc::new(RwLock::new(MemoryManager::new()));
        let nodes = Arc::new(RwLock::new(HashMap::new()));

        // Create vector index with embedding dimension
        let vector_index = Some(Arc::new(MemoryVectorIndex::new(embedding.dimension())) as Arc<dyn VectorIndex>);

        Self {
            graph,
            memory,
            nodes,
            store: None,
            vector_index,
            persistence_enabled: false,
            embedding: Some(embedding),
            collector: TraceCollector::global(),
        }
    }

    /// Create a new service instance with both store and embedding support.
    #[cfg(feature = "ml")]
    pub fn with_store_and_embedding(store: Arc<dyn Store>, embedding: Arc<EmbeddingService>) -> Self {
        let graph = Arc::new(RwLock::new(MemoryGraph::new()));
        let memory = Arc::new(RwLock::new(MemoryManager::new()));
        let nodes = Arc::new(RwLock::new(HashMap::new()));

        // Create vector index with embedding dimension
        let vector_index = Some(Arc::new(MemoryVectorIndex::new(embedding.dimension())) as Arc<dyn VectorIndex>);

        Self {
            graph,
            memory,
            nodes,
            store: Some(store),
            vector_index,
            persistence_enabled: true,
            embedding: Some(embedding),
            collector: TraceCollector::global(),
        }
    }

    /// Set the embedding service.
    #[cfg(feature = "ml")]
    pub fn set_embedding(&mut self, embedding: Arc<EmbeddingService>) {
        let vector_index = Some(Arc::new(MemoryVectorIndex::new(embedding.dimension())) as Arc<dyn VectorIndex>);
        self.embedding = Some(embedding);
        self.vector_index = vector_index;
    }

    /// Set the persistent store.
    pub fn set_store(&mut self, store: Arc<dyn Store>) {
        self.store = Some(store);
        self.persistence_enabled = true;
    }

    /// Set the vector index.
    pub fn set_vector_index(&mut self, index: Arc<dyn VectorIndex>) {
        self.vector_index = Some(index);
    }

    /// Get a reference to the embedding service.
    #[cfg(feature = "ml")]
    pub fn embedding(&self) -> Option<&Arc<EmbeddingService>> {
        self.embedding.as_ref()
    }

    /// Get a reference to the store.
    pub fn store(&self) -> Option<&Arc<dyn Store>> {
        self.store.as_ref()
    }

    /// Get a reference to the vector index.
    pub fn vector_index(&self) -> Option<&Arc<dyn VectorIndex>> {
        self.vector_index.as_ref()
    }

    /// Check if persistence is enabled.
    pub fn is_persistence_enabled(&self) -> bool {
        self.persistence_enabled
    }

    /// Initialize the service by loading data from persistent storage.
    pub async fn initialize_from_store(&self) -> ApiResult<()> {
        let Some(store) = &self.store else {
            return Ok(()); // No store configured, nothing to load
        };

        // Load all nodes from storage
        let mut nodes = Vec::new();
        let mut stream = store.scan_nodes(None).await?;
        while let Some(node_result) = futures::StreamExt::next(&mut stream).await {
            match node_result {
                Ok(node) => nodes.push(node),
                Err(e) => {
                    tracing::warn!("Failed to load node from storage: {}", e);
                }
            }
        }

        // Load all edges by scanning
        let mut edges = Vec::new();
        // For now, we'll load edges by iterating through nodes and getting their edges
        for node in &nodes {
            if let Ok(outgoing) = store.get_outgoing_edges(node.id).await {
                edges.extend(outgoing);
            }
        }

        self.initialize(nodes, edges).await
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

    // ========== Helper methods for add_node() ==========

    /// Create a node with optional embedding and attributes.
    async fn create_node_with_embedding(
        &self,
        request: &AddNodeRequest,
    ) -> ApiResult<Node> {
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

        let mut node = Node::new(request.content.clone(), request.node_type);
        if let Some(emb) = embedding {
            node = node.with_embedding(emb);
        }
        if let Some(ref attrs) = request.attributes {
            node = node.with_attributes(attrs.clone());
        }

        Ok(node)
    }

    /// Check if a node exists in memory or storage.
    async fn check_node_exists(&self, node_id: Uuid) -> (bool, bool) {
        let exists_in_memory = {
            let nodes = self.nodes.read().await;
            nodes.contains_key(&node_id)
        };

        let exists_in_storage = if self.persistence_enabled {
            if let Some(store) = &self.store {
                store.node_exists(node_id).await.unwrap_or(false)
            } else {
                false
            }
        } else {
            false
        };

        (exists_in_memory, exists_in_storage)
    }

    /// Load an existing node from storage into memory.
    async fn load_node_from_storage(&self, node_id: Uuid) -> ApiResult<Option<Node>> {
        if !self.persistence_enabled {
            return Ok(None);
        }

        let Some(store) = &self.store else {
            return Ok(None);
        };

        let Some(existing) = store.get_node(node_id).await.ok().flatten() else {
            return Ok(None);
        };

        // Add to memory structures
        {
            let mut graph = self.graph.write().await;
            let _ = graph.add_node(existing.clone());
        }
        {
            let mut nodes = self.nodes.write().await;
            nodes.insert(existing.id, existing.clone());
        }
        {
            let mut memory = self.memory.write().await;
            let _ = memory.register(existing.clone());
        }

        Ok(Some(existing))
    }

    /// Persist a node to storage if enabled.
    async fn persist_node(&self, node: &Node) -> ApiResult<()> {
        if !self.persistence_enabled {
            return Ok(());
        }

        let Some(store) = &self.store else {
            return Ok(());
        };

        store.put_node(node).await.map_err(|e| {
            tracing::error!("Failed to persist node: {}", e);
            ApiError::Storage(format!("Failed to persist node: {}", e))
        })
    }

    /// Add a node to all in-memory structures.
    async fn add_node_to_memory(&self, node: &Node) -> ApiResult<()> {
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

        Ok(())
    }

    /// Index a node's vector in the vector index if available.
    async fn index_node_vector(&self, node: &Node) {
        let Some(ref vector_index) = self.vector_index else {
            return;
        };

        let Some(ref embedding) = node.embedding else {
            return;
        };

        if let Err(e) = vector_index.insert(node.id, embedding.clone()).await {
            tracing::warn!("Failed to index node vector: {}", e);
        }
    }

    // ========== Public API methods ==========

    /// Add a node to the database.
    pub async fn add_node(&self, request: AddNodeRequest) -> ApiResult<AddNodeResponse> {
        // Create node with embedding
        let node = self.create_node_with_embedding(&request).await?;

        // Check if node already exists
        let (exists_in_memory, exists_in_storage) = self.check_node_exists(node.id).await;

        // Return existing node if found
        if exists_in_memory {
            let nodes = self.nodes.read().await;
            if let Some(existing) = nodes.get(&node.id).cloned() {
                return Ok(AddNodeResponse {
                    node: existing,
                    created: false,
                });
            }
        }

        if exists_in_storage {
            if let Some(existing) = self.load_node_from_storage(node.id).await? {
                return Ok(AddNodeResponse {
                    node: existing,
                    created: false,
                });
            }
        }

        // Persist to storage
        self.persist_node(&node).await?;

        // Add to memory structures
        self.add_node_to_memory(&node).await?;

        // Index vector
        self.index_node_vector(&node).await;

        Ok(AddNodeResponse {
            node,
            created: true,
        })
    }

    /// Add an edge to the database.
    pub async fn add_edge(&self, request: AddEdgeRequest) -> ApiResult<AddEdgeResponse> {
        let edge = Edge::with_weight(request.source, request.target, request.relation, request.weight);

        // Validate nodes exist (check memory first, then storage if enabled)
        let source_exists = {
            let nodes = self.nodes.read().await;
            nodes.contains_key(&request.source)
        };
        let target_exists = {
            let nodes = self.nodes.read().await;
            nodes.contains_key(&request.target)
        };

        let (source_valid, target_valid) = if self.persistence_enabled {
            let store_source = if !source_exists {
                if let Some(store) = &self.store {
                    store.node_exists(request.source).await.unwrap_or(false)
                } else {
                    false
                }
            } else {
                true
            };
            let store_target = if !target_exists {
                if let Some(store) = &self.store {
                    store.node_exists(request.target).await.unwrap_or(false)
                } else {
                    false
                }
            } else {
                true
            };
            (source_exists || store_source, target_exists || store_target)
        } else {
            (source_exists, target_exists)
        };

        if !source_valid {
            return Err(ApiError::NodeNotFound(request.source));
        }
        if !target_valid {
            return Err(ApiError::NodeNotFound(request.target));
        }

        // Add to persistent storage if enabled
        if self.persistence_enabled {
            if let Some(store) = &self.store {
                if let Err(e) = store.put_edge(&edge).await {
                    tracing::error!("Failed to persist edge: {}", e);
                    return Err(ApiError::Storage(format!("Failed to persist edge: {}", e)));
                }
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
    /// Get a node by ID.
    pub async fn get_node(&self, request: GetNodeRequest) -> ApiResult<GetNodeResponse> {
        // First check in-memory cache
        {
            let nodes = self.nodes.read().await;
            if let Some(node) = nodes.get(&request.id) {
                return Ok(GetNodeResponse {
                    node: Some(node.clone()),
                });
            }
        }

        // If not in memory and persistence is enabled, check storage
        if self.persistence_enabled {
            if let Some(store) = &self.store {
                match store.get_node(request.id).await {
                    Ok(Some(node)) => {
                        // Cache in memory
                        {
                            let mut nodes = self.nodes.write().await;
                            nodes.insert(node.id, node.clone());
                        }
                        {
                            let mut graph = self.graph.write().await;
                            let _ = graph.add_node(node.clone());
                        }
                        {
                            let mut memory = self.memory.write().await;
                            let _ = memory.register(node.clone());
                        }
                        return Ok(GetNodeResponse {
                            node: Some(node),
                        });
                    }
                    Ok(None) => {}
                    Err(e) => {
                        tracing::warn!("Failed to get node from storage: {}", e);
                    }
                }
            }
        }

        Ok(GetNodeResponse {
            node: None,
        })
    }

    /// Delete a node by ID.
    pub async fn delete_node(&self, request: DeleteNodeRequest) -> ApiResult<DeleteNodeResponse> {
        let was_in_memory = {
            let mut nodes = self.nodes.write().await;
            nodes.remove(&request.id)
        };

        // Remove from persistent storage if enabled
        let was_in_storage = if self.persistence_enabled {
            if let Some(store) = &self.store {
                match store.delete_node(request.id).await {
                    Ok(deleted) => deleted,
                    Err(e) => {
                        tracing::error!("Failed to delete node from storage: {}", e);
                        false
                    }
                }
            } else {
                false
            }
        } else {
            false
        };

        let deleted = was_in_memory.is_some() || was_in_storage;

        if deleted {
            // Also remove from memory manager
            let mut memory = self.memory.write().await;
            memory.unregister(request.id);
        }

        Ok(DeleteNodeResponse {
            deleted,
            id: request.id,
        })
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
        let truncated = request.limit.is_some_and(|l| nodes.len() > l);

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

    /// Hybrid search combining vector similarity and graph traversal.
    pub async fn hybrid_search(&self, query: &str, k: usize) -> ApiResult<Vec<Node>> {
        #[cfg(feature = "ml")]
        {
            // Generate query embedding
            let query_embedding = if let Some(embedding_service) = &self.embedding {
                match embedding_service.embed(query).await {
                    Ok(emb) => emb,
                    Err(e) => {
                        tracing::warn!("Failed to generate query embedding: {}", e);
                        return self.simple_text_search(query, Some(k)).await;
                    }
                }
            } else {
                return self.simple_text_search(query, Some(k)).await;
            };

            // Use vector index if available
            if let Some(vector_index) = &self.vector_index {
                match vector_index.search(&query_embedding, k).await {
                    Ok(search_results) => {
                        let mut result_nodes = Vec::new();
                        let nodes = self.nodes.read().await;

                        for result in search_results {
                            if let Some(node) = nodes.get(&result.id) {
                                result_nodes.push(node.clone());
                            }
                        }

                        return Ok(result_nodes);
                    }
                    Err(e) => {
                        tracing::warn!("Vector search failed: {}, falling back to text search", e);
                    }
                }
            }

            // Fallback to text search if vector index is not available
            self.simple_text_search(query, Some(k)).await
        }

        #[cfg(not(feature = "ml"))]
        {
            // No ML feature enabled, use simple text search
            self.simple_text_search(query, Some(k)).await
        }
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

    /// Ingest a document with automatic chunking.
    pub async fn ingest_document(
        &self,
        request: IngestDocumentRequest,
    ) -> ApiResult<IngestDocumentResponse> {
        let start = std::time::Instant::now();

        // Create document node
        let title = request.title.as_deref().unwrap_or("Untitled Document");

        let document_node = Node::new(
            format!("{}: {}", title, request.content),
            NodeType::Concept,
        );

        // Add document node
        let mut nodes = self.nodes.write().await;
        nodes.insert(document_node.id, document_node.clone());
        drop(nodes);

        // Chunk the document using the chunking crate
        let chunks = match request.chunking.as_ref().unwrap_or(&ApiChunkingStrategy::default()) {
            ApiChunkingStrategy::Fixed { chunk_size, overlap } => {
                let metadata = ChunkMetadata {
                    source: Some(title.to_string()),
                    ..Default::default()
                };
                let config = FixedChunkConfig::new(*chunk_size, *overlap);
                let chunker = FixedChunker::with_config(config)
                    .map_err(|e| ApiError::Internal(e.to_string()))?;
                chunker
                    .chunk(&request.content, metadata)
                    .await
                    .map_err(|e| ApiError::Internal(e.to_string()))?
            }
            ApiChunkingStrategy::Semantic {
                min_chunk_size: _,
                max_chunk_size,
                boundary_threshold,
            } => {
                let metadata = ChunkMetadata {
                    source: Some(title.to_string()),
                    ..Default::default()
                };

                // For semantic chunking, use the built-in semantic chunker
                let config = SemanticChunkConfig::new(*max_chunk_size, *boundary_threshold);
                let chunker = SemanticChunker::with_config(config)
                    .map_err(|e| ApiError::Internal(e.to_string()))?;
                chunker
                    .chunk(&request.content, metadata)
                    .await
                    .map_err(|e| ApiError::Internal(e.to_string()))?
            }
            ApiChunkingStrategy::Hierarchical {
                include_sentences: _,
                include_paragraphs: _,
            } => {
                let metadata = ChunkMetadata {
                    source: Some(title.to_string()),
                    ..Default::default()
                };
                let config = HierarchicalChunkConfig::new();
                let chunker = HierarchicalChunker::with_config(config)
                    .map_err(|e| ApiError::Internal(e.to_string()))?;
                chunker
                    .chunk(&request.content, metadata)
                    .await
                    .map_err(|e| ApiError::Internal(e.to_string()))?
            }
        };

        // Process chunks and create nodes
        let mut chunk_infos = Vec::new();
        let mut nodes = self.nodes.write().await;
        let mut graph = self.graph.write().await;

        for chunk in chunks {
            // Create node for chunk
            let chunk_node = Node::new(chunk.content.clone(), NodeType::Concept);

            // Link to document (chunk is part of document)
            let _ = graph.add_edge(synton_core::Edge::new(
                chunk_node.id,
                document_node.id,
                synton_core::Relation::IsPartOf,
            ));

            // Generate embeddings if requested
            if request.embed {
                #[cfg(feature = "ml")]
                if let Some(embedding_service) = &self.embedding {
                    if let Ok(embedding) = embedding_service
                        .embed(&chunk.content)
                        .await
                    {
                        let mut node_with_embedding = chunk_node.clone();
                        node_with_embedding.embedding = Some(embedding);
                        nodes.insert(chunk_node.id, node_with_embedding.clone());

                        // Add to vector index
                        if let Some(vector_index) = &self.vector_index {
                            if let Some(emb) = &node_with_embedding.embedding {
                                let _ = vector_index
                                    .insert(chunk_node.id, emb.clone())
                                    .await;
                            }
                        }
                        continue;
                    }
                }
            }

            nodes.insert(chunk_node.id, chunk_node.clone());

            chunk_infos.push(ChunkInfo {
                id: chunk.id,
                content: chunk.content,
                index: chunk.index,
                range: chunk.range,
                chunk_type: format!("{:?}", chunk.chunk_type),
                parent_id: chunk.parent_id,
                child_ids: chunk.child_ids,
            });
        }

        let processing_time_ms = start.elapsed().as_millis() as u64;

        Ok(IngestDocumentResponse {
            document_id: document_node.id,
            chunk_count: chunk_infos.len(),
            chunks: chunk_infos,
            embedded: request.embed,
            processing_time_ms,
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
