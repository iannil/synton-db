// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Neighbor expansion for Graph-RAG context.
//!
//! Expands retrieved context by including related nodes through
//! graph traversal, improving recall and coverage.

use crate::GraphRagResult;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Configuration for context expansion.
#[derive(Debug, Clone, Copy)]
pub struct ExpansionConfig {
    /// Maximum number of nodes to add through expansion.
    pub max_expanded_nodes: usize,

    /// Maximum hop distance for expansion.
    pub max_expansion_hops: usize,

    /// Minimum edge weight to traverse.
    pub min_edge_weight: f32,

    /// Whether to expand bidirectionally.
    pub bidirectional: bool,

    /// Expansion strategy.
    pub strategy: ExpansionStrategy,
}

impl Default for ExpansionConfig {
    fn default() -> Self {
        Self {
            max_expanded_nodes: 10,
            max_expansion_hops: 2,
            min_edge_weight: 0.3,
            bidirectional: true,
            strategy: ExpansionStrategy::Weighted,
        }
    }
}

impl ExpansionConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_nodes(mut self, max: usize) -> Self {
        self.max_expanded_nodes = max;
        self
    }

    pub fn with_max_hops(mut self, hops: usize) -> Self {
        self.max_expansion_hops = hops;
        self
    }

    pub fn with_min_weight(mut self, weight: f32) -> Self {
        self.min_edge_weight = weight;
        self
    }

    pub fn with_strategy(mut self, strategy: ExpansionStrategy) -> Self {
        self.strategy = strategy;
        self
    }
}

/// Strategy for selecting nodes during expansion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpansionStrategy {
    /// Unweighted BFS - explore all neighbors equally.
    Bfs,

    /// Weighted - prioritize by edge weight.
    Weighted,

    /// Relevant types only - expand to specific node types.
    Typed,

    /// Adaptive - adjust based on current context diversity.
    Adaptive,
}

/// Context expansion result.
#[derive(Debug, Clone)]
pub struct ExpansionResult {
    /// IDs of added nodes.
    pub added_ids: Vec<Uuid>,

    /// IDs of removed nodes (due to filtering).
    pub removed_ids: Vec<Uuid>,

    /// Final node IDs after expansion.
    pub final_ids: Vec<Uuid>,

    /// Expansion statistics.
    pub stats: ExpansionStats,
}

/// Statistics about the expansion operation.
#[derive(Debug, Clone)]
pub struct ExpansionStats {
    /// Number of seed nodes.
    pub seed_count: usize,

    /// Number of nodes explored.
    pub explored_count: usize,

    /// Number of hops traversed.
    pub hop_count: usize,

    /// Number of edges traversed.
    pub edge_count: usize,
}

/// Neighbor expander for context enhancement.
pub struct NeighborExpander {
    config: ExpansionConfig,
}

impl NeighborExpander {
    pub fn new(config: ExpansionConfig) -> Self {
        Self { config }
    }

    pub fn with_config(config: ExpansionConfig) -> Self {
        Self::new(config)
    }

    /// Expand context by traversing to neighboring nodes.
    ///
    /// # Arguments
    ///
    /// * `seed_ids` - Initial node IDs to expand from
    /// * `get_neighbors` - Function to get neighbors for a node
    /// * `max_total` - Maximum total nodes after expansion
    ///
    /// # Returns
    ///
    /// The expansion result with added and final node IDs.
    pub fn expand<F>(
        &self,
        seed_ids: &[Uuid],
        get_neighbors: F,
        max_total: usize,
    ) -> GraphRagResult<ExpansionResult>
    where
        F: Fn(Uuid) -> GraphRagResult<Vec<(Uuid, f32)>>,
    {
        let mut visited = HashSet::new();
        let mut to_visit = std::collections::VecDeque::new();
        let mut added_ids = Vec::new();
        let mut all_neighbors = HashMap::new();

        // Initialize with seeds
        for &id in seed_ids {
            visited.insert(id);
            to_visit.push_back((id, 0));
        }

        let mut explored_count = 0;
        let mut edge_count = 0;
        let mut max_hop = 0;

        while let Some((current_id, hop)) = to_visit.pop_front() {
            if hop >= self.config.max_expansion_hops {
                continue;
            }

            // Get neighbors
            let neighbors = get_neighbors(current_id)?;
            explored_count += 1;
            max_hop = max_hop.max(hop);

            for (neighbor_id, weight) in neighbors {
                edge_count += 1;

                // Filter by minimum weight
                if weight < self.config.min_edge_weight {
                    continue;
                }

                if visited.insert(neighbor_id) {
                    all_neighbors.insert(neighbor_id, weight);
                    added_ids.push(neighbor_id);

                    // Continue expansion
                    if added_ids.len() < self.config.max_expanded_nodes {
                        to_visit.push_back((neighbor_id, hop + 1));
                    }
                }

                // Stop if we've reached the limit
                if visited.len() >= max_total {
                    break;
                }
            }
        }

        // Combine seeds and expanded nodes
        let final_ids: Vec<_> = seed_ids
            .iter()
            .chain(added_ids.iter())
            .copied()
            .take(max_total)
            .collect();

        let stats = ExpansionStats {
            seed_count: seed_ids.len(),
            explored_count,
            hop_count: max_hop,
            edge_count,
        };

        Ok(ExpansionResult {
            added_ids,
            removed_ids: Vec::new(),
            final_ids,
            stats,
        })
    }

