// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

//! PaQL: Prompt as Query Language
//!
//! Natural language query parser for SYNTON-DB cognitive database.

#![warn(missing_docs)]
#![warn(clippy::all)]

mod ast;
mod error;
mod parser;

pub use ast::{BinaryOp, ComparisonOp, Query, QueryNode, SortField, SortFieldType, SortOrder};
pub use error::{ParseError, ParseResult};
pub use parser::Parser;

/// Re-exports commonly used types
pub mod prelude {
    pub use crate::{ParseError, ParseResult, Parser, Query};
}
