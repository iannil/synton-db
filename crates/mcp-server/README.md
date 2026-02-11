# SYNTON-DB MCP Server

Model Context Protocol (MCP) server for SYNTON-DB cognitive database.

Enables AI coding assistants (Claude Code, Gemini CLI, Cursor, Continue, Windsurf) to use SYNTON-DB as persistent external memory.

## Features

- **Natural Language Query**: Query the database using PaQL (Prompt as Query Language)
- **Knowledge Absorption**: Store information with automatic semantic extraction
- **Graph Traversal**: Explore relationships between concepts
- **Graph-RAG Hybrid Search**: Combines vector similarity with graph traversal
- **Cross-Session Memory**: Persistent knowledge across AI sessions

## Installation

### From Source

```bash
cd /path/to/synton-db
cargo build --release -p synton-mcp-server
```

The binary will be available at `target/release/synton-mcp-server`.

## Configuration

### Claude Code

Add to `~/.config/claude-code/mcp_servers.json`:

```json
{
  "mcpServers": {
    "synton-db": {
      "command": "/path/to/synton-db/target/release/synton-mcp-server",
      "args": ["--endpoint", "http://localhost:8080"]
    }
  }
}
```

### Environment Variables

- `SYNTONDB_ENDPOINT`: SYNTON-DB REST API endpoint (default: `http://localhost:8080`)
- `VERBOSE`: Enable verbose logging
- `TRACE`: Enable trace-level logging

## Usage

### Starting SYNTON-DB

First, ensure SYNTON-DB is running:

```bash
# Using Docker
docker-compose up -d

# Or directly
cargo run -p synton-bin
```

### Starting the MCP Server

```bash
synton-mcp-server --endpoint http://localhost:8080
```

### Available Tools

| Tool | Description |
|------|-------------|
| `synton_absorb` | Store knowledge in the database |
| `synton_query` | Natural language search query |
| `synton_hybrid_search` | Graph-RAG hybrid retrieval |
| `synton_get_node` | Get a node by UUID |
| `synton_traverse` | Traverse the knowledge graph |
| `synton_add_edge` | Create relationship between nodes |
| `synton_stats` | Get database statistics |
| `synton_list_nodes` | List all nodes in database |

### Example Usage

In Claude Code, you can now use SYNTON-DB tools directly:

```
Store information about my project architecture:
@synton_absorb(content="The project uses a microservices architecture with Rust", node_type="concept")

Search for related concepts:
@synton_query(query="microservices architecture")

Explore connections:
@synton_traverse(start_id="<uuid>", max_depth=2)
```

## Development

### Running Tests

```bash
cargo test -p synton-mcp-server
```

### Linting

```bash
cargo clippy -p synton-mcp-server
```

### Formatting

```bash
cargo fmt -p synton-mcp-server
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    AI Coding Assistants                          │
│  (Claude Code | Gemini CLI | Cursor | Continue | Windsurf)     │
└───────────────────────────┬─────────────────────────────────────┘
                            │ MCP Protocol (stdio)
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                   SYNTON-DB MCP Server                           │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │  synton_absorb  │  │  synton_query   │  │  synton_traverse│ │
│  │  synton_get_node│  │  synton_hybrid_ │  │  synton_add_edge│ │
│  │                 │  │     _search     │  │                 │ │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘ │
│           │                    │                     │          │
│           └────────────────────┼──────────────────────────────────┘
│                               ▼
│                    ┌──────────────────────┐
│                    │   SYNTON-DB Core     │
│                    │   (REST API Client)  │
│                    └──────────────────────┘
└─────────────────────────────────────────────────────────────────┘
                            │ HTTP/JSON
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                   SYNTON-DB Server                               │
│              (http://localhost:8080)                             │
└─────────────────────────────────────────────────────────────────┘
```

## License

Apache License 2.0

## See Also

- [SYNTON-DB Documentation](../../../docs)
- [MCP Specification](https://modelcontextprotocol.io)
- [Claude Code MCP Integration](https://code.claude.com/docs/en/mcp)
