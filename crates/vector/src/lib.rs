// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod index;

pub use error::{VectorError, VectorResult};
pub use index::VectorIndex;

/// Re-exports commonly used types
pub mod prelude {
    pub use crate::{VectorError, VectorIndex, VectorResult};
}
