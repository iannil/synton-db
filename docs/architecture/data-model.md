# SYNTON-DB 数据模型文档

**文档版本**: 1.0
**创建时间**: 2025-02-05
**作者**: SYNTON-DB Team

---

## 1. 概述

SYNTON-DB 的核心数据模型是**张量图 (Tensor-Graph)**，一种结合了向量表示和图关系的混合数据结构。

### 1.1 设计原则

1. **语义原子性**: 每个节点代表最小的语义单元
2. **关系显式化**: 边携带逻辑关系和权重
3. **时序保留**: 所有数据带时间戳，支持历史追溯
4. **动态演化**: 数据通过访问和反馈不断更新

---

## 2. 核心数据类型

### 2.1 节点 (Node)

节点是知识的基本单元，代表一个语义原子。

```rust
use std::borrow::Cow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 节点类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    /// 实体（如"埃隆·马斯克"、"特斯拉"）
    Entity,
    /// 概念（如"人工智能"、"电动汽车"）
    Concept,
    /// 事实（如"特斯拉CEO是马斯克"、"巴黎是法国首都"）
    Fact,
    /// 原始文本片段（未结构化的内容）
    RawChunk,
}

/// 节点元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMeta {
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 最后访问时间
    pub accessed_at: Option<DateTime<Utc>>,
    /// 访问分数（用于记忆衰退）
    pub access_score: f32,
    /// 置信度（0.0-1.0）
    pub confidence: f32,
    /// 数据来源
    pub source: Source,
    /// 原始文档ID（如果是分块产生的）
    pub document_id: Option<Uuid>,
    /// 块索引（如果是分块产生的）
    pub chunk_index: Option<usize>,
}

/// 数据来源
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    UserInput,
    FileUpload,
    WebCrawl,
    ApiImport,
    AutoExtracted,
    Custom(String),
}

/// 节点 - SYNTON-DB 的核心数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// 唯一标识符
    pub id: Uuid,
    /// 内容（使用 Cow 避免不必要的复制）
    pub content: Cow<'static, str>,
    /// 向量表示（可选，惰性计算）
    pub embedding: Option<Vec<f32>>,
    /// 节点元数据
    pub meta: NodeMeta,
    /// 节点类型
    pub node_type: NodeType,
    /// 额外属性（灵活扩展）
    pub attributes: serde_json::Value,
}

impl Node {
    /// 创建新节点
    pub fn new(content: impl Into<Cow<'static, str>>, node_type: NodeType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            content: content.into(),
            embedding: None,
            meta: NodeMeta {
                created_at: now,
                updated_at: now,
                accessed_at: None,
                access_score: 1.0,
                confidence: 1.0,
                source: Source::UserInput,
                document_id: None,
                chunk_index: None,
            },
            node_type,
            attributes: serde_json::json!({}),
        }
    }

    /// 获取嵌入向量（如果不存在则返回 None）
    pub fn embedding(&self) -> Option<&[f32]> {
        self.embedding.as_deref()
    }

    /// 更新访问分数（基于遗忘曲线）
    pub fn decay_access_score(&mut self, hours_passed: f64, lambda: f32) {
        // CurrentScore = InitialScore * e^(-λ * TimeElapsed)
        let decay = (-lambda * hours_passed).exp();
        self.meta.access_score = self.meta.access_score * decay;
    }

    /// 强化访问分数（被引用或用户反馈）
    pub fn reinforce_access_score(&mut self, delta: f32) {
        self.meta.access_score = (self.meta.access_score + delta).min(10.0);
        self.meta.accessed_at = Some(Utc::now());
    }
}
```

### 2.2 边 (Edge)

边表示节点间的逻辑关系，是 SYNTON-DB 推理能力的基础。

