# SYNTON-DB

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE)

[中文文档](README.zh-CN.md)

---

## Overview

SYNTON-DB is a specialized memory database designed for Large Language Models (LLMs). By combining knowledge graphs with vector retrieval, it provides semantic association, logical reasoning, and dynamic memory capabilities.

Unlike traditional databases (SQL, NoSQL, Vector) that focus on CRUD operations, SYNTON-DB is built on three core principles:

- Ingestion = Understanding - Automatic knowledge graph extraction from input
- Query = Reasoning - Hybrid vector similarity + graph traversal
- Output = Context - Returns preprocessed context packages, not raw data

### What Problem Does It Solve?

Traditional databases store and retrieve data but lack semantic understanding. SYNTON-DB:

1. Understands relationships between entities, not just content similarity
2. Maintains temporal context through memory decay and reinforcement
3. Reasons through multi-hop connections using graph traversal
4. Synthesizes context optimized for LLM consumption

### Key Differentiators

| Feature | Traditional DB | SYNTON-DB |
| --------- | --------------- | ----------- |
| Storage | Tables/Documents/Vector | Tensor-Graph (nodes with vectors + edges with relations) |
| Query | SQL/Vector Search | PaQL (Prompt as Query Language) |
| Retrieval | Similarity-based | Graph-RAG (vector + graph traversal) |
| Memory | Static | Dynamic (decay/reinforcement based on access) |
| Output | Raw rows/columns | Synthesized context packages |

---

## Core Features

### Tensor-Graph Storage

- Nodes contain content with optional vector embeddings
- Edges represent logical relationships (is_a, causes, contradicts, etc.)
- Supports 4 node types: `entity`, `concept`, `fact`, `raw_chunk`
- Supports 7 relation types: `is_a`, `is_part_of`, `causes`, `similar_to`, `contradicts`, `happened_after`, `belongs_to`

### Graph-RAG Hybrid Retrieval

- Combines vector similarity search with multi-hop graph traversal
- Configurable weights for vector vs. graph scoring
- Returns ranked results with confidence scores
- Configurable traversal depth and result limits

### PaQL (Prompt as Query Language)

- Natural language query parser
- Supports logical operators (AND, OR, NOT)
- Supports filters and graph traversal queries
- Optimized for LLM-generated queries

### Memory Decay Mechanism

- Ebbinghaus forgetting curve implementation
- Access score-based retention (0.0-10.0 scale)
- Periodic decay calculation
- Configurable retention thresholds

### ML Embedding Service

- Multiple backend support: Local (Candle), OpenAI, Ollama
- Embedding cache for performance
- Configurable model selection
- CPU/GPU device support

### Dual Protocol APIs

- REST API (Port 8080) - JSON over HTTP
- gRPC API (Port 50051) - High-performance binary protocol
- CORS enabled for web clients

---

## Quick Start

### Docker Compose (Recommended)

```bash
# Clone the repository
git clone https://github.com/synton-db/synton-db.git
cd synton-db

# Start all services (database + monitoring)
docker-compose up -d

# Check service status
docker-compose ps

# View logs
docker-compose logs -f synton-db
```

Services exposed:

- `8080` - REST API
- `50051` - gRPC API
- `9090` - Prometheus metrics
- `3000` - Grafana dashboard

### Build from Source

```bash
# Prerequisites: Rust 1.75+, Git

# Build the server
cargo build --release -p synton-db-server

# Build the CLI tool
cargo build --release -p synton-cli

# Run the server
./target/release/synton-db-server --config config.toml
```

### Verification

```bash
# Health check
curl http://localhost:8080/health

# Get statistics
curl http://localhost:8080/stats
```

---

## CLI Usage

The `synton-cli` tool provides a comprehensive command-line interface.

### Connection Options

```bash
synton-cli --host <host> --port <port> --format <text|json> [command]
```

### Node Operations

