// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod store;
pub mod rocksdb;

pub use error::{StorageError, StorageResult};
pub use store::{ColumnFamily, Store, WriteOp};

/// Re-exports commonly used types
pub mod prelude {
    pub use crate::{ColumnFamily, Store, StorageError, StorageResult, WriteOp};
}