```rust
/// 关系类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Relation {
    /// 组成关系（A是B的一部分）
    IsPartOf,
    /// 因果关系（A导致B）
    Causes,
    /// 矛盾关系（A与B矛盾）
    Contradicts,
    /// 时序关系（A发生在B之后）
    HappenedAfter,
    /// 相似关系（A与B相似）
    SimilarTo,
    /// 类别关系（A是B的一种）
    IsA,
    /// 位置关系（A位于B）
    LocatedAt,
    /// 从属关系（A属于B）
    BelongsTo,
    /// 自定义关系
    Custom(String),
}

impl Relation {
    /// 获取关系的反向关系
    pub fn reverse(&self) -> Relation {
        match self {
            Relation::IsPartOf => Relation::BelongsTo,
            Relation::Causes => Relation::Custom("caused_by".to_string()),
            Relation::HappenedAfter => Relation::Custom("happened_before".to_string()),
            Relation::LocatedAt => Relation::Custom("contains".to_string()),
            Relation::IsA => Relation::Custom("has_instance".to_string()),
            _ => Relation::RelatedTo,
        }
    }
}

/// 边 - 节点间的逻辑连接
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// 源节点 ID
    pub source: Uuid,
    /// 目标节点 ID
    pub target: Uuid,
    /// 关系类型
    pub relation: Relation,
    /// 关系权重（0.0-1.0，表示关系强度）
    pub weight: f32,
    /// 关系的向量表示（用于模糊关系查询）
    pub vector: Option<Vec<f32>>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 是否已过期（用于动态事实修正）
    pub expired: bool,
    /// 替换者（如果此边已被新边替换）
    pub replaced_by: Option<Uuid>,
    /// 额外属性
    pub attributes: serde_json::Value,
}

impl Edge {
    /// 创建新边
    pub fn new(source: Uuid, target: Uuid, relation: Relation) -> Self {
        Self {
            source,
            target,
            relation,
            weight: 1.0,
            vector: None,
            created_at: Utc::now(),
            expired: false,
            replaced_by: None,
            attributes: serde_json::json!({}),
        }
    }

    /// 获取边的唯一标识符
    pub fn id(&self) -> String {
        format!("{}::{}::{:?}", self.source, self.target, self.relation)
    }

    /// 标记为过期
    pub fn expire(&mut self) {
        self.expired = true;
    }
}
```

### 2.3 推理路径 (ReasoningPath)

表示图遍历的推理路径，是 Graph-RAG 的核心输出。

```rust
/// 推理路径 - 图遍历的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningPath {
    /// 路径中的节点序列
    pub nodes: Vec<Node>,
    /// 路径中的边序列
    pub edges: Vec<Edge>,
    /// 路径置信度
    pub confidence: f32,
    /// 路径解释（自然语言）
    pub explanation: String,
    /// 路径类型
    pub path_type: PathType,
}

/// 路径类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathType {
    /// 因果链
    Causal,
    /// 层次结构
    Hierarchical,
    /// 时序链
    Temporal,
    /// 关联链
    Associative,
    /// 混合路径
    Hybrid,
}

impl ReasoningPath {
    /// 获取起始节点
    pub fn start(&self) -> Option<&Node> {
        self.nodes.first()
    }

    /// 获取终止节点
    pub fn end(&self) -> Option<&Node> {
        self.nodes.last()
    }

    /// 路径长度（节点数）
    pub fn length(&self) -> usize {
        self.nodes.len()
    }

    /// 路径跳数（边数）
    pub fn hops(&self) -> usize {
        self.edges.len()
    }
}
```

---

## 3. 存储模型

### 3.1 Lance 数据集结构

Lance 数据集存储向量和元数据。

#### 节点数据集 (`nodes`)

```
Schema:
├── id: string (UUID)
├── content: string
├── embedding: fixed_size_list[1536] (float32)
├── node_type: string
├── created_at: timestamp
├── updated_at: timestamp
├── accessed_at: timestamp (nullable)
├── access_score: float32
├── confidence: float32
└── attributes: json (nullable)
```

#### 边数据集 (`edges`)

```
Schema:
├── source_id: string (UUID)
├── target_id: string (UUID)
├── relation: string
├── weight: float32
├── vector: fixed_size_list[768] (float32, nullable)
├── created_at: timestamp
├── expired: boolean
├── replaced_by: string (UUID, nullable)
└── attributes: json (nullable)
```

### 3.2 RocksDB 列族结构

RocksDB 作为 KV 存储，处理快速查找和图遍历。

| 列族 | Key | Value | 说明 |
|------|-----|-------|------|
| `nodes` | UUID | Node JSON | 节点完整数据 |
| `edges` | `src::tgt::rel` | Edge JSON | 边完整数据 |
| `edges_out` | `src::rel` | `[target_ids]` | 出边索引 |
| `edges_in` | `tgt::rel` | `[source_ids]` | 入边索引 |
| `metadata` | 字符串键 | JSON | 系统元数据 |
| `access_log` | `node_id::ts` | 访问信息 | 访问历史（用于衰减计算） |

---

## 4. 查询模型

### 4.1 PaQL 查询

