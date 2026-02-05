// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use uuid::Uuid;

/// Relevance score for a node in the context.
#[derive(Debug, Clone, PartialEq)]
pub struct RelevanceScore {
    /// Node ID
    pub node_id: Uuid,

    /// Vector similarity score (0.0 - 1.0)
    pub vector_similarity: f32,

    /// Graph proximity score (0.0 - 1.0)
    pub graph_proximity: f32,

    /// Combined final score (0.0 - 1.0)
    pub final_score: f32,

    /// Distance in hops from query node
    pub hop_distance: usize,
}

impl RelevanceScore {
    /// Create a new relevance score.
    pub fn new(
        node_id: Uuid,
        vector_similarity: f32,
        graph_proximity: f32,
        hop_distance: usize,
    ) -> Self {
        let final_score = Self::combine_scores(vector_similarity, graph_proximity, hop_distance);
        Self {
            node_id,
            vector_similarity,
            graph_proximity,
            final_score,
            hop_distance,
        }
    }

    /// Combine individual scores into a final score.
    ///
    /// Uses a weighted formula that favors:
    /// - High vector similarity (semantic relevance)
    /// - High graph proximity (structural relevance)
    /// - Low hop distance (closeness to query)
    fn combine_scores(vector_sim: f32, graph_prox: f32, hops: usize) -> f32 {
        // Decay factor based on hops: 1.0 at 0 hops, 0.5 at 1 hop, etc.
        let hop_decay = 2.0_f32.powi(-(hops as i32));

        // Weighted combination: 60% vector, 40% graph proximity
        let semantic_score = 0.6 * vector_sim + 0.4 * graph_prox;

        // Apply hop decay
        semantic_score * hop_decay
    }

    /// Create a direct match score (0 hops, maximum graph proximity).
    pub fn direct_match(node_id: Uuid, vector_similarity: f32) -> Self {
        Self::new(node_id, vector_similarity, 1.0, 0)
    }
}

/// Scorer for calculating relevance in Graph-RAG retrieval.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Scorer {
    /// Weight for vector similarity (default: 0.6)
    pub vector_weight: f32,

    /// Weight for graph proximity (default: 0.4)
    pub graph_weight: f32,

    /// Decay rate per hop (default: 0.5, i.e., half relevance per hop)
    pub hop_decay_rate: f32,
}

impl Scorer {
    /// Create a new scorer with default weights.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a scorer with custom weights.
    pub fn with_weights(vector_weight: f32, graph_weight: f32) -> Self {
        Self {
            vector_weight,
            graph_weight,
            ..Default::default()
        }
    }

    /// Set the hop decay rate.
    pub fn with_hop_decay(mut self, decay: f32) -> Self {
        self.hop_decay_rate = decay.clamp(0.0, 1.0);
        self
    }

    /// Score a direct vector match.
    pub fn score_direct(&self, node_id: Uuid, vector_similarity: f32) -> RelevanceScore {
        RelevanceScore::direct_match(node_id, vector_similarity)
    }

    /// Score a node based on graph traversal.
    pub fn score_traversal(
        &self,
        node_id: Uuid,
        vector_similarity: f32,
        hop_distance: usize,
    ) -> RelevanceScore {
        // Graph proximity decays with hops
        let graph_proximity = self.hop_decay_rate.powi(hop_distance as i32);

        let final_score = self.vector_weight * vector_similarity
            + self.graph_weight * graph_proximity;

        RelevanceScore {
            node_id,
            vector_similarity,
            graph_proximity,
            final_score: final_score.clamp(0.0, 1.0),
            hop_distance,
        }
    }

    /// Score a node without vector similarity (graph-only).
    pub fn score_graph_only(&self, node_id: Uuid, hop_distance: usize) -> RelevanceScore {
        let graph_proximity = self.hop_decay_rate.powi(hop_distance as i32);
        RelevanceScore {
            node_id,
            vector_similarity: 0.0,
            graph_proximity,
            final_score: (self.graph_weight * graph_proximity).clamp(0.0, 1.0),
            hop_distance,
        }
    }

    /// Re-rank nodes based on their scores.
    pub fn rerank(&self, mut scores: Vec<RelevanceScore>) -> Vec<RelevanceScore> {
        scores.sort_by(|a, b| {
            b.final_score
                .partial_cmp(&a.final_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scores
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relevance_score_direct_match() {
        let id = Uuid::new_v4();
        let score = RelevanceScore::direct_match(id, 0.95);

        assert_eq!(score.node_id, id);
        assert_eq!(score.vector_similarity, 0.95);
        assert_eq!(score.graph_proximity, 1.0);
        assert_eq!(score.hop_distance, 0);
        assert!(score.final_score > 0.9); // High score for direct match
    }

    #[test]
    fn test_relevance_score_hop_decay() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        let score0 = RelevanceScore::new(id1, 0.8, 1.0, 0);
        let score1 = RelevanceScore::new(id2, 0.8, 1.0, 1);
        let score2 = RelevanceScore::new(id3, 0.8, 1.0, 2);

        // Scores should decrease with hop distance
        assert!(score0.final_score > score1.final_score);
        assert!(score1.final_score > score2.final_score);
    }

    #[test]
    fn test_scorer_with_custom_weights() {
        let scorer = Scorer::with_weights(0.8, 0.2);
        let id = Uuid::new_v4();

        let score = scorer.score_traversal(id, 0.9, 1);

        assert_eq!(score.node_id, id);
        assert_eq!(score.hop_distance, 1);
        assert!(score.final_score > 0.0);
        assert!(score.final_score <= 1.0);
    }

    #[test]
    fn test_scorer_rerank() {
        let scorer = Scorer::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        let mut scores = vec![
            RelevanceScore::new(id1, 0.5, 1.0, 0),
            RelevanceScore::new(id2, 0.9, 1.0, 0),
            RelevanceScore::new(id3, 0.7, 1.0, 0),
        ];

        scores = scorer.rerank(scores);

        // Should be sorted by final_score descending
        assert_eq!(scores[0].node_id, id2); // 0.9
        assert_eq!(scores[1].node_id, id3); // 0.7
        assert_eq!(scores[2].node_id, id1); // 0.5
    }
}
