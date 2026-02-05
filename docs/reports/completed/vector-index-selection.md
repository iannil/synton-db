# 向量索引选型报告

**报告时间**: 2025-02-05
**决策状态**: 已完成
**推荐方案**: Lance

---

## 1. 候选技术

| 技术 | 语言 | 许可证 | 项目状态 |
|------|------|--------|----------|
| Lance | Rust | Apache 2.0 | LanceDB 团队开发，活跃维护 |
| Faiss | C++ | MIT | Meta 开源，成熟稳定 |
| usearch | C++ | Apache 2.0 | unum 团队，活跃 |
| Hnswlib | C++ | MIT | nmslib/hnswlib，稳定 |

---

## 2. 详细对比

### 2.1 技术架构

| 维度 | Lance | Faiss | usearch | Hnswlib |
|------|-------|-------|---------|---------|
| 核心语言 | Rust 原生 | C++ | C++ | C++ |
| Rust 绑定 | 原生 | FFI (unsafe) | FFI | FFI |
| 算法支持 | HNSW, IVF | HNSW, IVF, PQ, SQ, 等 | HNSW | HNSW |
| 磁盘索引 | 原生 DiskANN | 需自行管理 | 支持 | 不支持 |

### 2.2 性能特征

| 维度 | Lance | Faiss | usearch | Hnswlib |
|------|-------|-------|---------|---------|
| 构建速度 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| 查询速度 (QPS) | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| 内存效率 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| 磁盘支持 | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ | ⭐ |

### 2.3 元数据管理

| 维度 | Lance | Faiss | usearch | Hnswlib |
|------|-------|-------|---------|---------|
| 内置元数据存储 | ✅ Parquet 格式 | ❌ 需外部 KV | ✅ | ❌ |
| 元数据过滤 | ✅ 原生支持 | ❌ 需额外处理 | ✅ | ❌ |
| 元数据更新 | ✅ 支持 | ⚠️ 复杂 | ✅ | ⚠️ 复杂 |
| Schema 支持 | ✅ Arrow Schema | ❌ | ✅ | ❌ |

### 2.4 Rust 生态集成

| 维度 | Lance | Faiss | usearch | Hnswlib |
|------|-------|-------|---------|---------|
| Crate | `lance` | `faiss` | `usearch` | `hnswlib` |
| 维护状态 | 活跃 | 稳定 | 活跃 | 稳定 |
| 文档质量 | 优秀 | 良好 | 良好 | 中等 |
| 编译复杂度 | 简单（纯 Rust） | 复杂（C++ 依赖） | 中等 | 中等 |

---

## 3. SYNTON-DB 需求分析

### 3.1 核心需求

1. **向量 + 元数据一体存储**
   - 节点向量与节点属性需要紧密关联
   - 图遍历时需要快速过滤元数据

2. **图结构友好**
   - 边的向量表示（关系向量）
   - 多跳查询的向量操作

3. **Rust 原生优先**
   - 减少 unsafe FFI 边界
   - 便于编译和部署

4. **可扩展性**
   - 支持大规模向量（百万级以上）
   - 支持增量索引更新

### 3.2 需求匹配度

| 需求 | Lance | Faiss | usearch | Hnswlib |
|------|-------|-------|---------|---------|
| Rust 原生 | ✅ | ❌ FFI | ⚠️ FFI | ❌ FFI |
| 元数据存储 | ✅ 内置 | ❌ 需外部 | ✅ | ❌ |
| 磁盘索引 | ✅ DiskANN | ⚠️ 有限 | ✅ | ❌ |
| 图遍历友好 | ✅ Schema 灵活 | ⚠️ 需适配 | ⚠️ 需适配 | ❌ |

---

## 4. 最终决策

### 推荐：**Lance**

### 决策理由

1. **Rust 原生是关键优势**
   - SYNTON-DB 核心用 Rust 开发
   - 消除 FFI 边界，减少内存安全问题
   - 简化编译和部署（无需 C++ 工具链）

2. **内置元数据存储**
   - 基于 Apache Arrow/Parquet
   - 元数据与向量紧密耦合
   - 支持复杂过滤查询

3. **DiskANN 支持**
   - 大规模向量可部分驻留磁盘
   - 内存占用可控

4. **Schema 设计灵活性**
   - Arrow Schema 支持复杂数据类型
   - 易于表示节点、边等图结构

### 风险与缓解

| 风险 | 缓解措施 |
|------|----------|
| 项目相对较新 | 核心算法成熟，LanceDB 生产验证 |
| 社区小于 Faiss | Rust 生态活跃，Apache 基金会支持 |
| 极限性能 | 热路径可选择性用 Faiss |

---

## 5. Lance 数据模型设计

### 5.1 节点表设计

```rust
use arrow::array::{StringArray, Float32Array};
use arrow::datatypes::{Schema, Field, DataType};
use lance::LanceError;

// 节点 Schema
let schema = Schema::new(vec![
    Field::new("id", DataType::Utf8, false),           // UUID
    Field::new("content", DataType::Utf8, false),       // 文本内容
    Field::new("embedding", DataType::FixedSizeList(1536), false),  // 向量
    Field::new("node_type", DataType::Utf8, false),     // Entity/Concept/Fact
    Field::new("created_at", DataType::Timestamp(TimeUnit::Milli, None), false),
    Field::new("access_score", DataType::Float32, false),
    Field::new("confidence", DataType::Float32, false),
]);
```

### 5.2 边表设计

```rust
// 边 Schema
let edge_schema = Schema::new(vec![
    Field::new("source_id", DataType::Utf8, false),
    Field::new("target_id", DataType::Utf8, false),
    Field::new("relation", DataType::Utf8, false),      // 关系类型
    Field::new("weight", DataType::Float32, false),
    Field::new("vector", DataType::FixedSizeList(768), false),  // 关系向量
    Field::new("created_at", DataType::Timestamp(TimeUnit::Milli, None), false),
]);
```

---

## 6. 查询性能优化策略

### 6.1 混合查询

```rust
// 向量检索 + 元数据过滤
async fn hybrid_search(
    &self,
    query_vector: &[f32],
    node_type_filter: &str,
    min_confidence: f32,
    limit: usize,
) -> Result<Vec<Node>> {
    let dataset = lance::Dataset::open(&self.db_uri)?;
    let builder = dataset.scan()
        .filter(lance::format!("node_type = '{}' AND confidence >= {}", node_type_filter, min_confidence))
        .nearest(&query_vector)?
        .limit(limit);

    let results = builder.execute().await?;
    // ...
}
```

### 6.2 图遍历加速

- 预计算常用路径的向量表示
- 利用 Lance 的索引能力缓存热点节点

---

## 7. 备选方案：Faiss

### 适用场景

1. **极限性能要求**
   - QPS > 10000 的高并发场景
   - 需要极致的内存效率（PQ/SQ 压缩）

2. **特定算法需求**
   - 需要最新的量化算法
   - 需要 GPU 加速

### 混合策略

```
┌─────────────────┐      ┌──────────────────┐
│   Lance         │      │   Faiss          │
│   (元数据)      │ +    │   (纯向量索引)    │
│                 │      │                  │
└─────────────────┘      └──────────────────┘
        │                        │
        └──────────┬─────────────┘
                   ▼
         SYNTON-DB 向量层
```

---

## 8. 参考资料

- [Lance 官方文档](https://lancedb.github.io/lance/)
- [Lance Rust Crate](https://docs.rs/lance/)
- [Faiss GitHub](https://github.com/facebookresearch/faiss)
- [Vector Database Comparison](https://github.com/erikbern/ann-benchmarks)
