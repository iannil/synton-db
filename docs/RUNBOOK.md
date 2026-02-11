# SYNTON-DB 运维手册

**更新时间**: 2026-02-06

---

## 概述

本文档面向 SYNTON-DB 的运维人员，涵盖部署流程、监控告警、常见问题和故障恢复。

---

## 部署流程

### 前置要求

| 组件 | 版本要求 | 用途 |
|------|---------|------|
| Docker | 20.10+ | 容器运行时 |
| Docker Compose | 2.0+ | 服务编排 |
| 磁盘空间 | 10GB+ | 数据存储（根据实际需求调整） |
| 内存 | 2GB+ | 运行时内存（向量操作需要更多） |

### 初始部署

#### 1. 准备配置文件

```bash
# 克隆仓库
git clone https://github.com/synton-db/synton-db.git
cd synton-db

# 复制环境变量模板
cp .env.example .env

# 根据需要编辑 .env 文件
vim .env
```

#### 2. 配置检查清单

- [ ] 数据目录路径正确
- [ ] 端口无冲突（8080, 50051）
- [ ] 日志级别已设置
- [ ] ML 后端配置正确
- [ ] 网络配置（如需跨主机访问）

#### 3. 启动服务

```bash
# 构建并启动所有服务
docker-compose up -d

# 查看服务状态
docker-compose ps

# 查看日志
docker-compose logs -f
```

#### 4. 验证部署

```bash
# 健康检查
curl http://localhost:8080/health

# 获取统计信息
curl http://localhost:8080/stats

# 检查 gRPC 端口
nc -zv localhost 50051
```

### 滚动更新

```bash
# 拉取最新代码
git pull origin main

# 重建镜像
docker-compose build synton-db

# 重启服务（不停机）
docker-compose up -d synton-db

# 验证更新
docker-compose logs -f synton-db
```

### 回滚流程

```bash
# 1. 查看历史版本
git log --oneline

# 2. 切换到稳定版本
git checkout <stable-commit-hash>

# 3. 重建并重启
docker-compose build --no-cache synton-db
docker-compose up -d synton-db

# 4. 验证服务状态
curl http://localhost:8080/health
```

---

## 监控与告警

### 服务健康检查

```bash
# 自动健康检查（Docker 内置）
docker inspect --format='{{.State.Health.Status}}' synton-db

# 手动健康检查
curl http://localhost:8080/health
```

健康检查间隔：30秒
超时时间：10秒
重试次数：3次

### Prometheus 指标

访问地址：http://localhost:9090

#### 关键指标

| 指标 | 类型 | 说明 |
|------|------|------|
| `synton_node_count` | Gauge | 当前节点数量 |
| `synton_edge_count` | Gauge | 当前边数量 |
| `synton_query_duration_seconds` | Histogram | 查询耗时 |
| `synton_request_total` | Counter | 请求总数 |
| `synton_error_total` | Counter | 错误总数 |

### Grafana 仪表板

访问地址：http://localhost:3000

默认凭证：
- 用户名：`admin`
- 密码：`admin`

#### 推荐面板

1. **系统概览**
   - 节点/边数量趋势
   - QPS
   - 错误率

2. **性能指标**
   - 查询延迟（P50, P95, P99）
   - 存储操作耗时
   - 内存使用

3. **告警规则**
   - 错误率 > 5%
   - 查询延迟 > 1s
   - 服务不可用

---

## 常见问题与解决方案

### 问题 1: 服务无法启动

**症状**: `docker-compose up` 后服务立即退出

**排查步骤**:

```bash
# 1. 查看日志
docker-compose logs synton-db

# 2. 检查配置文件
docker-compose config

# 3. 验证数据目录权限
ls -la ./data/
```

**常见原因**:
- 数据目录权限不足
- 配置文件语法错误
- 端口被占用

**解决方案**:

```bash
# 修复目录权限
sudo chown -R 1000:1000 ./data/

# 更改端口（如果冲突）
# 编辑 docker-compose.yml 或 .env
```

### 问题 2: 查询超时

**症状**: API 请求无响应或超时

**排查步骤**:

```bash
# 1. 检查服务负载
curl http://localhost:8080/stats

# 2. 查看资源使用
docker stats synton-db

# 3. 检查日志中的错误
docker-compose logs synton-db | grep -i error
```

**解决方案**:

```bash
# 1. 增加资源限制
# 编辑 docker-compose.yml，添加：
services:
  synton-db:
    deploy:
      resources:
        limits:
          memory: 4G
        reservations:
          memory: 2G

# 2. 调整 Graph-RAG 配置
# 编辑 .env：
SYNTON_GRAPHRAG_MAX_TRAVERSAL_DEPTH=2
SYNTON_GRAPHRAG_MAX_RESULTS=5
```

