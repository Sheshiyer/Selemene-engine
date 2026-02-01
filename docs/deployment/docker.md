# Docker Deployment Guide

## Prerequisites

- Docker 20.10+
- Docker Compose 2.0+
- 4GB+ available RAM

## Quick Start

### 1. Clone and Configure

```bash
# Clone repository
git clone <repo-url>
cd selemene-engine

# Copy environment template
cp .env.example .env

# Edit configuration
vim .env
```

### 2. Start Services

```bash
# Development mode
docker-compose up -d

# With monitoring stack
docker-compose -f docker-compose.yml -f docker-compose.monitoring.yml up -d

# View logs
docker-compose logs -f noesis-api
```

### 3. Verify

```bash
# Health check
curl http://localhost:8080/health

# Test calculation
curl -X POST http://localhost:8080/api/v1/panchanga/calculate \
  -H "Content-Type: application/json" \
  -d '{"current_time": "2025-01-15T12:00:00Z", "location": {"latitude": 28.6, "longitude": 77.2}}'
```

---

## Docker Compose Services

### Core Services

```yaml
services:
  noesis-api:
    build: .
    ports:
      - "8080:8080"
    depends_on:
      - redis
      - postgres
    environment:
      - REDIS_URL=redis://redis:6379
      - DATABASE_URL=postgresql://user:pass@postgres:5432/noesis

  ts-engines:
    build: ./ts-engines
    ports:
      - "3001:3001"

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data

  postgres:
    image: postgres:16-alpine
    ports:
      - "5432:5432"
    environment:
      - POSTGRES_DB=noesis
      - POSTGRES_USER=noesis_user
      - POSTGRES_PASSWORD=noesis_password
    volumes:
      - postgres-data:/var/lib/postgresql/data
```

### Service Dependencies

```
noesis-api
├── redis (required for L2 cache)
├── postgres (required for metadata)
└── ts-engines (required for TS engines)

ts-engines
└── (no dependencies)
```

---

## Environment Variables

### Server Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level (trace, debug, info, warn, error) |
| `SERVER_HOST` | `0.0.0.0` | Bind address |
| `SERVER_PORT` | `8080` | HTTP port |

### Database Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | - | PostgreSQL connection string |
| `POSTGRES_DB` | `noesis` | Database name |
| `POSTGRES_USER` | `noesis_user` | Database user |
| `POSTGRES_PASSWORD` | - | Database password |

### Cache Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `REDIS_URL` | - | Redis connection string |
| `REDIS_POOL_SIZE` | `10` | Connection pool size |
| `CACHE_L1_SIZE` | `268435456` | L1 cache size (256MB) |
| `CACHE_L1_TTL` | `3600` | L1 TTL in seconds |
| `CACHE_L2_TTL` | `86400` | L2 TTL in seconds |

### Authentication

| Variable | Default | Description |
|----------|---------|-------------|
| `JWT_SECRET` | - | JWT signing secret (required) |
| `JWT_EXPIRY` | `3600` | Token expiry in seconds |

### Ephemeris

| Variable | Default | Description |
|----------|---------|-------------|
| `SWISS_EPHEMERIS_PATH` | `/app/data/ephemeris` | Path to ephemeris files |
| `DATA_PATH` | `/app/data` | General data directory |
| `WISDOM_DOCS_PATH` | `/app/data/wisdom-docs` | Wisdom documents path |

### Rate Limiting

| Variable | Default | Description |
|----------|---------|-------------|
| `RATE_LIMIT_REQUESTS` | `100` | Requests per window |
| `RATE_LIMIT_WINDOW` | `60` | Window in seconds |

### TypeScript Engines

| Variable | Default | Description |
|----------|---------|-------------|
| `TS_ENGINES_URL` | `http://ts-engines:3001` | TS engine bridge URL |

### Observability

| Variable | Default | Description |
|----------|---------|-------------|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | `http://jaeger:4317` | OpenTelemetry endpoint |
| `OTEL_SERVICE_NAME` | `noesis-api` | Service name for tracing |
| `LOG_FORMAT` | `json` | Log format (json, pretty) |
| `ENABLE_METRICS` | `true` | Enable Prometheus metrics |

---

## Production Dockerfile

```dockerfile
# Dockerfile.prod
FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/selemene-engine /usr/local/bin/
COPY --from=builder /app/data /app/data

ENV RUST_LOG=info
ENV SERVER_PORT=8080

EXPOSE 8080
CMD ["selemene-engine"]
```

### Build Production Image

```bash
docker build -f Dockerfile.prod -t selemene-engine:latest .
```

---

## Volume Mounts

### Ephemeris Data

Swiss Ephemeris files (required for accurate calculations):

```yaml
volumes:
  - ./data/ephemeris:/app/data/ephemeris:ro
```

Download from: https://www.astro.com/ftp/swisseph/

### Persistent Storage

```yaml
volumes:
  redis-data:
    driver: local
  postgres-data:
    driver: local
  ephemeris-data:
    driver: local
```

---

## Healthchecks

```yaml
services:
  noesis-api:
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  redis:
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

  postgres:
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U noesis_user -d noesis"]
      interval: 10s
      timeout: 5s
      retries: 5
```

---

## Scaling

### Horizontal Scaling

```bash
# Scale API instances
docker-compose up -d --scale noesis-api=3

# With load balancer (requires additional config)
```

### Resource Limits

```yaml
services:
  noesis-api:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          cpus: '0.5'
          memory: 512M
```

---

## Backup and Restore

### Backup PostgreSQL

```bash
docker-compose exec postgres pg_dump -U noesis_user noesis > backup.sql
```

### Restore PostgreSQL

```bash
cat backup.sql | docker-compose exec -T postgres psql -U noesis_user noesis
```

### Backup Redis

```bash
docker-compose exec redis redis-cli BGSAVE
docker cp noesis-redis:/data/dump.rdb ./redis-backup.rdb
```

---

## Troubleshooting

### Container Won't Start

```bash
# Check logs
docker-compose logs noesis-api

# Check container status
docker-compose ps

# Inspect container
docker inspect noesis-api
```

### Connection Issues

```bash
# Test Redis connectivity
docker-compose exec noesis-api redis-cli -h redis ping

# Test PostgreSQL connectivity
docker-compose exec noesis-api pg_isready -h postgres -U noesis_user
```

### Performance Issues

```bash
# Check resource usage
docker stats

# Increase memory limit
docker-compose up -d --memory 8g
```

---

## Security Considerations

1. **Never commit `.env` files** with secrets
2. **Use Docker secrets** for production credentials
3. **Enable TLS** for production deployments
4. **Restrict network access** using Docker networks
5. **Regularly update** base images

### Using Docker Secrets

```yaml
services:
  noesis-api:
    secrets:
      - jwt_secret
      - db_password

secrets:
  jwt_secret:
    external: true
  db_password:
    external: true
```

---

**Last Updated**: 2026-01
