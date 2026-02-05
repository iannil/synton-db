# SYNTON-DB

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE)

[English](README.md)

---

## 项目概述

SYNTON-DB 是一个专门为大语言模型设计的记忆数据库，通过结合知识图谱和向量检索，提供语义关联、逻辑推理和动态记忆能力。

与传统数据库（SQL、NoSQL、Vector）专注于 CRUD 操作不同，SYNTON-DB 基于三大核心原则构建：

- 入库即理解 - 自动从输入中提取知识图谱
- 查询即推理 - 混合向量相似度 + 图遍历
- 输出即上下文 - 返回预处理的上下文包，而非原始数据

### 解决什么问题？

传统数据库存储和检索数据但缺乏语义理解。SYNTON-DB：

1. 理解实体之间的关系，而不仅是内容相似度
2. 通过记忆衰减和强化维持时间上下文
3. 通过图遍历进行多跳推理
4. 为 LLM 优化合成上下文

### 核心差异

| 特性 | 传统数据库 | SYNTON-DB |
| ------ | ----------- | ----------- |
| 存储 | 表/文档/向量 | 张量图（带向量的节点 + 带关系的边） |
| 查询 | SQL/向量搜索 | PaQL（提示即查询语言） |
| 检索 | 基于相似度 | Graph-RAG（向量 + 图遍历） |
| 记忆 | 静态 | 动态（基于访问的衰减/强化） |
| 输出 | 原始行/列 | 合成的上下文包 |

---

## 核心特性

### 张量图存储（Tensor-Graph）

- 节点包含内容与可选向量嵌入
- 边代表逻辑关系（is_a、causes、contradicts 等）
- 支持 4 种节点类型：`entity`（实体）、`concept`（概念）、`fact`（事实）、`raw_chunk`（原始片段）
- 支持 7 种关系类型：`is_a`（是）、`is_part_of`（属于）、`causes`（导致）、`similar_to`（相似）、`contradicts`（矛盾）、`happened_after`（发生于）、`belongs_to`（归属于）

### Graph-RAG 混合检索

- 结合向量相似度搜索与多跳图遍历
- 可配置向量与图评分的权重
- 返回带置信度分数的排序结果
- 可配置遍历深度和结果限制

### PaQL（提示即查询语言）

- 自然语言查询解析器
- 支持逻辑运算符（AND、OR、NOT）
- 支持过滤器和图遍历查询
- 为 LLM 生成的查询优化

### 记忆衰减机制

- 艾宾浩斯遗忘曲线实现
- 基于访问分数的保留策略（0.0-10.0 分）
- 周期性衰减计算
- 可配置的保留阈值

### ML 嵌入服务

- 多后端支持：本地（Candle）、OpenAI、Ollama
- 嵌入缓存提升性能
- 可配置模型选择
- 支持 CPU/GPU 设备

### 双协议 API

- REST API（端口 8080）- 基于 HTTP 的 JSON
- gRPC API（端口 50051）- 高性能二进制协议
- 为 Web 客户端启用 CORS

---

## 快速开始

### Docker Compose（推荐）

```bash
# 克隆仓库
git clone https://github.com/synton-db/synton-db.git
cd synton-db

# 启动所有服务（数据库 + 监控）
docker-compose up -d

# 检查服务状态
docker-compose ps

# 查看日志
docker-compose logs -f synton-db
```

暴露的服务：

- `8080` - REST API
- `50051` - gRPC API
- `9090` - Prometheus 指标
- `3000` - Grafana 仪表板

### 从源码构建

```bash
# 前置要求：Rust 1.75+、Git

# 构建服务器
cargo build --release -p synton-db-server

# 构建 CLI 工具
cargo build --release -p synton-cli

# 运行服务器
./target/release/synton-db-server --config config.toml
```

### 验证

```bash
# 健康检查
curl http://localhost:8080/health

# 获取统计信息
curl http://localhost:8080/stats
```

---

## CLI 使用

`synton-cli` 工具提供全面的命令行界面。

### 连接选项

```bash
synton-cli --host <主机> --port <端口> --format <text|json> [命令]
```

### 节点操作

