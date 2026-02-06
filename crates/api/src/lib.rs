// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! API service layer for SYNTON-DB.
//!
//! Provides gRPC and REST endpoints for database operations.

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod grpc;
mod models;
/// OpenAPI documentation.
pub mod openapi;
/// REST API handlers and router.
pub mod rest;
mod service;

pub use error::{ApiError, ApiResult};
pub use grpc::create_grpc_router;
pub use models::*;
pub use rest::{AppState, create_router, run_server};
pub use service::SyntonDbService;

/// Re-exports commonly used types
pub mod prelude {
    pub use crate::{ApiError, ApiResult, SyntonDbService};
}
