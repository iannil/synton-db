// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::{
    error::GraphRagResult,
    retrieval::{ContextSource, RetrievedContext, RetrievedNode, RetrievalResult},
    scorer::Scorer,
    RetrievalConfig, RetrievalMode,
};
use synton_core::Node;
use synton_graph::{Graph, TraverseDirection, TraversalConfig};

/// Configuration for Graph-RAG operations.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphRagConfig {
    /// Default retrieval configuration.
    pub retrieval: RetrievalConfig,

    /// Scorer configuration.
    pub scorer: Scorer,

    /// Whether to enable caching.
    pub enable_cache: bool,

    /// Cache TTL in seconds.
    pub cache_ttl_secs: u64,
}

impl Default for GraphRagConfig {
    fn default() -> Self {
        Self {
            retrieval: RetrievalConfig::default(),
            scorer: Scorer::default(),
            enable_cache: true,
            cache_ttl_secs: 300, // 5 minutes
        }
    }
}

impl GraphRagConfig {
    /// Create a new config with retrieval settings.
    pub fn with_retrieval(mut self, retrieval: RetrievalConfig) -> Self {
        self.retrieval = retrieval;
        self
    }

    /// Create a new config with scorer settings.
    pub fn with_scorer(mut self, scorer: Scorer) -> Self {
        self.scorer = scorer;
        self
    }

    /// Create a new config with cache settings.
    pub fn with_cache(mut self, enabled: bool, ttl_secs: u64) -> Self {
        self.enable_cache = enabled;
        self.cache_ttl_secs = ttl_secs;
        self
    }
}

/// Graph-RAG: Combines vector search with graph traversal for enhanced retrieval.
#[async_trait]
pub trait GraphRag: Send + Sync {
    /// Retrieve relevant context based on a query embedding.
    async fn retrieve(
        &self,
        query_embedding: Vec<f32>,
        config: RetrievalConfig,
    ) -> GraphRagResult<RetrievalResult>;

    /// Retrieve and format context for LLM consumption.
    async fn retrieve_context(
        &self,
        query_embedding: Vec<f32>,
        config: RetrievalConfig,
    ) -> GraphRagResult<RetrievedContext>;

    /// Retrieve from seed nodes (graph-only mode).
    async fn retrieve_from_seeds(
        &self,
        seed_ids: Vec<Uuid>,
        config: RetrievalConfig,
    ) -> GraphRagResult<RetrievalResult>;

    /// Hybrid retrieval: vector search + graph expansion.
    async fn hybrid_retrieve(
        &self,
        query_embedding: Vec<f32>,
        top_k: usize,
        max_hops: usize,
    ) -> GraphRagResult<RetrievalResult>;
}

/// In-memory Graph-RAG implementation.
pub struct MemoryGraphRag<G> {
    /// The graph for traversal.
    graph: G,

    /// Node lookup by ID.
    nodes: HashMap<Uuid, Node>,

    /// Configuration.
    config: GraphRagConfig,
}

