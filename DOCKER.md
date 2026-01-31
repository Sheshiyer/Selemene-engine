# Docker Deployment Guide - Noesis API

## Overview
This directory contains Docker configuration for running the Noesis API with all dependencies (Redis, PostgreSQL) in a containerized environment.

## Files
- `Dockerfile`: Multi-stage build configuration for noesis-api
- `docker-compose.yml`: Orchestration for API + Redis + Postgres
- `.dockerignore`: Build optimization (excludes unnecessary files)
- `.env.example`: Environment variable template

## Quick Start

### 1. Prerequisites
```bash
# Ensure Docker and Docker Compose are installed
docker --version
docker-compose --version
```

### 2. Environment Setup
```bash
# Copy environment template
cp .env.example .env

# Edit .env with your configuration
# IMPORTANT: Change JWT_SECRET and POSTGRES_PASSWORD in production!
nano .env
```

### 3. Build and Run
```bash
# Build and start all services
docker-compose up --build

# Or run in detached mode
docker-compose up -d --build
```

### 4. Verify Deployment
```bash
# Check service health
curl http://localhost:8080/health

# Expected response:
# {"status":"healthy","timestamp":"..."}

# View logs
docker-compose logs -f noesis-api

# Check all services
docker-compose ps
```

## Service Access

| Service | Host Access | Container Network |
|---------|-------------|-------------------|
| Noesis API | http://localhost:8080 | noesis-api:8080 |
| PostgreSQL | localhost:5432 | postgres:5432 |
| Redis | localhost:6379 | redis:6379 |

## Common Operations

### Start Services
```bash
docker-compose up -d
```

### Stop Services
```bash
docker-compose down
```

### Stop and Remove Volumes (Clean Slate)
```bash
docker-compose down -v
```

### View Logs
```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f noesis-api
docker-compose logs -f postgres
docker-compose logs -f redis
```

### Rebuild After Code Changes
```bash
docker-compose up -d --build noesis-api
```

### Execute Commands in Container
```bash
# Access noesis-api shell
docker-compose exec noesis-api /bin/bash

# Access PostgreSQL
docker-compose exec postgres psql -U noesis_user -d noesis

# Access Redis CLI
docker-compose exec redis redis-cli
```

## Image Details

### Builder Stage
- Base: `rust:1.75`
- Compiles release binary with optimizations
- Build flags: LTO enabled, stripped binary

### Runtime Stage
- Base: `debian:bookworm-slim`
- Minimal runtime dependencies (ca-certificates, libssl3)
- Target size: ~200-400MB (depending on data files)

### Included Data
- Swiss Ephemeris data: `/app/data/ephemeris/`
- Wisdom docs: `/app/data/wisdom-docs/`
- Constants: `/app/data/constants/`

## Environment Variables

Key variables (see `.env.example` for full list):

```bash
# Server
SERVER_PORT=8080
RUST_LOG=info

# Database
DATABASE_URL=postgresql://user:pass@postgres:5432/noesis

# Redis
REDIS_URL=redis://redis:6379

# Security
JWT_SECRET=your_secret_key_here

# Data
SWISS_EPHEMERIS_PATH=/app/data/ephemeris
```

## Health Checks

All services have health checks configured:

- **noesis-api**: `curl http://localhost:8080/health`
  - Interval: 30s, Timeout: 10s, Start period: 40s
  
- **postgres**: `pg_isready`
  - Interval: 10s, Timeout: 5s, Start period: 10s
  
- **redis**: `redis-cli ping`
  - Interval: 10s, Timeout: 5s, Start period: 10s

## Volumes

Persistent data volumes:

- `redis-data`: Redis cache persistence
- `postgres-data`: PostgreSQL database
- `ephemeris-data`: Swiss Ephemeris data files (optional)

### Backup Volumes
```bash
# Backup PostgreSQL
docker-compose exec postgres pg_dump -U noesis_user noesis > backup.sql

# Restore PostgreSQL
cat backup.sql | docker-compose exec -T postgres psql -U noesis_user noesis
```

## Production Deployment

### Security Checklist
- [ ] Change `JWT_SECRET` to a strong random value
- [ ] Change `POSTGRES_PASSWORD` to a strong password
- [ ] Set `RUST_LOG=warn` or `error` for production
- [ ] Remove port bindings for internal services (postgres, redis)
- [ ] Use secrets management (Docker Secrets, Vault, etc.)
- [ ] Enable HTTPS/TLS termination (nginx, traefik)
- [ ] Configure firewall rules
- [ ] Set up log aggregation
- [ ] Enable backup strategy

### Resource Limits
Add to `docker-compose.yml` under each service:

```yaml
deploy:
  resources:
    limits:
      cpus: '2'
      memory: 2G
    reservations:
      cpus: '1'
      memory: 1G
```

## Troubleshooting

### Build Fails
```bash
# Clear build cache
docker-compose build --no-cache

# Check Dockerfile syntax
docker-compose config
```

### Service Won't Start
```bash
# Check logs
docker-compose logs noesis-api

# Check health status
docker-compose ps

# Verify environment
docker-compose exec noesis-api env | grep -E 'REDIS|POSTGRES|DATA'
```

### Connection Issues
```bash
# Test Redis connection
docker-compose exec noesis-api curl redis:6379

# Test PostgreSQL connection
docker-compose exec noesis-api nc -zv postgres 5432

# Check network
docker network inspect selemene-engine_noesis-network
```

### Performance Issues
```bash
# Monitor resource usage
docker stats

# Check container logs for errors
docker-compose logs --tail=100 noesis-api
```

## Development Workflow

### Local Development with Hot Reload
For active development, use cargo directly instead of Docker:

```bash
# Start only dependencies
docker-compose up -d redis postgres

# Run API locally with hot reload
cargo watch -x "run --bin noesis-server"
```

### Testing in Docker
```bash
# Build test image
docker-compose build noesis-api

# Run tests in container
docker-compose run --rm noesis-api cargo test

# Run specific test suite
docker-compose run --rm noesis-api cargo test --test integration_tests
```

## Architecture

```
┌─────────────────────────────────────────┐
│         Noesis API Container            │
│  ┌─────────────────────────────────┐   │
│  │   noesis-server (port 8080)     │   │
│  │                                 │   │
│  │  - Axum HTTP Server             │   │
│  │  - Calculation Orchestrator     │   │
│  │  - Engine Bridge                │   │
│  │  - Authentication               │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
           │                    │
           ▼                    ▼
    ┌──────────┐         ┌──────────┐
    │  Redis   │         │ Postgres │
    │  (Cache) │         │   (DB)   │
    └──────────┘         └──────────┘
```

## Support

For issues or questions:
1. Check logs: `docker-compose logs -f noesis-api`
2. Verify health: `curl http://localhost:8080/health`
3. Review configuration: `docker-compose config`
4. Consult main project documentation: `README.md`

## Build Time

Typical build times:
- Initial build: 10-15 minutes (downloading deps + compilation)
- Rebuild (no cache): 8-12 minutes
- Rebuild (with cache): 2-5 minutes
- Incremental rebuild: 1-2 minutes

## Image Size

Target image sizes:
- Builder stage: ~2-3GB (temporary)
- Runtime image: ~200-400MB (final)
- With full ephemeris data: ~300-500MB

Optimize further by:
- Using Alpine Linux base (may require musl compilation)
- Stripping debug symbols (already enabled)
- Minimizing data file inclusions
