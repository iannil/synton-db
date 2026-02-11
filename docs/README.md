# SYNTON-DB 文档索引

本文档提供 SYNTON-DB 项目文档的导航索引。

---

## 文档结构

```
docs/
├── README.md                      # 本文档 - 文档导航索引
├── PROJECT_STATUS.md              # 项目当前状态概览
├── RUNBOOK.md                     # 运维手册
├── CONTRIB.md                     # 贡献指南
├── standards/                    # 规范文档
│   └── documentation-conventions.md
├── templates/                    # 文档模板
│   ├── progress-report.md
│   ├── completion-report.md
│   └── acceptance-report.md
├── architecture/                 # 架构文档
├── progress/                     # 进行中的工作 (空=无进行中工作)
├── reports/
│   └── completed/                # 已完成阶段报告
└── archive/                      # 历史归档
    └── 2025/                     # 2025 年历史文档
```

---

## 快速导航

### 项目状态
- **[PROJECT_STATUS.md](PROJECT_STATUS.md)** - 项目当前阶段、测试覆盖、下一步计划

### 运维与贡献
- **[RUNBOOK.md](RUNBOOK.md)** - 部署、运维、故障排查
- **[CONTRIB.md](CONTRIB.md)** - 开发规范、提交指南、代码审查

### 规范与模板
- **[standards/documentation-conventions.md](standards/documentation-conventions.md)** - 文档编写规范
- **[templates/](templates/)** - 各类文档模板

---

## 已完成阶段报告

### 2026 年阶段 (Phase 0-4)

| 文档 | 内容 | 日期 |
|------|------|------|
| [phase0-candle-upgrade-2026-02-10.md](reports/completed/phase0-candle-upgrade-2026-02-10.md) | Candle 0.6.0 → 0.9.2 升级 | 2026-02-10 |
| [phase1-chunking-2026-02-10.md](reports/completed/phase1-chunking-2026-02-10.md) | 自适应分块实现 (19 tests) | 2026-02-10 |
| [phase2-lance-index-2026-02-10.md](reports/completed/phase2-lance-index-2026-02-10.md) | Lance 向量索引 (46 tests) | 2026-02-10 |
| [phase3-context-synthesis-2026-02-10.md](reports/completed/phase3-context-synthesis-2026-02-10.md) | 上下文合成优化 (30 tests) | 2026-02-10 |
| [phase4-candle-ml-2026-02-11.md](reports/completed/phase4-candle-ml-2026-02-11.md) | Candle Feature + Web UI | 2026-02-11 |

### 集成与 API

| 文档 | 内容 | 日期 |
|------|------|------|
| [api-document-ingestion-2026-02-10.md](reports/completed/api-document-ingestion-2026-02-10.md) | API 文档摄取 | 2026-02-10 |
| [mcp-integration.md](reports/completed/mcp-integration.md) | MCP 集成 | 2026-02-09 |
| [phases-0-3-summary-2026-02-10.md](reports/completed/phases-0-3-summary-2026-02-10.md) | Phase 0-3 总结 | 2026-02-10 |

### 早期集成测试 (2026-02-06)

| 文档 | 内容 | 日期 |
|------|------|------|
| [phase1-unit-tests-2026-02-06.md](reports/completed/phase1-unit-tests-2026-02-06.md) | 单元测试覆盖 | 2026-02-06 |
| [phase2-candle-ml-2026-02-06.md](reports/completed/phase2-candle-ml-2026-02-06.md) | Candle ML 实现 | 2026-02-06 |
| [phase3-api-docs-2026-02-06.md](reports/completed/phase3-api-docs-2026-02-06.md) | API 文档 | 2026-02-06 |
| [phase4-integration-tests-2026-02-06.md](reports/completed/phase4-integration-tests-2026-02-06.md) | 集成测试 | 2026-02-06 |

### 技术选型报告

| 文档 | 内容 | 日期 |
|------|------|------|
| [tech-stack-final.md](reports/completed/tech-stack-final.md) | 最终技术栈 | 2025-02-05 |
| [ml-framework-selection.md](reports/completed/ml-framework-selection.md) | ML 框架选型 | 2025-02-05 |
| [vector-index-selection.md](reports/completed/vector-index-selection.md) | 向量索引选型 | 2025-02-05 |
| [kv-storage-selection.md](reports/completed/kv-storage-selection.md) | KV 存储选型 | 2025-02-05 |

---

## 历史归档 (2025)

2025 年的历史 MVP 阶段文档已归档至 [`archive/2025/`](archive/2025/)。

归档内容：
- MVP0-MVP5 完成报告
- 技术选型决策
- E2E 测试结果
- Web UI 实现
- CLI 实现
- Docker 部署

---

## 文档命名规范

### 进度文档 (`docs/progress/`)
- 格式: `{YYYY-MM-DD}-{topic}.md`
- 状态: 进行中的工作
- 完成后移至 `docs/reports/completed/`

### 完成报告 (`docs/reports/completed/`)
- 格式: `{phase-name}-{date}.md` 或 `{topic}-{date}.md`
- 状态: 已完成的阶段
- 保留 2026 年文档，2025 年已归档

### 归档文档 (`docs/archive/{YYYY}/`)
- 格式: 按原始命名保留
- 状态: 历史参考
- 按年份组织

---

## 更新日志

| 日期 | 变更 |
|------|------|
| 2026-02-11 | 创建文档索引，归档 2025 文档，移动 Phase 0-4 至 completed |
| 2026-02-10 | Phase 0-3 完成报告 |
| 2025-02-05 | MVP0-MVP5 完成 |

---

*最后更新: 2026-02-11*
