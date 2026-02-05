// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A parsed PaQL query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Query {
    /// The root query node.
    pub root: QueryNode,

    /// Maximum number of results to return.
    pub limit: Option<usize>,

    /// Sort order for results.
    pub sort_fields: Vec<SortField>,
}

impl Query {
    /// Create a new query with a root node.
    pub fn new(root: QueryNode) -> Self {
        Self {
            root,
            limit: None,
            sort_fields: Vec::new(),
        }
    }

    /// Set the limit.
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Add a sort field.
    pub fn with_sort(mut self, field: SortField) -> Self {
        self.sort_fields.push(field);
        self
    }

    /// Check if the query is empty.
    pub fn is_empty(&self) -> bool {
        matches!(self.root, QueryNode::Empty)
    }
}

/// A node in the query AST.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryNode {
    /// Empty query node.
    Empty,

    /// Simple text search.
    TextSearch { query: String },

    /// Semantic search (vector similarity).
    SemanticSearch { embedding: Vec<f32> },

    /// Hybrid search (text + semantic).
    HybridSearch { query: String, embedding: Vec<f32> },

    /// Graph traversal from a seed node.
    GraphTraversal {
        seed_id: Uuid,
        direction: TraverseDirection,
        max_hops: usize,
    },

    /// Filtered search.
    Filter {
        input: Box<QueryNode>,
        filters: Vec<Filter>,
    },

    /// Combined query (AND).
    And { left: Box<QueryNode>, right: Box<QueryNode> },

    /// Combined query (OR).
    Or { left: Box<QueryNode>, right: Box<QueryNode> },

    /// Negated query.
    Not { input: Box<QueryNode> },
}

/// Traversal direction for graph queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraverseDirection {
    /// Forward (outgoing edges).
    Forward,
    /// Backward (incoming edges).
    Backward,
    /// Both directions.
    Both,
}

/// A filter condition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Filter {
    /// Field to filter on.
    pub field: FilterField,

    /// Comparison operation.
    pub op: ComparisonOp,

    /// Value to compare against.
    pub value: FilterValue,
}

impl Filter {
    /// Create a new filter.
    pub fn new(field: FilterField, op: ComparisonOp, value: FilterValue) -> Self {
        Self { field, op, value }
    }
}

/// Filter field.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FilterField {
    /// Node content.
    Content,

    /// Node type.
    NodeType,

    /// Access score.
    AccessScore,

    /// Confidence score.
    Confidence,

    /// Created timestamp.
    CreatedAt,

    /// Custom field.
    Custom(String),
}

/// Comparison operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOp {
    /// Equals.
    Eq,

    /// Not equals.
    Ne,

    /// Greater than.
    Gt,

    /// Greater than or equal.
    Ge,

    /// Less than.
    Lt,

    /// Less than or equal.
    Le,

    /// Contains (for text fields).
    Contains,

    /// In list.
    In,
}

/// Filter value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterValue {
    /// String value.
    String(String),

    /// Integer value.
    Integer(i64),

    /// Float value.
    Float(f64),

    /// Boolean value.
    Boolean(bool),

    /// List of values.
    List(Vec<FilterValue>),
}

/// Sort field specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SortField {
    /// Field to sort by.
    pub field: SortFieldType,

    /// Sort order.
    pub order: SortOrder,
}

impl SortField {
    /// Create a new sort field.
    pub fn new(field: SortFieldType, order: SortOrder) -> Self {
        Self { field, order }
    }
}

/// Sort field options.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SortFieldType {
    /// Relevance score.
    Relevance,

    /// Access score.
    AccessScore,

    /// Confidence score.
    Confidence,

    /// Created timestamp.
    CreatedAt,

    /// Custom field.
    Custom(String),
}

/// Sort order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    /// Ascending order.
    Asc,

    /// Descending order.
    Desc,
}

/// Binary operation for combining queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    /// AND operation.
    And,

    /// OR operation.
    Or,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_creation() {
        let node = QueryNode::TextSearch {
            query: "test".to_string(),
        };
        let query = Query::new(node);

        assert!(!query.is_empty());
        assert!(query.limit.is_none());
        assert!(query.sort_fields.is_empty());
    }

    #[test]
    fn test_query_with_limit() {
        let node = QueryNode::TextSearch {
            query: "test".to_string(),
        };
        let query = Query::new(node).with_limit(10);

        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn test_empty_query() {
        let query = Query::new(QueryNode::Empty);
        assert!(query.is_empty());
    }

    #[test]
    fn test_filter_creation() {
        let filter = Filter::new(
            FilterField::Content,
            ComparisonOp::Contains,
            FilterValue::String("test".to_string()),
        );

        assert_eq!(filter.field, FilterField::Content);
        assert_eq!(filter.op, ComparisonOp::Contains);
    }
}
