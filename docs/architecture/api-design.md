# SYNTON-DB API 设计文档

**文档版本**: 1.0
**创建时间**: 2025-02-05
**作者**: SYNTON-DB Team

---

## 1. 概述

SYNTON-DB 提供两类 API：

1. **gRPC**: 内部服务间通讯，高性能要求
2. **REST**: 对外 API，通用兼容性

### 1.1 设计原则

- **语义化**: 操作名称清晰表达意图
- **幂等性**: 写操作支持幂等
- **流式支持**: 大批量操作使用流式 API
- **错误友好**: 结构化错误信息

---

## 2. 核心概念

### 2.1 操作原语

SYNTON-DB 的核心操作不是 CRUD，而是：

| 操作 | 说明 | 类比 |
|------|------|------|
| `Absorb` | 吸收数据，自动处理 | INSERT + 自动分块+向量化 |
| `Query` | 自然语言查询 | SELECT + 图遍历+重排序 |
| `Recall` | 按ID召回节点 | SELECT by ID |
| `Forget` | 遗忘节点（软删除） | DELETE + 归档 |

### 2.2 ID 设计

```
节点 ID: UUID v4
边 ID:   {source_id}::{target_id}::{relation}
会话 ID: UUID v4
```

---

## 3. gRPC API

### 3.1 服务定义

