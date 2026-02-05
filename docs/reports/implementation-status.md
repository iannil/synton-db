# SYNTON-DB 实现状态

**更新时间**: 2025-02-05

---

## 概览

| 指标 | 状态 |
|------|------|
| MVP 进度 | 6/6 (100%) |
| Crate 完成度 | 8/11 (73%) |
| 编译状态 | ✅ 通过 |
| 测试模块 | 28 个 |

---

## Crate 实现状态

### 已完整实现 (8)

| Crate | 状态 | 关键导出类型 | 测试 |
|-------|------|-------------|------|
| `core` | ✅ 完整 | Node, Edge, Relation, NodeType, Source, Filter, Path, Error | ✅ |
| `graph` | ✅ 完整 | Graph, MemoryGraph, BFS/DFS, shortest_path, TraversalResult | ✅ |
| `graphrag` | ✅ 完整 | GraphRag, RetrievalConfig, RetrievedContext, Scorer | ✅ |
| `paql` | ✅ 完整 | Query, Parser, QueryNode, BinaryOp, ComparisonOp | ✅ |
| `memory` | ✅ 完整 | DecayCalculator, MemoryManager, Ebbinghaus curve | ✅ |
| `storage` | ✅ 完整 | Store, RocksdbStore, ColumnFamily, WriteOp | ✅ |
| `vector` | ✅ 基础 | VectorIndex, MemoryVectorIndex | ✅ |
| `api` | ✅ 完整 | SyntonDbService, REST/gRPC routers, AppState | ✅ |

### 占位符/未实现 (2)

| Crate | 状态 | 说明 |
|-------|------|------|
| `cli` | ⏸️ 占位符 | 预留给 CLI 工具实现 |
| `ml` | ⏸️ 占位符 | 预留给 ML 集成（Candle） |

### 已废弃 (1)

| Crate | 状态 | 说明 |
|-------|------|------|
| `query` | ❌ 已废弃 | 功能已被 `paql` 完全取代 |

---

## 核心类型导出清单

### core Crate - 核心数据模型

```rust
// 节点
pub struct Node {
    pub id: Uuid,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub node_type: NodeType,
    pub source: Source,
    pub metadata: HashMap<String, Value>,
    pub created_at: DateTime<Utc>,
}

pub enum NodeType {
    Entity,
    Concept,
    Fact,
    Fragment,
}

// 边
pub struct Edge {
    pub id: Uuid,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub relation: Relation,
    pub weight: f32,
    pub metadata: HashMap<String, Value>,
    pub created_at: DateTime<Utc>,
}

pub struct Relation(pub String);
pub enum RelationType {
    IsA,
    PartOf,
    Causes,
    RelatedTo,
    SimilarTo,
    Contradicts,
}

// 路径
pub struct ReasoningPath {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub confidence: f32,
    pub explanation: String,
    pub path_type: PathType,
}

pub enum PathType {
    Deductive,
    Inductive,
    Analogical,
    Abductive,
}

// 过滤器
pub enum Filter {
    True,
    False,
    Not(Box<Filter>),
    And(Vec<Filter>),
    Or(Vec<Filter>),
    Equals { field: String, value: FilterValue },
    NotEquals { field: String, value: FilterValue },
    GreaterThan { field: String, value: FilterValue },
    LessThan { field: String, value: FilterValue },
    In { field: String, values: Vec<FilterValue> },
}

pub enum FilterValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

// 来源
pub struct Source {
    pub source_type: SourceType,
    pub identifier: String,
    pub timestamp: DateTime<Utc>,
}

pub enum SourceType {
    Document,
    API,
    Inferred,
    Manual,
}

// 错误
pub enum Error {
    NodeNotFound(Uuid),
    EdgeNotFound(Uuid),
    InvalidFilter(String),
    InvalidEmbedding(usize),
    ContentTooLarge { size: usize, max: usize },
    StorageError(String),
}

pub type CoreResult<T> = Result<T, Error>;
```

### graph Crate - 图遍历引擎