### 问题 3: 磁盘空间不足

**症状**: 日志显示 "No space left on device"

**排查步骤**:

```bash
# 检查磁盘使用
df -h

# 检查数据目录大小
du -sh ./data/*
```

**解决方案**:

```bash
# 1. 清理 Docker 资源
docker system prune -a

# 2. 配置记忆衰减自动清理
# 编辑 .env：
SYNTON_MEMORY_RETENTION_THRESHOLD=0.3

# 3. 手动清理过期数据
# (需要通过 CLI 或 API 实现)
```

### 问题 4: 内存使用过高

**症状**: 容器 OOM 被杀死

**排查步骤**:

```bash
# 检查内存限制
docker inspect synton-db | grep -i memory

# 查看实际使用
docker stats synton-db
```

**解决方案**:

```bash
# 1. 禁用 ML 功能（如不需要）
SYNTON_ML_ENABLED=false

# 2. 减少向量缓存
SYNTON_ML_CACHE_SIZE=1000

# 3. 调整 RocksDB 缓存
# 编辑 config.toml：
[storage]
cache_size_mb = 128
```

### 问题 5: 网络连接问题

**症状**: 无法从其他主机访问服务

**排查步骤**:

```bash
# 1. 检查端口绑定
docker-compose ps

# 2. 检查防火墙
sudo ufw status

# 3. 测试本地访问
curl http://localhost:8080/health
curl http://<local-ip>:8080/health
```

**解决方案**:

```bash
# 1. 确认绑定地址
# .env 或 docker-compose.yml 中使用 0.0.0.0

# 2. 开放防火墙端口
sudo ufw allow 8080/tcp
sudo ufw allow 50051/tcp

# 3. 检查 Docker 网络模式
# 如需主机网络，在 docker-compose.yml 中添加：
network_mode: "host"
```

---

## 备份与恢复

### 备份

#### 数据备份

```bash
# 1. 停止服务（确保数据一致性）
docker-compose stop synton-db

# 2. 备份数据目录
tar -czf synton-db-backup-$(date +%Y%m%d).tar.gz ./data/

# 3. 备份配置文件
tar -czf synton-db-config-$(date +%Y%m%d).tar.gz \
    .env \
    release/docker/config.toml \
    docker-compose.yml

# 4. 重启服务
docker-compose start synton-db
```

#### 自动化备份脚本

```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="/backup/synton-db"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p "$BACKUP_DIR"

# 备份数据
docker exec synton-db tar -czf /tmp/data.tar.gz /data
docker cp synton-db:/tmp/data.tar.gz "$BACKUP_DIR/data-$DATE.tar.gz"

# 备份配置
cp .env "$BACKUP_DIR/env-$DATE"
cp release/docker/config.toml "$BACKUP_DIR/config-$DATE.toml"

# 清理 30 天前的备份
find "$BACKUP_DIR" -name "*.tar.gz" -mtime +30 -delete
```

### 恢复

```bash
# 1. 停止服务
docker-compose down

# 2. 备份当前数据（以防万一）
mv ./data ./data.failed

# 3. 恢复数据
tar -xzf synton-db-backup-20250206.tar.gz

# 4. 恢复配置
tar -xzf synton-db-config-20250206.tar.gz

# 5. 重启服务
docker-compose up -d

# 6. 验证
curl http://localhost:8080/stats
```

---

## 性能调优

### 内存调优

```toml
# config.toml

[storage]
# 减少 RocksDB 缓存
cache_size_mb = 128

[ml]
# 减少 ML 缓存
cache_size = 1000

[graphrag]
# 限制遍历深度
max_traversal_depth = 2
```

### 存储调优

```toml
[storage]
# 启用压缩
compression = true

# 调整写缓冲区
write_buffer_size = 67108864  # 64MB

# 调整 SST 文件大小
target_file_size_base = 67108864
```

### 查询调优

```toml
[graphrag]
# 减少返回结果数
max_results = 5

# 调整检索权重
vector_weight = 0.8
graph_weight = 0.2
```

---

## 日志管理

### 日志级别

| 级别 | 用途 |
|------|------|
| `trace` | 最详细的调试信息 |
| `debug` | 调试信息 |
| `info` | 一般信息（生产推荐） |
| `warn` | 警告信息 |
| `error` | 错误信息 |

### 日志配置

```bash
# .env
SYNTON_LOG_LEVEL=info
SYNTON_LOG_JSON_FORMAT=false  # 生产环境建议 true
```

### 日志轮转

