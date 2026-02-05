# SYNTON-DB 技术栈最终决策报告

**报告时间**: 2025-02-05
**报告状态**: 最终决策

---

## 1. 执行摘要

经过详细选型分析，SYNTON-DB 采用 **Rust 原生优先**的技术栈策略，在保证性能的同时最大化部署简化程度。

### 最终技术栈

| 层级 | 技术选择 | 理由 |
|------|----------|------|
| 核心语言 | **Rust** | 内存安全、高性能、数据库内核标准 |
| 向量存储 | **Lance** | Rust 原生、内置元数据、DiskANN |
| KV 存储 | **RocksDB** | 列族支持、写密集优化、成熟生态 |
| ML 推理 | **Candle** | Rust 原生、HuggingFace 集成、静态链接 |
| 网络协议 | **gRPC + REST** | 内部高效通讯 + 外部兼容 |

---

## 2. 决策矩阵

### 2.1 一致性原则

| 决策维度 | 选择 | 说明 |
|----------|------|------|
| Rust 原生优先 | ✅ | Lance + Candle 组合，减少 FFI 边界 |
| 部署简化 | ✅ | 静态链接，单一 Docker 镜像 |
| 生态成熟度 | ✅ | RocksDB + Lance 为组合 |
| 可扩展性 | ✅ | 模块化设计，可选择性替换组件 |

### 2.2 技术栈关联图

```
┌─────────────────────────────────────────────────────────────┐
│                      SYNTON-DB                              │
├─────────────────────────────────────────────────────────────┤
│  Interface Layer                                            │
│  ┌─────────────┐  ┌─────────────┐                          │
│  │   gRPC      │  │   REST      │                          │
│  │  (tonic)    │  │  (Axum)     │                          │
│  └─────────────┘  └─────────────┘                          │
├─────────────────────────────────────────────────────────────┤
│  Cognitive Compute Layer                                    │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  PaQL Parser  │  Reranker  │  Context Synthesizer   │   │
│  │    (Nom)      │  (Candle)  │       (Custom)         │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│  Tensor-Graph Storage Layer                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │    Lance     │  │   RocksDB    │  │   Graph      │     │
│  │  向量+元数据  │  │   KV 存储    │  │   遍历引擎    │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
├─────────────────────────────────────────────────────────────┤
│  ML Inference Layer                                         │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Embedding Model  │  Cross-Encoder  │  NLP SLM      │   │
│  │    (Candle)      │     (Candle)    │   (Candle)    │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│  Infrastructure Layer                                       │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Tokio Runtime  │  MMAP  │  NVMe Optimizations     │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. 分层技术栈详解

### 3.1 基础设施层

| 组件 | 技术 | 说明 |
|------|------|------|
| 异步运行时 | **Tokio** | Rust 标准异步运行时 |
| 内存映射 | **mmap / memmap2** | 零拷贝文件访问 |
| 错误处理 | **anyhow / thiserror** | 统一错误类型 |
| 日志 | **tracing** | 结构化日志 |
| 序列化 | **serde** | 数据序列化标准 |

### 3.2 存储层

| 组件 | 技术 | 说明 |
|------|------|------|
| 向量存储 | **Lance** | 主向量 + 元数据 |
| KV 存储 | **RocksDB** | 列族：nodes, edges, metadata, index |
| 图存储 | **自研 CSR** | 压缩稀疏行格式 |

### 3.3 认知计算层

| 组件 | 技术 | 说明 |
|------|------|------|
| 查询解析 | **Nom** | 组合子解析器 |
| 嵌入推理 | **Candle** | BERT / sentence-transformers |
| 重排序 | **Candle** | Cross-Encoder 模型 |
| 上下文合成 | **自研** | 模板 + 规则引擎 |

### 3.4 接口层

| 组件 | 技术 | 说明 |
|------|------|------|
| gRPC 框架 | **tonic + prost** | 内部服务通讯 |
| REST 框架 | **Axum** | 对外 API |
| WebSocket | **tokio-tungstenite** | 实时推送 |
| OpenAPI | **utoipa** | API 文档生成 |

---

## 4. 依赖声明 (Cargo.toml)

```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["SYNTON-DB Team"]
license = "Apache-2.0"

[workspace.dependencies]
# 基础设施
tokio = { version = "1.40", features = ["full"] }
async-trait = "0.1"
anyhow = "1.0"
thiserror = "2.0"
tracing = "0.1"
tracing-subscriber = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.10", features = ["v4", "serde"] }

# 存储
lance = "0.12"
rocksdb = "0.22"

# ML 推理
candle = { version = "0.6", features = ["--", "mkl"] }
candle-transformers = "0.6"
candle-nn = "0.6"
tokenizers = "0.20"
hf-hub = "0.3"

# 网络协议
tonic = "0.12"
prost = "0.13"
axum = "0.7"
tower = "0.5"

# 解析器
nom = "7.1"
```

---

## 5. 部署架构

### 5.1 Docker 镜像

```dockerfile
# Dockerfile
FROM rust:1.83 as builder

WORKDIR /app
COPY . .

# 静态链接构建
RUN cargo build --release --static

# 运行时镜像
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/synton-db /usr/local/bin/

EXPOSE 50051 8080
CMD ["synton-db"]
```

### 5.2 网络配置

```yaml
# docker-compose.yml
version: '3.8'

services:
  synton-db:
    build: .
    network_mode: synton-network
    ports:
      - "50051:50051"  # gRPC
      - "8080:8080"    # REST
    volumes:
      - ./data:/data
    environment:
      - RUST_LOG=info
      - LANCE_DATA_PATH=/data/lance
      - ROCKSDB_PATH=/data/rocksdb

networks:
  synton-network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.28.0.0/16
```

---

## 6. 迁移路径

### 6.1 MVP0: 基础验证

| 组件 | MVP0 选择 | 后续迁移 |
|------|-----------|----------|
| 向量存储 | Lance | - |
| KV 存储 | RocksDB | - |
| ML 推理 | Candle | 可选 ONNX Runtime |

### 6.2 潜在替换路径

```
┌────────────────────────────────────────────────────────┐
│                    MVP0 基础                           │
│  Lance + RocksDB + Candle                              │
└────────────────────────────────────────────────────────┘
                          │
                          ▼
┌────────────────────────────────────────────────────────┐
│                   生产优化阶段                          │
│  │                                                       │
│  ├─ 超大规模 → Lance + Faiss 混合                       │
│  ├─ 读密集场景 → 切换部分列族到 LMDB                    │
│  └─ GPU 推理 → Candle GPU / ONNX Runtime GPU           │
└────────────────────────────────────────────────────────┘
```

---

## 7. 风险评估与缓解

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| Lance 性能不足 | 高 | 中 | 基准测试验证，预留 Faiss 替换路径 |
| Candle 模型支持不全 | 中 | 低 | 主流模型已支持，可自定义实现 |
| RocksDB 写放大 | 中 | 高 | 配置调优，使用合适压缩策略 |
| Rust 编译时间长 | 低 | 高 | 使用 cargo check, sccache |

---

## 8. 参考资料

- [Rust 官方文档](https://www.rust-lang.org/)
- [Lance 文档](https://lancedb.github.io/lance/)
- [RocksDB 文档](https://rocksdb.org/)
- [Candle 文档](https://github.com/huggingface/candle)
- [tonic gRPC](https://github.com/hyperium/tonic)
