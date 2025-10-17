# Standard Dockerfile (Cloud Deployment - No Hardware)
# Multi-stage build for minimal runtime image

FROM rust:1.82-bullseye AS builder

# Install build dependencies (Debian-based)
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY benches ./benches

# Build release binary
RUN cargo build --release --bin mcp-utc-time-server

# Runtime stage
FROM debian:bookworm-slim

# Runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    tzdata \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -g 1000 mcpuser && useradd -m -u 1000 -g 1000 mcpuser

# Copy binary from builder
COPY --from=builder /build/target/release/mcp-utc-time-server /usr/local/bin/

# Set ownership
RUN chown mcpuser:mcpuser /usr/local/bin/mcp-utc-time-server

# Switch to non-root user
USER mcpuser

# Expose port
EXPOSE 3000

# Health check (use curl if available; busybox/curl not present by default)
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD /usr/bin/env bash -c 'if curl -sSf http://localhost:3000/health >/dev/null; then exit 0; else exit 1; fi'

# Run server
CMD ["/usr/local/bin/mcp-utc-time-server"]
