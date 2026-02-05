// Copyright 2025 SYNTON-DB Team
//
// Licensed under the Apache License, Version 2.0 (the "License");

use serde::{Deserialize, Serialize};
use std::fmt;

/// Source of data in the system.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// Direct user input
    UserInput,

    /// File upload
    FileUpload,

    /// Web crawling result
    WebCrawl,

    /// API import
    ApiImport,

    /// Automatically extracted from other content
    AutoExtracted,

    /// Custom source with identifier
    Custom(String),
}

impl Source {
    /// Create a custom source
    #[inline]
    pub fn custom(s: impl Into<String>) -> Self {
        Self::Custom(s.into())
    }

    /// Check if this is an automated source
    #[inline]
    pub const fn is_automated(&self) -> bool {
        matches!(self, Self::AutoExtracted | Self::WebCrawl | Self::ApiImport)
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UserInput => write!(f, "user_input"),
            Self::FileUpload => write!(f, "file_upload"),
            Self::WebCrawl => write!(f, "web_crawl"),
            Self::ApiImport => write!(f, "api_import"),
            Self::AutoExtracted => write!(f, "auto_extracted"),
            Self::Custom(s) => write!(f, "custom:{}", s),
        }
    }
}

impl Default for Source {
    #[inline]
    fn default() -> Self {
        Self::UserInput
    }
}

impl From<&str> for Source {
    fn from(s: &str) -> Self {
        match s {
            "user_input" => Self::UserInput,
            "file_upload" => Self::FileUpload,
            "web_crawl" => Self::WebCrawl,
            "api_import" => Self::ApiImport,
            "auto_extracted" => Self::AutoExtracted,
            other => Self::Custom(other.to_string()),
        }
    }
}
