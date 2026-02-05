// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! SYNTON-DB Core Data Types
//!
//! This crate provides the fundamental data structures for SYNTON-DB,
//! including nodes, edges, and related types that form the Tensor-Graph
//! data model.

#![warn(missing_docs)]
#![warn(clippy::all)]

mod node;
mod edge;
mod relation;
mod node_type;
mod error;
mod source;
mod filter;
mod path;

pub use node::{Node, NodeMeta, NodeBuilder};
pub use edge::{Edge, EdgeBuilder};
pub use relation::{Relation, RelationParseError};
pub use node_type::NodeType;
pub use error::{CoreError, CoreResult};
pub use source::Source;
pub use filter::{Filter, FilterValue, TraverseDirection};
pub use path::{ReasoningPath, PathType};

/// Re-exports commonly used types
pub mod prelude {
    pub use crate::{Edge, Filter, Node, NodeType, ReasoningPath, Relation};
}
