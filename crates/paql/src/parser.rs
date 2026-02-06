// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use crate::{
    ast::{
        ComparisonOp, Filter, FilterField, FilterValue, Query, QueryNode, SortField, SortFieldType,
        SortOrder, TraverseDirection,
    },
    error::ParseResult,
};

/// PaQL query parser.
pub struct Parser {
    /// Maximum nesting depth for queries.
    max_depth: usize,
}

impl Parser {
    /// Create a new parser with default settings.
    pub fn new() -> Self {
        Self { max_depth: 10 }
    }

    /// Create a new parser with custom max depth.
    pub fn with_max_depth(max_depth: usize) -> Self {
        Self { max_depth }
    }

    /// Parse a query string into a Query AST.
    pub fn parse(&self, input: &str) -> ParseResult<Query> {
        let input = input.trim();

        if input.is_empty() {
            return Ok(Query::new(QueryNode::Empty));
        }

        // Parse limit if present
        let limit = self.extract_limit(input);

        // Parse sort if present
        let sort_fields = self.extract_sort(input)?;

        // Remove limit and sort clauses for node parsing
        let query_input = self.strip_modifiers(input);

        // Parse the root query node
        let root = self.parse_query_node(&query_input)?;

        Ok(Query {
            root,
            limit,
            sort_fields,
        })
    }

    fn parse_query_node(&self, input: &str) -> ParseResult<QueryNode> {
        let input = input.trim();

        // Try different patterns in order
        // Check for graph traversal
        if let Some(node) = self.try_parse_graph_traversal(input) {
            return Ok(node);
        }

        // Check for filter queries
        if let Some(node) = self.try_parse_filter_query(input) {
            return Ok(node);
        }

        // Check for combined queries (AND/OR)
        if let Some(node) = self.try_parse_combined_query(input) {
            return Ok(node);
        }

        // Check for negation
        if let Some(node) = self.try_parse_not_query(input) {
            return Ok(node);
        }

        // Default: text search
        Ok(QueryNode::TextSearch {
            query: self.extract_search_term(input),
        })
    }

    fn try_parse_graph_traversal(&self, input: &str) -> Option<QueryNode> {
        let lower = input.to_lowercase();

        // Look for patterns like "from UUID", "traverse from UUID"
        let uuid_str = if lower.starts_with("from ") {
            input[4..].split_whitespace().next()?
        } else if lower.starts_with("traverse from ") {
            input[14..].split_whitespace().next()?
        } else if lower.starts_with("starting from ") {
            input[14..].split_whitespace().next()?
        } else {
            return None;
        };

        let seed_id = uuid::Uuid::parse_str(uuid_str).ok()?;

        // Check for direction
        let direction = if lower.contains("forward") {
            TraverseDirection::Forward
        } else if lower.contains("backward") {
            TraverseDirection::Backward
        } else {
            TraverseDirection::Both
        };

        // Check for max hops
        let max_hops = if let Some(pos) = lower.find("within") {
            let after = &lower[pos + 6..];
            after
                .split_whitespace()
                .next()?
                .parse()
                .unwrap_or(2)
        } else {
            2
        };

        Some(QueryNode::GraphTraversal {
            seed_id,
            direction,
            max_hops,
        })
    }

    fn try_parse_filter_query(&self, input: &str) -> Option<QueryNode> {
        let lower = input.to_lowercase();

        // Look for "where" or "with"
        let (base_query, filter_part) = if let Some(pos) = lower.find(" where ") {
            (&input[..pos], &input[pos + 7..])
        } else if let Some(pos) = lower.find(" with ") {
            (&input[..pos], &input[pos + 6..])
        } else {
            return None;
        };

        // Parse filters
        let filters = self.parse_filter_conditions(filter_part).ok()?;

        Some(QueryNode::Filter {
            input: Box::new(QueryNode::TextSearch {
                query: base_query.trim().to_string(),
            }),
            filters,
        })
    }

    fn try_parse_combined_query(&self, input: &str) -> Option<QueryNode> {
        let lower = input.to_lowercase();

        // Look for AND
        if let Some(pos) = lower.find(" and ") {
            let left = &input[..pos];
            let right = &input[pos + 5..];

            return Some(QueryNode::And {
                left: Box::new(QueryNode::TextSearch {
                    query: left.trim().to_string(),
                }),
                right: Box::new(QueryNode::TextSearch {
                    query: right.trim().to_string(),
                }),
            });
        }

        // Look for OR
        if let Some(pos) = lower.find(" or ") {
            let left = &input[..pos];
            let right = &input[pos + 4..];

            return Some(QueryNode::Or {
                left: Box::new(QueryNode::TextSearch {
                    query: left.trim().to_string(),
                }),
                right: Box::new(QueryNode::TextSearch {
                    query: right.trim().to_string(),
                }),
            });
        }

        None
    }

    fn try_parse_not_query(&self, input: &str) -> Option<QueryNode> {
        let lower = input.to_lowercase();

        if lower.starts_with("not ") {
            let query = &input[4..];
            return Some(QueryNode::Not {
                input: Box::new(QueryNode::TextSearch {
                    query: query.trim().to_string(),
                }),
            });
        }

        None
    }

