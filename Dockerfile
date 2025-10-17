# Standard Dockerfile (Cloud Deployment - No Hardware)
# Multi-stage build for minimal runtime image

FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig

WORKDIR /build

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY benches ./benches

# Build release binary
RUN cargo build --release --bin mcp-utc-time-server

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies only
RUN apk add --no-cache \
    ca-certificates \
    tzdata \
    libgcc

# Create non-root user
RUN addgroup -g 1000 mcpuser && \
    adduser -D -u 1000 -G mcpuser mcpuser

# Copy binary from builder
COPY --from=builder /build/target/release/mcp-utc-time-server /usr/local/bin/

# Set ownership
RUN chown mcpuser:mcpuser /usr/local/bin/mcp-utc-time-server

# Switch to non-root user
USER mcpuser

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1

# Run server
CMD ["/usr/local/bin/mcp-utc-time-server"]
