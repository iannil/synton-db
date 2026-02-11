# SYNTON-DB 项目状态

**最后更新**: 2026-02-11
**当前阶段**: 生产就绪 - 功能完整，测试覆盖完善

---

## 项目概述

SYNTON-DB 是一个**认知数据库**（Cognitive Database），专为 LLM 设计的外挂大脑/海马体。与传统数据库不同，SYNTON-DB 专注于记忆、推理和关联，而非简单的 CRUD 操作。

### 核心特性

1. **入库即理解** - 自动知识图谱提取
2. **查询即推理** - 混合向量搜索 + 图遍历
3. **输出即上下文** - 为 LLM 提供预处理上下文
4. **自适应分块** - 语义感知的文档分割
5. **记忆衰退机制** - 基于遗忘曲线的数据管理

---

## 当前状态

### 已完成功能

| 层级 | 功能 | 状态 | 说明 |
|------|------|------|------|
| 存储 | RocksDB + Lance | ✅ | KV 存储 + 向量索引 |
| 图引擎 | 张量图 (Node + Edge) | ✅ | 支持多跳遍历 |
| 检索 | Graph-RAG | ✅ | 混合检索 (向量 + 图) |
| 查询 | PaQL 解析器 | ✅ | 自然语言查询 |
| 记忆 | 遗忘曲线 | ✅ | 基于访问频率 |
| API | REST + gRPC | ✅ | 双协议支持 |
| 文档 | OpenAPI/Swagger | ✅ | 自动生成文档 |
| 测试 | 342+ 测试 | ✅ | 覆盖完善 |

### 测试覆盖

```
总计: 342+ tests passed
├── 单元测试: 188 tests
├── Candle ML: 37 tests
├── 集成测试: 19 tests
├── 自适应分块: 19 tests
├── Lance 索引: 46 tests
├── 上下文合成: 30 tests
└── 其他: 3+ tests
```

---

## 项目结构

### Crates (12 个子项目)

| Crate | 职责 | 测试 |
|-------|------|------|
| `synton-bin` | CLI 入口 | ✅ |
| `synton-api` | REST API 服务 | ✅ |
| `synton-storage` | RocksDB 封装 | ✅ |
| `synton-vector` | 向量索引 (Lance) | ✅ |
| `synton-graph` | 张量图引擎 | ✅ |
| `synton-ml` | Candle ML 集成 | ✅ |
| `synton-memory` | 记忆管理 | ✅ |
| `synton-chunking` | 自适应分块 | ✅ |
| `synton-graphrag` | Graph-RAG 检索 | ✅ |
| `synton-mcp-server` | MCP 协议支持 | ✅ |
| `synton-expansion` | 查询扩展 | ✅ |
| `synton-formatter` | 上下文合成 | ✅ |

### 前端

| 组件 | 状态 |
|------|------|
| Web UI | ✅ 完整 |
| 配置 | `web/package.json` |

---

## 技术栈

| 组件 | 技术 | 版本 |
|------|------|------|
| 语言 | Rust | stable |
| KV 存储 | RocksDB | latest |
| 向量索引 | Lance | 0.12.0 |
| ML 框架 | Candle | 0.9.2 |
| Web 框架 | Axum | latest |
| 序列化 | Serde | latest |
| 异步运行时 | Tokio | latest |

---

## 下一步计划

### 短期 (P0-P1)

- [ ] **性能基准测试** - Criterion 基准框架
- [ ] **并发压力测试** - 负载测试
- [ ] **分页支持** - 查询结果分页

### 中期 (P1-P2)

- [ ] **Swagger UI** - 可视化 API 文档界面
- [ ] **Docker 优化** - 多阶段构建、镜像压缩
- [ ] **监控与指标** - Prometheus 集成

### 长期 (P2-P3)

- [ ] **分布式存储** - 多节点集群
- [ ] **多租户隔离** - 命名空间支持
- [ ] **流式处理** - 实时数据摄入

---

## 技术债务

| 项 | 优先级 | 说明 |
|----|--------|------|
| 编译警告 | P2 | 部分 crate 存在未使用导入 |
| 错误处理 | P1 | 统一错误类型设计 |
| 配置管理 | P1 | 多环境配置支持 |
| 日志规范 | P2 | 统一日志格式与级别 |

---

## 部署

### 本地开发

```bash
cargo build --workspace
cargo run --bin synton-db
```

### Docker 部署

```bash
docker-compose up -d
```

### 配置文件

- 开发环境: `/config.toml`
- 生产环境: `/release/docker/config.toml`
- 测试环境: `/e2e/test-config.toml`

---

## 文档索引

- **[README.md](README.md)** - 文档导航
- **[RUNBOOK.md](RUNBOOK.md)** - 运维手册
- **[CONTRIB.md](CONTRIB.md)** - 贡献指南
- **[architecture/](architecture/)** - 架构设计文档
- **[reports/completed/](reports/completed/)** - 已完成阶段报告

---

## 发布

| 版本 | 日期 | 说明 |
|------|------|------|
| v0.1.0 | 2025-02-05 | MVP 完成 |
| v0.2.0 | 2026-02-10 | Phase 0-3 完成 |
| v0.3.0 | 2026-02-11 | Phase 4 + Web UI 集成 |

---

*最后更新: 2026-02-11*