    fn parse_filter_conditions(&self, input: &str) -> ParseResult<Vec<Filter>> {
        let mut filters = Vec::new();
        let lower = input.to_lowercase();

        // Simple filter parsing: "field op value"
        // For MVP, just handle content contains
        if lower.contains("contains") {
            filters.push(Filter::new(
                FilterField::Content,
                ComparisonOp::Contains,
                FilterValue::String(self.extract_search_term(input).to_string()),
            ));
        }

        Ok(filters)
    }

    fn extract_search_term(&self, input: &str) -> String {
        let input = input.trim();

        // Remove common prefixes
        let term = input
            .strip_prefix("find ")
            .or_else(|| input.strip_prefix("search for "))
            .or_else(|| input.strip_prefix("search "))
            .or_else(|| input.strip_prefix("look for "))
            .or_else(|| input.strip_prefix("show "))
            .or_else(|| input.strip_prefix("show me "))
            .unwrap_or(input);

        // Remove quotes if present
        if term.starts_with('"') && term.ends_with('"') {
            term[1..term.len() - 1].to_string()
        } else {
            term.to_string()
        }
    }

    fn strip_modifiers(&self, input: &str) -> String {
        let mut result = input.to_string();
        let lower = result.to_lowercase();

        // Remove sort clauses
        for kw in [" sort by ", " order by "] {
            if let Some(pos) = lower.find(kw) {
                // Find the end of the clause (next comma or end)
                let after = &result[pos..];
                let end_pos = after
                    .find(',')
                    .unwrap_or(after.len());

                result = format!("{}{}", &result[..pos], &result[pos + end_pos..]);
                break;
            }
        }

        // Remove limit clauses
        for kw in [" limit ", " top ", " first "] {
            if let Some(pos) = lower.find(kw) {
                let end = result[pos + kw.len()..]
                    .find(|c: char| !c.is_numeric() && c != ' ')
                    .unwrap_or(result.len() - pos - kw.len());

                result = format!("{}{}", &result[..pos], &result[pos + kw.len() + end..]);
                break;
            }
        }

        result.trim().to_string()
    }

    fn extract_limit(&self, input: &str) -> Option<usize> {
        let lower = input.to_lowercase();

        // Try "limit N"
        if let Some(pos) = lower.find("limit") {
            let after = &lower[pos + 5..];
            let num = after.split_whitespace().next()?;
            return num.parse().ok();
        }

        // Try "top N"
        if let Some(pos) = lower.find("top") {
            let after = &lower[pos + 3..];
            let num = after.split_whitespace().next()?;
            return num.parse().ok();
        }

        // Try "first N"
        if let Some(pos) = lower.find("first") {
            let after = &lower[pos + 5..];
            let num = after.split_whitespace().next()?;
            return num.parse().ok();
        }

        None
    }

    fn extract_sort(&self, input: &str) -> ParseResult<Vec<SortField>> {
        let mut sort_fields = Vec::new();
        let lower = input.to_lowercase();

        // Look for "sort by X", "order by X"
        let keywords = ["sort by", "order by"];
        for kw in keywords {
            if let Some(pos) = lower.find(kw) {
                let after = &input[pos + kw.len()..];
                let parts: Vec<&str> = after
                    .trim()
                    .split(',')
                    .map(|s| s.trim())
                    .collect();

                for part in parts {
                    let tokens: Vec<&str> = part.split_whitespace().collect();
                    if tokens.is_empty() {
                        continue;
                    }

                    let field = match tokens[0].to_lowercase().as_str() {
                        "relevance" | "score" => SortFieldType::Relevance,
                        "access" | "access score" => SortFieldType::AccessScore,
                        "confidence" => SortFieldType::Confidence,
                        "created" | "date" => SortFieldType::CreatedAt,
                        other => SortFieldType::Custom(other.to_string()),
                    };

                    let order = if tokens.len() > 1 {
                        match tokens[1].to_lowercase().as_str() {
                            "asc" | "ascending" => SortOrder::Asc,
                            "desc" | "descending" => SortOrder::Desc,
                            _ => SortOrder::Desc,
                        }
                    } else {
                        SortOrder::Desc
                    };

                    sort_fields.push(SortField::new(field, order));
                }

                break;
            }
        }

        Ok(sort_fields)
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_text_search() {
        let parser = Parser::new();
        let result = parser.parse("machine learning");

        assert!(result.is_ok());
        let query = result.unwrap();
        assert!(!query.is_empty());
    }

    #[test]
    fn test_parse_empty_query() {
        let parser = Parser::new();
        let result = parser.parse("");

        assert!(result.is_ok());
        let query = result.unwrap();
        assert!(query.is_empty());
    }

    #[test]
    fn test_parse_with_limit() {
        let parser = Parser::new();
        let result = parser.parse("find AI concepts limit 10");

        assert!(result.is_ok());
        let query = result.unwrap();
        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn test_parse_with_sort() {
        let parser = Parser::new();
        let result = parser.parse("find AI concepts sort by relevance");

        assert!(result.is_ok());
        let query = result.unwrap();
        assert_eq!(query.sort_fields.len(), 1);
    }
}
