# Load Tests

Performance and load testing for the Selemene/Noesis API using [k6](https://k6.io).

## Prerequisites

Install k6:

```bash
# macOS
brew install k6

# Linux (Debian/Ubuntu)
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg \
  --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D68
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" \
  | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update && sudo apt-get install k6

# Docker
docker pull grafana/k6
```

## Test Scripts

| Script | Purpose | VUs | Duration |
|--------|---------|-----|----------|
| `baseline.js` | Performance baseline across all endpoint types | 10-100 | ~95s |
| `smoke-test.js` | Quick sanity check (1 VU, 1 iteration) | 1 | ~5s |
| `scenario1-steady.js` | Steady-state engine calculations | varied | varied |
| `scenario2-spike.js` | Spike traffic pattern | varied | varied |
| `scenario3-workflows.js` | Workflow execution under load | varied | varied |
| `scenario4-cache.js` | Cache hit/miss behavior | varied | varied |
| `scenario5-ratelimit.js` | Rate limiter validation | varied | varied |
| `k6/workflow-load.js` | Heavy workflow load (500 VUs) | 5-500 | ~7m |

## Running the Baseline Test

### Against local server

Start the server first, then run:

```bash
# Start the server
cargo run --bin noesis-server

# In another terminal, run the baseline
k6 run \
  --env BASE_URL=http://localhost:8080 \
  --env JWT_TOKEN=<your-jwt-token> \
  tests/load/baseline.js
```

### Against Railway deployment

```bash
k6 run \
  --env BASE_URL=https://selemene.railway.app \
  --env JWT_TOKEN=<your-jwt-token> \
  tests/load/baseline.js
```

### With Docker

```bash
docker run --rm -i \
  -v $(pwd)/tests/load:/scripts \
  -e BASE_URL=http://host.docker.internal:8080 \
  -e JWT_TOKEN=<your-jwt-token> \
  grafana/k6 run /scripts/baseline.js
```

## Obtaining a JWT Token

Generate a token by registering or logging in:

```bash
# Register a new user
curl -s -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"loadtest@example.com","password":"TestPass123!","name":"Load Test"}' \
  | jq -r '.token'

# Or log in with existing credentials
curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"loadtest@example.com","password":"TestPass123!"}' \
  | jq -r '.token'
```

Export it for convenience:

```bash
export JWT_TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"loadtest@example.com","password":"TestPass123!"}' \
  | jq -r '.token')
```

## Baseline Scenarios

The `baseline.js` test runs four scenarios:

### 1. Health Check (100 VUs, 30s)
- Endpoint: `GET /health`
- No authentication required
- Threshold: p95 < 100ms, error rate < 1%

### 2. Engine List (50 VUs, 30s)
- Endpoint: `GET /api/v1/engines`
- Bearer token authentication
- Threshold: p95 < 200ms, error rate < 1%

### 3. Panchanga Calculate (20 VUs, 60s)
- Endpoint: `POST /api/v1/engines/panchanga/calculate`
- Bearer token authentication, JSON body with birth data
- Threshold: p95 < 500ms, error rate < 1%

### 4. Workflow Execute (10 VUs, 60s)
- Endpoint: `POST /api/v1/workflows/birth-blueprint/execute`
- Bearer token authentication, JSON body with birth data
- Threshold: p95 < 500ms, error rate < 1%

## Interpreting Results

### Console Output

k6 prints a summary table after each run. Key columns:

| Metric | What it means |
|--------|---------------|
| `http_req_duration` | End-to-end latency (DNS + connect + TLS + send + wait + receive) |
| `p(95)` | 95th percentile -- 95% of requests completed within this time |
| `http_req_failed` | Percentage of non-2xx responses |
| `iterations` | Total number of completed VU iterations |
| `vus_max` | Peak concurrent virtual users |

### Threshold Pass/Fail

Thresholds appear with a checkmark or cross:

```
health_latency_ms..........: avg=2.1ms  min=1.2ms  p(95)=4.8ms   p(99)=8.1ms
  { p(95)<100 }............: pass
```

If any threshold fails, k6 exits with code 99.

### JSON Results

Full results are written to `tests/load/results/baseline-results.json`. The structured summary (printed to stdout) contains:

```json
{
  "test": "baseline",
  "timestamp": "2026-02-10T...",
  "scenarios": {
    "health_check": { "latency": { "p95": 4.8 }, "error_rate": 0.0 },
    "engine_list": { "latency": { "p95": 12.3 }, "error_rate": 0.0 },
    "calculate": { "latency": { "p95": 85.2 }, "error_rate": 0.0 },
    "workflow": { "latency": { "p95": 210.5 }, "error_rate": 0.0 }
  },
  "thresholds_passed": true
}
```

### Comparing Baselines

Save each run's JSON output and compare p95 values over time:

```bash
# Run baseline and save
k6 run --env BASE_URL=http://localhost:8080 --env JWT_TOKEN=$JWT_TOKEN \
  tests/load/baseline.js 2>&1 | tee tests/load/results/baseline-$(date +%Y%m%d).log

# Compare p95 values from two runs
jq '.scenarios | to_entries[] | {scenario: .key, p95: .value.latency.p95}' \
  tests/load/results/baseline-results.json
```

### Warning Signs

- **p95 > 2x baseline**: Performance regression -- investigate before merging
- **Error rate > 0.5%**: Stability issue -- check server logs
- **p99 >> p95**: Tail latency problem -- likely GC pauses, DB connection pool exhaustion, or lock contention
- **Declining RPS during sustained phase**: Resource leak or connection exhaustion

## Running the Full Suite

Use the orchestrator script:

```bash
./tests/load/run-load-tests.sh
```

This runs smoke, then all scenarios in sequence, collecting results into `tests/load/results/`.
