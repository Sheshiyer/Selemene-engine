# Multi-stage build for noesis-api
# Builder stage
FROM rust:1.75 AS builder

WORKDIR /build

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy all crates source code
COPY crates/ ./crates/

# Copy additional source if exists
COPY src/ ./src/ 2>/dev/null || true

# Build release binary
RUN cargo build --release --bin noesis-server

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /build/target/release/noesis-server /app/noesis-server

# Copy Swiss Ephemeris data files (if exists)
COPY data/ephemeris/ /app/data/ephemeris/ 2>/dev/null || mkdir -p /app/data/ephemeris

# Copy wisdom-docs JSON files
COPY data/wisdom-docs/ /app/data/wisdom-docs/

# Copy additional data directories that may be needed
COPY data/constants/ /app/data/constants/ 2>/dev/null || true
COPY data/validation/ /app/data/validation/ 2>/dev/null || true

# Set environment variables
ENV RUST_LOG=info
ENV DATA_PATH=/app/data

# Expose API port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Install curl for healthcheck
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl && \
    rm -rf /var/lib/apt/lists/*

# Set entrypoint
ENTRYPOINT ["./noesis-server"]
