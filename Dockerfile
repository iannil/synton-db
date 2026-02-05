# Copyright 2025 SYNTON-DB Team
#
# Licensed under the Apache License, Version 2.0 (the "License");

# SYNTON-DB Dockerfile
# Multi-stage build for minimal production image

#########################################
# Stage 1: Builder
#########################################
FROM rust:1.83-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    git \
    musl-dev \
    pkgconfig \
    openssl-dev \
    clang \
    llvm-dev \
    lld

WORKDIR /build

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build the binary in release mode
RUN cargo build --release -p synton-db

#########################################
# Stage 2: Runtime
#########################################
FROM alpine:3.20

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libgcc

# Create non-root user
RUN addgroup -g 1000 synton && \
    adduser -D -u 1000 -G synton synton

# Create data directories
RUN mkdir -p /data/rocksdb /data/lance && \
    chown -R synton:synton /data

# Copy binary from builder
COPY --from=builder /build/target/release/synton-db-server /usr/local/bin/

# Copy entrypoint and default config
COPY release/docker/entrypoint.sh /entrypoint.sh
COPY release/docker/config.toml /etc/synton-db/config.toml

RUN chmod +x /entrypoint.sh

# Switch to non-root user
USER synton

# Expose ports
EXPOSE 50051 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD wget -q -O- http://localhost:8080/health || exit 1

# Set working directory
WORKDIR /data

# Set default entrypoint
ENTRYPOINT ["/entrypoint.sh"]

# Default command runs the server
CMD ["server"]
