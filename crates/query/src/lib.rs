//! # synton-query
//!
//! ⚠️ **已废弃 (Deprecated)** - 此 crate 的功能已被 `synton-paql` 完全取代。
//!
//! ## 迁移指南
//!
//! 请将依赖从 `synton-query` 迁移到 `synton-paql`：
//!
//! ```toml
//! # 旧版（已废弃）
//! [dependencies]
//! synton-query = { path = "../query" }
//!
//! # 新版
//! [dependencies]
//! synton-paql = { path = "../paql" }
//! ```
//!
//! ## 废弃原因
//!
//! - `synton-paql` 实现了完整的 PaQL (Prompt as Query Language) 解析器
//! - 功能更强大，支持更复杂的查询语法
//! - 使用 Nom 解析器组合子，性能更优
//! - 与项目架构设计更一致
//!
//! ## 废弃日期
//!
//! 2025-02-05
