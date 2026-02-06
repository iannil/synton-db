# SYNTON-DB 贡献指南

**更新时间**: 2026-02-06

---

## 概述

本文档面向 SYNTON-DB 项目的贡献者，涵盖开发工作流、可用脚本、环境配置和测试程序。

---

## 开发环境设置

### 前置要求

| 工具 | 最低版本 | 用途 |
|------|---------|------|
| Rust | 1.75+ | 核心开发语言 |
| Git | - | 版本控制 |
| Docker | 20.10+ | 容器化部署和测试 |
| Docker Compose | 2.0+ | 多容器编排 |
| Node.js | 18+ | E2E 测试 |
| Make | - | 构建自动化（可选） |

### 克隆仓库

```bash
git clone https://github.com/synton-db/synton-db.git
cd synton-db
```

### 安装依赖

```bash
# Rust 依赖（自动通过 Cargo 管理）
cargo fetch

# E2E 测试依赖
cd e2e
npm install
npx playwright install
cd ..
```

---

## 可用脚本

### Cargo 命令（核心）

| 命令 | 说明 |
|------|------|
| `cargo build` | Debug 构建 |
| `cargo build --release` | Release 优化构建 |
| `cargo check` | 快速类型检查（不构建二进制） |
| `cargo test` | 运行所有单元测试 |
| `cargo test -- --nocapture` | 运行测试并显示输出 |
| `cargo test <test_name>` | 运行特定测试 |
| `cargo clippy` | Lint 检查 |
| `cargo clippy -- -D warnings` | Lint 并将警告视为错误 |
| `cargo fmt` | 格式化代码 |
| `cargo fmt --check` | 检查代码格式 |
| `cargo doc --open` | 生成并打开文档 |
| `cargo run -p synton-db` | 运行服务器 |

### 工作区命令

| 命令 | 说明 |
|------|------|
| `cargo build --workspace` | 构建所有 crates |
| `cargo test --workspace` | 测试所有 crates |
| `cargo clean -p synton-db` | 清理特定 crate |
| `cargo tree` | 显示依赖树 |

### E2E 测试命令

在 `e2e/` 目录下：

| NPM 脚本 | 说明 |
|----------|------|
| `npm test` | 运行所有 E2E 测试 |
| `npm run test:headed` | 运行 E2E 测试（显示浏览器） |
| `npm run test:debug` | 调试模式运行 E2E 测试 |
| `npm run test:report` | 打开测试报告 |

### Docker 命令

| 命令 | 说明 |
|------|------|
| `docker build -t synton-db:dev .` | 构建开发镜像 |
| `docker-compose up -d` | 启动所有服务 |
| `docker-compose down` | 停止所有服务 |
| `docker-compose logs -f synton-db` | 查看服务日志 |
| `docker-compose ps` | 查看服务状态 |

---

## 项目结构

```
synton-db/
├── crates/
│   ├── bin/          # 服务器二进制
│   ├── cli/          # CLI 工具
│   ├── core/         # 核心类型定义
│   ├── storage/      # RocksDB + Lance 存储层
│   ├── vector/       # 向量索引
│   ├── graph/        # 图遍历算法
│   ├── graphrag/     # Graph-RAG 混合检索
│   ├── paql/         # 查询语言解析器
│   ├── memory/       # 记忆衰减管理
│   ├── ml/           # ML 嵌入服务
│   └── api/          # REST + gRPC API 层
├── e2e/              # E2E 测试 (Playwright)
├── release/          # 发布配置
│   └── docker/       # Docker 相关文件
├── docs/             # 项目文档
├── memory/           # 记忆系统文件
├── docker-compose.yml
├── Dockerfile
├── Cargo.toml        # 工作区配置
└── .env.example      # 环境变量模板
```

---

## 测试程序

### 单元测试

