# SYNTON-DB 集成阶段进度

**状态**: 进行中
**开始时间**: 2025-02-05
**阶段**: 集成与部署准备

---

## 背景

MVP 阶段（MVP0-MVP5）已全部完成，包括：
- ✅ 存储基础 (RocksDB + Lance)
- ✅ 张量图 (Node + Edge + Graph traversal)
- ✅ Graph-RAG (混合检索)
- ✅ PaQL (查询解析器)
- ✅ 记忆机制 (遗忘曲线)
- ✅ API 服务 (REST + gRPC)

现在进入集成阶段，需要将各模块组装成可运行的服务。

---

## 目标

1. 实现服务端主程序 (bin)
2. 准备 Docker 部署环境
3. 实现基础 CLI 工具
4. 补全缺失功能
5. 准备 E2E 测试

---

## 进展

### 待完成

- [ ] 服务端主程序实现
- [ ] Docker 部署配置
- [ ] CLI 工具实现
- [ ] 代码清理（编译警告、废弃标记）
- [ ] E2E 测试准备

---

## 详细任务

### 1. 服务端主程序实现

**文件**: `crates/bin/src/main.rs` (新建)

**功能**:
- 初始化所有组件（Store, Graph, VectorIndex, GraphRag, MemoryManager）
- 启动 gRPC 服务器
- 启动 REST 服务器
- 配置日志和追踪
- 优雅关闭处理

**配置**: `config.toml`
```toml
[server]
grpc_port = 50051
rest_port = 8080

[storage]
rocksdb_path = "./data/rocksdb"

[memory]
decay_scale = 20.0
retention_threshold = 0.1
```

### 2. Docker 部署配置

**文件清单**:
| 文件 | 说明 |
|------|------|
| `Dockerfile` | 服务端镜像 |
| `docker-compose.yml` | 完整部署配置 |
| `.dockerignore` | 排除文件 |
| `release/docker/entrypoint.sh` | 启动脚本 |

**Docker Compose 服务**:
- `synton-db` - 主服务
- `prometheus` - 监控
- `grafana` - 可视化

### 3. CLI 工具实现

**文件**: `crates/cli/src/main.rs`

**子命令**:
```bash
synton-cli node create <content>          # 创建节点
synton-cli edge create <from> <to>        # 创建边
synton-cli query execute "<query>"        # 执行查询
synton-cli export                         # 导出数据
synton-cli import <file>                  # 导入数据
```

### 4. 代码清理

#### 4.1 标记废弃 Crate

**文件**: `crates/query/Cargo.toml`
```toml
[package]
name = "synton-query"
deprecated = "Use synton-paql instead"
```

**文件**: `crates/query/src/lib.rs`
```rust
//! # synton-query
//!
//! ⚠️ **已废弃** - 此 crate 的功能已被 `synton-paql` 完全取代。
//!
//! 请迁移到 `synton-paql` crate。
```

#### 4.2 修复编译警告

| 警告 | 文件 | 修复方案 |
|------|------|----------|
| `PathBuilder` unused | `core/src/path.rs` | 添加 `#[allow(dead_code)]` 或移除 |
| `PathBuildError` unused | `core/src/path.rs` | 添加 `#[allow(dead_code)]` 或移除 |
| `RelatedTo` naming | `core/src/relation.rs` | 重命名为 `RELATED_TO` |

### 5. E2E 测试准备

**测试框架**: Playwright

**测试场景**:
1. 创建节点
2. 创建边
3. 执行查询
4. 验证 Graph-RAG 检索
5. 验证记忆衰减

**文件**: `tests/e2e/`

---

## 项目当前状态

### 已完成模块

| Crate | 状态 |
|-------|------|
| `core` | ✅ |
| `graph` | ✅ |
| `graphrag` | ✅ |
| `paql` | ✅ |
| `memory` | ✅ |
| `storage` | ✅ |
| `vector` | ✅ |
| `api` | ✅ |

### 占位符模块

| Crate | 状态 |
|-------|------|
| `cli` | ⏸️ 待实现 |
| `ml` | ⏸️ 待实现 |
| `query` | ❌ 已废弃 |

### 编译状态

```bash
✅ cargo check --workspace  # 通过（有少量警告）
```

---

## 遇到的问题

暂无

---

## 下一步计划

1. **立即**: 创建 `bin` crate 并实现主程序
2. **今天**: 完成 Docker 配置
3. **本周**: 完成 CLI 基础功能
4. **下周**: ML 集成 + E2E 测试

---

## 变更日志

### 2025-02-05
- 创建集成阶段进度文档
- MVP 阶段确认完成
- 开始集成准备