    /// Expand with type filtering.
    pub fn expand_with_types<F, T>(
        &self,
        seed_ids: &[Uuid],
        get_neighbors: F,
        get_type: T,
        allowed_types: &[&str],
        max_total: usize,
    ) -> GraphRagResult<ExpansionResult>
    where
        F: Fn(Uuid) -> GraphRagResult<Vec<(Uuid, f32)>>,
        T: Fn(Uuid) -> GraphRagResult<String>,
    {
        let result = self.expand(seed_ids, |id| {
            let neighbors = get_neighbors(id)?;
            // Filter by type
            let filtered: Vec<_> = neighbors
                .into_iter()
                .filter(|(nid, _)| {
                    if let Ok(node_type) = get_type(*nid) {
                        allowed_types.iter().any(|&t| t == node_type)
                    } else {
                        false
                    }
                })
                .collect();
            Ok(filtered)
        }, max_total)?;

        Ok(result)
    }

    /// Calculate expansion priority based on node properties.
    pub fn calculate_priority(
        &self,
        _node_id: Uuid,
        seed_relevance: f32,
        edge_weight: f32,
        hop_distance: usize,
    ) -> f32 {
        let hop_penalty = 1.0 / (1.0 + hop_distance as f32);
        seed_relevance * edge_weight * hop_penalty
    }

    /// Adaptive expansion based on context diversity.
    pub fn expand_adaptive<F>(
        &self,
        seed_ids: &[Uuid],
        get_neighbors: F,
        diversity_threshold: f32,
        max_total: usize,
    ) -> GraphRagResult<ExpansionResult>
    where
        F: Fn(Uuid) -> GraphRagResult<Vec<(Uuid, f32)>>,
    {
        // First, get all potential expansions
        let initial = self.expand(seed_ids, &get_neighbors, max_total * 2)?;

        // Calculate diversity (simplified as unique content types)
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        for id in &initial.final_ids {
            if let Ok(neighbors) = get_neighbors(*id) {
                for _nid in neighbors {
                    // In production, would get actual type
                    *type_counts.entry("concept".to_string()).or_insert(0) += 1;
                }
            }
        }

        // Filter based on diversity
        let mut filtered = Vec::new();
        for id in &initial.final_ids {
            if !filtered.contains(id) {
                filtered.push(*id);
            }

            // Check diversity threshold
            if type_counts.len() as f32 >= diversity_threshold * 10.0 {
                break;
            }
        }

        Ok(ExpansionResult {
            added_ids: initial.added_ids,
            removed_ids: initial
                .final_ids
                .iter()
                .filter(|id| !filtered.contains(id))
                .copied()
                .collect(),
            final_ids: filtered.into_iter().take(max_total).collect(),
            stats: initial.stats,
        })
    }
}

impl Default for NeighborExpander {
    fn default() -> Self {
        Self::new(ExpansionConfig::default())
    }
}

/// Relation-based expander - follows specific relation types.
pub struct RelationExpander {
    config: ExpansionConfig,
    target_relations: Vec<String>,
}

impl RelationExpander {
    pub fn new(target_relations: Vec<String>) -> Self {
        Self {
            config: ExpansionConfig::default(),
            target_relations,
        }
    }

    pub fn with_config(mut self, config: ExpansionConfig) -> Self {
        self.config = config;
        self
    }

    /// Expand following only specific relation types.
    pub fn expand_by_relation<F>(
        &self,
        seed_ids: &[Uuid],
        get_relations: F,
        max_total: usize,
    ) -> GraphRagResult<ExpansionResult>
    where
        F: Fn(Uuid) -> GraphRagResult<Vec<(Uuid, String, f32)>>, // (target_id, relation_type, weight)
    {
        let mut visited = HashSet::new();
        let mut added_ids = Vec::new();
        let mut final_ids = seed_ids.to_vec();

        for &id in seed_ids {
            visited.insert(id);
        }

        for &seed_id in seed_ids {
            let relations = get_relations(seed_id)?;

            for (target_id, relation_type, weight) in relations {
                // Check if this relation type is in our target list
                if !self.target_relations.contains(&relation_type) {
                    continue;
                }

                // Filter by weight
                if weight < self.config.min_edge_weight {
                    continue;
                }

                if visited.insert(target_id) && added_ids.len() < self.config.max_expanded_nodes {
                    added_ids.push(target_id);
                    final_ids.push(target_id);

                    if final_ids.len() >= max_total {
                        break;
                    }
                }
            }
        }

        let stats = ExpansionStats {
            seed_count: seed_ids.len(),
            explored_count: seed_ids.len(),
            hop_count: 1,
            edge_count: added_ids.len(),
        };

        Ok(ExpansionResult {
            added_ids,
            removed_ids: Vec::new(),
            final_ids,
            stats,
        })
    }
}