```bash
# 运行所有测试
cargo test --workspace

# 运行特定 crate 的测试
cargo test -p synton-core

# 运行特定模块的测试
cargo test test_add_node

# 带输出的测试
cargo test -- --nocapture

# 测试并发问题
cargo test -- --test-threads=1
```

### 集成测试

```bash
# 运行集成测试
cargo test --test '*'

# 运行特定集成测试
cargo test --test integration_test
```

### E2E 测试

```bash
# 启动服务（后台）
docker-compose up -d

# 等待服务就绪
sleep 10

# 运行 E2E 测试
cd e2e
npm test

# 清理
docker-compose down
```

### 测试覆盖率

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --workspace --out Html --output-dir ./coverage
```

---

## 环境变量配置

复制 `.env.example` 到 `.env` 并根据需要修改：

```bash
cp .env.example .env
```

### 关键环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `SYNTON_SERVER_HOST` | `0.0.0.0` | 服务绑定地址 |
| `SYNTON_SERVER_GRPC_PORT` | `50051` | gRPC 端口 |
| `SYNTON_SERVER_REST_PORT` | `8080` | REST API 端口 |
| `SYNTON_STORAGE_ROCKSDB_PATH` | `./data/rocksdb` | RocksDB 数据目录 |
| `SYNTON_STORAGE_LANCE_PATH` | `./data/lance` | Lance 数据目录 |
| `SYNTON_LOG_LEVEL` | `info` | 日志级别 |
| `SYNTON_ML_BACKEND` | `local` | ML 后端类型 |

详细配置请参考 `.env.example` 文件。

---

## 代码质量标准

### 格式化

提交前运行：

```bash
cargo fmt
```

CI 检查格式：

```bash
cargo fmt --check
```

### Lint

```bash
# 基本检查
cargo clippy

# 严格模式（将警告视为错误）
cargo clippy -- -D warnings
```

### 文档

```bash
# 生成文档
cargo doc --no-deps

# 检查文档链接
cargo doc --no-deps --document-private-items
```

---

## 提交规范

使用 Conventional Commits 格式：

```
<type>: <description>

[optional body]
```

| 类型 | 说明 |
|------|------|
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `refactor` | 代码重构 |
| `docs` | 文档更新 |
| `test` | 测试相关 |
| `chore` | 构建/工具链相关 |
| `perf` | 性能优化 |
| `ci` | CI 配置 |

### 示例

```bash
git commit -m "feat: add hybrid search support to GraphRAG"
git commit -m "fix: resolve memory leak in vector index"
git commit -m "docs: update API documentation for v0.2.0"
```

---

## Pull Request 工作流

1. **创建功能分支**

```bash
git checkout -b feat/your-feature
```

2. **开发和测试**

```bash
# 运行测试
cargo test --workspace
cd e2e && npm test

# 检查代码质量
cargo clippy -- -D warnings
cargo fmt --check
```

3. **提交更改**

```bash
git add .
git commit -m "feat: description of your changes"
```

4. **推送并创建 PR**

```bash
git push origin feat/your-feature
```

在 GitHub 上创建 Pull Request，包含：
- 清晰的描述
- 关联的 Issue
- 测试计划
- 截图（如适用）

---

## 故障排除

### 构建失败

```bash
# 清理构建缓存
cargo clean

# 更新依赖
cargo update

# 重新构建
cargo build
```

### 测试失败

```bash
# 单线程运行测试（排除并发问题）
cargo test -- --test-threads=1

# 显示测试输出
cargo test -- --nocapture

# 运行特定测试
cargo test test_name
```

### Docker 问题

```bash
# 重建镜像
docker-compose build --no-cache

# 查看日志
docker-compose logs -f

# 重启服务
docker-compose restart
```

---

## 获取帮助

- GitHub Issues: [https://github.com/synton-db/synton-db/issues](https://github.com/synton-db/synton-db/issues)
- Discussions: [https://github.com/synton-db/synton-db/discussions](https://github.com/synton-db/synton-db/discussions)
- 文档: [docs/](./docs/)