```bash
# Create a node
synton-cli node create "Paris is the capital of France" --node-type fact

# Get a node by ID
synton-cli node get <uuid>

# Delete a node (with confirmation)
synton-cli node delete <uuid>

# Delete without confirmation
synton-cli node delete <uuid> --force

# List all nodes
synton-cli node list --limit 100
```

### Edge Operations

```bash
# Create an edge between nodes
synton-cli edge create <source-id> <target-id> --relation is_part_of --weight 0.9

# List edges for a node
synton-cli edge list <node-id> --limit 100
```

### Query Operations

```bash
# Execute a PaQL query
synton-cli query execute "capital city" --limit 10
```

### System Operations

```bash
# Get database statistics
synton-cli stats

# Get detailed statistics
synton-cli stats --detailed

# Export data to JSON
synton-cli export --format json --output backup.json

# Import data from JSON
synton-cli import --format json --input backup.json

# Import with continue-on-error
synton-cli import --format json --input backup.json --continue-on-error
```

---

## API Endpoints

### REST API (Port 8080)

| Endpoint | Method | Description |
| ---------- | -------- | ------------- |
| `/health` | GET | Health check |
| `/stats` | GET | Database statistics |
| `/nodes` | GET | List all nodes |
| `/nodes` | POST | Create a new node |
| `/nodes/:id` | GET | Get node by ID |
| `/nodes/:id` | DELETE | Delete node by ID |
| `/edges` | POST | Create a new edge |
| `/query` | POST | Execute PaQL query |
| `/traverse` | POST | Graph traversal |
| `/bulk` | POST | Bulk operations |

#### Request/Response Examples

Health Check

```bash
curl http://localhost:8080/health
```

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_secs": 0
}
```

Create Node

```bash
curl -X POST http://localhost:8080/nodes \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Paris is the capital of France",
    "node_type": "fact"
  }'
```

```json
{
  "node": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "content": "Paris is the capital of France",
    "node_type": "fact",
    "embedding": null,
    "meta": {
      "created_at": "2025-02-05T10:00:00Z",
      "access_score": 5.0
    }
  },
  "created": true
}
```

Execute Query

```bash
curl -X POST http://localhost:8080/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": "capital",
    "limit": 10,
    "include_metadata": false
  }'
```

```json
{
  "nodes": [...],
  "total_count": 5,
  "execution_time_ms": 12,
  "truncated": false
}
```

Create Edge

```bash
curl -X POST http://localhost:8080/edges \
  -H "Content-Type: application/json" \
  -d '{
    "source": "<uuid-1>",
    "target": "<uuid-2>",
    "relation": "is_part_of",
    "weight": 0.9
  }'
```

Bulk Operations

```bash
curl -X POST http://localhost:8080/bulk \
  -H "Content-Type: application/json" \
  -d '{
    "nodes": [
      {"content": "Node 1", "node_type": "entity"},
      {"content": "Node 2", "node_type": "concept"}
    ],
    "edges": []
  }'
