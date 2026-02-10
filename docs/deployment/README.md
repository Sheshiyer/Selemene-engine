# Selemene Engine Deployment Guide

## Overview

This guide covers running and deploying Selemene Engine in a platform-agnostic way.

## Prerequisites

- **Rust 1.75+** - [Install Rust](https://rustup.rs/)
- **Git** - [Install Git](https://git-scm.com/)

## Local Development

### Quick Start

1. **Clone the repository**
   ```bash
   git clone https://github.com/Sheshiyer/Selemene-engine.git
   cd Selemene-engine
   ```

2. **Set environment variables**
   ```bash
   export RUST_LOG=debug
   export PORT=8080
   # Optional integrations (enable if you have these services available)
   # export REDIS_URL=redis://localhost:6379
   # export DATABASE_URL=postgresql://postgres:password@localhost:5432/selemene
   # export SWISS_EPHEMERIS_PATH=./data/ephemeris
   ```

3. **Start the engine**
   ```bash
   cargo run
   ```

4. **Verify**
   ```bash
   curl http://localhost:8080/health/live
   ```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_ENV` | Environment (`development` / `production`) | `development` |
| `RUST_LOG` | Logging level | `info` |
| `HOST` | Bind host | `0.0.0.0` |
| `PORT` | Bind port | `8080` |
| `JWT_SECRET` | JWT signing key (required) | — |
| `DATABASE_URL` | PostgreSQL connection URL (optional — runs degraded without DB) | — |
| `REDIS_URL` | Redis connection URL (optional — in-memory fallback) | — |
| `SENTRY_DSN` | Sentry error tracking DSN (optional) | — |
| `SWISS_EPHEMERIS_PATH` | Path to Swiss Ephemeris data | `/app/data/ephemeris` |
| `FREE_ASTROLOGY_API_KEY` | FreeAstrologyAPI.com key (optional) | — |

## Deployment

### Build a Release Binary

```bash
cargo build --release
./target/release/noesis-server
```

### Containerization

This repository does not ship provider-specific deployment assets. If you want to run in containers, create your own Dockerfile (or equivalent) that:

- Builds the release binary
- Copies the `data/` directory if you need ephemeris files
- Sets `PORT` and any required environment variables

## CI/CD Pipeline

### GitHub Actions

The project includes automated CI/CD workflows:

#### Test Workflow (`.github/workflows/test.yml`)
- Runs on push to `main` and `develop`
- Executes unit, integration, and performance tests
- Runs security audits with `cargo audit`
- Checks code quality with Clippy and rustfmt
- Builds release binary and checks size

### Automated Testing

```bash
# Run all tests
cargo test --all-features

# Run integration tests
cargo test --test integration

# Run performance tests
cargo test --test performance --release

# Run benchmarks
cargo bench
```

### Security Scanning

```bash
# Security audit
cargo audit

# Dependency vulnerability check
cargo audit --deny warnings
```

## Monitoring and Observability

### Prometheus Metrics

The engine exposes Prometheus metrics at `/metrics`:

- **Request metrics**: Total requests, duration, success rate
- **Calculation metrics**: Engine usage, backend selection
- **Cache metrics**: Hit rates, miss rates, performance
- **System metrics**: Memory usage, CPU usage, uptime

### Grafana Dashboards

Pre-configured dashboards include:

- **Selemene Engine Dashboard**: Core metrics and performance
- **Custom dashboards**: Application-specific monitoring

### Health Checks

Health check endpoints:

- `/health/live`: Liveness probe — always returns 200
- `/health/ready`: Readiness probe — checks DB, Redis, orchestrator
- `/metrics`: Prometheus metrics

## Performance Optimization

### Local Benchmarking

```bash
# Run performance benchmarks
./scripts/benchmark.sh
```

## Troubleshooting

### Common Issues

1. **Build failures**
   ```bash
   # Clean and rebuild
   cargo clean
   cargo build --release
   ```

2. **Dependency issues**
   ```bash
   # Update dependencies
   cargo update
   cargo build
   ```

3. **Database connection issues**
   ```bash
   # Verify DATABASE_URL and connectivity (example for psql)
   psql "$DATABASE_URL" -c 'select 1'
   ```

4. **Redis connection issues**
   ```bash
   # Test Redis connection
   redis-cli ping
   ```

### Logs and Debugging

```bash
# Set debug logging
export RUST_LOG=debug
```

### Performance Issues

1. **Check cache hit rates**
   ```bash
   curl http://localhost:8080/api/v1/cache/stats
   ```

2. **Monitor system resources**
   ```bash
   # Use your platform tooling (top/htop, container metrics, etc.)
   top
   ```

3. **Run performance benchmarks**
   ```bash
   ./scripts/benchmark.sh
   ```

## Scaling

### Horizontal Scaling

The engine is designed to scale horizontally behind a load balancer. Use your hosting platform’s autoscaling primitives and ensure any shared state is externalized (e.g., Redis for shared cache layers).

### Load Balancing

The engine supports multiple instances behind a load balancer:

- Stateless design for horizontal scaling
- Shared Redis cache for session data
- Database connection pooling

## Security

### Authentication

- JWT token-based authentication
- API key management
- Rate limiting per user tier
- Permission-based access control

### Network Security

- HTTPS enforcement in production
- CORS configuration
- Rate limiting middleware
- Input validation and sanitization

### Secrets Management

- Environment variable-based configuration
- No hardcoded secrets in code

## Backup and Recovery

### Database Backups

```bash
# Create backup (example)
pg_dump "$DATABASE_URL" > backup.sql

# Restore backup (example)
psql "$DATABASE_URL" < backup.sql
```

### Configuration Backups

- Version control for configuration files
- Environment-specific configurations
- Backup of Swiss Ephemeris data

## Support and Maintenance

### Regular Maintenance

1. **Dependency updates**
   ```bash
   cargo update
   cargo audit
   ```

2. **Security patches**
   ```bash
   cargo audit --deny warnings
   ```

3. **Performance monitoring**
   - Monitor cache hit rates
   - Track response times
   - Monitor resource usage

### Support Resources

- **GitHub Issues**: [https://github.com/Sheshiyer/Selemene-engine/issues](https://github.com/Sheshiyer/Selemene-engine/issues)
