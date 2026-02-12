# Multi-stage build for noesis-api
# Builder stage
FROM rust:1.75 AS builder

WORKDIR /build

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy all crates source code
COPY crates/ ./crates/

# Build release binary
RUN cargo build --release --bin noesis-server

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies + curl for healthcheck
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /build/target/release/noesis-server /app/noesis-server

# Copy all data directories in one pass
COPY data/ /app/data/

# Set environment variables
ENV RUST_LOG=info
ENV DATA_PATH=/app/data

# Expose API port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Set entrypoint
ENTRYPOINT ["./noesis-server"]
