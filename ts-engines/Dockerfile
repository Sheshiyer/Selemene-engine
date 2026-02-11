# TypeScript Engines Dockerfile
# Uses Bun runtime for high performance

# Build stage
FROM oven/bun:1-alpine AS builder

WORKDIR /build

# Copy package files
COPY package.json bun.lock* ./

# Install dependencies
RUN bun install --frozen-lockfile --production

# Copy source code
COPY . .

# Build if needed (TypeScript compilation)
# RUN bun run build

# Production stage
FROM oven/bun:1-alpine AS runtime

WORKDIR /app

# Create non-root user (skip if group/user already exists)
RUN addgroup -g 1001 appgroup 2>/dev/null || true && \
    adduser -u 1001 -G appgroup -s /bin/sh -D appuser 2>/dev/null || true

# Copy dependencies and source from builder
COPY --from=builder /build/node_modules ./node_modules
COPY --from=builder /build/package.json ./
COPY --from=builder /build/src ./src
COPY --from=builder /build/tsconfig.json ./

# Set environment variables
ENV NODE_ENV=production \
    PORT=3001

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 3001

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD wget -qO- http://localhost:3001/health || exit 1

# Run the application
CMD ["bun", "run", "src/index.ts"]