```bash
# 创建节点
synton-cli node create "巴黎是法国的首都" --node-type fact

# 通过 ID 获取节点
synton-cli node get <uuid>

# 删除节点（带确认）
synton-cli node delete <uuid>

# 强制删除（跳过确认）
synton-cli node delete <uuid> --force

# 列出所有节点
synton-cli node list --limit 100
```

### 边操作

```bash
# 在节点间创建边
synton-cli edge create <源节点ID> <目标节点ID> --relation is_part_of --weight 0.9

# 列出节点的边
synton-cli edge list <节点ID> --limit 100
```

### 查询操作

```bash
# 执行 PaQL 查询
synton-cli query execute "首都城市" --limit 10
```

### 系统操作

```bash
# 获取数据库统计
synton-cli stats

# 获取详细统计
synton-cli stats --detailed

# 导出数据为 JSON
synton-cli export --format json --output backup.json

# 从 JSON 导入数据
synton-cli import --format json --input backup.json

# 导入时遇到错误继续
synton-cli import --format json --input backup.json --continue-on-error
```

---

## API 端点

### REST API（端口 8080）

| 端点 | 方法 | 描述 |
| ------ | ------ | ------ |
| `/health` | GET | 健康检查 |
| `/stats` | GET | 数据库统计 |
| `/nodes` | GET | 列出所有节点 |
| `/nodes` | POST | 创建新节点 |
| `/nodes/:id` | GET | 按 ID 获取节点 |
| `/nodes/:id` | DELETE | 按 ID 删除节点 |
| `/edges` | POST | 创建新边 |
| `/query` | POST | 执行 PaQL 查询 |
| `/traverse` | POST | 图遍历 |
| `/bulk` | POST | 批量操作 |

#### 请求/响应示例

健康检查

```bash
curl http://localhost:8080/health
```

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_secs": 0
}
```

创建节点

```bash
curl -X POST http://localhost:8080/nodes \
  -H "Content-Type: application/json" \
  -d '{
    "content": "巴黎是法国的首都",
    "node_type": "fact"
  }'
```

```json
{
  "node": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "content": "巴黎是法国的首都",
    "node_type": "fact",
    "embedding": null,
    "meta": {
      "created_at": "2025-02-05T10:00:00Z",
      "access_score": 5.0
    }
  },
  "created": true
}
```

执行查询

```bash
curl -X POST http://localhost:8080/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": "首都",
    "limit": 10,
    "include_metadata": false
  }'
```

```json
{
  "nodes": [...],
  "total_count": 5,
  "execution_time_ms": 12,
  "truncated": false
}
```

创建边

```bash
curl -X POST http://localhost:8080/edges \
  -H "Content-Type: application/json" \
  -d '{
    "source": "<uuid-1>",
    "target": "<uuid-2>",
    "relation": "is_part_of",
    "weight": 0.9
  }'
```

批量操作

```bash
curl -X POST http://localhost:8080/bulk \
  -H "Content-Type: application/json" \
  -d '{
    "nodes": [
      {"content": "节点1", "node_type": "entity"},
      {"content": "节点2", "node_type": "concept"}
    ],
    "edges": []
  }'
