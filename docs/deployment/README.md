# Selemene Engine Deployment Guide

## Overview

This guide covers deploying the Selemene Engine to various environments, from local development to production on Railway.com.

## Prerequisites

- **Rust 1.75+** - [Install Rust](https://rustup.rs/)
- **Docker** - [Install Docker](https://docs.docker.com/get-docker/)
- **Railway CLI** - [Install Railway CLI](https://docs.railway.app/develop/cli)
- **Git** - [Install Git](https://git-scm.com/)

## Local Development

### Quick Start

1. **Clone the repository**
   ```bash
   git clone https://github.com/selemene/selemene-engine.git
   cd selemene-engine
   ```

2. **Set environment variables**
   ```bash
   export RUST_LOG=debug
   export ENVIRONMENT=development
   export HOST=0.0.0.0
   export PORT=8080
   export REDIS_URL=redis://localhost:6379
   export DATABASE_URL=postgresql://postgres:password@localhost:5432/selemene
   ```

3. **Run with Docker Compose**
   ```bash
   docker-compose up -d
   ```

4. **Start the engine**
   ```bash
   cargo run
   ```

5. **Verify deployment**
   ```bash
   curl http://localhost:8080/health
   ```

### Docker Compose Services

The `docker-compose.yml` includes:

- **selemene-engine**: Main application
- **postgres**: PostgreSQL database
- **redis**: Redis cache
- **prometheus**: Metrics collection
- **grafana**: Monitoring dashboard

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Logging level | `info` |
| `ENVIRONMENT` | Environment name | `development` |
| `HOST` | Bind host | `0.0.0.0` |
| `PORT` | Bind port | `8080` |
| `WORKERS` | Number of worker threads | `4` |
| `REDIS_URL` | Redis connection URL | `redis://localhost:6379` |
| `DATABASE_URL` | PostgreSQL connection URL | `postgresql://postgres:password@localhost:5432/selemene` |
| `SWISS_EPHEMERIS_PATH` | Path to Swiss Ephemeris data | `/app/data/ephemeris` |
| `NATIVE_ENGINE_ENABLED` | Enable native engines | `true` |
| `CROSS_VALIDATION_ENABLED` | Enable cross-validation | `true` |
| `CACHE_SIZE_MB` | L1 cache size in MB | `256` |
| `MAX_CONCURRENT_CALCULATIONS` | Max concurrent calculations | `100` |

## Railway.com Deployment

### Staging Environment

1. **Login to Railway**
   ```bash
   railway login
   ```

2. **Deploy to staging**
   ```bash
   ./scripts/deploy-staging.sh
   ```

3. **Verify staging deployment**
   ```bash
   curl https://selemene-staging.railway.app/health
   ```

### Production Environment

1. **Deploy to production**
   ```bash
   ./scripts/deploy-production.sh
   ```

2. **Verify production deployment**
   ```bash
   curl https://api.selemene.io/health
   ```

### Railway Configuration

The `railway.toml` file configures:

- **Build settings**: Dockerfile-based builds
- **Deploy settings**: Health checks, restart policies
- **Environment variables**: Production and staging configurations
- **Service scaling**: Horizontal scaling policies
- **Resource limits**: CPU and memory constraints

## Docker Deployment

### Building the Image

```bash
# Build for local use
docker build -t selemene-engine:latest .

# Build for production
docker build --target runtime -t selemene-engine:prod .
```

### Running the Container

```bash
# Basic run
docker run -p 8080:8080 selemene-engine:latest

# With environment variables
docker run -p 8080:8080 \
  -e RUST_LOG=info \
  -e ENVIRONMENT=production \
  -e REDIS_URL=redis://redis:6379 \
  selemene-engine:latest

# With volume mounts
docker run -p 8080:8080 \
  -v $(pwd)/data:/app/data \
  -v $(pwd)/logs:/app/logs \
  selemene-engine:latest
```

### Multi-stage Dockerfile

The Dockerfile uses multi-stage builds:

1. **Builder stage**: Compiles Rust application
2. **Runtime stage**: Minimal runtime image

Benefits:
- Smaller production images
- Faster builds with caching
- Security through minimal attack surface

## CI/CD Pipeline

### GitHub Actions

The project includes automated CI/CD workflows:

#### Test Workflow (`.github/workflows/test.yml`)
- Runs on push to `main` and `develop`
- Executes unit, integration, and performance tests
- Runs security audits with `cargo audit`
- Checks code quality with Clippy and rustfmt
- Builds release binary and checks size

#### Staging Deployment (`.github/workflows/deploy-staging.yml`)
- Triggers on push to `develop`
- Runs tests and security audits
- Deploys to Railway staging
- Performs post-deployment verification

#### Production Deployment (`.github/workflows/deploy-production.yml`)
- Triggers on push to `main` or release
- Comprehensive testing and validation
- Deploys to Railway production
- Extensive post-deployment verification

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

- `/health`: Basic health status
- `/status`: Detailed system status
- `/metrics`: Prometheus metrics

## Performance Optimization

### Cache Optimization

```bash
# Run cache optimization
curl -X POST https://api.selemene.io/api/v1/performance/optimize

# Run benchmarks
curl -X POST https://api.selemene.io/api/v1/performance/benchmark
```

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
   # Check database status
   docker-compose ps postgres
   
   # Check logs
   docker-compose logs postgres
   ```

4. **Redis connection issues**
   ```bash
   # Check Redis status
   docker-compose ps redis
   
   # Test Redis connection
   redis-cli ping
   ```

### Logs and Debugging

```bash
# Set debug logging
export RUST_LOG=debug

# View application logs
docker-compose logs -f selemene-engine

# View specific service logs
docker-compose logs -f postgres
docker-compose logs -f redis
```

### Performance Issues

1. **Check cache hit rates**
   ```bash
   curl https://api.selemene.io/api/v1/cache/stats
   ```

2. **Monitor system resources**
   ```bash
   docker stats
   ```

3. **Run performance benchmarks**
   ```bash
   ./scripts/benchmark.sh
   ```

## Scaling

### Horizontal Scaling

Railway.com supports automatic scaling:

```toml
# railway.toml
[services.selemene-api.scaling]
min_instances = 2
max_instances = 10
target_cpu_utilization = 70
```

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
- Railway.com secrets management
- No hardcoded secrets in code

## Backup and Recovery

### Database Backups

```bash
# Create backup
docker-compose exec postgres pg_dump -U postgres selemene > backup.sql

# Restore backup
docker-compose exec -T postgres psql -U postgres selemene < backup.sql
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

- **Documentation**: [https://docs.selemene.io](https://docs.selemene.io)
- **GitHub Issues**: [https://github.com/selemene/selemene-engine](https://github.com/selemene/selemene-engine)
- **Support Email**: support@selemene.io
- **Status Page**: [https://status.selemene.io](https://status.selemene.io)
