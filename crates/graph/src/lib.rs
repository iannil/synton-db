// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod graph;
mod path;
mod traversal;

pub use error::{GraphError, GraphResult};
pub use graph::{Graph, MemoryGraph, TraverseDirection, TraversalConfig, TraversalResult};
pub use path::GraphPaths;

/// Re-exports commonly used types
pub mod prelude {
    pub use crate::{Graph, GraphError, GraphPaths, GraphResult, MemoryGraph, TraverseDirection, TraversalConfig, TraversalResult};
}
