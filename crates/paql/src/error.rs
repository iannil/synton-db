// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use std::fmt;

/// PaQL parsing errors.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Unexpected end of input.
    UnexpectedEndOfInput,

    /// Unexpected token.
    UnexpectedToken { expected: String, found: String },

    /// Invalid query syntax.
    InvalidSyntax(String),

    /// Unknown keyword.
    UnknownKeyword(String),

    /// Invalid filter expression.
    InvalidFilter(String),

    /// Invalid sort expression.
    InvalidSort(String),

    /// Query is empty.
    EmptyQuery,

    /// Query too complex (nesting limit exceeded).
    QueryTooComplex { max_depth: usize, actual: usize },

    /// Invalid limit value.
    InvalidLimit(String),

    /// Custom error.
    Custom(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEndOfInput => write!(f, "Unexpected end of input"),
            Self::UnexpectedToken { expected, found } => {
                write!(f, "Expected '{}', found '{}'", expected, found)
            }
            Self::InvalidSyntax(e) => write!(f, "Invalid syntax: {}", e),
            Self::UnknownKeyword(kw) => write!(f, "Unknown keyword: {}", kw),
            Self::InvalidFilter(e) => write!(f, "Invalid filter: {}", e),
            Self::InvalidSort(e) => write!(f, "Invalid sort: {}", e),
            Self::EmptyQuery => write!(f, "Query is empty"),
            Self::QueryTooComplex { max_depth, actual } => {
                write!(f, "Query too complex: max depth {}, actual {}", max_depth, actual)
            }
            Self::InvalidLimit(e) => write!(f, "Invalid limit: {}", e),
            Self::Custom(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ParseError {}

/// Result type for PaQL parsing.
pub type ParseResult<T> = Result<T, ParseError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        assert_eq!(
            ParseError::EmptyQuery.to_string(),
            "Query is empty"
        );
        assert_eq!(
            ParseError::UnexpectedEndOfInput.to_string(),
            "Unexpected end of input"
        );
        assert_eq!(
            ParseError::InvalidSyntax("test".to_string()).to_string(),
            "Invalid syntax: test"
        );
    }
}
