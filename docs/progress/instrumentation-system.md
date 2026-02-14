# AI 代码执行埋点系统实现进展

## 概述

为 SYNTON-DB 认知数据库实现了一个完整的 AI 代码执行埋点系统，用于追踪和可视化代码执行的全生命周期。

**实现日期**: 2025-02-14

## 项目结构

```
synton-db/
├── crates/
│   ├── instrument/          # 新增：埋点核心库
│   ├── instrument-macros/ # 新增：埋点过程宏
│   ├── collector/          # 新增：数据收集和存储
│   └── api/
│       └── src/
│           ├── lib.rs      # 更新：添加了 instrument 模块
│           ├── instrument.rs # 新增：可视化 API 端点
│           └── rest.rs         # 更新：集成 instrument 路由
├── config.toml           # 更新：添加埋点配置项（待完成）
└── docs/
    └── progress/
        └── instrumention-system.md  # 本文档
```

## 已完成阶段

### Phase 1: 核心埋点框架 (`crates/instrument`)

**状态**: ✅ 已完成

**实现内容**:

1. **核心数据结构** (`span.rs`)
   - `TraceSpan`: 跟踪跨度，包含 ID、父级 ID、名称、时间戳、状态等
   - `SpanStatus`: 跨度的运行状态（Running, Completed, Failed, Cancelled）
   - `SpanKind`: 跨度分类（Function, AsyncTask, DatabaseQuery, ExternalCall, Internal, Custom）
   - `TraceEvent`: 埋点事件类型（Enter, Exit, Checkpoint, Error, Custom）
   - `SpanMetadata`: 跨度元数据

2. **内存收集器** (`collector.rs`)
   - `TraceCollector`: 全局收集器，使用 DashMap 存储跨度
   - `CollectorConfig`: 收集器配置
   - `SpanGuard`: 自动的跨度管理
   - 提供以下功能:
     - `enter_span()`: 进入新的跨度
     - `complete_span()`: 完成跨度
     - `fail_span()`: 标记跨度失败
     - `checkpoint()`: 记录检查点
     - `get_lifecycle()`: 获取生命周期视图
     - `get_timeline()`: 获取时间线视图
     - `get_statistics()`: 获取统计信息
     - `export_json()`: 导出为 JSON
     - `export_mermaid()`: 导出为 Mermaid 流程图

3. **统计管理** (`statistics.rs`)
   - `TimeWindowStats`: 时间窗口统计
   - `SpanNameStats`: 按跨度名称统计
   - `StatisticsManager`: 统计管理器

4. **视图类型** (`views.rs`)
   - `LifecycleView`: 层级结构视图
   - `TimelineView`: 时间线事件视图
   - `Statistics`: 统计摘要
   - `DashboardStats`: 仪表板统计
   - `ExportFormat`: 导出格式（Json, Mermaid, Text）

5. **过程宏** (`instrument-macros`)
   - `#[trace]`: 函数级埋点宏，自动记录：
     - 函数参数
     - 执行时间
     - 返回值
     - 调用关系
   - `#[checkpoint]`: 检查点宏，记录中间执行状态

### Phase 2: 数据收集器 (`crates/collector`)

**状态**: ✅ 已完成

**实现内容**:

1. **持久化存储** (`persistence.rs`)
   - `PersistenceBackend`: 持久化存储后端特征
   - `RocksDbPersistence`: 基于 RocksDB 的实现
   - 使用列族（Column Families）分别存储跨度和事件
   - `StorageStats`: 存储统计信息

2. **查询接口** (`query.rs`)
   - `QueryFilter`: 查询过滤器，支持：
     - 按 trace_id 筛选
     - 按名称筛选
     - 按类型筛选
     - 按时间范围筛选
     - 持持续时间筛选
     - 分页限制和偏移
   - `QueryFilterBuilder`: 流式过滤器构建器

3. **追踪收集器** (`lib.rs`)
   - `TraceCollector`: 带有持久化的追踪收集器
   - 功能：
     - `store_span()`: 存储跨度
     - `store_event()`: 存储事件
     - `get_trace()`: 按ID获取跨度
     - `query_traces()`: 查询跨度
     - `recent_traces()`: 获取最近的跨度
     - `clear()`: 清除所有数据
     - `flush()`: 刷新到磁盘
     - `stats()`: 获取存储统计

