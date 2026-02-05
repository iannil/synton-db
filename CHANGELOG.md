# Changelog

All notable changes to SYNTON-DB will be documented in this file.

## [0.1.0] - 2025-02-05

### Added
- **Core Data Model**
  - Node, Edge, Relation, Filter, Path types
  - Builder pattern for Node and Edge construction
  - 4 node types: Entity, Concept, Fact, RawChunk
  - 7 relation types: is_a, is_part_of, causes, similar_to, contradicts, happened_after, belongs_to

- **Storage Layer**
  - RocksDB integration for key-value storage
  - Lance integration for vector storage
  - Column family support for data organization
  - Async operations with tokio

- **Vector Index**
  - In-memory vector index implementation
  - Cosine similarity calculation
  - Support for variable dimension vectors

- **Graph Algorithms**
  - BFS/DFS traversal
  - Path finding between nodes
  - Confidence scoring
  - Graph paths and reasoning paths support

- **Graph-RAG**
  - Hybrid retrieval combining vector search and graph traversal
  - Configurable vector/graph weights
  - Context size management
  - Node relevance scoring and sorting

- **PaQL (Prompt as Query Language)**
  - Natural language query parser using Nom
  - Support for text, semantic, hybrid, and traversal queries
  - Boolean operations (AND, OR, NOT)
  - Filter expressions

- **Memory Management**
  - Forgetting curve implementation (Exponential, Power Law, Linear)
  - Access score tracking
  - Memory decay calculation
  - Configurable retention thresholds

- **REST API**
  - Health check endpoint
  - Node CRUD operations
  - Edge creation
  - Query execution
  - Statistics endpoint

- **gRPC API**
  - Full gRPC service implementation
  - Protocol buffer definitions
  - Unary and streaming operations

- **Server Binary** (`synton-db-server`)
  - Dual gRPC + REST server
  - TOML configuration file support
  - Environment variable overrides
  - Graceful shutdown handling
  - Structured logging (text/JSON)

- **CLI Tool** (`synton-cli`)
  - Node operations: create, get, list, delete
  - Edge operations: create, list
  - Query operations: execute
  - Stats command
  - Export/Import (JSON format)
  - Text and JSON output formats

- **Docker Deployment**
  - Multi-stage Dockerfile (Alpine-based)
  - docker-compose configuration with:
    - synton-db service
    - Prometheus monitoring
    - Grafana dashboards
  - Independent network configuration
  - Production-ready configuration

- **E2E Testing**
  - 39 test cases covering:
    - Node operations (6 tests)
    - Edge operations (4 tests)
    - Query operations (5 tests)
    - Statistics and health (5 tests)
    - API response format validation (8 tests)
    - Concurrency and edge cases (11 tests)
  - Playwright test framework
  - API client helper library

### Changed
- Optimized release build configuration
- Reduced binary size with LTO and strip

### Known Issues
- `/stats` endpoint returns cached data, not real-time counts
- `/nodes` list may have pagination limits for large datasets

### Technical Details
- Language: Rust 1.75+
- Async runtime: Tokio
- Storage: RocksDB, Lance
- Vector operations: Custom implementation
- Parsing: Nom combinator parsers
- Networking: Axum (REST), Tonic (gRPC)
- CLI: Clap
- Testing: Playwright (E2E)

---

## [Unreleased]

### Planned
- ML embedding model integration (Candle/ONNX)
- Distributed storage support
- WebUI console
- Enhanced PaQL syntax features