```

### gRPC API (Port 50051)

The gRPC API provides the same functionality with better performance for high-throughput scenarios. See `crates/api/src/grpc.rs` for the Protocol Buffers definition.

---

## Project Structure

```text
synton-db/
├── crates/
│   ├── bin/          # Server binary ✅
│   ├── cli/          # Command-line tool ✅
│   ├── core/         # Core types (Node, Edge, Relation) ✅
│   ├── storage/      # RocksDB + Lance storage ✅
│   ├── vector/       # Vector indexing ✅
│   ├── graph/        # Graph traversal algorithms ✅
│   ├── graphrag/     # Hybrid search implementation ✅
│   ├── paql/         # Query language parser ✅
│   ├── memory/       # Memory decay management ✅
│   ├── ml/           # ML embedding service ✅
│   └── api/          # REST + gRPC API layer ✅
├── e2e/              # End-to-end tests ✅
├── release/          # Release artifacts
│   └── docker/       # Docker configuration files
├── docs/             # Documentation
│   ├── progress/     # Work-in-progress documentation
│   └── reports/      # Completed reports
├── docker-compose.yml
├── Dockerfile
└── Cargo.toml
```

### Architecture Overview

```text
┌─────────────────────────────────────────────────────────────┐
│                     Interface Layer                         │
│  ┌──────────────────┐        ┌──────────────────┐          │
│  │   REST API       │        │    gRPC API      │          │
│  │   (Axum)         │        │    (Tonic)       │          │
│  └──────────────────┘        └──────────────────┘          │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                   Cognitive Compute Layer                   │
│  ┌──────────────────┐  ┌─────────────┐  ┌───────────────┐  │
│  │     PaQL         │  │  Graph-RAG  │  │  Memory Mgmt  │  │
│  │   Parser         │  │   Search    │  │   (Decay)     │  │
│  └──────────────────┘  └─────────────┘  └───────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                  Tensor-Graph Storage Layer                 │
│  ┌──────────────────┐        ┌──────────────────┐          │
│  │    RocksDB       │        │     Lance        │          │
│  │  (Graph Store)   │        │  (Vector Store)  │          │
│  └──────────────────┘        └──────────────────┘          │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                   Infrastructure Layer                       │
│                   Rust + Tokio Runtime                      │
└─────────────────────────────────────────────────────────────┘
```

---

## Configuration

### Configuration File

Create a `config.toml` file or use the default in `release/docker/config.toml`:

```toml
[server]
# Host address to bind to
host = "0.0.0.0"

# gRPC server port
grpc_port = 50051

# REST API server port
rest_port = 8080

# Enable/disable servers
grpc_enabled = true
rest_enabled = true

[storage]
# RocksDB data directory
rocksdb_path = "./data/rocksdb"

# Lance data directory
lance_path = "./data/lance"

# Maximum open files for RocksDB
max_open_files = 5000

# Cache size for RocksDB (in MB)
cache_size_mb = 256

# Enable write-ahead log
wal_enabled = true

[memory]
# Decay scale for the forgetting curve (days)
decay_scale = 20.0

# Retention threshold (0.0-1.0)
retention_threshold = 0.1

# Initial access score for new nodes
initial_access_score = 5.0

# Access score boost per access
access_boost = 0.5

# Enable periodic decay calculation
periodic_decay_enabled = false

# Interval for decay calculation (seconds)
decay_interval_secs = 3600

[logging]
# Log level: trace, debug, info, warn, error
level = "info"

# Enable JSON formatted logs
json_format = false

# Enable tracing output
tracing_enabled = true

[graphrag]
# Maximum depth for graph traversal
max_traversal_depth = 3

# Maximum nodes to return from hybrid search
max_results = 10

# Weight for vector similarity (0.0-1.0)
vector_weight = 0.7

# Weight for graph proximity (0.0-1.0)
graph_weight = 0.3

# Enable confidence scoring
confidence_scoring = true

[ml]
# Enable ML features
enabled = true

# Backend type: local, openai, ollama
backend = "local"

# Local model configuration
local_model = "sentence-transformers/all-MiniLM-L6-v2"
device = "cpu"
max_length = 512

# API configuration (for openai/ollama backends)
api_endpoint = "https://api.openai.com/v1"
api_model = "text-embedding-3-small"
timeout_secs = 30

# Embedding cache
cache_enabled = true
cache_size = 10000
```

### Environment Variables

Configuration can be overridden with environment variables:

| Variable | Description | Default |
| ---------- | ------------- | --------- |
| `SYNTON_SERVER_HOST` | Server bind address | `0.0.0.0` |
| `SYNTON_SERVER_GRPC_PORT` | gRPC port | `50051` |
| `SYNTON_SERVER_REST_PORT` | REST API port | `8080` |
| `SYNTON_STORAGE_ROCKSDB_PATH` | RocksDB data path | `./data/rocksdb` |
| `SYNTON_STORAGE_LANCE_PATH` | Lance data path | `./data/lance` |
| `SYNTON_LOG_LEVEL` | Log level | `info` |

---

## Development

### Prerequisites

- Rust 1.75+
- Node.js 18+ (for E2E tests)
- Docker & Docker Compose (for containerized testing)

### Running Tests

```bash
# Unit tests
cargo test

