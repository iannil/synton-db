// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod index;

#[cfg(feature = "lance")]
mod lance;

pub use error::{VectorError, VectorResult};
pub use index::{MemoryVectorIndex, SearchResult, VectorIndex, memory_index_dump};

#[cfg(feature = "lance")]
pub use lance::{
    HnswParams, IvfParams, IndexType, LanceIndexConfig,
    LanceVectorIndex, MemoryToLanceMigrator, default_progress_reporter,
};

/// Re-exports commonly used types
pub mod prelude {
    pub use crate::{MemoryVectorIndex, VectorError, VectorIndex, VectorResult};
}
