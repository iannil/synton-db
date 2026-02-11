# ML 集成开发进展

**日期**: 2026-02-05
**阶段**: ML 集成阶段（P0）
**状态**: 基础架构完成

---

## 一、已完成工作

### 1.1 ML 模块基础架构 ✅

| 模块 | 文件 | 说明 |
|------|------|------|
| 错误类型 | `crates/ml/src/error.rs` | MlError 枚举定义 |
| 后端抽象 | `crates/ml/src/backend.rs` | EmbeddingBackend trait、BackendType、DeviceType |
| 配置结构 | `crates/ml/src/config.rs` | EmbeddingConfig、LocalModelConfig、ApiConfig |
| 本地后端 | `crates/ml/src/local.rs` | LocalEmbeddingBackend（使用 Candle） |
| OpenAI 后端 | `crates/ml/src/openai.rs` | OpenAiEmbeddingBackend |
| Ollama 后端 | `crates/ml/src/ollama.rs` | OllamaEmbeddingBackend |
| 统一服务 | `crates/ml/src/service.rs` | EmbeddingService |

### 1.2 系统集成 ✅

| 组件 | 修改内容 |
|------|----------|
| `crates/ml/Cargo.toml` | 添加 serde、reqwest、url 等依赖 |
| `crates/ml/src/lib.rs` | 导出公共接口 |
| `crates/api/Cargo.toml` | 添加 synton-ml 可选依赖 |
| `crates/api/src/service.rs` | 集成 EmbeddingService，自动生成嵌入 |
| `crates/bin/Cargo.toml` | 添加 synton-ml 可选依赖 |
| `crates/bin/src/config.rs` | 添加 MlConfig 配置段 |
| `crates/bin/src/server.rs` | 初始化 EmbeddingService |
| `release/docker/config.toml` | 添加 ML 配置段 |

---

## 二、核心设计

### 2.1 多后端架构

```
EmbeddingService (统一接口)
    ├── LocalBackend (Candle 本地模型)
    ├── OpenAiBackend (OpenAI API)
    └── OllamaBackend (Ollama 本地 API)
```

### 2.2 配置示例

```toml
[ml]
enabled = true
backend = "local"  # 或 "openai", "ollama"

# 本地模型配置
local_model = "sentence-transformers/all-MiniLM-L6-v2"
device = "cpu"

# API 配置
api_endpoint = "https://api.openai.com/v1"
api_model = "text-embedding-3-small"
timeout_secs = 30

# 缓存配置
cache_enabled = true
cache_size = 10000
```

### 2.3 特性门控

- ML 功能默认启用（`default = ["ml"]`）
- Candle 本地模型需要 `--features candle` 启用
- API 后端（OpenAI、Ollama）无需额外 feature

---

## 三、API 变更

### 3.1 新增类型

```rust
// ML 模块
pub use synton_ml::{
    EmbeddingService, EmbeddingStats,
    BackendType, DeviceType,
    EmbeddingConfig, ApiConfig, LocalModelConfig,
    MlError, MlResult,
};

// 主配置
pub struct MlConfig {
    pub enabled: bool,
    pub backend: String,
    pub local_model: String,
    pub device: String,
    pub api_endpoint: String,
    pub api_key: Option<String>,
    pub api_model: String,
    pub timeout_secs: u64,
    pub cache_enabled: bool,
    pub cache_size: usize,
}
```

### 3.2 API 服务增强

```rust
// 创建带嵌入支持的服务
pub fn with_embedding(embedding: Arc<EmbeddingService>) -> Self

// 设置嵌入服务
pub fn set_embedding(&mut self, embedding: Arc<EmbeddingService>)

// 获取嵌入服务引用
pub fn embedding(&self) -> Option<&Arc<EmbeddingService>>
```

---

## 四、测试结果

### 4.1 单元测试

```bash
$ cargo test -p synton-ml
test result: ok. 33 passed; 0 failed
```

### 4.2 编译验证

```bash
$ cargo build --workspace
    Finished `dev` profile
```

---

## 五、后续工作

| 优先级 | 任务 | 说明 |
|--------|------|------|
| P1 | Candle 完整集成 | 当前为 mock 实现，需要实际的模型加载和推理 |
| P1 | 验证测试 | Docker 构建、E2E 测试 |
| P1 | API 文档 | 自动生成的 API 文档 |
| P2 | 性能测试 | 负载测试、基准测试 |
| P2 | Candle feature 优化 | 添加模型下载、缓存、量化支持 |

---

## 六、已知限制

1. **Candle 集成**: 当前为 mock 实现，需要实际的模型加载和推理代码
2. **错误处理**: API 错误处理可以更细致（如区分速率限制、超时等）
3. **缓存**: 当前为内存 LRU，重启后丢失
4. **批量处理**: Ollama 后端不支持原生批量，使用并行请求

---

## 七、文件清单

### 新建文件（7个）

| 文件 | 行数 | 说明 |
|------|------|------|
| `crates/ml/src/error.rs` | ~90 | 错误类型定义 |
| `crates/ml/src/backend.rs` | ~200 | 后端 Trait 定义 |
| `crates/ml/src/config.rs` | ~250 | 配置结构 |
| `crates/ml/src/local.rs` | ~330 | 本地模型实现 |
| `crates/ml/src/openai.rs` | ~330 | OpenAI API 实现 |
| `crates/ml/src/ollama.rs` | ~250 | Ollama API 实现 |
| `crates/ml/src/service.rs` | ~500 | 统一服务层 |

### 修改文件（8个）

| 文件 | 主要变更 |
|------|----------|
| `crates/ml/Cargo.toml` | 添加依赖 |
| `crates/ml/src/lib.rs` | 导出接口 |
| `crates/api/Cargo.toml` | 添加 synton-ml 依赖 |
| `crates/api/src/service.rs` | 集成嵌入服务 |
| `crates/bin/Cargo.toml` | 添加 synton-ml 依赖 |
| `crates/bin/src/config.rs` | 添加 MlConfig |
| `crates/bin/src/server.rs` | 初始化嵌入服务 |
| `release/docker/config.toml` | 添加 ML 配置 |