# Unit tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_add_node

# E2E tests
cd e2e
npm install
npx playwright install
npm test

# E2E tests with visible browser
npm run test:headed

# E2E test report
npm run test:report
```

### Code Quality

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Lint with Clippy
cargo clippy

# Lint with warnings as errors
cargo clippy -- -D warnings

# Generate documentation
cargo doc --open

# Generate documentation for all crates
cargo doc --document-private-items --open
```

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Build specific crate
cargo build -p synton-db-server

# Build with features
cargo build --features all
```

### Docker Development

```bash
# Build Docker image
docker build -t synton-db:dev .

# Run container
docker run -p 8080:8080 -p 50051:50051 synton-db:dev

# Run with custom config
docker run -v $(pwd)/config.toml:/etc/synton-db/config.toml synton-db:dev
```

---

## Design Philosophy

> Traditional databases focus on CRUD, pursuing ACID or CAP.
> Cognitive databases focus on: perception, association, recall, and evolution.

### Ingestion = Understanding

Traditional databases store data as-is. SYNTON-DB automatically:

- Extracts entities and relationships
- Builds knowledge graphs
- Creates semantic embeddings
- Establishes temporal context

### Query = Reasoning

Traditional databases match patterns. SYNTON-DB:

- Combines vector similarity with graph traversal
- Follows logical chains through connected nodes
- Weights results by confidence and relevance
- Returns contextualized information

### Output = Context

Traditional databases return raw rows. SYNTON-DB:

- Synthesizes related information
- Compresses and prioritizes context
- Formats output for LLM consumption
- Maintains provenance and confidence

---

## Roadmap

### Completed ✅

- [x] Core data model (Node, Edge, Relation)
- [x] Storage layer (RocksDB + Lance backends)
- [x] Vector indexing (Lance integration)
- [x] Graph traversal (BFS/DFS algorithms)
- [x] Graph-RAG hybrid retrieval
- [x] PaQL query parser
- [x] Memory decay management
- [x] REST + gRPC dual API
- [x] CLI tool with full feature set
- [x] Docker deployment
- [x] E2E test suite
- [x] Prometheus + Grafana monitoring
- [x] Configuration management
- [x] ML embedding service (Local/OpenAI/Ollama)

### In Progress 🚧

- [ ] Advanced PaQL syntax features
- [ ] Query caching layer

### Planned 📋

- [ ] WebUI console
- [ ] Backup/restore utilities
- [ ] Access control and authentication
- [ ] Distributed storage support
- [ ] Advanced alerting system

---

## Contributing

We welcome contributions! Please follow these guidelines:

1. Code Style: Follow Rust conventions and use `cargo fmt`
2. Tests: Write tests for new features (target 80% coverage)
3. Commits: Use conventional commit format (`feat:`, `fix:`, `docs:`, etc.)
4. Documentation: Update relevant docs for any changes
5. PRs: Provide clear descriptions and link related issues

### Development Workflow

```bash
# 1. Fork and clone the repository
git clone https://github.com/synton-db/synton-db.git

# 2. Create a feature branch
git checkout -b feat/your-feature

# 3. Make changes and test
cargo test
cargo clippy

# 4. Commit with conventional format
git commit -m "feat: add your feature description"

# 5. Push and create PR
git push origin feat/your-feature
```

---

## License

Apache License 2.0

---

## Links

- Repository: [https://github.com/synton-db/synton-db](https://github.com/synton-db/synton-db)
- Documentation: [docs/](./docs/)
- Issues: [https://github.com/synton-db/synton-db/issues](https://github.com/synton-db/synton-db/issues)
- Discussions: [https://github.com/synton-db/synton-db/discussions](https://github.com/synton-db/synton-db/discussions)