```protobuf
syntax = "proto3";

package synton.v1;

import "google/protobuf/timestamp.proto";
import "google/protobuf/struct.proto";

// ============================================================
// 服务定义
// ============================================================

service SyntonDB {
  // 数据摄入
  rpc Absorb(AbsorbRequest) returns (AbsorbResponse);
  rpc AbsorbStream(stream AbsorbRequest) returns (stream AbsorbResponse);

  // 查询
  rpc Query(QueryRequest) returns (QueryResponse);
  rpc QueryStream(QueryRequest) returns (stream NodeStream);

  // 节点操作
  rpc GetNode(GetNodeRequest) returns (GetNodeResponse);
  rpc GetNodes(GetNodesRequest) returns (GetNodesResponse);
  rpc UpdateNode(UpdateNodeRequest) returns (UpdateNodeResponse);
  rpc DeleteNode(DeleteNodeRequest) returns (DeleteNodeResponse);

  // 边操作
  rpc CreateEdge(CreateEdgeRequest) returns (CreateEdgeResponse);
  rpc GetEdges(GetEdgesRequest) returns (GetEdgesResponse);

  // 图遍历
  rpc Traverse(TraverseRequest) returns (TraverseResponse);
  rpc FindPath(FindPathRequest) returns (FindPathResponse);

  // 批量操作
  rpc Batch(BatchRequest) returns (BatchResponse);

  // 管理
  rpc Stats(StatsRequest) returns (StatsResponse);
  rpc Export(ExportRequest) returns (stream ExportChunk);
  rpc Import(stream ImportChunk) returns (ImportResponse);
}

// ============================================================
// 数据类型
// ============================================================

enum NodeType {
  NODE_TYPE_UNSPECIFIED = 0;
  NODE_TYPE_ENTITY = 1;      // 实体
  NODE_TYPE_CONCEPT = 2;     // 概念
  NODE_TYPE_FACT = 3;        // 事实
  NODE_TYPE_RAW_CHUNK = 4;   // 原始片段
}

enum Relation {
  RELATION_UNSPECIFIED = 0;
  RELATION_IS_PART_OF = 1;
  RELATION_CAUSES = 2;
  RELATION_CONTRADICTS = 3;
  RELATION_HAPPENED_AFTER = 4;
  RELATION_RELATED_TO = 5;
  RELATION_CUSTOM = 99;
}

message Node {
  string id = 1;
  string content = 2;
  NodeType node_type = 3;
  google.protobuf.Struct metadata = 4;
  google.protobuf.Timestamp created_at = 5;
  google.protobuf.Timestamp updated_at = 6;
  float access_score = 7;
  float confidence = 8;
}

message Edge {
  string source_id = 1;
  string target_id = 2;
  Relation relation = 3;
  float weight = 4;
  google.protobuf.Struct metadata = 5;
}

message ReasoningPath {
  repeated Node nodes = 1;
  repeated Edge edges = 2;
  float confidence = 3;
  string explanation = 4;
}

// ============================================================
// 请求/响应消息
// ============================================================

// Absorb: 数据摄入
message AbsorbRequest {
  string content = 1;
  google.protobuf.Struct metadata = 2;
  string source = 3;           // 数据来源标识
  bool auto_chunk = 4;         // 是否自动分块
  bool auto_extract = 5;       // 是否自动抽取实体/关系
}

message AbsorbResponse {
  repeated string node_ids = 1;
  int32 chunks_created = 2;
  int32 entities_extracted = 3;
  int32 edges_created = 4;
}

// Query: 自然语言查询
message QueryRequest {
  oneof query {
    string paql = 1;           // PaQL 查询字符串
    SemanticQuery semantic = 2; // 结构化语义查询
    GraphQuery graph = 3;      // 图查询
  }
  int32 limit = 10;
  int32 offset = 11;
  bool include_paths = 20;     // 是否返回推理路径
  bool rerank = 21;            // 是否重排序
}

message SemanticQuery {
  string query = 1;
  repeated Filter filters = 2;
}

message GraphQuery {
  repeated string start_nodes = 1;
  Relation relation = 2;
  int32 max_depth = 3;
  repeated Filter filters = 4;
}

message Filter {
  string field = 1;
  oneof condition {
    string equals = 10;
    string contains = 11;
    double greater_than = 12;
    double less_than = 13;
    repeated string in_list = 14;
  }
}

message QueryResponse {
  repeated Node nodes = 1;
  repeated ReasoningPath paths = 2;
  string synthesized_context = 3;
  float confidence = 4;
  int32 total_results = 5;
  QueryMetadata query_metadata = 10;
}

message QueryMetadata {
  int64 latency_ms = 1;
  int32 nodes_scanned = 2;
  int32 graph_hops = 3;
  string search_strategy = 4;
}

message NodeStream {
  oneof item {
    Node node = 1;
    QueryMetadata metadata = 2;
  }
}

// GetNode: 获取单个节点
message GetNodeRequest {
  string id = 1;
}

message GetNodeResponse {
  Node node = 1;
}

// GetNodes: 批量获取节点
message GetNodesRequest {
  repeated string ids = 1;
}

message GetNodesResponse {
  repeated Node nodes = 1;
  repeated string not_found = 2;
}

// UpdateNode: 更新节点
message UpdateNodeRequest {
  string id = 1;
  oneof update {
    string content = 2;
    google.protobuf.Struct metadata = 3;
    float access_score_delta = 4;
  }
}

message UpdateNodeResponse {
  Node node = 1;
}

// DeleteNode: 删除节点
message DeleteNodeRequest {
  string id = 1;
  bool soft_delete = 2;  // 软删除
}

message DeleteNodeResponse {
  bool deleted = 1;
}

// CreateEdge: 创建边
message CreateEdgeRequest {
  string source_id = 1;
  string target_id = 2;
  Relation relation = 3;
  float weight = 4;
  google.protobuf.Struct metadata = 5;
}

message CreateEdgeResponse {
  Edge edge = 1;
}

// GetEdges: 获取边
message GetEdgesRequest {
  string source_id = 1;
  Relation relation = 2;
}

message GetEdgesResponse {
  repeated Edge edges = 1;
}

// Traverse: 图遍历
message TraverseRequest {
  string start_node = 1;
  TraverseDirection direction = 2;
  int32 max_depth = 3;
  Relation relation_filter = 4;
  repeated Filter filters = 5;
}

enum TraverseDirection {
  DIRECTION_OUTGOING = 0;
  DIRECTION_INCOMING = 1;
  DIRECTION_BOTH = 2;
}

message TraverseResponse {
  repeated Node nodes = 1;
  repeated Edge edges = 2;
  repeated ReasoningPath paths = 3;
}

// FindPath: 寻找路径
message FindPathRequest {
  string from = 1;
  string to = 2;
  int32 max_hops = 3;
  Relation relation_filter = 4;
}

message FindPathResponse {
  ReasoningPath path = 1;
  bool found = 2;
}

// Batch: 批量操作
message BatchRequest {
  repeated BatchOperation operations = 1;
}

message BatchOperation {
  oneof operation {
    AbsorbRequest absorb = 1;
    UpdateNodeRequest update = 2;
    CreateEdgeRequest create_edge = 3;
    DeleteNodeRequest delete = 4;
  }
}

message BatchResponse {
  repeated BatchResult results = 1;
}

message BatchResult {
  bool success = 1;
  string error = 2;
  oneof result {
    AbsorbResponse absorb = 10;
    Node node = 11;
    Edge edge = 12;
  }
}

// Stats: 统计信息
message StatsRequest {
  bool detailed = 1;
}

message StatsResponse {
  int64 total_nodes = 1;
  int64 total_edges = 2;
  map<string, int64> nodes_by_type = 3;
  map<string, int64> edges_by_relation = 4;
  StorageStats storage = 5;
  QueryStats query = 6;
}

message StorageStats {
  int64 size_bytes = 1;
  int64 vector_count = 2;
  map<string, int64> column_family_size = 3;
}

message QueryStats {
  int64 total_queries = 1;
  double avg_latency_ms = 2;
  int64 queries_last_hour = 3;
}

// Export/Import
message ExportRequest {
  repeated string filters = 1;
  bool include_embeddings = 2;
}

message ExportChunk {
  bytes data = 1;
  int32 chunk_number = 2;
  bool is_last = 3;
}

message ImportChunk {
  bytes data = 1;
  int32 chunk_number = 2;
  bool is_last = 3;
}

message ImportResponse {
  int64 nodes_imported = 1;
  int64 edges_imported = 2;
}
```

