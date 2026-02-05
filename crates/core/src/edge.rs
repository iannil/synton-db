// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{CoreError, CoreResult, Relation};

/// An edge in the Tensor-Graph.
///
/// Edges represent logical relationships between nodes and enable
/// graph-based reasoning and traversal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Edge {
    /// Source node ID
    pub source: Uuid,

    /// Target node ID
    pub target: Uuid,

    /// Relation type
    pub relation: Relation,

    /// Weight/strength (0.0 - 1.0)
    pub weight: f32,

    /// Optional relation vector
    pub vector: Option<Vec<f32>>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Whether this edge is expired (for dynamic fact correction)
    pub expired: bool,

    /// If expired, the replacement edge ID
    pub replaced_by: Option<Uuid>,

    /// Additional flexible attributes
    pub attributes: serde_json::Value,
}

impl Edge {
    /// Create a new edge with default values.
    pub fn new(source: Uuid, target: Uuid, relation: Relation) -> Self {
        Self {
            source,
            target,
            relation,
            weight: 1.0,
            vector: None,
            created_at: Utc::now(),
            expired: false,
            replaced_by: None,
            attributes: serde_json::json!({}),
        }
    }

    /// Create an edge with a specific weight.
    pub fn with_weight(source: Uuid, target: Uuid, relation: Relation, weight: f32) -> Self {
        Self::new(source, target, relation).with_weight_value(weight)
    }

    /// Get a unique identifier for this edge.
    pub fn id(&self) -> String {
        format!("{}::{}::{}", self.source, self.target, self.relation)
    }

    /// Get the relation vector if available.
    #[inline]
    pub fn vector(&self) -> Option<&[f32]> {
        self.vector.as_deref()
    }

    /// Get the vector dimension if available.
    #[inline]
    pub fn vector_dim(&self) -> Option<usize> {
        self.vector.as_ref().map(|v| v.len())
    }

    /// Check if this edge is active (not expired).
    #[inline]
    pub fn is_active(&self) -> bool {
        !self.expired
    }

    /// Check if this edge is expired.
    #[inline]
    pub fn is_expired(&self) -> bool {
        self.expired
    }

    /// Mark this edge as expired.
    pub fn expire(&mut self) {
        self.expired = true;
    }

    /// Set a replacement edge.
    pub fn replace_with(&mut self, replacement_id: Uuid) {
        self.expired = true;
        self.replaced_by = Some(replacement_id);
    }

    /// Validate the edge's data.
    pub fn validate(&self) -> CoreResult<()> {
        if self.source == self.target {
            return Err(CoreError::SelfReferentialEdge);
        }
        if !(0.0..=1.0).contains(&self.weight) {
            return Err(CoreError::InvalidWeight(self.weight));
        }
        Ok(())
    }

    /// Create a reverse edge (swapping source and target).
    pub fn reverse(&self) -> Self {
        Self {
            source: self.target,
            target: self.source,
            relation: self.relation.reverse(),
            weight: self.weight,
            vector: self.vector.clone(),
            created_at: Utc::now(),
            expired: false,
            replaced_by: None,
            attributes: self.attributes.clone(),
        }
    }

    /// Set the weight.
    pub fn with_weight_value(mut self, weight: f32) -> Self {
        self.weight = weight.clamp(0.0, 1.0);
        self
    }

    /// Set the relation vector.
    pub fn with_vector(mut self, vector: Vec<f32>) -> Self {
        self.vector = Some(vector);
        self
    }

    /// Set attributes.
    pub fn with_attributes(mut self, attributes: serde_json::Value) -> Self {
        self.attributes = attributes;
        self
    }

    /// Set the expired flag.
    pub fn with_expired(mut self, expired: bool) -> Self {
        self.expired = expired;
        self
    }
}

/// Builder for constructing edges.
#[derive(Debug)]
pub struct EdgeBuilder {
    edge: Edge,
}

impl EdgeBuilder {
    /// Create a new builder.
    pub fn new(source: Uuid, target: Uuid, relation: Relation) -> Self {
        Self {
            edge: Edge::new(source, target, relation),
        }
    }

