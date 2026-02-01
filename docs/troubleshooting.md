# Troubleshooting Guide

This guide covers common issues and their solutions when running Selemene Engine.

---

## 1. Swiss Ephemeris File Not Found

### Symptoms
```
Error: Ephemeris data not found at path: /app/data/ephemeris
Error: swe_set_ephe_path failed
```

### Causes
- Ephemeris data files not downloaded
- Incorrect path configuration
- Volume mount not working (Docker)

### Solutions

**1. Download ephemeris files:**
```bash
mkdir -p data/ephemeris
cd data/ephemeris

# Download required files (minimal set)
curl -O https://www.astro.com/ftp/swisseph/ephe/sepl_18.se1
curl -O https://www.astro.com/ftp/swisseph/ephe/semo_18.se1
curl -O https://www.astro.com/ftp/swisseph/ephe/seas_18.se1

# Or download full set for extended date ranges
# See: https://www.astro.com/ftp/swisseph/ephe/
```

**2. Verify environment variable:**
```bash
# Check path
echo $SWISS_EPHEMERIS_PATH

# Set correctly
export SWISS_EPHEMERIS_PATH=/path/to/data/ephemeris
```

**3. Docker volume mount:**
```yaml
volumes:
  - ./data/ephemeris:/app/data/ephemeris:ro
```

**4. Verify files exist:**
```bash
ls -la $SWISS_EPHEMERIS_PATH
# Should show .se1 files
```

---

## 2. Redis Connection Failed

### Symptoms
```
Error: Connection refused (os error 111)
Error: Could not connect to Redis at redis:6379
L2 cache unavailable, falling back to L1 only
```

### Causes
- Redis not running
- Incorrect connection URL
- Network/firewall issues
- Docker network misconfiguration

### Solutions

**1. Check Redis is running:**
```bash
# Local
redis-cli ping
# Should return PONG

# Docker
docker-compose ps redis
docker-compose logs redis
```

**2. Verify connection URL:**
```bash
# Test connection
redis-cli -h <host> -p <port> ping

# Check environment variable
echo $REDIS_URL
# Expected: redis://localhost:6379 or redis://redis:6379
```

**3. Docker network issues:**
```bash
# Ensure services are on same network
docker network inspect noesis-network

# Restart with network
docker-compose down && docker-compose up -d
```

**4. Firewall check:**
```bash
# Check port is accessible
nc -zv localhost 6379
```

**Note:** The system degrades gracefully - L1 cache still works without Redis.

---

## 3. TypeScript Engine Timeout

### Symptoms
```
Error: TS engine bridge timeout after 30s
Error: Connection refused to ts-engines:3001
Workflow returned partial results (TS engines missing)
```

### Causes
- TS engines not running
- Incorrect bridge URL
- Network issues
- TS engine crashed

### Solutions

**1. Start TS engines:**
```bash
# Local development
cd ts-engines
bun install
bun run dev

# Docker
docker-compose up -d ts-engines
docker-compose logs ts-engines
```

**2. Verify health:**
```bash
curl http://localhost:3001/health
# Should return {"status":"healthy"}
```

**3. Check environment variable:**
```bash
echo $TS_ENGINES_URL
# Expected: http://localhost:3001 or http://ts-engines:3001
```

**4. Increase timeout (if engines are slow):**
```bash
export TS_ENGINE_TIMEOUT_SECONDS=60
```

**5. Check for errors in TS engine logs:**
```bash
docker-compose logs -f ts-engines
```

---

## 4. Cache Miss Rate High

### Symptoms
- Slow response times
- High CPU usage
- Metrics show cache hit rate <50%

### Causes
- Cache size too small
- TTL too short
- Cache key collisions
- Requests with unique parameters

### Solutions

**1. Increase L1 cache size:**
```bash
# Default is 256MB, increase to 1GB
export CACHE_L1_SIZE=1073741824
```

**2. Adjust TTL:**
```bash
# Increase L1 TTL (default 1 hour)
export CACHE_L1_TTL=7200  # 2 hours

# Increase L2 TTL (default 24 hours)
export CACHE_L2_TTL=172800  # 48 hours
```

**3. Monitor cache stats:**
```bash
curl http://localhost:8080/api/v1/cache/stats
```

**4. Verify cache is being used:**
- Check `X-Cache-Status` header in responses
- Review metrics: `noesis_cache_hits_total`, `noesis_cache_misses_total`

---

## 5. Authentication Errors

### Symptoms
```
401 Unauthorized
{"error": "Invalid or expired token"}
{"error": "Missing authorization header"}
```

### Causes
- Token expired
- Invalid JWT secret
- Malformed token
- Missing header

### Solutions

**1. Check token format:**
```bash
# Header should be:
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
```

**2. Refresh expired token:**
```bash
curl -X POST http://localhost:8080/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{"refresh_token": "rt_xxx"}'
```

**3. Verify JWT_SECRET matches:**
```bash
# Ensure same secret is used for signing and verification
echo $JWT_SECRET
```

**4. Decode and inspect token:**
```bash
# Use jwt.io or:
echo "eyJhbGciOiJIUzI1NiIs..." | cut -d'.' -f2 | base64 -d
```

**5. Check token expiration:**
```bash
curl http://localhost:8080/api/v1/auth/validate \
  -H "Authorization: Bearer $TOKEN"
```

---

## 6. Rate Limiting Issues

### Symptoms
```
429 Too Many Requests
{"error": "Rate limit exceeded. Try again in 45 seconds."}
```