impl<G> MemoryGraphRag<G>
where
    G: Graph + Send + Sync,
{
    /// Create a new MemoryGraphRag.
    pub fn new(graph: G, nodes: Vec<Node>) -> Self {
        let node_map = nodes.into_iter().map(|n| (n.id, n)).collect();
        Self {
            graph,
            nodes: node_map,
            config: GraphRagConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(graph: G, nodes: Vec<Node>, config: GraphRagConfig) -> Self {
        let node_map = nodes.into_iter().map(|n| (n.id, n)).collect();
        Self {
            graph,
            nodes: node_map,
            config,
        }
    }

    /// Add a node to the index.
    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id, node);
    }

    /// Remove a node from the index.
    pub fn remove_node(&mut self, id: Uuid) -> Option<Node> {
        self.nodes.remove(&id)
    }

    /// Get a node by ID.
    pub fn get_node(&self, id: Uuid) -> Option<&Node> {
        self.nodes.get(&id)
    }

    /// Get the graph reference.
    pub fn graph(&self) -> &G {
        &self.graph
    }

    /// Get mutable reference to the graph.
    pub fn graph_mut(&mut self) -> &mut G {
        &mut self.graph
    }

    /// Format nodes as context string.
    fn format_context(&self, nodes: &[RetrievedNode]) -> String {
        nodes
            .iter()
            .enumerate()
            .map(|(i, rn)| {
                format!(
                    "[{}] {} (relevance: {:.2})",
                    i + 1,
                    rn.node.content(),
                    rn.score
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Extract sources from retrieved nodes.
    fn extract_sources(&self, nodes: &[RetrievedNode]) -> Vec<ContextSource> {
        nodes
            .iter()
            .map(|rn| {
                let preview = if rn.node.content().len() > 100 {
                    format!("{}...", &rn.node.content()[..97])
                } else {
                    rn.node.content().to_string()
                };
                ContextSource::new(rn.id(), preview, rn.score, rn.node.node_type)
            })
            .collect()
    }

    /// Deduplicate nodes by ID.
    fn deduplicate(&self, mut nodes: Vec<RetrievedNode>) -> Vec<RetrievedNode> {
        let mut seen = HashSet::new();
        nodes.retain(|n| seen.insert(n.id()));
        nodes
    }

    /// Calculate approximate context size in tokens.
    fn calculate_context_size(&self, nodes: &[RetrievedNode]) -> usize {
        nodes
            .iter()
            .map(|n| n.node.content().len() / 4) // Rough estimate: 4 chars per token
            .sum()
    }

    /// Filter nodes by minimum relevance.
    fn filter_by_relevance(&self, nodes: Vec<RetrievedNode>, min_rel: f32) -> Vec<RetrievedNode> {
        nodes
            .into_iter()
            .filter(|n| n.score >= min_rel)
            .collect()
    }

    /// Sort nodes by relevance score.
    fn sort_by_relevance(&self, mut nodes: Vec<RetrievedNode>) -> Vec<RetrievedNode> {
        nodes.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        nodes
    }

    /// Truncate to max context size.
    fn truncate_to_size(&self, nodes: Vec<RetrievedNode>, max_size: usize) -> Vec<RetrievedNode> {
        let mut total_size = 0;
        let mut result = Vec::new();

        for node in nodes {
            let node_size = node.node.content().len() / 4;
            if total_size + node_size <= max_size {
                total_size += node_size;
                result.push(node);
            } else {
                break;
            }
        }

        result
    }
}

#[async_trait]
impl<G> GraphRag for MemoryGraphRag<G>
where
    G: Graph + Send + Sync,
{
    async fn retrieve(
        &self,
        query_embedding: Vec<f32>,
        config: RetrievalConfig,
    ) -> GraphRagResult<RetrievalResult> {
        match config.mode {
            RetrievalMode::VectorOnly => {
                self.vector_retrieve(query_embedding, config).await
            }
            RetrievalMode::GraphOnly => {
                // Vector-only for now (would need seed nodes for graph-only)
                self.vector_retrieve(query_embedding, config).await
            }
            RetrievalMode::Hybrid => self.hybrid_retrieve(
                query_embedding,
                config.max_vector_results,
                config.max_hops,
            ).await,
        }
    }

    async fn retrieve_context(
        &self,
        query_embedding: Vec<f32>,
        config: RetrievalConfig,
    ) -> GraphRagResult<RetrievedContext> {
        let result = self.retrieve(query_embedding, config).await?;

        let context = self.format_context(&result.nodes);
        let sources = self.extract_sources(&result.nodes);

        Ok(RetrievedContext::new(result, context, sources))
    }

    async fn retrieve_from_seeds(
        &self,
        seed_ids: Vec<Uuid>,
        config: RetrievalConfig,
    ) -> GraphRagResult<RetrievalResult> {
        let mut all_nodes = Vec::new();
        let mut all_scores = Vec::new();
        let mut visited = HashSet::new();

        for seed_id in seed_ids {
            if visited.contains(&seed_id) {
                continue;
            }

            // BFS traversal from seed
            let trav_config = TraversalConfig::with_depth(config.max_hops)
                .with_direction(TraverseDirection::Both);

            if let Ok(result) = self.graph.bfs(seed_id, trav_config).await {
                for (hop, node) in result.nodes.iter().enumerate() {
                    if visited.insert(node.id) {
                        let score = self.config.scorer.score_graph_only(node.id, hop);
                        all_nodes.push(RetrievedNode::new(
                            node.clone(),
                            score.final_score,
                            score.hop_distance,
                            0.0,
                            false,
                        ));
                        all_scores.push(score);
                    }
                }
            }
        }

        // Post-process
        let mut nodes = self.sort_by_relevance(all_nodes);
        nodes = self.filter_by_relevance(nodes, config.min_relevance);

        if config.deduplicate {
            nodes = self.deduplicate(nodes);
        }

        let context_size = self.calculate_context_size(&nodes);

        Ok(RetrievalResult::new(nodes, all_scores, context_size))
    }

    async fn hybrid_retrieve(
        &self,
        query_embedding: Vec<f32>,
        top_k: usize,
        max_hops: usize,
    ) -> GraphRagResult<RetrievalResult> {
        let mut all_nodes = Vec::new();
        let mut all_scores = Vec::new();
        let mut visited = HashSet::new();

        // Step 1: Vector similarity search (simulated - find all nodes with embeddings)
        for (id, node) in &self.nodes {
            if node.embedding.is_some() {
                // Calculate cosine similarity
                if let Some(embedding) = &node.embedding {
                    let similarity = cosine_similarity(&query_embedding, embedding);

                    if similarity > 0.5 {
                        // Treat as direct match
                        visited.insert(*id);
                        let score = self.config.scorer.score_direct(*id, similarity);

                        all_nodes.push(RetrievedNode::new(
                            node.clone(),
                            score.final_score,
                            0,
                            similarity,
                            true,
                        ));
                        all_scores.push(score);
                    }
                }
            }
        }

        // Sort by similarity and take top_k
        all_nodes.sort_by(|a, b| b.vector_similarity.partial_cmp(&a.vector_similarity).unwrap());
        all_nodes.truncate(top_k);

        // Step 2: Graph expansion from top matches
        let top_matches: Vec<_> = all_nodes.iter().take(top_k.min(5)).map(|n| n.id()).collect();

        for match_id in top_matches {
            let trav_config = TraversalConfig::with_depth(max_hops)
                .with_direction(TraverseDirection::Both);

            if let Ok(result) = self.graph.bfs(match_id, trav_config).await {
                for node in result.nodes {
                    if visited.insert(node.id) {
                        let hop = 1; // Simplified - should track actual hop distance
                        let score = self.config.scorer.score_graph_only(node.id, hop);

                        all_nodes.push(RetrievedNode::new(
                            node,
                            score.final_score,
                            score.hop_distance,
                            0.0,
                            false,
                        ));
                        all_scores.push(score);
                    }
                }
            }
        }

        // Step 3: Re-rank and filter
        let mut nodes = self.sort_by_relevance(all_nodes);

        if self.config.retrieval.deduplicate {
            nodes = self.deduplicate(nodes);
        }

        nodes = self.filter_by_relevance(nodes, self.config.retrieval.min_relevance);

        let context_size = self.calculate_context_size(&nodes);

        Ok(RetrievalResult::new(nodes, all_scores, context_size))
    }
}

impl<G> MemoryGraphRag<G>
where
    G: Graph + Send + Sync,
{
    /// Vector-only retrieval.
    async fn vector_retrieve(
        &self,
        query_embedding: Vec<f32>,
        config: RetrievalConfig,
    ) -> GraphRagResult<RetrievalResult> {
        let mut nodes = Vec::new();
        let mut scores = Vec::new();

        for (id, node) in &self.nodes {
            if let Some(embedding) = &node.embedding {
                let similarity = cosine_similarity(&query_embedding, embedding);

                if similarity >= config.min_relevance {
                    let score = self.config.scorer.score_direct(*id, similarity);

                    nodes.push(RetrievedNode::new(
                        node.clone(),
                        score.final_score,
                        0,
                        similarity,
                        true,
                    ));
                    scores.push(score);
                }
            }
        }

        nodes = self.sort_by_relevance(nodes);
        nodes.truncate(config.max_vector_results);

        let context_size = self.calculate_context_size(&nodes);

        Ok(RetrievalResult::new(nodes, scores, context_size))
    }
}

/// Calculate cosine similarity between two vectors.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for (x, y) in a.iter().zip(b.iter()) {
        dot_product += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a.sqrt() * norm_b.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scorer::Scorer;
    use synton_core::NodeType;
    use synton_graph::MemoryGraph;

    #[tokio::test]
    async fn test_graph_rag_retrieve() {
        let graph = MemoryGraph::new();
        let nodes = vec![
            Node::new("Machine learning is a subset of AI", NodeType::Concept),
            Node::new("Deep learning uses neural networks", NodeType::Concept),
        ];

        let rag = MemoryGraphRag::new(graph, nodes);

        let query = vec![0.1, 0.2, 0.3];
        let result = rag.vector_retrieve(query, RetrievalConfig::default()).await.unwrap();

        // Should return empty result (no embeddings on nodes)
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_graph_rag_retrieve_context() {
        let graph = MemoryGraph::new();
        let mut node1 = Node::new("Test content", NodeType::Concept);
        node1.embedding = Some(vec![0.1, 0.2, 0.3]);

        let nodes = vec![node1.clone()];
        let rag = MemoryGraphRag::new(graph, nodes);

        let query = vec![0.1, 0.2, 0.3];
        let context = rag
            .retrieve_context(query, RetrievalConfig::default())
            .await
            .unwrap();

        assert!(!context.context.is_empty());
        assert_eq!(context.sources.len(), 1);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&a, &c) < 0.001);

        let d = vec![1.0, 1.0, 0.0];
        let e = vec![1.0, 0.0, 1.0];
        let sim = cosine_similarity(&d, &e);
        assert!(sim > 0.0 && sim < 1.0);
    }

    #[test]
    fn test_format_context() {
        let graph = MemoryGraph::new();
        let node = Node::new("Test content", NodeType::Concept);
        let rag = MemoryGraphRag::new(graph, vec![]);

        let retrieved = RetrievedNode::new(node, 0.9, 0, 0.9, true);
        let context = rag.format_context(&[retrieved]);

        assert!(context.contains("Test content"));
        assert!(context.contains("0.90"));
    }

    #[test]
    fn test_deduplicate() {
        let graph = MemoryGraph::new();
        let node = Node::new("Test", NodeType::Concept);
        let rag = MemoryGraphRag::new(graph, vec![]);

        let retrieved = RetrievedNode::new(node.clone(), 0.9, 0, 0.9, true);
        let nodes = vec![retrieved.clone(), retrieved];

        let deduped = rag.deduplicate(nodes);
        assert_eq!(deduped.len(), 1);
    }
}
