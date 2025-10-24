# Multi-stage build for GLIBC compatibility and NTPsec SHM support
FROM rust:1.84-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install nightly Rust for edition2024 support
RUN rustup toolchain install nightly && rustup default nightly

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

# Install runtime dependencies (NTP not needed in container - time comes from host)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    tzdata \
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Set environment variable to indicate container environment
ENV CONTAINER_APP_NAME=mcp-utc-time-server

# Create non-privileged user
RUN groupadd -g 1000 mcpuser && useradd -m -u 1000 -g 1000 mcpuser

# Copy binary from builder
COPY --from=builder /build/target/release/mcp-utc-time-server /usr/local/bin/mcp-utc-time-server
RUN chmod +x /usr/local/bin/mcp-utc-time-server && \
    chown mcpuser:mcpuser /usr/local/bin/mcp-utc-time-server

# Create NTP config directory
RUN mkdir -p /etc/ntpsec && chown -R mcpuser:mcpuser /etc/ntpsec

# Note: NTP daemon is not run in containers
# - Container time comes from the host
# - NTP SHM segments won't be available
# - The code detects container environment and skips NTP checks

USER mcpuser
EXPOSE 3000
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD /usr/bin/env bash -c 'if curl -sSf http://localhost:3000/health >/dev/null; then exit 0; else exit 1; fi'
CMD ["/usr/local/bin/mcp-utc-time-server"]