```rust
// 图抽象
pub trait Graph: Send + Sync {
    fn add_node(&mut self, node: Node) -> Result<(), Error>;
    fn add_edge(&mut self, edge: Edge) -> Result<(), Error>;
    fn get_node(&self, id: Uuid) -> Option<&Node>;
    fn get_edge(&self, id: Uuid) -> Option<&Edge>;
    fn get_neighbors(&self, id: Uuid) -> Vec<&Node>;
    fn find_path(&self, from: Uuid, to: Uuid) -> Option<ReasoningPath>;
}

// 内存实现
pub struct MemoryGraph {
    nodes: HashMap<Uuid, Node>,
    edges: HashMap<Uuid, Edge>,
    adjacency: HashMap<Uuid, Vec<Uuid>>,
}

impl MemoryGraph {
    pub fn new() -> Self;
    pub fn traverse(&self, start: Uuid, config: TraversalConfig) -> TraversalResult;
    pub fn bfs(&self, start: Uuid, goal: Option<Uuid>) -> TraversalResult;
    pub fn dfs(&self, start: Uuid, goal: Option<Uuid>) -> TraversalResult;
    pub fn shortest_path(&self, from: Uuid, to: Uuid) -> Option<ReasoningPath>;
}

// 遍历配置
pub struct TraversalConfig {
    pub order: TraversalOrder,
    pub max_depth: usize,
    pub max_nodes: usize,
    pub goal: Option<Uuid>,
}

pub enum TraversalOrder {
    BFS,
    DFS,
}

// 遍历结果
pub struct TraversalResult {
    pub visited: Vec<Uuid>,
    pub path: Option<ReasoningPath>,
    pub edges_traversed: usize,
}
```

### graphrag Crate - 混合检索

```rust
// Graph-RAG 引擎
pub struct GraphRag<G, V, S>
where
    G: Graph,
    V: VectorIndex,
    S: Scorer,
{
    graph: G,
    vector_index: V,
    scorer: S,
}

impl<G, V, S> GraphRag<G, V, S>
where
    G: Graph,
    V: VectorIndex,
    S: Scorer,
{
    pub fn new(graph: G, vector_index: V, scorer: S) -> Self;
    pub fn retrieve(&self, query: &str, config: RetrievalConfig) -> Result<RetrievedContext>;
    pub fn add_node(&mut self, node: Node) -> Result<()>;
    pub fn add_edge(&mut self, edge: Edge) -> Result<()>;
}

// 检索配置
pub struct RetrievalConfig {
    pub mode: RetrievalMode,
    pub top_k: usize,
    pub max_depth: usize,
    pub alpha: f32,  // 向量检索权重
    pub beta: f32,   // 图遍历权重
}

pub enum RetrievalMode {
    VectorOnly,
    GraphOnly,
    Hybrid,
}

// 检索结果
pub struct RetrievedContext {
    pub nodes: Vec<RetrievedNode>,
    pub query_embedding: Vec<f32>,
    pub retrieval_mode: RetrievalMode,
}

pub struct RetrievedNode {
    pub node: Node,
    pub score: f32,
    pub vector_score: f32,
    pub graph_score: f32,
}

// 评分器
pub trait Scorer: Send + Sync {
    fn score(&self, vector_score: f32, graph_score: f32, config: &RetrievalConfig) -> f32;
}

pub struct DefaultScorer;
```

### paql Crate - 查询语言

```rust
// 查询 AST
pub enum Query {
    Select {
        node: QueryNode,
        filters: Vec<Filter>,
        limit: Option<usize>,
        order: Vec<SortField>,
    },
    Explain {
        from: String,
        to: String,
        max_depth: Option<usize>,
    },
    Similar {
        to: String,
        top_k: Option<usize>,
    },
}

pub struct QueryNode {
    pub id: Option<String>,
    pub node_type: Option<NodeType>,
    pub relation: Option<Relation>,
    pub filters: Vec<Filter>,
}

// 运算符
pub enum BinaryOp {
    And,
    Or,
}

pub enum ComparisonOp {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    Contains,
}

pub struct SortField {
    pub field: String,
    pub ascending: bool,
}

// 解析器
pub struct Parser;

impl Parser {
    pub fn new() -> Self;
    pub fn parse(&self, input: &str) -> Result<Query>;
}

// 查询构建器
pub struct QueryBuilder;

impl QueryBuilder {
    pub fn new() -> Self;
    pub fn select(mut self, node: QueryNode) -> Self;
    pub fn filter(mut self, filter: Filter) -> Self;
    pub fn limit(mut self, limit: usize) -> Self;
    pub fn build(self) -> Query;
}
```