```

### gRPC API（端口 50051）

gRPC API 提供相同功能，在高吞吐量场景下性能更佳。请参阅 `crates/api/src/grpc.rs` 了解 Protocol Buffers 定义。

---

## 项目结构

```text
synton-db/
├── crates/
│   ├── bin/          # 服务器二进制 ✅
│   ├── cli/          # 命令行工具 ✅
│   ├── core/         # 核心类型（Node、Edge、Relation）✅
│   ├── storage/      # RocksDB + Lance 存储 ✅
│   ├── vector/       # 向量索引 ✅
│   ├── graph/        # 图遍历算法 ✅
│   ├── graphrag/     # 混合搜索实现 ✅
│   ├── paql/         # 查询语言解析器 ✅
│   ├── memory/       # 记忆衰减管理 ✅
│   ├── ml/           # ML 嵌入服务 ✅
│   └── api/          # REST + gRPC API 层 ✅
├── e2e/              # 端到端测试 ✅
├── release/          # 发布文件
│   └── docker/       # Docker 配置文件
├── docs/             # 文档
│   ├── progress/     # 进行中的文档
│   └── reports/      # 已完成的报告
├── docker-compose.yml
├── Dockerfile
└── Cargo.toml
```

### 架构概览

```text
┌─────────────────────────────────────────────────────────────┐
│                      接口层                                  │
│  ┌──────────────────┐        ┌──────────────────┐          │
│  │   REST API       │        │    gRPC API      │          │
│  │   (Axum)         │        │    (Tonic)       │          │
│  └──────────────────┘        └──────────────────┘          │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                    认知计算层                               │
│  ┌──────────────────┐  ┌─────────────┐  ┌───────────────┐  │
│  │     PaQL         │  │  Graph-RAG  │  │  记忆管理     │  │
│  │   解析器         │  │   搜索      │  │   (衰减)      │  │
│  └──────────────────┘  └─────────────┘  └───────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                 张量图存储层                                │
│  ┌──────────────────┐        ┌──────────────────┐          │
│  │    RocksDB       │        │     Lance        │          │
│  │  (图存储)        │        │  (向量存储)      │          │
│  └──────────────────┘        └──────────────────┘          │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                    基础设施层                               │
│                   Rust + Tokio 运行时                       │
└─────────────────────────────────────────────────────────────┘
```

---

## 配置

### 配置文件

创建 `config.toml` 文件或使用 `release/docker/config.toml` 中的默认配置：

```toml
[server]
# 绑定地址
host = "0.0.0.0"

# gRPC 服务器端口
grpc_port = 50051

# REST API 服务器端口
rest_port = 8080

# 启用/禁用服务器
grpc_enabled = true
rest_enabled = true

[storage]
# RocksDB 数据目录
rocksdb_path = "./data/rocksdb"

# Lance 数据目录
lance_path = "./data/lance"

# RocksDB 最大打开文件数
max_open_files = 5000

# RocksDB 缓存大小（MB）
cache_size_mb = 256

# 启用预写日志
wal_enabled = true

[memory]
# 遗忘曲线衰减尺度（天）
decay_scale = 20.0

# 保留阈值（0.0-1.0）
retention_threshold = 0.1

# 新节点初始访问分数
initial_access_score = 5.0

# 每次访问的分数提升
access_boost = 0.5

# 启用周期性衰减计算
periodic_decay_enabled = false

# 衰减计算间隔（秒）
decay_interval_secs = 3600

[logging]
# 日志级别：trace、debug、info、warn、error
level = "info"

# 启用 JSON 格式日志
json_format = false

# 启用追踪输出
tracing_enabled = true

[graphrag]
# 图遍历最大深度
max_traversal_depth = 3

# 混合搜索返回的最大节点数
max_results = 10

# 向量相似度权重（0.0-1.0）
vector_weight = 0.7

# 图邻近度权重（0.0-1.0）
graph_weight = 0.3

# 启用置信度评分
confidence_scoring = true

[ml]
# 启用 ML 功能
enabled = true

# 后端类型：local、openai、ollama
backend = "local"

# 本地模型配置
local_model = "sentence-transformers/all-MiniLM-L6-v2"
device = "cpu"
max_length = 512

# API 配置（用于 openai/ollama 后端）
api_endpoint = "https://api.openai.com/v1"
api_model = "text-embedding-3-small"
timeout_secs = 30

# 嵌入缓存
cache_enabled = true
cache_size = 10000
```

### 环境变量

可通过环境变量覆盖配置：

| 变量 | 描述 | 默认值 |
| ------ | ------ | -------- |
| `SYNTON_SERVER_HOST` | 服务器绑定地址 | `0.0.0.0` |
| `SYNTON_SERVER_GRPC_PORT` | gRPC 端口 | `50051` |
| `SYNTON_SERVER_REST_PORT` | REST API 端口 | `8080` |
| `SYNTON_STORAGE_ROCKSDB_PATH` | RocksDB 数据路径 | `./data/rocksdb` |
| `SYNTON_STORAGE_LANCE_PATH` | Lance 数据路径 | `./data/lance` |
| `SYNTON_LOG_LEVEL` | 日志级别 | `info` |

---

## 开发

### 前置要求

- Rust 1.75+
- Node.js 18+（用于 E2E 测试）
- Docker & Docker Compose（用于容器化测试）

### 运行测试

```bash
# 单元测试
cargo test

