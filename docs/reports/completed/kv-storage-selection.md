# KV 存储选型报告

**报告时间**: 2025-02-05
**决策状态**: 已完成
**推荐方案**: RocksDB

---

## 1. 候选技术

| 技术 | 语言 | 许可证 | 项目状态 |
|------|------|--------|----------|
| RocksDB | C++ | GPL 2.0 / BSD 3-clause | Facebook/Meta 开源，活跃维护 |
| LMDB | C | OpenLDAP Public License | 特定用途维护，较稳定 |
| sled | Rust | MIT / Apache-2.0 | 纯 Rust 实现，维护较少 |
| Redb | Rust | MIT | 新兴纯 Rust 实现，活跃 |

---

## 2. 详细对比

### 2.1 架构设计

| 维度 | RocksDB | LMDB | sled | Redb |
|------|---------|------|------|------|
| 存储结构 | LSM Tree (Log Structured Merge Tree) | B+ Tree 内存映射 | LSM Tree (纯 Rust) | B+ Tree (纯 Rust) |
| 写入模式 | 批量写入优化，写放大高 | 直接写入，写放大低 | 批量写入优化 | 直接写入 |
| 读取性能 | 需要压缩可能需多次查找 | 内存映射，直接访问 | 类 RocksDB | 内存映射友好 |

### 2.2 性能特征

| 场景 | RocksDB | LMDB | sled | Redb |
|------|---------|------|------|------|
| 顺序写入 | ⭐⭐⭐⭐⭐ 极高 | ⭐⭐⭐ 中等 | ⭐⭐⭐⭐ 高 | ⭐⭐⭐⭐ 高 |
| 随机写入 | ⭐⭐⭐⭐⭐ 极高 | ⭐⭐ 中低 | ⭐⭐⭐⭐ 高 | ⭐⭐⭐⭐ 高 |
| 读取性能 | ⭐⭐⭐ 中等（可能需压缩） | ⭐⭐⭐⭐⭐ 极高 | ⭐⭐⭐ 中等 | ⭐⭐⭐⭐⭐ 极高 |
| 内存占用 | 可配置，较低 | 需要完整索引在内存 | 可配置 | 较低 |

### 2.3 并发模型

| 维度 | RocksDB | LMDB | sled | Redb |
|------|---------|------|------|------|
| 写入者 | 单写者（无锁队列） | 多写入者（MVCC） | 单写入者 | 单写入者 |
| 读取者 | 多读取者 | 多读取者 | 多读取者 | 多读取者 |
| 一致性 | 最终一致 | ACID | ACID | ACID |

### 2.4 Rust 生态

| 维度 | RocksDB | LMDB | sled | Redb |
|------|---------|------|------|------|
| Crate | `rocksdb` | `heed` / `lmdb-rkv` | `sled` | `redb` |
| 绑定类型 | FFI (sys crate) | FFI | 纯 Rust | 纯 Rust |
| 维护状态 | 活跃 | 稳定 | 较少更新 | 活跃 |
| 文档质量 | 优秀（官方文档） | 中等 | 良好 | 良好 |

### 2.5 特性对比

| 特性 | RocksDB | LMDB | sled | Redb |
|------|---------|------|------|------|
| 列族 (Column Families) | ✅ 支持 | ❌ | ❌ | ❌ |
| 前缀 Bloom Filter | ✅ | ❌ | ✅ | ✅ |
| 事务支持 | ✅ (原子写入组) | ✅ (完整 ACID) | ✅ | ✅ |
| 备份/检查点 | ✅ | 需手动 | ✅ | ✅ |
| 增量备份 | ✅ | ❌ | ❌ | ❌ |
| SST 文件格式 | ✅ 可定制 | ❌ | ❌ | ❌ |

---

## 3. SYNTON-DB 需求分析

### 3.1 核心需求

1. **写密集型工作负载**
   - 大量数据摄入（文档、知识图谱节点）
   - 批量写入场景多

2. **灵活的数据组织**
   - 需要多种数据类型（节点、边、向量索引）
   - 列族可支持逻辑分离

3. **向量索引集成**
   - 需要与向量存储（Faiss/Lance）协同工作
   - RocksDB + Faiss 是成熟组合

4. **内存可控性**
   - 嵌入式部署场景
   - 内存占用可配置

### 3.2 需求匹配度

| 需求 | RocksDB | LMDB | sled | Redb |
|------|---------|------|------|------|
| 写入吞吐量 | ✅ 优秀 | ⚠️ 中等 | ✅ 良好 | ✅ 良好 |
| 列族支持 | ✅ 关键特性 | ❌ | ❌ | ❌ |
| 向量集成 | ✅ 成熟 | ⚠️ 需适配 | ⚠️ 未知 | ⚠️ 新项目 |
| 生产案例 | ✅ 广泛 | ✅ 有限 | ⚠️ 较少 | ❌ 无 |

---

## 4. 最终决策

### 推荐：**RocksDB**

### 决策理由

1. **列族支持是关键差异点**
   - SYNTON-DB 需要存储多种数据类型（节点、边、元数据）
   - 列族允许逻辑隔离，简化数据管理

2. **写入性能优势**
   - 认知数据库需要持续摄入新知识
   - LSM Tree 架构天然适合写密集场景

3. **成熟生态**
   - 广泛的生产验证（TiDB、Kafka、SQLite4 等）
   - Rust `rocksdb` crate 稳定可靠

4. **向量索引集成**
   - RocksDB + Faiss 组合经过大量验证
   - 可作为 Lance 的元数据补充存储

### 风险与缓解

| 风险 | 缓解措施 |
|------|----------|
| GPL 2.0 许可证 | 使用 BSD 3-clause 双重许可选项 |
| 写放大问题 | 配置合适的压缩策略，使用 SST 定制 |
| 非 Rust 原生 | 使用维护良好的 `rocksdb` crate |

---

## 5. 备选方案：LMDB

### 适用场景

如果 SYNTON-DB 演进为**读密集型**应用（如纯推理查询服务），可考虑：

- LMDB 的内存映射读取性能更优
- 完整的 ACID 事务支持
- 更低的写放大

### 迁移路径

可在 MVP 阶段使用 RocksDB，通过抽象层设计，未来可切换到 LMDB。

---

## 6. 配置建议

### 推荐配置（针对 SYNTON-DB）

```rust
use rocksdb::{Options, DB};

let mut opts = Options::default();
opts.create_if_missing(true);
opts.create_missing_column_families(true);

// 列族定义
let cf_names = vec!["nodes", "edges", "metadata", "vector_index"];

// 性能调优
opts.increase_parallelism(4);  // 并行度
opts.set_max_write_buffer_number(4);
opts.set_write_buffer_size(512 * 1024 * 1024);  // 512MB
opts.set_max_background_jobs(4);

// 压缩配置
opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
```

---

## 7. 参考资料

- [RocksDB 官方文档](https://rocksdb.org/)
- [rocksdb crate](https://docs.rs/rocksdb/)
- [LMDB Documentation](https://www.lmdb.tech/doc/)
- [heed: LMDB for Rust](https://docs.rs/heed/)
