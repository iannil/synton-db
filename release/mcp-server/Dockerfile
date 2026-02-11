# Copyright 2025 SYNTON-DB Team
#
# Licensed under the Apache License, Version 2.0 (the "License");

# Stage 1: Build the MCP server
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

WORKDIR /app

# Copy cargo config
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

# Build the MCP server
RUN cargo build --release -p synton-mcp-server

# Stage 2: Create minimal runtime image
FROM alpine:latest

RUN apk add --no-cache ca-certificates

# Copy the binary
COPY --from=builder /app/target/release/synton-mcp-server /usr/local/bin/

# Create a non-root user
RUN addgroup -g 1000 synton && \
    adduser -D -u 1000 -G synton synton && \
    chown -R synton:synton /home/synton

USER synton

ENTRYPOINT ["synton-mcp-server"]
CMD ["--endpoint", "http://synton-db:8080"]
