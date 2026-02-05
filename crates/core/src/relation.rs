// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Relation type between nodes in the Tensor-Graph.
///
/// Relations define the semantic connections between nodes and enable
/// graph-based reasoning and traversal.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Relation {
    /// Composition: A is part of B
    /// (e.g., "Wheel" is_part_of "Car")
    IsPartOf,

    /// Causality: A causes B
    /// (e.g., "Supply shortage" causes "Production delay")
    Causes,

    /// Contradiction: A contradicts B
    /// (e.g., "Statement A" contradicts "Statement B")
    Contradicts,

    /// Temporal: A happened after B
    /// (e.g., "Event A" happened_after "Event B")
    HappenedAfter,

    /// Similarity: A is similar to B
    /// (e.g., "Concept A" is similar to "Concept B")
    SimilarTo,

    /// Taxonomy: A is a kind of B
    /// (e.g., "Tesla" is_a "Car manufacturer")
    IsA,

    /// Location: A is located at B
    /// (e.g., "Tesla HQ" located_at "Austin, Texas")
    LocatedAt,

    /// Belonging: A belongs to B
    /// (e.g., "Employee" belongs_to "Company")
    BelongsTo,

    /// Custom relation type with identifier
    Custom(String),
}

impl Relation {
    /// All standard relation types
    pub const STANDARD: &'static [Relation] = &[
        Relation::IsPartOf,
        Relation::Causes,
        Relation::Contradicts,
        Relation::HappenedAfter,
        Relation::SimilarTo,
        Relation::IsA,
        Relation::LocatedAt,
        Relation::BelongsTo,
    ];

    /// Get the reverse relation
    #[must_use]
    pub fn reverse(&self) -> Relation {
        match self {
            Self::IsPartOf => Self::BelongsTo,
            Self::BelongsTo => Self::IsPartOf,
            Self::Causes => Self::Custom("caused_by".to_string()),
            Self::HappenedAfter => Self::Custom("happened_before".to_string()),
            Self::LocatedAt => Self::Custom("contains".to_string()),
            Self::IsA => Self::Custom("has_instance".to_string()),
            Self::SimilarTo | Self::Contradicts => Self::RELATED_TO,
            Self::Custom(s) => Self::Custom(format!("reverse_{}", s)),
        }
    }

    /// Check if this is a transitive relation
    #[inline]
    pub fn is_transitive(&self) -> bool {
        matches!(
            self,
            Self::IsPartOf | Self::IsA | Self::Causes | Self::LocatedAt
        )
    }

    /// Check if this is a symmetric relation
    #[inline]
    pub fn is_symmetric(&self) -> bool {
        matches!(self, Self::SimilarTo | Self::Contradicts)
    }

    /// Check if this is a directional relation
    #[inline]
    pub fn is_directional(&self) -> bool {
        !self.is_symmetric()
    }
}

// Auto-generated RELATED_TO variant for reverse calculations
impl Relation {
    const RELATED_TO: Self = Self::SimilarTo;
}

impl fmt::Display for Relation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IsPartOf => write!(f, "is_part_of"),
            Self::Causes => write!(f, "causes"),
            Self::Contradicts => write!(f, "contradicts"),
            Self::HappenedAfter => write!(f, "happened_after"),
            Self::SimilarTo => write!(f, "similar_to"),
            Self::IsA => write!(f, "is_a"),
            Self::LocatedAt => write!(f, "located_at"),
            Self::BelongsTo => write!(f, "belongs_to"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl FromStr for Relation {
    type Err = RelationParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "is_part_of" => Ok(Self::IsPartOf),
            "causes" => Ok(Self::Causes),
            "contradicts" => Ok(Self::Contradicts),
            "happened_after" => Ok(Self::HappenedAfter),
            "similar_to" => Ok(Self::SimilarTo),
            "is_a" => Ok(Self::IsA),
            "located_at" => Ok(Self::LocatedAt),
            "belongs_to" => Ok(Self::BelongsTo),
            other => Ok(Self::Custom(other.to_string())),
        }
    }
}

/// Error when parsing a relation string
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationParseError;

impl fmt::Display for RelationParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse relation")
    }
}

impl std::error::Error for RelationParseError {}

impl From<&str> for Relation {
    fn from(s: &str) -> Self {
        s.parse().unwrap_or_else(|_| Self::Custom(s.to_string()))
    }
}

impl From<String> for Relation {
    fn from(s: String) -> Self {
        s.parse().unwrap_or(Self::Custom(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relation_display() {
        assert_eq!(Relation::IsPartOf.to_string(), "is_part_of");
        assert_eq!(Relation::Causes.to_string(), "causes");
        assert_eq!(Relation::Custom("custom_rel".to_string()).to_string(), "custom_rel");
    }

    #[test]
    fn test_relation_from_str() {
        assert_eq!("is_part_of".parse::<Relation>().unwrap(), Relation::IsPartOf);
        assert_eq!("CAUSES".parse::<Relation>().unwrap(), Relation::Causes);
        assert_eq!(
            "custom_rel".parse::<Relation>().unwrap(),
            Relation::Custom("custom_rel".to_string())
        );
    }

    #[test]
    fn test_relation_reverse() {
        assert_eq!(Relation::IsPartOf.reverse(), Relation::BelongsTo);
        assert_eq!(Relation::BelongsTo.reverse(), Relation::IsPartOf);
        assert_eq!(Relation::Causes.reverse(), Relation::Custom("caused_by".to_string()));
    }

    #[test]
    fn test_relation_properties() {
        assert!(Relation::IsPartOf.is_transitive());
        assert!(Relation::IsA.is_transitive());
        assert!(Relation::SimilarTo.is_symmetric());
        assert!(!Relation::Causes.is_symmetric());
    }

    #[test]
    fn test_relation_is_directional() {
        assert!(Relation::Causes.is_directional());
        assert!(Relation::IsPartOf.is_directional());
        assert!(!Relation::SimilarTo.is_directional());
        assert!(!Relation::Contradicts.is_directional());
    }
}
