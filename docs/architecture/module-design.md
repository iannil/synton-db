# SYNTON-DB 模块设计文档

**文档版本**: 1.0
**创建时间**: 2025-02-05
**作者**: SYNTON-DB Team

---

## 1. 概述

本文档定义 SYNTON-DB 的模块结构和职责划分，采用**高内聚、低耦合**的设计原则。

### 1.1 设计原则

1. **分层架构**: 接口层 → 认知层 → 存储层 → 基础设施层
2. **模块独立**: 每个模块可独立测试、编译、部署
3. **依赖单向**: 上层依赖下层，下层不依赖上层
4. **接口稳定**: 公共 API 使用语义化版本控制

---

## 2. 模块结构

### 2.1 顶层目录结构

```
synton-db/
├── Cargo.toml                    # Workspace 根配置
├── crates/
│   ├── core/                     # 核心数据模型和类型
│   ├── storage/                  # 存储引擎抽象和实现
│   ├── vector/                   # 向量索引和检索
│   ├── graph/                    # 图遍历和推理引擎
│   ├── ml/                       # ML 模型推理
│   ├── query/                    # PaQL 查询解析和执行
│   ├── api/                      # gRPC + REST 服务
│   └── cli/                      # 命令行工具
├── release/
│   └── rust/                     # 发布产物
├── docker/
│   ├── Dockerfile
│   └── docker-compose.yml
└── tests/
    ├── integration/              # 集成测试
    └── e2e/                      # 端到端测试
```

### 2.2 依赖关系图

```
                    ┌─────────────┐
                    │     cli     │
                    └─────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │     api     │
                    └─────────────┘
                           │
                           ▼
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│    query    │──────│    core     │──────│    graph    │
└─────────────┘      └─────────────┘      └─────────────┘
      │                   │                      │
      ▼                   ▼                      ▼
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│     ml      │      │   storage   │◄─────│   vector    │
└─────────────┘      └─────────────┘      └─────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │  基础设施层   │
                    │  (tokio等)   │
                    └─────────────┘
```

---

## 3. 核心模块详解

### 3.1 `core` - 核心数据模型

**职责**: 定义所有共享的数据结构和类型

**内容**:
- 节点 (`Node`)、边 (`Edge`) 结构
- 类型定义：`NodeType`, `Relation`, `NodeMeta`
- 错误类型：`SyntonError`
- ID 类型：`NodeId`, `EdgeId`

**关键结构**:

```rust
// 节点类型
pub enum NodeType {
    Entity,      // 实体（如"埃隆·马斯克"）
    Concept,     // 概念（如"人工智能"）
    Fact,        // 事实（如"特斯拉CEO是马斯克"）
    RawChunk,    // 原始文本片段
}

// 节点
pub struct Node {
    pub id: NodeId,
    pub content: Cow<str>,
    pub embedding: Option<Vec<f32>>,
    pub meta: NodeMeta,
    pub node_type: NodeType,
}

// 节点元数据
pub struct NodeMeta {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub access_score: f32,
    pub confidence: f32,
    pub source: Source,
}

// 边
pub struct Edge {
    pub source: NodeId,
    pub target: NodeId,
    pub relation: Relation,
    pub weight: f32,
    pub vector: Option<Vec<f32>>,
}

// 关系类型
pub enum Relation {
    IsPartOf,
    Causes,
    Contradicts,
    HappenedAfter,
    RelatedTo,
    Custom(String),
}
```

**依赖**: 无（最底层模块）

---

### 3.2 `storage` - 存储引擎

**职责**: 数据持久化和检索的抽象层

**内容**:
- `Store` trait 定义
- RocksDB 实现
- 列族管理
- 事务支持

**关键接口**:

```rust
#[async_trait]
pub trait Store: Send + Sync {
    // 节点操作
    async fn get_node(&self, id: NodeId) -> Result<Option<Node>, StorageError>;
    async fn put_node(&self, node: &Node) -> Result<(), StorageError>;
    async fn delete_node(&self, id: NodeId) -> Result<(), StorageError>;

    // 边操作
    async fn get_edge(&self, source: NodeId, target: NodeId) -> Result<Option<Edge>, StorageError>;
    async fn put_edge(&self, edge: &Edge) -> Result<(), StorageError>;
    async fn get_outgoing_edges(&self, source: NodeId) -> Result<Vec<Edge>, StorageError>;

    // 批量操作
    async fn batch_write(&self, ops: Vec<WriteOp>) -> Result<(), StorageError>;

    // 遍历
    async fn scan_nodes(&self, filter: NodeFilter) -> Result<BoxStream<Node>, StorageError>;
}
```

**列族设计**:

| 列族 | 数据类型 | Key | Value |
|------|----------|-----|-------|
| nodes | 节点 | NodeId (UUID) | Node JSON |
| edges | 边 | SourceID:TargetID:Relation | Edge JSON |
| edges_by_source | 边索引 | SourceID | [EdgeID] |
| metadata | 元数据 | String | JSON |
| access_log | 访问日志 | NodeID:Timestamp | AccessInfo |

**依赖**: `core`

---

### 3.3 `vector` - 向量索引

**职责**: 向量存储和相似度检索

**内容**:
- `VectorIndex` trait 定义
- Lance 实现
- HNSW 索引配置

**关键接口**:

```rust
#[async_trait]
pub trait VectorIndex: Send + Sync {
    // 插入向量
    async fn insert(&self, id: NodeId, vector: Vec<f32>) -> Result<(), VectorError>;

    // 批量插入
    async fn insert_batch(&self, vectors: Vec<(NodeId, Vec<f32>)>) -> Result<(), VectorError>;

    // 向量检索
    async fn search(&self, query: &[f32], k: usize) -> Result<Vec<SearchResult>, VectorError>;

    // 混合检索（向量 + 过滤）
    async fn search_with_filter(
        &self,
        query: &[f32],
        filter: Filter,
        k: usize,
    ) -> Result<Vec<SearchResult>, VectorError>;

    // 删除
    async fn remove(&self, id: NodeId) -> Result<(), VectorError>;

    // 更新
    async fn update(&self, id: NodeId, vector: Vec<f32>) -> Result<(), VectorError>;
}

pub struct SearchResult {
    pub id: NodeId,
    pub score: f32,
    pub metadata: HashMap<String, Value>,
}
```

**依赖**: `core`, `storage`

---

### 3.4 `graph` - 图遍历引擎

**职责**: 图遍历和推理操作

**内容**:
- `Graph` trait 定义
- BFS/DFS 遍历
- 最短路径
- 子图提取

**关键接口**:

```rust
#[async_trait]
pub trait Graph: Send + Sync {
    // 邻居查询
    async fn neighbors(&self, id: NodeId) -> Result<Vec<Node>, GraphError>;
    async fn neighbors_with_relation(
        &self,
        id: NodeId,
        relation: Relation,
    ) -> Result<Vec<Node>, GraphError>;

    // 遍历
    async fn bfs(
        &self,
        start: NodeId,
        depth: usize,
        filter: Option<EdgeFilter>,
    ) -> Result<Vec<Node>, GraphError>;

    async fn dfs(
        &self,
        start: NodeId,
        depth: usize,
        filter: Option<EdgeFilter>,
    ) -> Result<Vec<Node>, GraphError>;

    // 路径
    async fn shortest_path(
        &self,
        from: NodeId,
        to: NodeId,
    ) -> Result<Option<Vec<Node>>, GraphError>;

    // 子图
    async fn subgraph(
        &self,
        seeds: Vec<NodeId>,
        radius: usize,
    ) -> Result<SubGraph, GraphError>;
}
```

**依赖**: `core`, `storage`

---

### 3.5 `ml` - ML 推理

**职责**: 嵌入式模型推理

