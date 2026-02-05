// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! Memory management for SYNTON-DB.
//!
//! This module implements memory decay and strengthening based on
//! the Ebbinghaus forgetting curve.

#![warn(missing_docs)]
#![warn(clippy::all)]

mod config;
mod decay;
mod error;
mod manager;

pub use config::DecayConfig;
pub use decay::{DecayCalculator, DecayCurve, ForgettingCurve};
pub use error::{MemoryError, MemoryResult};
pub use manager::{MemoryManager, MemoryStats, PruneResult};

/// Re-exports commonly used types
pub mod prelude {
    pub use crate::{DecayCalculator, DecayConfig, DecayCurve, ForgettingCurve, MemoryError, MemoryManager, MemoryResult};
}
