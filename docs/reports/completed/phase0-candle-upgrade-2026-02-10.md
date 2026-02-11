# Phase 0: Candle 依赖升级 - 进展报告

**日期**: 2026-02-10

## 目标

解决 Candle 0.6.0 依赖冲突问题，升级到最新版本使本地嵌入模型可用。

## 已完成工作

### 1. 版本升级

- **Candle**: 0.6.0 → 0.9.2
- **candle-nn**: 0.6.0 → 0.9.2
- **candle-transformers**: 0.6.0 → 0.9.2
- 添加 **safetensors** 0.7.0 依赖（直接使用 safetensors crate）

### 2. API 适配

修改文件：
- `Cargo.toml` - 更新 workspace 依赖
- `crates/ml/Cargo.toml` - 更新 ML crate 依赖
- `crates/ml/src/error.rs` - 添加缺失的 `EmbeddingFailed` 错误变体
- `crates/ml/src/loader.rs` - 适配 Candle 0.9.2 API
- `crates/ml/src/local.rs` - 适配 API 调用

主要 API 变更：
1. `VarBuilder::from_safetensors` → 使用 safetensors crate 直接加载
2. `BertModel::new()` → `BertModel::load()`
3. `forward(&input, &mask)` → `forward(&input, &mask, None)`
4. `to_dtype(dtype, device)` → `to_dtype(dtype)`
5. `ApiBuilder::with_cache()` → `ApiBuilder::with_cache_dir()`
6. 添加 `use candle::{Device as CandleDevice, ...}` 类型别名

### 3. Bug 修复

- 添加缺失的 `MlError::EmbeddingFailed` 变体（在 `local.rs` 中被使用但未定义）
- 修复缓存键类型不匹配（`&str` vs `&String`）

## 测试结果

```
running 38 tests
test result: FAILED. 35 passed; 3 failed; 0 ignored
```

**通过的测试 (35/38)**：
- 所有配置测试
- 所有错误处理测试
- 所有本地后端创建测试
- 所有 Ollama 后端测试
- 所有 OpenAI 后端测试

**失败的测试 (3/38)**：
- `service::tests::test_embed_cache` - 需要 HF 网络访问
- `service::tests::test_clear_cache` - 需要 HF 网络访问
- `local::tests::test_embed_cache` - 需要 HF 网络访问

这些失败是预期的，因为测试尝试从 Hugging Face Hub 下载模型。在生产环境中，模型应该预先缓存或使用其他后端（OpenAI/Ollama）。

## 编译状态

✅ `cargo build --package synton-ml --features candle` 成功

## 平台兼容性

- 移除了 `mkl` feature flag，因为它在 macOS ARM64 上不可用
- CPU 后端在所有平台上都可以工作

## 后续步骤

1. **Phase 1**: 自适应分块实现
2. **Phase 2**: Lance 向量索引扩展
3. **Phase 3**: 上下文合成优化

## 文件变更清单

### 修改的文件
- `/Users/iannil/Code/synton-db/Cargo.toml`
- `/Users/iannil/Code/synton-db/crates/ml/Cargo.toml`
- `/Users/iannil/Code/synton-db/crates/ml/src/error.rs`
- `/Users/iannil/Code/synton-db/crates/ml/src/loader.rs`
- `/Users/iannil/Code/synton-db/crates/ml/src/local.rs`

## 技术债务

1. MKL 优化不可用于 macOS ARM64 - 后续可考虑使用 Accelerate 框架
2. 测试中的 3 个失败需要 mock HF Hub API
3. 需要添加嵌入质量验证测试

## 参考资料

- [Candle GitHub](https://github.com/huggingface/candle)
- [Candle 0.9.2 发布说明](https://github.com/huggingface/candle/releases/tag/v0.9.2)
