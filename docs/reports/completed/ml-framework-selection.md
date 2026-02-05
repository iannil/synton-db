# 嵌入式 ML 框架选型报告

**报告时间**: 2025-02-05
**决策状态**: 已完成
**推荐方案**: Candle

---

## 1. 候选技术

| 技术 | 语言 | 许可证 | 项目状态 |
|------|------|--------|----------|
| Candle | Rust | Apache 2.0 / MIT | HuggingFace 开发，活跃 |
| ONNX Runtime | C++ | MIT | 微软主导，成熟稳定 |
| tch-rs | Rust 绑定 | Apache 2.0 | PyTorch C++ 绑定 |
| tract | Rust | Apache 2.0 | Sonos 开发，中等活跃 |
| Burn | Rust | Apache 2.0 / MIT | Tracel AI，新兴 |

---

## 2. 详细对比

### 2.1 技术架构

| 维度 | Candle | ONNX Runtime | tch-rs | tract | Burn |
|------|--------|--------------|--------|-------|------|
| 核心语言 | Rust 原生 | C++ | C++ (LibTorch) | Rust | Rust |
| 模型格式 | Safetensors, GGUF | ONNX | PyTorch (.pt) | ONNX, NNEF | 自定义 |
| Rust 绑定 | 原生 | FFI (unsafe) | FFI | 原生 | 原生 |
| 静态链接 | ✅ | ⚠️ 需配置 | ⚠️ 复杂 | ✅ | ✅ |

### 2.2 模型支持

| 维度 | Candle | ONNX Runtime | tch-rs | tract | Burn |
|------|--------|--------------|--------|-------|------|
| Transformer | ✅ BERT, LLaMA, 等 | ✅ 转换后 | ✅ | ✅ | ⚠️ 有限 |
| 嵌入模型 | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| 重排序模型 | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| 量化支持 | ✅ GGUF, GPTQ | ✅ | ⚠️ | ⚠️ | ⚠️ |

### 2.3 性能特征

| 维度 | Candle | ONNX Runtime | tch-rs | tract | Burn |
|------|--------|--------------|--------|-------|------|
| 推理速度 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| 内存效率 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| 启动速度 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| 后端支持 | CPU/CUDA/Metal | CPU/CUDA/OpenVINO | CPU/CUDA | CPU/CUDA | CPU/CUDA/WGSL |

### 2.4 部署友好性

| 维度 | Candle | ONNX Runtime | tch-rs | tract | Burn |
|------|--------|--------------|--------|-------|------|
| 部署复杂度 | ⭐⭐⭐⭐⭐ 简单 | ⭐⭐⭐ 中等 | ⭐⭐ 复杂 | ⭐⭐⭐⭐ 简单 | ⭐⭐⭐⭐ 简单 |
| 二进制大小 | 小 | 大（运行时） | 大 | 中 | 小 |
| 依赖管理 | Cargo | 系统 | 系统 | Cargo | Cargo |
| 容器化 | 友好 | 需运行时 | 需 LibTorch | 友好 | 友好 |

---

## 3. SYNTON-DB 需求分析

### 3.1 核心需求

1. **嵌入式推理**
   - 在数据库进程内部运行小模型
   - 不能有外部依赖（独立部署）

2. **模型类型**
   - 嵌入模型（BERT, sentence-transformers）
   - 重排序模型（Cross-Encoder）
   - 未来可能：小型生成模型（SLM）

3. **Rust 原生优先**
   - 与核心存储层紧密集成
   - 避免跨语言边界

4. **部署简化**
   - Docker 镜像优化
   - 静态链接避免外部依赖

### 3.2 需求匹配度

| 需求 | Candle | ONNX Runtime | tch-rs | tract | Burn |
|------|--------|--------------|--------|-------|------|
| Rust 原生 | ✅ | ❌ | ❌ | ✅ | ✅ |
| 嵌入模型 | ✅ 丰富 | ✅ 转换后 | ✅ | ✅ | ⚠️ |
| 部署简单 | ✅ | ⚠️ 运行时 | ❌ 重 | ✅ | ✅ |
| HF 集成 | ✅ 原生 | ✅ 转换 | ⚠️ | ⚠️ | ⚠️ |

---

## 4. 最终决策

### 推荐：**Candle**

### 决策理由

1. **HuggingFace 原生支持**
   - 直接加载 Safetensors 格式
   - 无需模型转换
   - 与 Hub 深度集成

2. **纯 Rust 实现**
   - 无外部 C++ 依赖
   - 内存安全保证
   - 编译时优化

3. **部署优势**
   - 静态链接，单一二进制
   - 容器镜像体积小
   - 跨平台一致性好

4. **性能足够**
   - 对于 SLM 推理（< 1B 参数）
   - 性能损失可接受（相比 C++ 实现）
   - 支持硬件加速（CUDA/Metal）

### 风险与缓解

| 风险 | 缓解措施 |
|------|----------|
| 相对较新 | HuggingFace 官方支持，活跃开发 |
| 模型覆盖 | 主流模型已支持，可自定义实现 |
| 性能未知 | 先行基准测试，热点可优化 |

---

## 5. Candle 使用示例

### 5.1 嵌入模型推理