**内容**:
- 嵌入模型 (`EmbeddingModel`)
- 重排序模型 (`Reranker`)
- NLP 处理 (`NlpProcessor`)

**关键接口**:

```rust
#[async_trait]
pub trait EmbeddingModel: Send + Sync {
    async fn embed(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>, MlError>;
    async fn embed_single(&self, text: &str) -> Result<Vec<f32>, MlError>;
    fn dimension(&self) -> usize;
}

#[async_trait]
pub trait RerankerModel: Send + Sync {
    async fn rerank(
        &self,
        query: &str,
        documents: Vec<Document>,
        top_k: usize,
    ) -> Result<Vec<RerankResult>, MlError>;
}

pub struct NlpProcessor {
    tokenizer: Tokenizer,
    sentence_splitter: SentenceSplitter,
}

impl NlpProcessor {
    // 语义分块
    pub async fn semantic_chunk(&self, text: &str, max_tokens: usize) -> Result<Vec<Chunk>, NlpError>;

    // 实体抽取
    pub async fn extract_entities(&self, text: &str) -> Result<Vec<Entity>, NlpError>;

    // 关系抽取
    pub async fn extract_relations(&self, text: &str) -> Result<Vec<Relation>, NlpError>;
}
```

**依赖**: `core`

---

### 3.6 `query` - 查询引擎

**职责**: PaQL 解析和执行

**内容**:
- PaQL 词法/语法解析 (`nom`)
- 查询计划生成
- 查询执行引擎
- 结果合并

**关键结构**:

```rust
// PaQL 查询
pub enum PaqlQuery {
    SemanticSearch {
        query: String,
        filters: Vec<Filter>,
        limit: usize,
    },
    GraphTraversal {
        start: String,
        relation: Option<Relation>,
        depth: usize,
    },
    Hybrid {
        query: String,
        graph_hops: usize,
        limit: usize,
    },
}

// 查询结果
pub struct QueryResult {
    pub nodes: Vec<Node>,
    pub paths: Vec<ReasoningPath>,
    pub context: String,
    pub confidence: f32,
}

// 查询引擎
pub struct QueryEngine {
    store: Arc<dyn Store>,
    vector_index: Arc<dyn VectorIndex>,
    graph: Arc<dyn Graph>,
    embedding_model: Arc<dyn EmbeddingModel>,
    reranker: Option<Arc<dyn RerankerModel>>,
}

impl QueryEngine {
    pub async fn execute(&self, query: PaqlQuery) -> Result<QueryResult, QueryError> {
        match query {
            PaqlQuery::SemanticSearch { query, filters, limit } => {
                self.semantic_search(&query, filters, limit).await
            }
            PaqlQuery::GraphTraversal { start, relation, depth } => {
                self.graph_traversal(start, relation, depth).await
            }
            PaqlQuery::Hybrid { query, graph_hops, limit } => {
                self.hybrid_search(&query, graph_hops, limit).await
            }
        }
    }
}
```

**依赖**: `core`, `storage`, `vector`, `graph`, `ml`

---

### 3.7 `api` - API 服务

**职责**: 对外服务接口

**内容**:
- gRPC 服务 (`tonic`)
- REST 服务 (`axum`)
- WebSocket 支持

**gRPC 定义** (`api.proto`):

```protobuf
syntax = "proto3";

package synton;

service SyntonDB {
    rpc Absorb(AbsorbRequest) returns (AbsorbResponse);
    rpc Query(QueryRequest) returns (QueryResponse);
    rpc GetNode(GetNodeRequest) returns (GetNodeResponse);
    rpc DeleteNode(DeleteNodeRequest) returns (DeleteNodeResponse);
}

message AbsorbRequest {
    string content = 1;
    map<string, string> metadata = 2;
}

message AbsorbResponse {
    repeated string node_ids = 1;
}

message QueryRequest {
    string paql = 1;
    uint32 limit = 2;
}

message QueryResponse {
    repeated Node nodes = 1;
    string context = 2;
    float confidence = 3;
}

message Node {
    string id = 1;
    string content = 2;
    string node_type = 3;
    map<string, Value> metadata = 4;
}
```

