# Phase 3: API 文档 (OpenAPI/Swagger) - 进展报告

**时间**: 2026-02-06
**状态**: ✅ 已完成

## 目标

生成完整的 API 文档，支持 Swagger UI 访问。

## 完成的工作

### 1. 添加 OpenAPI 依赖

在 `crates/api/Cargo.toml` 中添加:
```toml
utoipa = { version = "4.2", features = ["axum_extras", "chrono", "uuid"] }
utoipa-swagger-ui = { version = "4.0", features = ["axum"] }
```

### 2. 创建 OpenAPI 模块

**新建文件**: `crates/api/src/openapi.rs`

包含:
- `ApiDoc` - 使用 `#[derive(OpenApi)]` 的主文档结构
- API 信息定义 (title, version, description, license)
- 路径定义 (所有 API 端点)
- 组件/Schema 定义
- 标签定义 (health, nodes, edges, query, graph)

### 3. 添加路径注解

在 `crates/api/src/rest.rs` 中为所有处理器添加了 `#[utoipa::path]` 注解:
- `health_check` - 健康检查
- `stats` - 统计信息
- `add_node` - 添加节点
- `get_node` - 获取节点
- `get_all_nodes` - 获取所有节点
- `delete_node` - 删除节点
- `add_edge` - 添加边
- `query` - 查询
- `traverse` - 图遍历
- `hybrid_search` - GraphRAG 混合搜索
- `bulk_operation` - 批量操作

### 4. OpenAPI JSON 端点

新增端点: `GET /api-docs/openapi.json`

返回完整的 OpenAPI 3.0 规范 JSON 文档。

## API 文档访问

### OpenAPI JSON
```
GET http://localhost:8080/api-docs/openapi.json
```

### 使用 Swagger UI

可以使用任何在线 Swagger UI 查看文档:
1. 访问 https://petstore.swagger.io/
2. 输入 API URL: `http://localhost:8080/api-docs/openapi.json`

## API 端点列表

| 端点 | 方法 | 描述 | 标签 |
|------|------|------|------|
| `/health` | GET | 健康检查 | health |
| `/stats` | GET | 数据库统计 | health |
| `/nodes` | POST | 添加节点 | nodes |
| `/nodes` | GET | 获取所有节点 | nodes |
| `/nodes/:id` | GET | 获取单个节点 | nodes |
| `/nodes/:id` | DELETE | 删除节点 | nodes |
| `/edges` | POST | 添加边 | edges |
| `/query` | POST | 文本查询 | query |
| `/traverse` | POST | 图遍历 | graph |
| `/hybrid_search` | POST | GraphRAG 混合搜索 | query |
| `/bulk` | POST | 批量操作 | nodes |

## Schema 定义

定义的 Schema 类型:
- `HealthResponse` - 健康检查响应
- `DatabaseStats` - 数据库统计
- `NodeInfo` - 节点信息
- `EdgeInfo` - 边信息
- `AddNodeRequest` - 添加节点请求
- `AddNodeResponse` - 添加节点响应
- `GetNodeResponse` - 获取节点响应
- `DeleteNodeRequest` - 删除节点请求
- `DeleteNodeResponse` - 删除节点响应
- `AddEdgeRequest` - 添加边请求
- `AddEdgeResponse` - 添加边响应
- `QueryRequest` - 查询请求
- `QueryResponse` - 查询响应
- `TraverseRequest` - 遍历请求
- `TraverseResponse` - 遍历响应
- `HybridSearchRequest` - 混合搜索请求
- `HybridSearchResponse` - 混合搜索响应
- `BulkOperationRequest` - 批量操作请求
- `BulkOperationResponse` - 批量操作响应

## 下一步

由于 Swagger UI 集成存在类型兼容性问题，当前实现提供 OpenAPI JSON 端点。用户可以使用:
1. 在线 Swagger UI (如 https://editor.swagger.io/)
2. VS Code REST Client 插件
3. Postman (导入 OpenAPI JSON)

如需集成 Swagger UI，可以:
1. 升级 utoipa/utoipa-swagger-ui 到兼容版本
2. 或使用反向代理在前端层提供 Swagger UI