```rust
use candle::{Device, Tensor};
use candle_transformers::models::bert::{BertModel, Config};
use candle_nn::VarBuilder;
use tokenizers::Tokenizer;

// 加载 BERT 嵌入模型
async fn load_embedding_model() -> Result<BertModel, CandleError> {
    let device = Device::Cpu;

    // 从 HuggingFace Hub 加载
    let api = hf_hub::api::sync::Api::new()?;
    let repo = api.model("sentence-transformers/all-MiniLM-L6-v2".to_string());
    let config_file = repo.get("config.json")?;
    let weights_file = repo.get("model.safetensors")?;
    let tokenizer_file = repo.get("tokenizer.json")?;

    let config = Config::from_file(config_file)?;
    let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[weights_file], candle::DType::F32, &device)? };
    let model = BertModel::load(vb, &config)?;

    let tokenizer = Tokenizer::from_file(tokenizer_file).map_err(|e| CandleError::Msg(e.to_string()))?;

    Ok(model)
}

// 生成嵌入向量
async fn embed(
    model: &BertModel,
    tokenizer: &Tokenizer,
    text: &str,
) -> Result<Vec<f32>, CandleError> {
    let tokens = tokenizer.encode(text, true).map_err(|e| CandleError::Msg(e.to_string()))?;
    let input_ids = Tensor::new(tokens.get_ids(), &Device::Cpu)?.unsqueeze(0)?;
    let token_type_ids = Tensor::new(tokens.get_type_ids(), &Device::Cpu)?.unsqueeze(0)?;
    let attention_mask = Tensor::new(tokens.get_attention_mask(), &Device::Cpu)?.unsqueeze(0)?;

    let embeddings = model.forward(&input_ids, &token_type_ids, &attention_mask)?;
    // Mean pooling
    let (_n_batch, n_tokens, hidden_size) = embeddings.dims3()?;
    let embedding = embeddings.broadcast_div(&Tensor::new(n_tokens as f32, &Device::Cpu)?)?
        .sum(1)?;

    Ok(embedding.to_vec1()?)
}
```

### 5.2 重排序模型

```rust
use candle_transformers::models::with_tracing::BertForSequenceClassification;

async fn rerank(
    model: &BertForSequenceClassification,
    tokenizer: &Tokenizer,
    query: &str,
    documents: Vec<&str>,
) -> Result<Vec<(usize, f32)>, CandleError> {
    let mut scores = Vec::new();

    for (idx, doc) in documents.iter().enumerate() {
        let input = format!("[CLS] {} [SEP] {} [SEP]", query, doc);
        let tokens = tokenizer.encode(&input, true)?;
        let input_ids = Tensor::new(tokens.get_ids(), &Device::Cpu)?.unsqueeze(0)?;
        let attention_mask = Tensor::new(tokens.get_attention_mask(), &Device::Cpu)?.unsqueeze(0)?;

        let output = model.forward(&input_ids, &attention_mask)?;
        let score = output.squeeze(0)?.squeeze(0)?.to_scalar::<f32>()?;

        scores.push((idx, score));
    }

    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    Ok(scores)
}
```

---

## 6. SYNTON-DB ML 模块设计

### 6.1 模块结构

```
synton-db/
├── crates/
│   └── ml/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── embedding.rs      # 嵌入模型
│           ├── reranker.rs       # 重排序模型
│           ├── nlp.rs            # NLP 处理
│           └── models/           # 模型定义
```

### 6.2 核心接口

```rust
// ml/embedding.rs
use async_trait::async_trait;

#[async_trait]
pub trait EmbeddingModel: Send + Sync {
    async fn embed(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>, MlError>;
    async fn embed_single(&self, text: &str) -> Result<Vec<f32>, MlError>;
    fn dimension(&self) -> usize;
}

// ml/reranker.rs
#[async_trait]
pub trait RerankerModel: Send + Sync {
    async fn rerank(
        &self,
        query: &str,
        documents: Vec<&str>,
        top_k: usize,
    ) -> Result<Vec<(usize, f32)>, MlError>;
}

// ml/nlp.rs
pub struct NlpProcessor {
    tokenizer: Tokenizer,
    sentence_splitter: SentenceSplitter,
}

impl NlpProcessor {
    pub async fn semantic_chunk(&self, text: &str, max_tokens: usize) -> Result<Vec<Chunk>, NlpError> {
        // 语义感知分块
    }

    pub async fn extract_entities(&self, text: &str) -> Result<Vec<Entity>, NlpError> {
        // 实体抽取
    }

    pub async fn extract_relations(&self, text: &str) -> Result<Vec<Relation>, NlpError> {
        // 关系抽取
    }
}
```

---

## 7. 备选方案：ONNX Runtime

### 适用场景

1. **跨语言部署**
   - 模型需要在不同语言间共享
   - 已有 ONNX 模型资产

2. **成熟度要求**
   - 生产环境需要最稳定的运行时
   - 需要厂商技术支持

### 混合策略

```
┌─────────────────┐      ┌──────────────────┐
│   Candle        │      │   ONNX Runtime   │
│   (默认)        │      │   (可选)         │
│   • 嵌入        │      │   • GPU 推理     │
│   • 重排序      │      │   • 特殊模型     │
└─────────────────┘      └──────────────────┘
```

---

## 8. 参考资料

- [Candle GitHub](https://github.com/huggingface/candle)
- [Candle Examples](https://github.com/huggingface/candle/tree/main/candle-examples/examples)
- [ONNX Runtime Rust](https://github.com/microsoft/onnxruntime-rust)
- [Burn Framework](https://github.com/tracel-ai/burn)
