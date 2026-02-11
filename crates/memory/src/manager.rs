// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

use crate::{
    config::DecayConfig,
    decay::{DecayCalculator, DecayCurve},
    error::{MemoryError, MemoryResult},
};
use synton_core::Node;

/// Statistics about memory state.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStats {
    /// Total number of nodes being tracked.
    pub total_nodes: usize,

    /// Nodes above minimum threshold.
    pub active_nodes: usize,

    /// Nodes below minimum threshold (candidates for pruning).
    pub decayed_nodes: usize,

    /// Average access score.
    pub average_score: f32,

    /// Memory load ratio (active / total).
    pub load_factor: f32,
}

impl MemoryStats {
    /// Create new memory stats.
    pub fn new(
        total_nodes: usize,
        active_nodes: usize,
        decayed_nodes: usize,
        average_score: f32,
    ) -> Self {
        let load_factor = if total_nodes > 0 {
            active_nodes as f32 / total_nodes as f32
        } else {
            0.0
        };

        Self {
            total_nodes,
            active_nodes,
            decayed_nodes,
            average_score,
            load_factor,
        }
    }

    /// Create empty stats.
    pub fn empty() -> Self {
        Self {
            total_nodes: 0,
            active_nodes: 0,
            decayed_nodes: 0,
            average_score: 0.0,
            load_factor: 0.0,
        }
    }
}

/// Result of a pruning operation.
#[derive(Debug, Clone, PartialEq)]
pub struct PruneResult {
    /// IDs of nodes that were pruned.
    pub pruned_ids: Vec<Uuid>,

    /// Number of nodes pruned.
    pub count: usize,

    /// Total score reclaimed.
    pub score_reclaimed: f32,

    /// Duration of the pruning operation.
    pub duration_ms: u64,
}

impl PruneResult {
    /// Create a new prune result.
    pub fn new(pruned_ids: Vec<Uuid>, score_reclaimed: f32, duration_ms: u64) -> Self {
        let count = pruned_ids.len();
        Self {
            pruned_ids,
            count,
            score_reclaimed,
            duration_ms,
        }
    }

    /// Create an empty result.
    pub fn empty() -> Self {
        Self {
            pruned_ids: Vec::new(),
            count: 0,
            score_reclaimed: 0.0,
            duration_ms: 0,
        }
    }

    /// Check if anything was pruned.
    pub fn is_empty(&self) -> bool {
        self.pruned_ids.is_empty()
    }
}

/// Memory manager for tracking and managing node access scores.
#[derive(Debug, Clone)]
pub struct MemoryManager {
    calculator: DecayCalculator,
    nodes: HashMap<Uuid, Node>,
}

impl MemoryManager {
    /// Create a new memory manager with default settings.
    pub fn new() -> Self {
        Self {
            calculator: DecayCalculator::new(),
            nodes: HashMap::new(),
        }
    }

    /// Create a new memory manager with custom config.
    pub fn with_config(config: DecayConfig) -> Self {
        Self {
            calculator: DecayCalculator::with_config(config),
            nodes: HashMap::new(),
        }
    }

    /// Create a new memory manager with custom curve.
    pub fn with_curve(curve: DecayCurve) -> Self {
        Self {
            calculator: DecayCalculator::new().with_curve(curve),
            nodes: HashMap::new(),
        }
    }

    /// Register a node for memory tracking.
    pub fn register(&mut self, node: Node) -> MemoryResult<()> {
        self.nodes.insert(node.id, node);
        Ok(())
    }

    /// Unregister a node from tracking.
    pub fn unregister(&mut self, id: Uuid) -> Option<Node> {
        self.nodes.remove(&id)
    }

    /// Record an access to a node (strengthens memory).
    pub fn record_access(&mut self, id: Uuid) -> MemoryResult<()> {
        if let Some(node) = self.nodes.get_mut(&id) {
            let current = self.calculator.current_score(node);
            let boosted = self.calculator.boost(current, 1);

            // Update the node's metadata
            node.meta.access_score = boosted;
            node.meta.accessed_at = Some(chrono::Utc::now());
        }

        Ok(())
    }

    /// Record multiple accesses (e.g., from a batch operation).
    pub fn record_access_batch(&mut self, ids: &[Uuid]) -> MemoryResult<()> {
        for id in ids {
            self.record_access(*id)?;
        }
        Ok(())
    }

    /// Get the current decayed score for a node.
    pub fn get_score(&self, id: Uuid) -> MemoryResult<f32> {
        self.nodes
            .get(&id)
            .map(|node| self.calculator.current_score(node))
            .ok_or(MemoryError::NodeNotFound(id))
    }

    /// Get all node scores.
    pub fn get_all_scores(&self) -> HashMap<Uuid, f32> {
        self.nodes
            .iter()
            .map(|(id, node)| (*id, self.calculator.current_score(node)))
            .collect()
    }

    /// Get the retention rate for a node (0.0 - 1.0).
    pub fn get_retention(&self, id: Uuid) -> MemoryResult<f64> {
        self.nodes
            .get(&id)
            .map(|node| self.calculator.retention(node))
            .ok_or(MemoryError::NodeNotFound(id))
    }

