# Phase 4: Candle ML Feature + Web UI 集成

**日期**: 2026-02-11
**状态**: 完成

## 问题 1: Candle Feature 未启用

服务启动后，生成嵌入时出现警告：
```
WARN Failed to generate embedding: Failed to load model: Candle feature is not enabled.
Please enable the 'candle' feature to use local models.
```

## 问题 2: Web 服务未集成

运维脚本没有包含 Web UI 启动功能。

---

## 修改内容

### 1. Feature 传递链

**crates/api/Cargo.toml**:
```toml
[features]
default = ["ml"]
ml = ["dep:synton-ml"]
candle = ["ml", "synton-ml/candle"]  # 新增
```

**crates/bin/Cargo.toml**:
```toml
[features]
default = ["ml"]
ml = ["dep:synton-ml", "synton-api/ml"]
candle = ["ml", "synton-api/candle"]  # 新增
```

### 2. 构建脚本更新

**synton-ops.sh**:
```bash
cargo build --release --bins --features candle
```

### 3. Web 服务集成

**synton-ops.sh** 新增内容：

1. 添加 Web 相关变量：
   - `WEB_SERVER_BIN`: 指向 `web/` 目录
   - `WEB_SERVER_PID`: Web 服务 PID 文件
   - `WEB_SERVER_PORT`: 默认 5173

2. `cmd_start_local()`: 检查并启动 npm dev

3. `cmd_stop_local()`: 停止 Web 服务

4. `show_status()`: 显示 Web 服务状态

5. `cmd_logs_local()`: 支持 Web 日志查看

---

## 验证

```bash
# 1. 构建
cargo build --release --features candle
# ✅ 构建成功

# 2. 启动服务
./synton-ops.sh start
# ✅ DB Server 启动
# ✅ Web Server 启动
```

### 使用方式

```bash
# 启动所有服务
./synton-ops.sh start

# 查看状态
./synton-ops.sh status

# 查看日志
./synton-ops.sh logs         # 所有日志
./synton-ops.sh logs web       # 仅 Web 日志

# 停止服务
./synton-ops.sh stop
```

---

## 配置说明

### ML Backend 配置

**本地模型 (Candle)**:
```toml
[ml]
enabled = true
backend = "local"
local_model = "sentence-transformers/all-MiniLM-L6-v2"
```
- 需要 `--features candle` 构建
- 首次运行会从 HuggingFace 下载模型
- 需要网络访问 `huggingface.co`

**在线 API (OpenAI 兼容)**:
```toml
[ml]
enabled = true
backend = "openai"
api_endpoint = "https://api.openai.com/v1"
api_key = "sk-your-key"
api_model = "text-embedding-3-small"
```

**常用 API 提供商**:
- OpenAI: `https://api.openai.com/v1`
- DeepSeek: `https://api.deepseek.com/v1`
- Together AI: `https://api.together.xyz/v1`
- Ollama (本地): `http://localhost:11434/v1`

### Web UI 配置

Web 服务默认运行在 `http://localhost:5173`

可通过环境变量 `SYNTON_WEB_PORT` 修改端口：
```bash
SYNTON_WEB_PORT=3000 ./synton-ops.sh start
```

---

## 文件变更

| 文件 | 修改 |
|------|------|
| `crates/api/Cargo.toml` | 添加 `candle` feature |
| `crates/bin/Cargo.toml` | 添加 `candle` feature |
| `synton-ops.sh` | 添加 Web 服务支持 |
| `config.toml` | backend 改为 `openai` |
| `docs/progress/phase4-candle-ml-2026-02-11.md` | 本文档 |
