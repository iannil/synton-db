# Copyright 2025 SYNTON-DB Team
#
# Licensed under the Apache License, Version 2.0 (the "License");

# SYNTON-DB Dockerfile
# Multi-stage build using Debian for better C++ library compatibility

#########################################
# Stage 1: Builder
#########################################
FROM rust:1.83-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    git \
    pkg-config \
    libssl-dev \
    clang \
    libclang-dev \
    llvm-dev \
    lld \
    zstd \
    libzstd-dev \
    protobuf-compiler \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set LIBCLANG_PATH for bindgen
ENV LIBCLANG_PATH=/usr/lib/llvm-14/lib
ENV BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/llvm-14/include

WORKDIR /build

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build all binaries in release mode
RUN cargo build --release -p synton-db -p synton-mcp-server

#########################################
# Stage 2: Main Server Runtime
#########################################
FROM debian:bookworm-slim AS synton-db

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    wget \
    libgcc1 \
    zstd \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r -g 1000 synton && \
    useradd -r -u 1000 -g synton synton

# Create data directories
RUN mkdir -p /data/rocksdb /data/lance && \
    chown -R synton:synton /data

# Copy binary from builder
COPY --from=builder /build/target/release/synton-db-server /usr/local/bin/

# Copy entrypoint and default config
COPY release/docker/entrypoint.sh /entrypoint.sh
COPY release/docker/config.toml /etc/synton-db/config.toml

# Create optional web directory
RUN mkdir -p /usr/local/share/synton-db/web

RUN chmod +x /entrypoint.sh

# Switch to non-root user
USER synton

# Expose ports
EXPOSE 5570 5571

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD wget -q -O- http://localhost:8080/health || exit 1

# Set working directory
WORKDIR /data

# Set default entrypoint
ENTRYPOINT ["/entrypoint.sh"]

# Default command runs the server
CMD ["server"]

#########################################
# Stage 3: MCP Server Runtime
#########################################
FROM debian:bookworm-slim AS synton-mcp

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r -g 1001 mcp && \
    useradd -r -u 1001 -g mcp mcp

# Copy binary from builder
COPY --from=builder /build/target/release/synton-mcp-server /usr/local/bin/

# Switch to non-root user
USER mcp

# Set default environment
ENV SYNTONDB_ENDPOINT=http://synton-db:8080

# Set default entrypoint
ENTRYPOINT ["/usr/local/bin/synton-mcp-server"]

# Default args
CMD ["--endpoint", "http://synton-db:8080"]