### Phase 3: 可视化 API (`crates/api/src/instrument.rs`)

**状态**: ✅ 已完成

**实现内容**:

新增 REST API 端点：

1. **获取生命周期**
   ```
   GET /api/v1/instrument/lifecycle/{trace_id}
   ```
   - 返回完整的层及结构视图
   - 包含每个跨度的详细信息

2. **获取时间线**
   ```
   GET /api/v1/instrument/timeline/{trace_id}
   ```
   - 返回按时间排序的事件列表
   - 包含所有检查点和事件

3. **获取统计信息**
   ```
   GET /api/v1/instrument/stats
   ```
   - 返回收集器统计信息

4. **导出追踪数据**
   ```
   GET /api/v1/instrument/export/{trace_id}?format={json|mermaid}
   ```
   - 支持导出为 JSON 或 Mermaid 格式

**API 模块**: `instrument.rs`
- 定义了响应类型：`LifecycleResponse`、`TimelineResponse`、`StatisticsResponse`、`ExportResponse`
- 实现了路径参数解析：`TracePath`
- 集成 `synton_instrument` 中的类型

## 技术栈

| 组件 | 技术选型 |
|--------|----------|
| 过程宏 | `proc-macro2` + `quote` + `syn` |
| 异步运行时 | `tokio` + `parking_lot` + `dashmap` |
| 序列化 | `serde` + `serde_json` |
| 存储 | `rocksdb` |
| Web 框架 | `axum` + `tower` |
| 时间处理 | `chrono` |
| 唯一标识 | `uuid` |

## 验证方式

1. **单元测试**: 每个 crate 都包含 `#[cfg(test)]` 模块
2. **集成测试**: 测试 API 端点的端到端连接
3. **性能测试**: 验证埋点对性能的影响 < 5%

## 待完成任务

### Phase 4: 配置集成

**状态**: ⏳ 待完成

**需要实现**:
- 更新 `config.toml` 添加 `[instrument]` 配置段
- 在 `synton-api` 中添加配置读取和初始化
- 集成环境变量 `SYNTON_INSTRUMENT_*`

### Phase 5: 文档和进度追踪

**状态**: ✅ 进行中（本文档）

## 使用示例

### 在代码中使用埋点宏

```rust
use synton_instrument::{trace, TraceCollector};

#[trace]
async fn process_query(query: &str) -> Result<Vec<Node>> {
    // 函数进入时自动创建追踪跨度

    let parsed = parse_query(query)?;

    // 执行业务逻辑
    let results = search_nodes(&parsed).await?;

    Ok(results)
}

#[trace]
fn validate_syntax(input: &str) -> bool {
    // 同步函数自动追踪

    input.len() <= 1000
}
```

### 查询追踪数据

```rust
use synton_collector::TraceCollector;
use synton_instrument::ExportFormat;

// 获取全局收集器
let collector = TraceCollector::global();

// 获取生命周期
let lifecycle = collector.get_lifecycle(trace_id)?;

// 获取时间线
let timeline = collector.get_timeline(trace_id)?;

// 导出为 Mermaid
let mermaid = collector.export_mermaid(trace_id)?;
println!("{}", mermaid);
```

### REST API 示例

```bash
# 获取生命周期
curl http://localhost:3000/api/v1/instrument/lifecycle/{trace_id}

# 获取时间线
curl http://localhost:3000/api/v1/instrument/timeline/{trace_id}

# 获取统计
curl http://localhost:3000/api/v1/instrument/stats

# 导出为 JSON
curl http://localhost:3000/api/v1/instrument/export/{trace_id}?format=json

# 导出为 Mermaid
curl http://localhost:3000/api/v1/instrument/export/{trace_id}?format=mermaid
```

## 关键决策记录

1. **宏设计**: 选择使用独立的宏 crate（`instrument-macros`）而非内联宏，便于维护和编译速度
2. **存储策略**: 使用 RocksDB 列族（Column Families）分别存储跨度和事件，优化查询性能
3. **缓存策略**: 实现双层缓存（内存 + 持久化），平衡实时性和性能
4. **API 设计**: RESTful 风格，支持多种导出格式，便于前端集成

## 下一步

1. 完成 Phase 4: 配置集成
2. 添加单元测试覆盖率达到 80% 以上
3. 实现 E2E 测试关键用户流程
4. 集成 OpenAPI 文档