---

## 4. REST API

### 4.1 基础规范

- **Base URL**: `/api/v1`
- **Content-Type**: `application/json`
- **认证**: Bearer Token (未来)

### 4.2 端点定义

#### 数据摄入

```http
POST /api/v1/absorb
Content-Type: application/json

{
  "content": "特斯拉是一家美国电动汽车和清洁能源公司...",
  "metadata": {
    "source": "user_upload",
    "topic": "technology"
  },
  "auto_chunk": true,
  "auto_extract": true
}

Response 200:
{
  "node_ids": ["uuid-1", "uuid-2", ...],
  "chunks_created": 5,
  "entities_extracted": 12,
  "edges_created": 8
}
```

#### 查询

```http
POST /api/v1/query
Content-Type: application/json

{
  "paql": "特斯拉的供应链如何影响股价？",
  "limit": 10,
  "include_paths": true,
  "rerank": true
}

Response 200:
{
  "nodes": [...],
  "paths": [
    {
      "nodes": [...],
      "edges": [...],
      "confidence": 0.87,
      "explanation": "供应链问题导致产量下降，进而影响股价"
    }
  ],
  "synthesized_context": "...",
  "confidence": 0.85,
  "total_results": 42,
  "query_metadata": {
    "latency_ms": 127,
    "nodes_scanned": 1523,
    "graph_hops": 3,
    "search_strategy": "hybrid_vector_graph"
  }
}
```

#### 获取节点

```http
GET /api/v1/nodes/{id}

Response 200:
{
  "id": "uuid-1",
  "content": "...",
  "node_type": "ENTITY",
  "metadata": {...},
  "created_at": "2025-02-05T10:00:00Z",
  "updated_at": "2025-02-05T10:00:00Z",
  "access_score": 0.75,
  "confidence": 0.95
}
```

#### 批量获取

```http
POST /api/v1/nodes/batch
Content-Type: application/json

{
  "ids": ["uuid-1", "uuid-2", "uuid-3"]
}

Response 200:
{
  "nodes": [...],
  "not_found": ["uuid-3"]
}
```

#### 更新节点

```http
PATCH /api/v1/nodes/{id}
Content-Type: application/json

{
  "access_score_delta": 0.1
}

Response 200:
{
  "node": {...}
}
```

#### 图遍历