    /// Prune nodes that have decayed below the threshold.
    pub fn prune(&mut self) -> MemoryResult<PruneResult> {
        let start = std::time::Instant::now();
        let min_score = self.calculator.config().min_score;

        let mut pruned_ids = Vec::new();
        let mut score_reclaimed = 0.0;

        self.nodes.retain(|id, node| {
            let score = self.calculator.current_score(node);

            if score < min_score {
                pruned_ids.push(*id);
                score_reclaimed += score;
                false // Remove from map
            } else {
                true // Keep in map
            }
        });

        let duration = start.elapsed().as_millis() as u64;

        Ok(PruneResult::new(pruned_ids, score_reclaimed, duration))
    }

    /// Get memory statistics.
    pub fn stats(&self) -> MemoryStats {
        let total_nodes = self.nodes.len();

        let mut active_nodes = 0;
        let mut decayed_nodes = 0;
        let mut total_score = 0.0;

        for node in self.nodes.values() {
            let score = self.calculator.current_score(node);
            total_score += score;

            if score >= self.calculator.config().min_score {
                active_nodes += 1;
            } else {
                decayed_nodes += 1;
            }
        }

        let average_score = if total_nodes > 0 {
            total_score / total_nodes as f32
        } else {
            0.0
        };

        MemoryStats::new(total_nodes, active_nodes, decayed_nodes, average_score)
    }

    /// Get all nodes (for iteration/export).
    pub fn nodes(&self) -> &HashMap<Uuid, Node> {
        &self.nodes
    }

    /// Get a specific node.
    pub fn get_node(&self, id: Uuid) -> Option<&Node> {
        self.nodes.get(&id)
    }

    /// Get the calculator for custom decay calculations.
    pub fn calculator(&self) -> &DecayCalculator {
        &self.calculator
    }

    /// Update the calculator config.
    pub fn update_config(&mut self, config: DecayConfig) -> MemoryResult<()> {
        config.validate()?;
        self.calculator = DecayCalculator::with_config(config);
        Ok(())
    }

    /// Run periodic decay update on all nodes.
    pub async fn run_decay_update(&mut self) -> MemoryResult<usize> {
        // In a real implementation, this would:
        // 1. Calculate decayed scores for all nodes
        // 2. Update their access_score in storage
        // 3. Return count of updated nodes

        let count = self.nodes.len();

        // For MVP, we just calculate scores on-demand
        // Storage updates would happen in the background
        Ok(count)
    }

    /// Start background decay task.
    pub fn spawn_decay_task(
        &mut self,
        interval: Duration,
    ) -> tokio::task::JoinHandle<()> {
        let nodes_map = std::mem::take(&mut self.nodes);

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                // Update decay for all nodes
                // In production, this would persist to storage
                tracing::debug!("Running decay update for {} nodes", nodes_map.len());
            }
        })
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use synton_core::{NodeType, Source};

    #[tokio::test]
    async fn test_memory_manager() {
        let mut manager = MemoryManager::new();

        let node = Node::new("Test memory", NodeType::Concept);
        let id = node.id;

        manager.register(node).unwrap();

        // Check initial score
        let score = manager.get_score(id).unwrap();
        assert_eq!(score, 1.0);

        // Record access
        manager.record_access(id).unwrap();

        // Score should be boosted
        let boosted_score = manager.get_score(id).unwrap();
        assert!(boosted_score > 1.0);
    }

    #[tokio::test]
    async fn test_prune() {
        let config = DecayConfig::new().with_min_score(5.0).with_max_score(10.0);
        let mut manager = MemoryManager::with_config(config);

        let node = Node::new("Decay test", NodeType::Concept);
        let id = node.id;

        manager.register(node).unwrap();

        // Score should be low (initial is 1.0, min is 5.0)
        let stats = manager.stats();
        assert_eq!(stats.decayed_nodes, 1);

        let result = manager.prune().unwrap();
        assert_eq!(result.count, 1);
        assert!(result.pruned_ids.contains(&id));
    }

    #[tokio::test]
    async fn test_stats() {
        let mut manager = MemoryManager::new();

        manager
            .register(Node::new("Node 1", NodeType::Concept))
            .unwrap();
        manager
            .register(Node::new("Node 2", NodeType::Concept))
            .unwrap();

        let stats = manager.stats();
        assert_eq!(stats.total_nodes, 2);
        assert_eq!(stats.active_nodes, 2); // Both above min_score (0.1)
    }

    #[tokio::test]
    async fn test_retention() {
        let mut manager = MemoryManager::new();

        let mut node = Node::new("Test", NodeType::Concept);
        let id = node.id;
        // Simulate access 1000 hours ago
        node.meta.accessed_at = Some(chrono::Utc::now() - chrono::Duration::hours(1000));
        node.meta.access_score = 1.0;

        manager.register(node).unwrap();

        // After 1000 hours, retention should be ~22%
        let retention = manager.get_retention(id).unwrap();
        assert!((retention - 0.22).abs() < 0.01);
    }
}