```rust
/// PaQL (Prompt as Query Language) 查询
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PaqlQuery {
    /// 语义搜索（向量检索）
    Semantic {
        query: String,
        filters: Vec<Filter>,
        limit: usize,
    },
    /// 图遍历
    Graph {
        start: Vec<Uuid>,
        direction: TraverseDirection,
        max_depth: usize,
        relation_filter: Option<Relation>,
    },
    /// 混合查询（Graph-RAG）
    Hybrid {
        query: String,
        graph_hops: usize,
        filters: Vec<Filter>,
        limit: usize,
    },
    /// 自然语言查询（需要解析）
    Natural {
        text: String,
    },
}

/// 过滤条件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Filter {
    /// 字段等于
    Equals { field: String, value: serde_json::Value },
    /// 字段包含
    Contains { field: String, value: String },
    /// 字段大于
    GreaterThan { field: String, value: f64 },
    /// 字段小于
    LessThan { field: String, value: f64 },
    /// 字段在列表中
    InList { field: String, values: Vec<serde_json::Value> },
}

/// 遍历方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraverseDirection {
    Outgoing,
    Incoming,
    Both,
}
```

### 4.2 查询结果

```rust
/// 查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// 匹配的节点
    pub nodes: Vec<Node>,
    /// 推理路径（如果存在）
    pub paths: Vec<ReasoningPath>,
    /// 合成的上下文
    pub context: String,
    /// 整体置信度
    pub confidence: f32,
    /// 查询元数据
    pub metadata: QueryMetadata,
}

/// 查询元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetadata {
    /// 查询延迟（毫秒）
    pub latency_ms: u64,
    /// 扫描的节点数
    pub nodes_scanned: usize,
    /// 图遍历跳数
    pub graph_hops: usize,
    /// 搜索策略
    pub search_strategy: SearchStrategy,
}

/// 搜索策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchStrategy {
    /// 纯向量搜索
    VectorOnly,
    /// 纯图遍历
    GraphOnly,
    /// 混合搜索（Graph-RAG）
    HybridVectorGraph,
    /// 关键词搜索
    Keyword,
}
```

---

## 5. 数据操作

### 5.1 写入操作 (Absorb)

```rust
/// 数据摄入请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorbRequest {
    /// 内容
    pub content: String,
    /// 元数据
    pub metadata: serde_json::Value,
    /// 数据来源
    pub source: Source,
    /// 是否自动分块
    pub auto_chunk: bool,
    /// 是否自动抽取实体和关系
    pub auto_extract: bool,
}

/// 数据摄入响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorbResponse {
    /// 创建的节点 ID
    pub node_ids: Vec<Uuid>,
    /// 分块数量
    pub chunks_created: usize,
    /// 抽取的实体数量
    pub entities_extracted: usize,
    /// 创建的边数量
    pub edges_created: usize,
}
```

### 5.2 更新操作

```rust
/// 节点更新
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeUpdate {
    /// 更新内容
    Content { content: String },
    /// 更新属性
    Attributes { attributes: serde_json::Value },
    /// 调整访问分数
    AccessScoreDelta { delta: f32 },
    /// 标记为过期
    Expire,
}
```

---

## 6. 数据约束

### 6.1 节点约束

- `id`: 必须是有效的 UUID v4
- `content`: 非空，最大长度 10MB
- `confidence`: 范围 [0.0, 1.0]
- `access_score`: 范围 [0.0, 10.0]

### 6.2 边约束

- `source` != `target`: 不允许自环
- `weight`: 范围 [0.0, 1.0]
- 同一 `(source, target, relation)` 组合允许存在多条边

---

## 7. 数据迁移

### 7.1 版本化

数据模型使用语义版本控制：

```
v1.0.0 - 初始版本
v1.1.0 - 添加 NodeMeta.accessed_at
v1.2.0 - 添加 Edge.expired, Edge.replaced_by
```

### 7.2 兼容性策略

- **向后兼容**: 新字段可选，旧数据自动填充默认值
- **向前兼容**: 忽略未知字段
- **迁移脚本**: 结构变更提供自动迁移

---

## 8. 性能考虑

### 8.1 向量维度

| 用途 | 推荐维度 | 模型示例 |
|------|----------|----------|
| 通用语义 | 384-768 | MiniLM, E5 |
| 高精度 | 1536 | OpenAI Embeddings |
| 关系向量 | 256-512 | 专用关系编码器 |

### 8.2 存储估算

假设 100万 节点：

- **向量存储** (1536维 float32): 100万 × 1536 × 4字节 ≈ 5.7GB
- **元数据**: 约 1-2GB
- **边**: 假设每节点 5 条边，约 3-5GB

**总计**: 约 10-15GB

---

## 参考资料

- [Apache Arrow Data Types](https://arrow.apache.org/docs/python/api/datatypes.html)
- [Rust Serialization](https://serde.rs/)
- [Vector Database Design](https://zilliz.com/learn/what_is_vector_database)
