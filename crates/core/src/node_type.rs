// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Node type in the Tensor-Graph.
///
/// Each node represents a semantic unit with a specific type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    /// Entity (e.g., "Elon Musk", "Tesla", "Paris")
    Entity,

    /// Concept (e.g., "Artificial Intelligence", "Electric Vehicle")
    Concept,

    /// Fact (e.g., "Tesla CEO is Musk", "Paris is the capital of France")
    Fact,

    /// Raw text chunk (unstructured content segment)
    RawChunk,
}

impl NodeType {
    /// All node type variants
    pub const ALL: [NodeType; 4] = [
        NodeType::Entity,
        NodeType::Concept,
        NodeType::Fact,
        NodeType::RawChunk,
    ];

    /// Check if this node type is structured (has semantic meaning)
    #[inline]
    pub const fn is_structured(&self) -> bool {
        matches!(self, Self::Entity | Self::Concept | Self::Fact)
    }

    /// Check if this node type is unstructured (raw content)
    #[inline]
    pub const fn is_raw(&self) -> bool {
        matches!(self, Self::RawChunk)
    }
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Entity => write!(f, "entity"),
            Self::Concept => write!(f, "concept"),
            Self::Fact => write!(f, "fact"),
            Self::RawChunk => write!(f, "raw_chunk"),
        }
    }
}

impl std::str::FromStr for NodeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "entity" => Ok(Self::Entity),
            "concept" => Ok(Self::Concept),
            "fact" => Ok(Self::Fact),
            "raw_chunk" => Ok(Self::RawChunk),
            _ => Err(format!("Unknown node type: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type_display() {
        assert_eq!(NodeType::Entity.to_string(), "entity");
        assert_eq!(NodeType::Concept.to_string(), "concept");
        assert_eq!(NodeType::Fact.to_string(), "fact");
        assert_eq!(NodeType::RawChunk.to_string(), "raw_chunk");
    }

    #[test]
    fn test_node_type_from_str() {
        assert_eq!("entity".parse::<NodeType>().unwrap(), NodeType::Entity);
        assert_eq!("ENTITY".parse::<NodeType>().unwrap(), NodeType::Entity);
        assert_eq!("concept".parse::<NodeType>().unwrap(), NodeType::Concept);
        assert_eq!("fact".parse::<NodeType>().unwrap(), NodeType::Fact);
        assert_eq!("raw_chunk".parse::<NodeType>().unwrap(), NodeType::RawChunk);
        assert!("unknown".parse::<NodeType>().is_err());
    }

    #[test]
    fn test_node_type_is_structured() {
        assert!(NodeType::Entity.is_structured());
        assert!(NodeType::Concept.is_structured());
        assert!(NodeType::Fact.is_structured());
        assert!(!NodeType::RawChunk.is_structured());
    }

    #[test]
    fn test_node_type_is_raw() {
        assert!(!NodeType::Entity.is_raw());
        assert!(!NodeType::Concept.is_raw());
        assert!(!NodeType::Fact.is_raw());
        assert!(NodeType::RawChunk.is_raw());
    }
}