### Causes
- Exceeding tier limits
- Burst traffic
- No caching on client side
- Misconfigured limits

### Solutions

**1. Check current limits:**
- Review `X-RateLimit-*` headers in responses
- Check your tier allocation

**2. Implement client-side caching:**
```javascript
// Cache responses for repeated identical requests
const cache = new Map();
```

**3. Add retry logic with backoff:**
```javascript
async function fetchWithRetry(url, options, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    const response = await fetch(url, options);
    if (response.status !== 429) return response;
    
    const retryAfter = response.headers.get('Retry-After') || 60;
    await sleep(retryAfter * 1000 * Math.pow(2, i));
  }
  throw new Error('Rate limit exceeded after retries');
}
```

**4. Adjust server-side limits (admin):**
```bash
export RATE_LIMIT_REQUESTS=200
export RATE_LIMIT_WINDOW=60
```

---

## 7. Calculation Accuracy Concerns

### Symptoms
- Results differ from reference software
- Planetary positions seem off
- Panchanga elements don't match expected values

### Causes
- Ayanamsa settings
- Timezone handling
- Coordinate precision
- Different calculation methods

### Solutions

**1. Verify input precision:**
```json
{
  "birth_data": {
    "latitude": 40.712800,   // Use 6 decimal places
    "longitude": -74.006000,
    "timezone": "America/New_York"  // Use IANA timezone
  }
}
```

**2. Check ayanamsa setting:**
```bash
# Selemene uses Lahiri ayanamsa by default
# Verify if reference uses same
```

**3. Compare with Swiss Ephemeris directly:**
```bash
# Use Swiss Ephemeris command line tool
swetest -b15.3.1990 -ut14:30 -p0123456789 -fPZS -g, -head
```

**4. Use validation endpoint:**
```bash
curl -X POST http://localhost:8080/api/v1/panchanga/validate \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"...": "..."}'
```

**5. Request High precision:**
```json
{
  "precision": "High"
}
```

---

## 8. Memory Issues

### Symptoms
```
Error: Out of memory
Container OOMKilled
Slow response times with high memory usage
```

### Causes
- L1 cache too large
- Memory leak
- Too many concurrent connections
- Large batch requests

### Solutions

**1. Reduce L1 cache size:**
```bash
export CACHE_L1_SIZE=134217728  # 128MB instead of 256MB
```

**2. Set container memory limits:**
```yaml
deploy:
  resources:
    limits:
      memory: 4G
```

**3. Monitor memory usage:**
```bash
# Docker
docker stats

# Kubernetes
kubectl top pods -n noesis
```

**4. Enable memory profiling:**
```bash
export RUST_LOG=debug
# Check for memory patterns in logs
```

**5. Limit concurrent requests:**
```bash
export MAX_CONNECTIONS=100
```

---

## 9. Log Analysis Tips

### View Logs

```bash
# Docker Compose
docker-compose logs -f noesis-api

# Kubernetes
kubectl logs -f deployment/noesis-api -n noesis

# Filter by level
docker-compose logs noesis-api 2>&1 | grep "ERROR"
```

### Parse JSON Logs

```bash
# Extract specific fields
docker-compose logs noesis-api | jq -r 'select(.level == "ERROR") | .message'

# Count errors by type
docker-compose logs noesis-api | jq -r 'select(.level == "ERROR") | .error_code' | sort | uniq -c
```

### Common Log Patterns

| Pattern | Meaning |
|---------|---------|
| `calculation_time_ms > 100` | Slow calculation |
| `cache_hit: false` | Cache miss |
| `backend: "swiss"` | Using Swiss Ephemeris fallback |
| `ts_bridge_error` | TS engine communication issue |

---

## 10. Performance Troubleshooting

### Symptoms
- Slow API responses
- High latency on calculations
- Timeouts under load

### Diagnostic Steps

**1. Check metrics:**
```bash
curl http://localhost:8080/metrics | grep noesis_request_duration
```

**2. Profile specific endpoint:**
```bash
# Time a request
time curl -X POST http://localhost:8080/api/v1/engines/human-design/calculate \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"birth_data": {...}}'
```

**3. Check for resource constraints:**
```bash
# CPU/Memory
docker stats
top -c

# Network
netstat -an | grep 8080
```

**4. Enable debug logging:**
```bash
export RUST_LOG=debug,tower_http=trace
```

**5. Run benchmarks:**
```bash
cargo bench
```

### Performance Targets

| Operation | Target | Investigate If |
|-----------|--------|----------------|
| Single engine | <100ms | >500ms |
| Workflow (3 engines) | <500ms | >2s |
| Full spectrum (14 engines) | <2s | >5s |
| Cache hit (L1) | <1ms | >10ms |
| Cache hit (L2) | <10ms | >50ms |

---

## Getting Help

### Collect Debug Information

Before reporting issues, gather:

1. **Environment details:**
```bash
uname -a
cargo --version
docker --version
```

2. **Configuration:**
```bash
env | grep -E "(NOESIS|REDIS|SWISS|CACHE)"
```

3. **Logs:**
```bash
docker-compose logs --tail=100 noesis-api > logs.txt
```

4. **Metrics:**
```bash
curl http://localhost:8080/metrics > metrics.txt
```

5. **Health status:**
```bash
curl http://localhost:8080/health
curl http://localhost:8080/ready
```

### Support Channels

- GitHub Issues: Report bugs with debug information
- Documentation: Check for updates at `/docs/`
- Logs: Include relevant log excerpts

---

**Last Updated**: 2026-01