### memory Crate - 记忆机制

```rust
// 记忆管理器
pub struct MemoryManager<D>
where
    D: DecayCalculator,
{
    calculator: D,
    access_counts: HashMap<Uuid, u64>,
    last_access: HashMap<Uuid, DateTime<Utc>>,
}

impl<D> MemoryManager<D>
where
    D: DecayCalculator,
{
    pub fn new(calculator: D) -> Self;
    pub fn record_access(&mut self, id: Uuid);
    pub fn get_score(&self, id: Uuid, created_at: DateTime<Utc>) -> f32;
    pub fn should_retain(&self, id: Uuid, created_at: DateTime<Utc>, threshold: f32) -> bool;
}

// 衰减计算器
pub trait DecayCalculator: Send + Sync {
    fn calculate(&self, days_since_creation: f32, access_count: u32) -> f32;
}

pub struct EbbinghausDecay {
    config: DecayConfig,
}

impl DecayCalculator for EbbinghausDecay {
    fn calculate(&self, days: f32, access_count: u32) -> f32 {
        // R = e^(-d/S) * (1 + log(access_count + 1))
    }
}

// 衰减配置
pub struct DecayConfig {
    pub scale: f32,  // 遗忘曲线尺度参数
}
```

### storage Crate - 存储层

```rust
// 存储抽象
pub trait Store: Send + Sync {
    async fn put(&mut self, key: &[u8], value: &[u8], cf: ColumnFamily) -> Result<()>;
    async fn get(&self, key: &[u8], cf: ColumnFamily) -> Result<Option<Vec<u8>>>;
    async fn delete(&mut self, key: &[u8], cf: ColumnFamily) -> Result<()>;
    async fn batch(&mut self, ops: Vec<WriteOp>) -> Result<()>;
    async fn scan(&self, prefix: &[u8], cf: ColumnFamily) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
}

// RocksDB 实现
pub struct RocksdbStore {
    db: Db,
    path: PathBuf,
}

impl RocksdbStore {
    pub fn new(path: impl AsRef<Path>) -> Result<Self>;
    pub fn open_cf(&mut self, cf: ColumnFamily) -> Result<()>;
}

// 列族定义
pub enum ColumnFamily {
    Default,
    Nodes,
    Edges,
    Metadata,
    Indices,
}

// 写操作
pub enum WriteOp {
    Put { key: Vec<u8>, value: Vec<u8>, cf: ColumnFamily },
    Delete { key: Vec<u8>, cf: ColumnFamily },
}
```

### vector Crate - 向量索引

```rust
// 向量索引抽象
pub trait VectorIndex: Send + Sync {
    fn add(&mut self, id: Uuid, embedding: Vec<f32>) -> Result<()>;
    fn search(&self, query: &[f32], k: usize) -> Result<Vec<(Uuid, f32)>>;
    fn remove(&mut self, id: Uuid) -> Result<()>;
    fn len(&self) -> usize;
}

// 内存实现（测试用）
pub struct MemoryVectorIndex {
    embeddings: HashMap<Uuid, Vec<f32>>,
}

impl MemoryVectorIndex {
    pub fn new() -> Self;
    pub fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32;
}
```

### api Crate - API 服务

```rust
// 服务状态
pub struct AppState {
    pub graph: Arc<RwLock<MemoryGraph>>,
    pub vector_index: Arc<RwLock<MemoryVectorIndex>>,
    pub graphrag: Arc<RwLock<GraphRag<MemoryGraph, MemoryVectorIndex, DefaultScorer>>>,
    pub memory_manager: Arc<RwLock<MemoryManager<EbbinghausDecay>>>,
}

// gRPC 服务
pub struct SyntonDbService {
    pub state: AppState,
}

impl SyntonDbService {
    pub fn new(state: AppState) -> Self;
}

// REST 路由
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/nodes", post(create_node).get(get_nodes))
        .route("/nodes/:id", get(get_node).delete(delete_node))
        .route("/edges", post(create_edge))
        .route("/query", post(execute_query))
}
```