```bash
# 查看 Docker 日志大小
docker logs synton-db --tail 0 | wc -l

# 限制日志大小
# 在 docker-compose.yml 中添加：
logging:
  driver: "json-file"
  options:
    max-size: "100m"
    max-file: "3"
```

---

## 安全加固

### 1. 网络隔离

```yaml
# docker-compose.yml
networks:
  synton-network:
    driver: bridge
    internal: false  # 设为 true 禁止外网访问
```

### 2. 只读根文件系统

```yaml
# docker-compose.yml
services:
  synton-db:
    read_only: true
    tmpfs:
      - /tmp
```

### 3. 资源限制

```yaml
# docker-compose.yml
services:
  synton-db:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          cpus: '1'
          memory: 2G
```

### 4. 健康检查优化

```yaml
# docker-compose.yml
healthcheck:
  test: ["CMD", "wget", "-q", "-O-", "http://localhost:8080/health"]
  interval: 30s
  timeout: 10s
  retries: 3
  start_period: 10s
```

---

## 应急响应

### 服务完全宕机

```bash
# 1. 检查容器状态
docker-compose ps

# 2. 尝试重启
docker-compose restart synton-db

# 3. 如果重启失败，重建
docker-compose down
docker-compose up -d

# 4. 检查系统资源
free -h
df -h
```

### 数据损坏

```bash
# 1. 停止服务
docker-compose down

# 2. 从备份恢复
# 见"备份与恢复"章节

# 3. 启动服务
docker-compose up -d
```

### 性能严重下降

```bash
# 1. 临时禁用 ML 功能
export SYNTON_ML_ENABLED=false
docker-compose up -d

# 2. 重启服务
docker-compose restart synton-db

# 3. 监控恢复情况
watch -n 5 'curl -s http://localhost:8080/stats'
```

---

## 联系支持

- GitHub Issues: [https://github.com/synton-db/synton-db/issues](https://github.com/synton-db/synton-db/issues)
- 文档: [docs/](./docs/)

---

## MCP 服务器配置（2026-02-09 更新）

SYNTON-DB 现在支持 MCP (Model Context Protocol)，可作为 AI 编程助手的外挂记忆库。

### 启动 MCP 服务器

```bash
# 构建二进制
cargo build --release -p synton-mcp-server

# 启动服务器
./target/release/synton-mcp-server --endpoint http://localhost:8080
```

### Docker 部署

```bash
cd release/mcp-server
docker-compose up -d
```

### Claude Code 配置

在 `~/.config/claude-code/mcp_servers.json` 添加：

```json
{
  "mcpServers": {
    "synton-db": {
      "command": "/usr/local/bin/synton-mcp-server",
      "args": ["--endpoint", "http://localhost:8080"]
    }
  }
}
```

### 可用工具

| 工具 | 描述 |
|------|------|
| `synton_absorb` | 存储知识到数据库 |
| `synton_query` | 自然语言查询 |
| `synton_hybrid_search` | Graph-RAG 混合检索 |
| `synton_get_node` | 按 UUID 获取节点 |
| `synton_traverse` | 图遍历 |
| `synton_add_edge` | 创建节点关系 |
| `synton_stats` | 获取数据库统计 |
| `synton_list_nodes` | 列出所有节点 |

详细文档：[MCP Integration Report](./reports/completed/mcp-integration.md)

---

## Web UI 管理界面（2025-02-09 更新）

SYNTON-DB 现在提供完整的 Web 管理界面，支持节点/边管理、图可视化、查询和遍历功能。

### 访问地址

- 开发环境：http://localhost:5173
- 生产环境：http://localhost:8080/ (通过 Rust API 服务器提供)

### 主要功能

| 页面 | 路径 | 功能 |
|------|------|------|
| Dashboard | `/` | 统计概览、快捷操作、最近节点 |
| Nodes | `/nodes` | 节点列表、创建、删除、详情 |
| Edges | `/edges` | 边列表、创建关系 |
| Graph | `/graph` | 图可视化（Cytoscape.js） |
| Query | `/query` | 自然语言查询、GraphRAG 搜索 |
| Traverse | `/traverse` | 图遍历可视化 |

### 开发环境启动

```bash
cd web
npm install
npm run dev
```

### 生产构建

```bash
cd web
npm run build
```

构建文件输出到 `web/dist/`，Rust API 服务器会自动提供这些静态文件。

### 验收状态

- [x] 节点管理功能完整
- [x] 边管理功能完整
- [x] 图可视化正常显示
- [x] 查询和遍历功能正常
- [x] 深色主题 UI
- [x] 响应式设计

详细文档：[Web UI Implementation Report](./reports/completed/2025-02-09-web-ui-implementation.md)
