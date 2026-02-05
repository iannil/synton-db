// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use serde::{Deserialize, Serialize};
use std::fmt;

/// Filter condition for queries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Filter {
    /// Field equality check
    Equals { field: String, value: FilterValue },

    /// Field contains substring
    Contains { field: String, value: String },

    /// Field greater than numeric value
    GreaterThan { field: String, value: f64 },

    /// Field less than numeric value
    LessThan { field: String, value: f64 },

    /// Field in list of values
    InList { field: String, values: Vec<FilterValue> },

    /// Field range check
    Range {
        field: String,
        min: f64,
        max: f64,
    },

    /// Logical AND of multiple filters
    And(Vec<Filter>),

    /// Logical OR of multiple filters
    Or(Vec<Filter>),

    /// Logical NOT
    Not(Box<Filter>),
}

/// Values that can be used in filter conditions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterValue {
    String(String),
    Number(i64),
    Float(f64),
    Boolean(bool),
}

impl fmt::Display for FilterValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => write!(f, "\"{}\"", s),
            Self::Number(n) => write!(f, "{}", n),
            Self::Float(v) => write!(f, "{}", v),
            Self::Boolean(b) => write!(f, "{}", b),
        }
    }
}

impl From<String> for FilterValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for FilterValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<i64> for FilterValue {
    fn from(n: i64) -> Self {
        Self::Number(n)
    }
}

impl From<f64> for FilterValue {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<bool> for FilterValue {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl Filter {
    /// Create an equals filter
    pub fn equals(field: impl Into<String>, value: impl Into<FilterValue>) -> Self {
        Self::Equals {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create a contains filter
    pub fn contains(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Contains {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create a greater than filter
    pub fn greater_than(field: impl Into<String>, value: f64) -> Self {
        Self::GreaterThan {
            field: field.into(),
            value,
        }
    }

    /// Create a less than filter
    pub fn less_than(field: impl Into<String>, value: f64) -> Self {
        Self::LessThan {
            field: field.into(),
            value,
        }
    }

    /// Create an in-list filter
    pub fn in_list(field: impl Into<String>, values: Vec<FilterValue>) -> Self {
        Self::InList {
            field: field.into(),
            values,
        }
    }

    /// Combine filters with AND
    pub fn and(filters: impl IntoIterator<Item = Filter>) -> Self {
        Self::And(filters.into_iter().collect())
    }

    /// Combine filters with OR
    pub fn or(filters: impl IntoIterator<Item = Filter>) -> Self {
        Self::Or(filters.into_iter().collect())
    }

    /// Negate a filter
    pub fn not(filter: Filter) -> Self {
        Self::Not(Box::new(filter))
    }
}

/// Direction for graph traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraverseDirection {
    /// Traverse outgoing edges (source -> target)
    Outgoing,

    /// Traverse incoming edges (target -> source)
    Incoming,

    /// Traverse both directions
    Both,
}

impl TraverseDirection {
    /// Check if this direction includes outgoing traversal
    #[inline]
    pub const fn includes_outgoing(&self) -> bool {
        matches!(self, Self::Outgoing | Self::Both)
    }

    /// Check if this direction includes incoming traversal
    #[inline]
    pub const fn includes_incoming(&self) -> bool {
        matches!(self, Self::Incoming | Self::Both)
    }
}

impl fmt::Display for TraverseDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Outgoing => write!(f, "outgoing"),
            Self::Incoming => write!(f, "incoming"),
            Self::Both => write!(f, "both"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_constructors() {
        let f = Filter::equals("name", "test");
        assert_eq!(f, Filter::Equals { field: "name".to_string(), value: FilterValue::String("test".to_string()) });

        let f = Filter::contains("content", "keyword");
        assert!(matches!(f, Filter::Contains { .. }));

        let f = Filter::greater_than("score", 0.5);
        assert!(matches!(f, Filter::GreaterThan { .. }));
    }

    #[test]
    fn test_filter_combinations() {
        let f = Filter::and(vec![
            Filter::equals("type", "entity"),
            Filter::greater_than("confidence", 0.8),
        ]);
        assert!(matches!(f, Filter::And(_)));

        let f = Filter::or(vec![
            Filter::equals("status", "active"),
            Filter::equals("status", "pending"),
        ]);
        assert!(matches!(f, Filter::Or(_)));

        let f = Filter::not(Filter::equals("deleted", "true"));
        assert!(matches!(f, Filter::Not(_)));
    }

    #[test]
    fn test_traverse_direction() {
        assert!(TraverseDirection::Outgoing.includes_outgoing());
        assert!(!TraverseDirection::Outgoing.includes_incoming());

        assert!(TraverseDirection::Incoming.includes_incoming());
        assert!(!TraverseDirection::Incoming.includes_outgoing());

        assert!(TraverseDirection::Both.includes_outgoing());
        assert!(TraverseDirection::Both.includes_incoming());
    }

    #[test]
    fn test_filter_value_from() {
        assert!(matches!(FilterValue::from("test"), FilterValue::String(_)));
        assert!(matches!(FilterValue::from(42i64), FilterValue::Number(42)));
        assert!(matches!(FilterValue::from(3.14f64), FilterValue::Float(_)));
        assert!(matches!(FilterValue::from(true), FilterValue::Boolean(true)));
    }
}
