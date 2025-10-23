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

# Install runtime dependencies including NTP tools
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    tzdata \
    curl \
    libssl3 \
    ntp \
    && rm -rf /var/lib/apt/lists/*

# Verify ntpq is available, create stub if missing
RUN if ! command -v ntpq &> /dev/null; then \
    echo '#!/bin/sh' > /usr/bin/ntpq && \
    echo 'echo "NTP query tool not available"' >> /usr/bin/ntpq && \
    echo 'exit 1' >> /usr/bin/ntpq && \
    chmod +x /usr/bin/ntpq; \
    fi

# Create non-privileged user
RUN groupadd -g 1000 mcpuser && useradd -m -u 1000 -g 1000 mcpuser

# Copy binary from builder
COPY --from=builder /build/target/release/mcp-utc-time-server /usr/local/bin/mcp-utc-time-server
RUN chmod +x /usr/local/bin/mcp-utc-time-server && \
    chown mcpuser:mcpuser /usr/local/bin/mcp-utc-time-server

# Create NTP config directory
RUN mkdir -p /etc/ntpsec && chown -R mcpuser:mcpuser /etc/ntpsec

# Copy NTP configuration template
COPY config/ntp.conf.template /etc/ntpsec/ntp.conf.template

USER mcpuser
EXPOSE 3000
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD /usr/bin/env bash -c 'if curl -sSf http://localhost:3000/health >/dev/null; then exit 0; else exit 1; fi'
CMD ["/usr/local/bin/mcp-utc-time-server"]
