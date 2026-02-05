# SYNTON-DB 长期记忆

本文件记录 SYNTON-DB 项目的长期知识、用户偏好和关键决策。

---

## 项目上下文

### 项目定位
SYNTON-DB 是一个认知数据库（Cognitive Database），也称为"神经符号数据库"。这是一个专为 LLM 设计的数据库，旨在作为大语言模型的外挂大脑/海马体。

### 与传统数据库的区别
- 传统数据库：SQL、NoSQL、文档、向量 - 专注于 CRUD 操作
- SYNTON-DB：专注于记忆、推理和关联

### 项目阶段
当前处于**MVP 阶段完成，进入集成阶段**。

**已完成**:
- MVP0: 存储基础 (RocksDB + Lance)
- MVP1: 张量图 (Node + Edge + Graph traversal)
- MVP2: Graph-RAG (混合检索)
- MVP3: PaQL (查询解析器)
- MVP4: 记忆机制 (遗忘曲线)
- MVP5: API 服务 (REST + gRPC)

**进行中**: 集成与部署准备

---

## 核心设计理念

SYNTON-DB 基于四大原则：

1. **入库即理解** - 自动知识图谱提取
2. **查询即推理** - 混合向量搜索 + 图遍历
3. **输出即上下文** - 为 LLM 提供预处理上下文，而非原始数据

---

## 架构设计

### 4 层架构 (CortexDB)

1. **接口层 (Interface Layer)**
   - PaQL (Prompt as Query Language) 解析器
   - 接受自然语言查询而非 SQL

2. **认知计算层 (Cognitive Compute Layer)**
   - 使用嵌入式小语言模型 (SLM)
   - 处理推理、重排序和上下文压缩

3. **张量图存储层 (Tensor-Graph Storage Layer)**
   - 核心引擎
   - 语义单元存储为节点（带向量）和边（带逻辑关系）

4. **基础设施层 (Infrastructure Layer)**
   - 基于 Rust
   - 利用 mmap 和 NVMe 优化

### 核心数据模型：张量图 (Tensor-Graph)

**节点 (语义原子)**:
- `ID`: UUID
- `Content`: 文本/图像数据
- `Embedding`: 向量表示（如 Float32[^1536]）
- `Meta`: 时间戳、来源、置信度分数、访问频率
- `Type`: 实体/概念/事实/原始片段

**边 (逻辑链接)**:
- `SourceID` → `TargetID`
- `Relation`: 自然语言关系类型（如 "is_part_of"、"contradicts"、"happened_after"）
- `Weight`: 关联强度 (0.0-1.0)
- `Vector`: 关系向量表示，用于模糊关系查询

---

## 独有特性

1. **Graph-RAG**: 结合向量相似度搜索与多跳图遍历的混合检索
2. **自适应分块**: 语义感知的文档分割（非固定字符数）
3. **分层存储**: 摘要层 → 段落层 → 句子层
4. **记忆衰退与强化**: 基于遗忘曲线的数据管理
5. **动态事实修正**: 冲突检测与时序边版本管理
6. **上下文合成**: 返回结构化上下文包，而非原始行数据

---

## 技术栈方向

| 组件 | 最终决策 | 决策日期 |
|------|----------|----------|
| 语言 | Rust | 2025-02-05 |
| KV 存储 | RocksDB | 2025-02-05 |
| 向量索引 | Lance | 2025-02-05 |
| 嵌入式 ML | Candle | 2025-02-05 |
| 协议 | gRPC + REST | 2025-02-05 |

### 技术栈决策理由

1. **Rust 原生优先**：选择 Lance 和 Candle 实现 Rust 原生技术栈，减少 FFI 边界
2. **列族支持是关键**：RocksDB 的列族功能适合多数据类型存储（节点、边、元数据）
3. **部署简化**：静态链接、单一 Docker 镜像

---

## 用户偏好

### 交流语言
- 交流与文档：**中文**
- 代码：**英文**

### 文档原则
- 强类型、可测试、分层解耦
- 清晰可读、模式统一（便于 LLM 理解与改写）

### 发布约定
- 发布固定在 `/release` 文件夹
- 例如：Rust 服务固定发布在 `/release/rust`
- 发布成果物必须以生产环境为标准

### 环境约定
- 尽量使用 Docker 部署
- 配置独立网络，避免与其他项目冲突

---

## 关键决策

### 决策 1: 双层记忆系统
**日期**: 2025-02-05
**决策**: 采用基于 Markdown 文件的透明双层记忆架构
**原因**:
- 禁止使用复杂的嵌入检索
- 所有记忆操作必须对人类可读且对 Git 友好

**架构**:
- 第一层：每日笔记（流）- `./memory/daily/{YYYY-MM-DD}.md`
- 第二层：长期记忆（沉积）- `./memory/MEMORY.md`

### 决策 2: 技术栈选型
**日期**: 2025-02-05
**决策**: 确定最终技术栈组合

**选型**:
- KV 存储：RocksDB（列族支持、写密集优化）
- 向量索引：Lance（Rust 原生、内置元数据）
- ML 框架：Candle（Rust 原生、HuggingFace 集成）
- 网络协议：gRPC (tonic) + REST (Axum)

**原因**: Rust 原生优先，减少外部依赖，简化部署

### 决策 3: 文档规范
**日期**: 2025-02-05
**决策**: 建立严格的文档分类和命名规范

**分类**:
- 规范文档：`docs/standards/`
- 模板文档：`docs/templates/`
- 架构文档：`docs/architecture/`
- 进度文档：`docs/progress/`
- 选型报告：`docs/reports/`
- 完成报告：`docs/reports/completed/`
- 验收报告：`docs/reports/`

**命名格式**:
- 进度：`{YYYY-MM-DD}-{topic}.md`
- 完成：`{YYYY-MM-DD}-{topic}-completed.md`
- 验收：`{YYYY-MM-DD}-{topic}-acceptance.md`

---

## 经验教训

（此部分将在项目进展中逐步填充）