**依赖**: `core`, `query`

---

### 3.8 `cli` - 命令行工具

**职责**: 管理和调试 CLI

**内容**:
- 数据导入/导出
- 查询执行
- 状态检查
- 性能分析

**命令结构**:

```bash
synton-db              # 启动服务
synton-db import FILE  # 导入数据
synton-db query PAQL   # 执行查询
synton-db status       # 检查状态
synton-db bench        # 性能测试
```

---

## 4. 模块编译配置

### 4.1 Workspace Cargo.toml

```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
authors = ["SYNTON-DB Team"]
license = "Apache-2.0"
repository = "https://github.com/synton-db/synton-db"

[workspace.dependencies]
# 基础设施
tokio = { version = "1.40", features = ["full"] }
async-trait = "0.1"
anyhow = "1.0"
thiserror = "2.0"
tracing = "0.1"
tracing-subscriber = "0.3"

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.10", features = ["v4", "serde"] }

# 存储
lance = "0.12"
rocksdb = "0.22"

# ML
candle = { version = "0.6", features = ["mkl"] }
candle-transformers = "0.6"
tokenizers = "0.20"

# 网络
tonic = "0.12"
prost = "0.13"
axum = "0.7"

# 解析
nom = "7.1"
```

### 4.2 各模块配置

```toml
# crates/core/Cargo.toml
[package]
name = "synton-core"

[dependencies]
uuid = { workspace = true }
serde = { workspace = true }
chrono = { version = "0.4", features = ["serde"] }
thiserror = { workspace = true }
```

```toml
# crates/storage/Cargo.toml
[package]
name = "synton-storage"

[dependencies]
synton-core = { path = "../core" }
tokio = { workspace = true }
anyhow = { workspace = true }
rocksdb = { workspace = true }
async-trait = { workspace = true }
```

```toml
# crates/api/Cargo.toml
[package]
name = "synton-api"

[dependencies]
synton-core = { path = "../core" }
synton-query = { path = "../query" }
tokio = { workspace = true }
tonic = { workspace = true }
prost = { workspace = true }
axum = { workspace = true }
```

---

## 5. 模块演进路径

### 5.1 MVP0: 最小验证

| 模块 | 范围 | 状态 |
|------|------|------|
| core | 基础数据结构 | ✅ |
| storage | RocksDB 基础操作 | 🔄 |
| vector | Lance 基础检索 | 🔄 |
| query | 简单向量检索 | 🔄 |

### 5.2 MVP1: 图基础

| 模块 | 新增功能 | 状态 |
|------|----------|------|
| storage | 边存储 | 📋 |
| graph | BFS 遍历 | 📋 |
| query | Graph-RAG 混合 | 📋 |

### 5.3 MVP2: 认知层

| 模块 | 新增功能 | 状态 |
|------|----------|------|
| ml | 嵌入推理 | 📋 |
| ml | 重排序 | 📋 |
| query | PaQL 解析 | 📋 |

### 5.4 MVP3: 完整服务

| 模块 | 新增功能 | 状态 |
|------|----------|------|
| api | gRPC 服务 | 📋 |
| api | REST 服务 | 📋 |
| cli | 管理工具 | 📋 |

---

## 6. 测试策略

### 6.1 单元测试

- 每个模块 `tests/` 目录
- 核心算法单元测试
- Mock 依赖

### 6.2 集成测试

- `tests/integration/`
- 跨模块交互测试
- 使用测试 fixture

### 6.3 端到端测试

- `tests/e2e/`
- 完整工作流测试
- 性能基准测试

---

## 参考资料

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [The Rust API Design Book](https://rust-lang.github.io/rust-api-guidelines/)