# 单元测试（带输出）
cargo test -- --nocapture

# 运行特定测试
cargo test test_add_node

# E2E 测试
cd e2e
npm install
npx playwright install
npm test

# E2E 测试（可见浏览器）
npm run test:headed

# E2E 测试报告
npm run test:report
```

### 代码质量

```bash
# 格式化代码
cargo fmt

# 检查格式
cargo fmt --check

# Clint 检查
cargo clippy

# 将警告视为错误
cargo clippy -- -D warnings

# 生成文档
cargo doc --open

# 生成所有 crate 的文档
cargo doc --document-private-items --open
```

### 构建

```bash
# Debug 构建
cargo build

# Release 构建（优化）
cargo build --release

# 构建特定 crate
cargo build -p synton-db-server

# 使用特性构建
cargo build --features all
```

### Docker 开发

```bash
# 构建 Docker 镜像
docker build -t synton-db:dev .

# 运行容器
docker run -p 8080:8080 -p 50051:50051 synton-db:dev

# 使用自定义配置运行
docker run -v $(pwd)/config.toml:/etc/synton-db/config.toml synton-db:dev
```

---

## 设计理念

> 传统数据库的核心是 CRUD，追求 ACID 或 CAP。
> 认知数据库的核心是：感知、关联、回忆和进化。

### 入库即理解

传统数据库按原样存储数据。SYNTON-DB 自动：

- 提取实体和关系
- 构建知识图谱
- 创建语义嵌入
- 建立时间上下文

### 查询即推理

传统数据库匹配模式。SYNTON-DB：

- 结合向量相似度与图遍历
- 通过连接节点跟踪逻辑链
- 按置信度和相关性加权结果
- 返回上下文相关的信息

### 输出即上下文

传统数据库返回原始行。SYNTON-DB：

- 合成相关信息
- 压缩和优先排序上下文
- 为 LLM 消费格式化输出
- 维护出处和置信度

---

## 路线图

### 已完成 ✅

- [x] 核心数据模型（Node、Edge、Relation）
- [x] 存储层（RocksDB + Lance 后端）
- [x] 向量索引（Lance 集成）
- [x] 图遍历（BFS/DFS 算法）
- [x] Graph-RAG 混合检索
- [x] PaQL 查询解析器
- [x] 记忆衰减管理
- [x] REST + gRPC 双 API
- [x] 全功能 CLI 工具
- [x] Docker 部署
- [x] E2E 测试套件
- [x] Prometheus + Grafana 监控
- [x] 配置管理
- [x] ML 嵌入服务（本地/OpenAI/Ollama）

### 进行中 🚧

- [ ] 高级 PaQL 语法特性
- [ ] 查询缓存层

### 计划中 📋

- [ ] WebUI 控制台
- [ ] 备份/恢复工具
- [ ] 访问控制和身份验证
- [ ] 分布式存储支持
- [ ] 高级告警系统

---

## 贡献

欢迎贡献！请遵循以下准则：

1. 代码风格：遵循 Rust 约定并使用 `cargo fmt`
2. 测试：为新功能编写测试（目标覆盖率 80%）
3. 提交：使用约定式提交格式（`feat:`、`fix:`、`docs:` 等）
4. 文档：更新相关文档以反映变更
5. PR：提供清晰的描述并链接相关 issue

### 开发流程

```bash
# 1. Fork 并克隆仓库
git clone https://github.com/synton-db/synton-db.git

# 2. 创建功能分支
git checkout -b feat/your-feature

# 3. 进行更改并测试
cargo test
cargo clippy

# 4. 使用约定格式提交
git commit -m "feat: 添加功能描述"

# 5. 推送并创建 PR
git push origin feat/your-feature
```

---

## 许可证

Apache License 2.0

---

## 链接

- 仓库：[https://github.com/synton-db/synton-db](https://github.com/synton-db/synton-db)
- 文档：[docs/](./docs/)
- 问题：[https://github.com/synton-db/synton-db/issues](https://github.com/synton-db/synton-db/issues)
- 讨论：[https://github.com/synton-db/synton-db/discussions](https://github.com/synton-db/synton-db/discussions)