---

## 依赖关系图

```
┌─────────────────────────────────────────────────────────────┐
│                          CLI                                 │
│                       (占位符)                                │
└────────────────────────────┬────────────────────────────────┘
                             │
┌────────────────────────────┴────────────────────────────────┐
│                          API                                 │
│                    (Axum + tonic)                            │
└───────────┬──────────────────────┬──────────────────────────┘
            │                      │
┌───────────┴───────────┐  ┌──────┴────────┐
│        PAQL           │  │    GRAPHRAG   │
│     (Nom parser)      │  │  (混合检索)    │
└───────────┬───────────┘  └──────┬────────┘
            │                      │
┌───────────┴──────────────────────┴──────────────────────────┐
│                      GRAPH                                   │
│                (BFS/DFS 遍历)                                 │
└───────────┬──────────────────────┬──────────────────────────┘
            │                      │
┌───────────┴───────────┐  ┌──────┴────────┐
│       MEMORY         │  │    VECTOR     │
│   (遗忘曲线)          │  │  (向量索引)    │
└───────────┬───────────┘  └──────┬────────┘
            │                      │
┌───────────┴──────────────────────┴──────────────────────────┐
│                     STORAGE                                  │
│                   (RocksDB)                                  │
└───────────┬──────────────────────────────────────────────────┘
            │
┌───────────┴──────────────────────────────────────────────────┐
│                      CORE                                    │
│            (Node, Edge, Filter, Error)                       │
└─────────────────────────────────────────────────────────────┘
            ▲
            │
┌───────────┴──────────────────────────────────────────────────┐
│                        ML                                     │
│                     (占位符)                                  │
└─────────────────────────────────────────────────────────────┘
```

---

## 编译状态

```bash
$ cargo check --workspace
    Checking synton-core v0.1.0 (/Users/iannil/Code/synton-db/crates/core)
    Checking synton-storage v0.1.0 (/Users/iannil/Code/synton-db/crates/storage)
    Checking synton-vector v0.1.0 (/Users/iannil/Code/synton-db/crates/vector)
    Checking synton-graph v0.1.0 (/Users/iannil/Code/synton-db/crates/graph)
    Checking synton-graphrag v0.1.0 (/Users/iannil/Code/synton-db/crates/graphrag)
    Checking synton-paql v0.1.0 (/Users/iannil/Code/synton-db/crates/paql)
    Checking synton-memory v0.1.0 (/Users/iannil/Code/synton-db/crates/memory)
    Checking synton-api v0.1.0 (/Users/iannil/Code/synton-db/crates/api)
    Checking synton-cli v0.1.0 (/Users/iannil/Code/synton-db/crates/cli)
    Checking synton-ml v0.1.0 (/Users/iannil/Code/synton-db/crates/ml)
    Checking synton-query v0.1.0 (/Users/iannil/Code/synton-db/crates/query)
    Checking synton-db v0.1.0 (/Users/iannil/Code/synton-db)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.42s
```

### 警告清单

| 警告 | 位置 | 说明 |
|------|------|------|
| dead_code | `core/src/path.rs:169` | `PathBuilder` 未使用 |
| dead_code | `core/src/path.rs:245` | `PathBuildError` 未使用 |
| non_upper_case_globals | `core/src/relation.rs:104` | `RelatedTo` 命名风格 |
| missing_docs | `core/src/error.rs` | 字段缺少文档 |

---

## 测试覆盖

```bash
$ cargo test --workspace --no-run
   Compiling [...]
    Finished test profile [unoptimized + debuginfo] target(s)

# 测试模块数量: 28 个
```

---

## 下一步

| 任务 | 优先级 | 预估 |
|------|--------|------|
| 服务端主程序实现 | 高 | - |
| Docker 部署配置 | 高 | - |
| CLI 工具实现 | 中 | - |
| ML 集成（Candle） | 中 | - |
| E2E 测试准备 | 低 | - |