    /// Set the weight.
    pub fn weight(mut self, weight: f32) -> Self {
        self.edge.weight = weight.clamp(0.0, 1.0);
        self
    }

    /// Set the relation vector.
    pub fn vector(mut self, vector: Vec<f32>) -> Self {
        self.edge.vector = Some(vector);
        self
    }

    /// Set the created_at timestamp.
    pub fn created_at(mut self, timestamp: DateTime<Utc>) -> Self {
        self.edge.created_at = timestamp;
        self
    }

    /// Add an attribute.
    pub fn attribute(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        if let Some(obj) = self.edge.attributes.as_object_mut() {
            obj.insert(key.into(), value.into());
        }
        self
    }

    /// Build the edge, validating before returning.
    pub fn build(self) -> CoreResult<Edge> {
        self.edge.validate()?;
        Ok(self.edge)
    }

    /// Build without validation.
    pub fn build_unvalidated(self) -> Edge {
        self.edge
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_creation() {
        let source = Uuid::new_v4();
        let target = Uuid::new_v4();
        let edge = Edge::new(source, target, Relation::Causes);

        assert_eq!(edge.source, source);
        assert_eq!(edge.target, target);
        assert_eq!(edge.relation, Relation::Causes);
        assert_eq!(edge.weight, 1.0);
        assert!(edge.is_active());
    }

    #[test]
    fn test_edge_validation() {
        let id = Uuid::new_v4();
        let edge = Edge::new(id, id, Relation::Causes);
        assert!(matches!(edge.validate(), Err(CoreError::SelfReferentialEdge)));

        let mut edge = Edge::new(Uuid::new_v4(), Uuid::new_v4(), Relation::Causes);
        edge.weight = 1.5;
        assert!(matches!(edge.validate(), Err(CoreError::InvalidWeight(_))));
    }

    #[test]
    fn test_edge_id() {
        let source = Uuid::new_v4();
        let target = Uuid::new_v4();
        let edge = Edge::new(source, target, Relation::IsPartOf);

        assert!(edge.id().contains(&source.to_string()));
        assert!(edge.id().contains(&target.to_string()));
        assert!(edge.id().contains("is_part_of"));
    }

    #[test]
    fn test_edge_expire() {
        let mut edge = Edge::new(Uuid::new_v4(), Uuid::new_v4(), Relation::Causes);
        assert!(edge.is_active());

        edge.expire();
        assert!(edge.is_expired());
    }

    #[test]
    fn test_edge_replace_with() {
        let mut edge = Edge::new(Uuid::new_v4(), Uuid::new_v4(), Relation::Causes);
        let replacement_id = Uuid::new_v4();

        edge.replace_with(replacement_id);
        assert!(edge.is_expired());
        assert_eq!(edge.replaced_by, Some(replacement_id));
    }

    #[test]
    fn test_edge_reverse() {
        let source = Uuid::new_v4();
        let target = Uuid::new_v4();
        let edge = Edge::new(source, target, Relation::IsPartOf);

        let reversed = edge.reverse();
        assert_eq!(reversed.source, target);
        assert_eq!(reversed.target, source);
        assert_eq!(reversed.relation, Relation::BelongsTo);
    }

    #[test]
    fn test_edge_with_vector() {
        let edge = Edge::new(Uuid::new_v4(), Uuid::new_v4(), Relation::Causes)
            .with_vector(vec![0.1, 0.2, 0.3]);

        assert!(edge.vector().is_some());
        assert_eq!(edge.vector_dim(), Some(3));
    }

    #[test]
    fn test_edge_with_weight() {
        let edge = Edge::with_weight(Uuid::new_v4(), Uuid::new_v4(), Relation::Causes, 0.7);
        assert_eq!(edge.weight, 0.7);
    }

    #[test]
    fn test_edge_builder() {
        let source = Uuid::new_v4();
        let target = Uuid::new_v4();

        let edge = EdgeBuilder::new(source, target, Relation::SimilarTo)
            .weight(0.85)
            .attribute("label", "test")
            .build()
            .unwrap();

        assert_eq!(edge.weight, 0.85);
        assert_eq!(
            edge.attributes.get("label"),
            Some(&serde_json::json!("test"))
        );
    }
}