/// Scorer for ranking expanded nodes.
#[derive(Debug, Clone)]
pub struct ExpansionScorer {
    /// Weight for original seed relevance.
    pub seed_weight: f32,

    /// Weight for edge strength.
    pub edge_weight: f32,

    /// Penalty for each hop.
    pub hop_penalty: f32,
}

impl Default for ExpansionScorer {
    fn default() -> Self {
        Self {
            seed_weight: 0.7,
            edge_weight: 0.3,
            hop_penalty: 0.1,
        }
    }
}

impl ExpansionScorer {
    /// Score an expanded node.
    pub fn score_expanded_node(
        &self,
        seed_relevance: f32,
        edge_weight: f32,
        hop_distance: usize,
    ) -> f32 {
        let hop_factor = (1.0 - self.hop_penalty).powi(hop_distance as i32);
        (seed_relevance * self.seed_weight + edge_weight * self.edge_weight) * hop_factor
    }

    /// Re-rank nodes after expansion.
    pub fn rerank(
        &self,
        nodes: Vec<(Uuid, f32, usize)>, // (id, original_score, hop_distance)
        edge_weights: &HashMap<Uuid, f32>,
    ) -> Vec<(Uuid, f32)> {
        let mut scored: Vec<_> = nodes
            .into_iter()
            .map(|(id, orig_score, hops)| {
                let edge_weight = edge_weights.get(&id).copied().unwrap_or(0.5);
                let new_score = self.score_expanded_node(orig_score, edge_weight, hops);
                (id, new_score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expansion_config() {
        let config = ExpansionConfig::new()
            .with_max_nodes(20)
            .with_max_hops(3)
            .with_min_weight(0.5)
            .with_strategy(ExpansionStrategy::Bfs);

        assert_eq!(config.max_expanded_nodes, 20);
        assert_eq!(config.max_expansion_hops, 3);
        assert_eq!(config.min_edge_weight, 0.5);
        assert_eq!(config.strategy, ExpansionStrategy::Bfs);
    }

    #[test]
    fn test_neighbor_expander() {
        let expander = NeighborExpander::new(ExpansionConfig {
            max_expanded_nodes: 5,
            max_expansion_hops: 2,
            min_edge_weight: 0.0,
            ..Default::default()
        });

        let seed1 = Uuid::new_v4();
        let seed2 = Uuid::new_v4();
        let neighbor1 = Uuid::new_v4();
        let neighbor2 = Uuid::new_v4();

        // Mock neighbor function
        let get_neighbors = |id: Uuid| -> GraphRagResult<Vec<(Uuid, f32)>> {
            if id == seed1 {
                Ok(vec![(neighbor1, 0.9), (neighbor2, 0.7)])
            } else if id == seed2 {
                Ok(vec![(neighbor1, 0.5)])
            } else {
                Ok(vec![])
            }
        };

        let result = expander.expand(&[seed1, seed2], get_neighbors, 10).unwrap();

        assert!(!result.added_ids.is_empty());
        assert!(result.final_ids.contains(&seed1));
        assert!(result.final_ids.contains(&seed2));
    }

    #[test]
    fn test_expansion_score() {
        let scorer = ExpansionScorer::default();

        let score1 = scorer.score_expanded_node(0.9, 0.8, 0);
        let score2 = scorer.score_expanded_node(0.9, 0.8, 1);
        let score3 = scorer.score_expanded_node(0.9, 0.8, 2);

        // Closer nodes should score higher
        assert!(score1 > score2);
        assert!(score2 > score3);
    }

    #[test]
    fn test_relation_expander() {
        let expander = RelationExpander::new(vec!["RELATES_TO".to_string(), "SIMILAR_TO".to_string()]);

        let seed = Uuid::new_v4();
        let target1 = Uuid::new_v4();
        let target2 = Uuid::new_v4();

        let get_relations = |_: Uuid| -> GraphRagResult<Vec<(Uuid, String, f32)>> {
            Ok(vec![
                (target1, "RELATES_TO".to_string(), 0.9),
                (target2, "CAUSES".to_string(), 0.8), // Should be filtered
            ])
        };

        let result = expander.expand_by_relation(&[seed], get_relations, 10).unwrap();

        assert!(result.added_ids.contains(&target1));
        assert!(!result.added_ids.contains(&target2));
    }
}