```http
POST /api/v1/traverse
Content-Type: application/json

{
  "start_node": "uuid-1",
  "direction": "outgoing",
  "max_depth": 2,
  "relation_filter": "CAUSES"
}

Response 200:
{
  "nodes": [...],
  "edges": [...],
  "paths": [...]
}
```

#### 路径查找

```http
POST /api/v1/paths/find
Content-Type: application/json

{
  "from": "uuid-1",
  "to": "uuid-2",
  "max_hops": 3
}

Response 200:
{
  "path": {
    "nodes": [...],
    "edges": [...],
    "confidence": 0.92,
    "explanation": "..."
  },
  "found": true
}
```

#### 统计信息

```http
GET /api/v1/stats

Response 200:
{
  "total_nodes": 15234,
  "total_edges": 45678,
  "nodes_by_type": {
    "ENTITY": 2341,
    "CONCEPT": 4523,
    "FACT": 5234,
    "RAW_CHUNK": 3136
  },
  "edges_by_relation": {
    "IS_PART_OF": 12345,
    "CAUSES": 5678,
    ...
  },
  "storage": {
    "size_bytes": 524288000,
    "vector_count": 15234
  },
  "query": {
    "total_queries": 1234,
    "avg_latency_ms": 87.5,
    "queries_last_hour": 45
  }
}
```

#### 流式查询 (SSE)

```http
GET /api/v1/query/stream?paql=...

Response:
Content-Type: text/event-stream

data: {"type":"node","node":{...}}
data: {"type":"node","node":{...}}
data: {"type":"metadata","metadata":{...}}
data: {"type":"done"}
```

---

## 5. 错误处理

### 5.1 错误格式

```json
{
  "error": {
    "code": "NODE_NOT_FOUND",
    "message": "Node with ID 'xxx' not found",
    "details": {
      "node_id": "xxx"
    },
    "request_id": "req-123"
  }
}
```

### 5.2 错误代码

| 代码 | HTTP | 说明 |
|------|------|------|
| `INVALID_ARGUMENT` | 400 | 请求参数无效 |
| `UNAUTHORIZED` | 401 | 未授权 |
| `FORBIDDEN` | 403 | 禁止访问 |
| `NOT_FOUND` | 404 | 资源不存在 |
| `ALREADY_EXISTS` | 409 | 资源已存在 |
| `RESOURCE_EXHAUSTED` | 429 | 请求过多 |
| `INTERNAL` | 500 | 内部错误 |
| `NOT_IMPLEMENTED` | 501 | 未实现 |
| `UNAVAILABLE` | 503 | 服务不可用 |

---

## 6. 流式 API

### 6.1 gRPC 流式

```protobuf
// 服务端流
rpc QueryStream(QueryRequest) returns (stream NodeStream);

// 客户端流
rpc AbsorbStream(stream AbsorbRequest) returns (AbsorbResponse);

// 双向流
rpc ChatStream(stream ChatMessage) returns (stream ChatMessage);
```

### 6.2 REST 流式

- **Server-Sent Events (SSE)**: 单向流
- **WebSocket**: 双向流

---

## 7. 认证与授权（未来）

### 7.1 Bearer Token

```http
GET /api/v1/nodes/{id}
Authorization: Bearer <token>
```

### 7.2 API Key

```http
GET /api/v1/nodes/{id}
X-API-Key: <api-key>
```

---

## 8. SDK 设计（未来）

### 8.1 Rust SDK

```rust
use synton_client::SyntonClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SyntonClient::connect("http://localhost:50051").await?;

    // 吸收数据
    let result = client.absorb("特斯拉是一家...").await?;

    // 查询
    let result = client.query("特斯拉的供应链如何影响股价？").await?;

    Ok(())
}
```

### 8.2 Python SDK

```python
from synton import SyntonClient

client = SyntonClient("http://localhost:8080")

# 吸收数据
result = client.absorb("特斯拉是一家...")

# 查询
result = client.query("特斯拉的供应链如何影响股价？")
```

---

## 参考资料

- [gRPC Style Guide](https://google.aip.dev/general)
- [OpenAPI Specification](https://swagger.io/specification/)
- [REST API Design Best Practices](https://restfulapi.net/)
