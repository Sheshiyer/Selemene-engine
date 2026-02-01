# Comprehensive Test Suite

This directory contains the comprehensive test suite for Selemene Engine created as part of Wave 2 Phase 4 (W2-S8).

## Test Categories

### 1. End-to-End Engine Tests (`tests/e2e/engines/`)
Tests all 14 consciousness engines for valid input, invalid input, edge cases, and error responses.

**Engines covered:**
- Rust: human-design, gene-keys, vimshottari, panchanga, numerology, biorhythm, vedic-clock, biofield, face-reading
- TypeScript: tarot, i-ching, enneagram, sacred-geometry, sigil-forge

**Run:**
```bash
# Copy to noesis-api tests directory and run
cp tests/e2e/engines/e2e_all_engines.rs crates/noesis-api/tests/
cargo test --test e2e_all_engines -- --nocapture
```

### 2. End-to-End Workflow Tests (`tests/e2e/workflows/`)
Tests all 6 workflows for execution, partial execution, and error handling.

**Workflows covered:**
- birth-blueprint
- daily-practice
- decision-support
- self-inquiry
- creative-expression
- full-spectrum

**Run:**
```bash
cp tests/e2e/workflows/e2e_all_workflows.rs crates/noesis-api/tests/
cargo test --test e2e_all_workflows -- --nocapture
```

### 3. Load Tests (`tests/load/k6/`)
k6 load tests for production readiness.

**Scripts:**
- `engine-load.js` - Test individual engines (target: 1000 VUs, p95 < 1s)
- `workflow-load.js` - Test workflows (target: 500 VUs, p95 < 2s)
- `full-spectrum.js` - Test full-spectrum workflow (target: 100 VUs, p95 < 5s)

**Run:**
```bash
# Quick smoke test
./tests/load/run-load-tests.sh quick

# Full load test suite (30+ minutes)
JWT_TOKEN="your-token" ./tests/load/run-load-tests.sh full

# Individual tests
k6 run -e JWT_TOKEN="your-token" tests/load/k6/engine-load.js
```

### 4. Chaos Engineering Tests (`tests/chaos/`)
Tests graceful degradation under failure scenarios.

**Scenarios:**
- Redis connection failure
- TypeScript engine timeout/unavailability
- Swiss Ephemeris file missing
- High CPU/memory pressure
- Edge case inputs

**Run:**
```bash
# All scenarios
./tests/chaos/run-chaos-tests.sh all

# Individual scenarios
./tests/chaos/run-chaos-tests.sh redis
./tests/chaos/run-chaos-tests.sh ts
./tests/chaos/run-chaos-tests.sh ephemeris
./tests/chaos/run-chaos-tests.sh load
./tests/chaos/run-chaos-tests.sh edge
```

### 5. Security Tests (`tests/security/`)
Security penetration tests for authentication, injection, and rate limiting.

**Test files:**
- `auth_bypass.rs` - Authentication bypass attempts
- `injection.rs` - SQL/command/JSON injection tests
- `rate_limit.rs` - Rate limiting verification
- `input_validation.rs` - Malformed input handling

**Run:**
```bash
cp tests/security/*.rs crates/noesis-api/tests/
cargo test --test auth_bypass -- --nocapture
cargo test --test injection -- --nocapture
cargo test --test rate_limit -- --nocapture
cargo test --test input_validation -- --nocapture
```

### 6. Accuracy Validation (`tests/validation/`)
Validates calculation accuracy against reference data.

**Test files:**
- `human_design_accuracy.rs` - HD type, authority, profile, gate validation
- `gene_keys_accuracy.rs` - Key mapping, frequency assessment validation
- `vimshottari_accuracy.rs` - Nakshatra, dasha sequence validation

**Run:**
```bash
cp tests/validation/*_accuracy.rs crates/noesis-api/tests/
cargo test --test human_design_accuracy -- --nocapture
cargo test --test gene_keys_accuracy -- --nocapture
cargo test --test vimshottari_accuracy -- --nocapture
```

## Test Fixtures (`tests/fixtures/`)

- `birth_data.json` - Sample birth data for testing
- `expected_outputs/engine_schemas.json` - Expected response schemas
- `reference_calculations/reference_data.json` - Reference data for accuracy validation

## Test Count Summary

| Category | File | Test Count |
|----------|------|------------|
| E2E Engines | e2e_all_engines.rs | 56 |
| E2E Workflows | e2e_all_workflows.rs | 25 |
| Chaos Tests | chaos_tests.rs | 20 |
| Auth Bypass | auth_bypass.rs | 11 |
| Injection | injection.rs | 10 |
| Rate Limit | rate_limit.rs | 7 |
| Input Validation | input_validation.rs | 22 |
| HD Accuracy | human_design_accuracy.rs | 9 |
| Gene Keys Accuracy | gene_keys_accuracy.rs | 8 |
| Vimshottari Accuracy | vimshottari_accuracy.rs | 10 |
| **Total** | | **~178** |

## Prerequisites

### For Rust Tests
```bash
# Ensure noesis-api builds
cargo build -p noesis-api
```

### For Load Tests
```bash
# Install k6
brew install k6  # macOS
# or see https://k6.io/docs/getting-started/installation/

# Generate test token
cargo run --bin generate_test_credentials
export JWT_TOKEN="<generated-token>"
```

### For Full Testing
```bash
# Start the API server
cargo run --release

# In another terminal, run tests
./tests/load/run-load-tests.sh full
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| JWT_TOKEN | JWT token for authenticated requests | (required) |
| API_URL | Base URL for the API | http://localhost:8080 |
| REDIS_URL | Redis connection URL | (optional) |
| TS_ENGINE_URL | TypeScript engine server URL | http://localhost:3001 |
| JWT_SECRET | JWT signing secret | noesis-dev-secret-... |

## Known Issues

1. **TypeScript engine tests**: Will fail/skip if TS engine server (port 3001) is not running
2. **Load tests**: Require valid JWT token in environment
3. **Chaos tests**: Some tests modify environment variables temporarily
4. **Rate limit tests**: May need adjustment based on configured limits
