# Selemene Engine - Codebase Index

**Generated:** 2026-02-08  
**Project:** Tryambakam Noesis - Consciousness Transformation Platform  
**Status:** Wave 2 Complete (187/187 tasks), FreeAstrologyAPI Integration In Progress

---

## 📊 Project Overview

### Architecture
- **Backend:** Rust (Axum framework)
- **TypeScript Engines:** Bun + Elysia
- **Database:** PostgreSQL + Redis
- **Deployment:** Docker + Kubernetes
- **Monitoring:** Prometheus + Grafana + Loki + Jaeger

### Core Components
1. **16 Consciousness Engines** (11 Rust + 5 TypeScript)
2. **6 Multi-Engine Workflows** (synthesis patterns)
3. **FreeAstrologyAPI Integration** (Vedic calculations)
4. **Production Infrastructure** (K8s, CI/CD, monitoring)

---

## 🗂️ Directory Structure

```
Selemene-engine/
├── crates/                    # Rust workspace crates
│   ├── engine-*/             # 11 Rust consciousness engines
│   ├── noesis-api/           # Main HTTP API server
│   ├── noesis-vedic-api/     # FreeAstrologyAPI client ⭐
│   ├── noesis-western-api/   # Western astrology API
│   ├── noesis-orchestrator/  # Multi-engine coordination
│   ├── noesis-cache/         # L1/L2/L3 caching
│   ├── noesis-auth/          # JWT + API key auth
│   ├── noesis-metrics/       # Prometheus metrics
│   └── noesis-*/             # Supporting crates
├── ts-engines/               # TypeScript engines (Bun)
│   ├── tarot/
│   ├── i-ching/
│   ├── enneagram/
│   ├── sacred-geometry/
│   └── sigil-forge/
├── data/                     # Wisdom data + ephemeris
├── tests/                    # E2E, load, chaos, security tests
├── k8s/                      # Kubernetes manifests
├── monitoring/               # Observability stack configs
├── .context/                 # Project documentation
│   ├── engines/             # Engine specifications
│   ├── reports/             # Implementation reports
│   └── tests/               # Test methodology
└── docs/                     # API docs, deployment guides

```

---

## 🧩 Rust Crates (20 total)

### Consciousness Engines (9 Rust)

| Crate | Phase | Status | Description |
|-------|-------|--------|-------------|
| `engine-human-design` | 0 | ✅ Complete | Body graph, centers, channels, gates (sequential mapping) |
| `engine-gene-keys` | 0 | ✅ Complete | 64 Gene Keys, Shadow→Gift→Siddhi transformation |
| `engine-vimshottari` | 0 | ✅ Complete | Vedic planetary periods (120-year cycle) |
| `engine-panchanga` | 0 | ✅ Complete | Vedic calendar (Tithi, Nakshatra, Yoga, Karana) |
| `engine-numerology` | 0 | ✅ Complete | Life Path, Expression, Soul Urge calculations |
| `engine-biorhythm` | 0 | ✅ Complete | Physical, Emotional, Intellectual cycles |
| `engine-vedic-clock` | 0 | ✅ Complete | TCM organ clock + Vedic timing synthesis |
| `engine-biofield` | 1 | ✅ Stub | Mock biofield metrics (future: PIP hardware) |
| `engine-face-reading` | 1 | ✅ Stub | Mock face analysis (future: MediaPipe) |

### API & Infrastructure (11 crates)

| Crate | Purpose | Key Features |
|-------|---------|--------------|
| **noesis-api** | Main HTTP server | Axum, auth middleware, rate limiting, CORS |
| **noesis-vedic-api** ⭐ | FreeAstrologyAPI client | Panchang, Dasha, Charts, Muhurtas, caching |
| **noesis-western-api** | Western astrology API | Placeholder for future expansion |
| **noesis-orchestrator** | Multi-engine coordination | Parallel execution, consciousness gating |
| **noesis-cache** | Multi-tier caching | L1 (memory), L2 (Redis), L3 (disk) |
| **noesis-auth** | Authentication | JWT + API key, consciousness level tracking |
| **noesis-metrics** | Observability | Prometheus metrics, engine latency tracking |
| **noesis-core** | Shared types | EngineInput, EngineOutput, EngineError |
| **noesis-witness** | Prompt generation | Non-prescriptive witness prompts |
| **noesis-bridge** | TypeScript bridge | HTTP bridge to Bun engines |
| **noesis-data** | Data loading | Wisdom docs, ephemeris, validation data |

---

## 🎯 FreeAstrologyAPI Integration (noesis-vedic-api)

### Status: 92/120 tasks complete (77%)

### Completed Phases (10/11)

#### Phase 1: Foundation ✅ (10/10 tasks)
- HTTP client with reqwest
- API key authentication
- Retry logic + exponential backoff
- Circuit breaker pattern
- Response caching (LRU)
- Request/response logging

#### Phase 2: Panchang ✅ (20/20 tasks)
- Complete Panchang (Tithi, Nakshatra, Yoga, Karana, Vara)
- 12 Muhurta endpoints (Abhijit, Rahu Kalam, Yama Gandam, etc.)
- Hora timings (24 planetary hours)
- Choghadiya Muhurtas
- Sunrise/sunset calculations

#### Phase 3: Vimshottari Dasha ✅ (14/14 tasks)
- All 4 levels: Maha, Antar, Pratyantar, Sookshma
- Current Dasha calculation
- Upcoming transitions
- Dasha lord by date
- Enrichment with wisdom data

#### Phase 4: Birth Chart ✅ (8/8 tasks)
- Rashi chart (D1) with planets + houses
- Dignities (exalted, debilitated, moolatrikona)
- Retrograde/combust status
- Planetary aspects

#### Phase 5: Navamsa & Vargas ✅ (10/10 tasks)
- Navamsa (D9) for marriage/spirituality
- Dasamsa (D10) for career
- Dwadasamsa (D12) for parents
- Saptamsa (D7) for children
- Varga strength calculator

#### Phase 6: Advanced Features ✅ (10/10 tasks - STUB)
- Yoga detection (Raj Yogas, Dhana Yogas)
- Shadbala (6-fold planetary strength)
- Ashtakavarga (bindu points)

#### Phase 7: Transits ✅ (8/8 tasks - STUB)
- Current transit positions
- Transit aspects to natal
- Sade Sati detection
- Jupiter transit blessings

#### Phase 8: Muhurta ✅ (6/6 tasks - STUB)
- Marriage Muhurta
- Business Muhurta
- Travel Muhurta
- General activity Muhurta

#### Phase 9: Vedic Clock Enhancement ✅ (5/5 tasks)
- API-backed organ clock timing
- Hora integration
- Choghadiya integration
- Panchang quality overlay

#### Phase 10: Integration & Testing ⚠️ (11/19 tasks)
- ✅ Unified VedicApiService
- ✅ API response mocks (FAPI-093)
- ✅ Integration tests (FAPI-094)
- ✅ JHora validation (FAPI-095)
- ✅ Reference validation (FAPI-096, FAPI-097)
- ✅ Fallback to native (FAPI-098)
- ✅ Metrics (FAPI-099)
- ✅ Rate limit handling (FAPI-105)
- ✅ Batch optimization (FAPI-106)
- ✅ API versioning (FAPI-107)
- ✅ Migration guide (FAPI-108)
- ✅ Full test suite (FAPI-110)

### Module Structure (noesis-vedic-api)

```
src/
├── lib.rs                    # Main exports + convenience functions
├── config.rs                 # API configuration
├── client.rs                 # HTTP client
├── cached_client.rs          # Main interface with caching
├── service.rs                # Unified VedicApiService
├── error.rs                  # Error types
├── cache.rs                  # LRU cache layer
├── rate_limiter.rs           # 50 req/day limit
├── retry.rs                  # Exponential backoff
├── circuit_breaker.rs        # Failure protection
├── logging.rs                # Request/response logging
├── metrics.rs                # Prometheus metrics
├── resilience.rs             # Fallback chain
├── batch.rs                  # Batch optimization
├── versioning.rs             # API versioning
├── types.rs                  # Common types
├── mocks.rs                  # Test mocks
├── dasha.rs                  # Dasha types
├── chart.rs                  # Chart types
├── panchang/                 # Panchang module (9 files)
│   ├── api.rs               # API calls
│   ├── types.rs             # Core types
│   ├── dto.rs               # Request/response DTOs
│   ├── mappers.rs           # DTO→domain mapping
│   ├── data.rs              # Wisdom data
│   ├── muhurta.rs           # Muhurta calculations
│   ├── hora.rs              # Hora timings
│   └── choghadiya.rs        # Choghadiya timings
├── vimshottari/             # Vimshottari module (8 files)
│   ├── api.rs               # API calls
│   ├── types.rs             # Dasha types
│   ├── mappers.rs           # DTO→domain mapping
│   ├── current.rs           # Current Dasha finder
│   ├── query.rs             # Dasha lord by date
│   ├── transitions.rs       # Upcoming transitions
│   └── enrichment.rs        # Wisdom enrichment
├── birth_chart/             # Birth chart module (7 files)
│   ├── api.rs               # API calls
│   ├── types.rs             # Chart types
│   ├── mappers.rs           # DTO→domain mapping
│   ├── dignities.rs         # Exalted/debilitated
│   ├── status.rs            # Retrograde/combust
│   └── aspects.rs           # Planetary aspects
├── vargas/                  # Divisional charts (10 files)
│   ├── navamsa.rs           # D9 chart
│   ├── dasamsa.rs           # D10 chart
│   ├── dwadasamsa.rs        # D12 chart
│   ├── saptamsa.rs          # D7 chart
│   └── strength.rs          # Varga strength
├── transits/                # Transit module (7 files - STUB)
├── yogas/                   # Yoga module (5 files - STUB)
├── shadbala/                # Shadbala module (4 files - STUB)
├── ashtakavarga/            # Ashtakavarga module (4 files - STUB)
├── muhurta/                 # Muhurta module (7 files - STUB)
├── progressions/            # Progressions module (3 files)
├── daily_panchang/          # Daily service (4 files - STUB)
├── hora_alarms/             # Hora alarms (4 files - STUB)
├── dasha_alerts/            # Dasha alerts (4 files - STUB)
├── festivals/               # Festival calendar (4 files - STUB)
├── eclipses/                # Eclipse predictions (4 files - STUB)
├── fasting/                 # Fasting calendar (4 files - STUB)
├── naming/                  # Name suggestions (4 files - STUB)
├── remedies/                # Planetary remedies (5 files - STUB)
└── report_generator/        # Report generation (4 files - STUB)
```

### API Endpoints Covered

| Category | Endpoints | Status |
|----------|-----------|--------|
| **Panchang** | 12 endpoints | ✅ Complete |
| **Vimshottari** | 1 endpoint (4 levels) | ✅ Complete |
| **Birth Chart** | 3 endpoints | ✅ Complete |
| **Vargas** | 5 divisional charts | ✅ Complete |
| **Transits** | 4 endpoints | 🟡 Stub |
| **Yogas** | 2 endpoints | 🟡 Stub |
| **Shadbala** | 1 endpoint | 🟡 Stub |
| **Ashtakavarga** | 1 endpoint | 🟡 Stub |
| **Muhurta** | 4 endpoints | 🟡 Stub |

### Test Coverage

```
tests/
├── client_tests.rs           # HTTP client tests (wiremock)
├── panchang_tests.rs         # Panchang API tests
├── panchang_integration_test.rs  # Cached integration
├── panchang_validation.rs    # Accuracy validation
├── vimshottari_validation.rs # Dasha validation
├── dasha_validation.rs       # Additional Dasha tests
├── birth_chart_validation.rs # Chart accuracy tests
├── navamsa_tests.rs          # D9 calculation tests
├── shesh_chart_validation.rs # Real profile validation
├── integration_tests.rs      # Full integration suite
└── resilience_tests.rs       # Fallback + retry tests
```

---

## 🌐 TypeScript Engines (5 total)

| Engine | Phase | Status | Description |
|--------|-------|--------|-------------|
| `tarot` | 0 | ✅ Complete | 78-card Rider-Waite, 5 spread types |
| `i-ching` | 0 | ✅ Complete | 64 hexagrams, three-coin method |
| `enneagram` | 1 | ✅ Complete | 9 types, 45-question assessment |
| `sacred-geometry` | 0 | 🟡 Stub | 12 sacred forms (visual gen deferred) |
| `sigil-forge` | 1 | 🟡 Stub | 4 creation methods (visual gen deferred) |

**Runtime:** Bun + Elysia (port 3001)  
**Bridge:** noesis-bridge (Rust HTTP client)

---

## 🔄 Multi-Engine Workflows (6 total)

| Workflow | Engines | Purpose | Status |
|----------|---------|---------|--------|
| `birth-blueprint` | Numerology, HD, Vimshottari | Natal analysis | ✅ |
| `daily-practice` | Panchanga, VedicClock, Biorhythm | Temporal calibration | ✅ |
| `decision-support` | Tarot, I-Ching, HD Authority | Multi-perspective guidance | ✅ |
| `self-inquiry` | Gene Keys, Enneagram | Shadow work | ✅ |
| `creative-expression` | Sigil Forge, Sacred Geometry | Generative guidance | ✅ |
| `full-spectrum` | All 16 engines | Complete self-portrait | ✅ |

**Execution:** Parallel via `futures::join_all`  
**Synthesis:** Theme extraction, alignment detection, tension framing

---

## 📦 Data Files

```
data/
├── ephemeris/               # Swiss Ephemeris files (sepl_18.se1, etc.)
├── wisdom-docs/             # Engine wisdom data (JSON)
│   ├── human_design/
│   ├── gene_keys/
│   ├── vimshottari/
│   ├── tarot/
│   └── i-ching/
├── validation/              # Reference charts for testing
│   ├── human_design_reference_charts.json
│   └── gene_keys_reference_charts.json
├── vedic-clock/             # TCM organ clock data
├── biorhythm/               # Biorhythm cycle data
└── constants/               # Astronomical constants
```

---

## 🧪 Testing Infrastructure

### Test Categories

| Category | Location | Count | Purpose |
|----------|----------|-------|---------|
| **Unit Tests** | `crates/*/tests/` | 100+ | Module-level testing |
| **Integration Tests** | `tests/integration/` | 26 | API endpoint testing |
| **E2E Tests** | `tests/e2e/` | 81 | Full workflow testing |
| **Load Tests** | `tests/load/k6/` | 3 | Performance testing |
| **Chaos Tests** | `tests/chaos/` | 20 | Resilience testing |
| **Security Tests** | `tests/security/` | 50 | Auth + injection testing |
| **Validation Tests** | `tests/validation/` | 27 | Accuracy validation |

### Test Execution

```bash
# Unit tests
cargo test --workspace

# Integration tests (API)
cargo test --package noesis-api

# Vedic API tests
cargo test --package noesis-vedic-api

# E2E tests
cargo test --test e2e_*

# Load tests
k6 run tests/load/k6/engine_load_test.js

# Full suite
cargo test --workspace --release
```

---

## 🚀 Deployment

### Docker

```bash
# Build production image
docker build -f Dockerfile.prod -t noesis-api:latest .

# Run with docker-compose
docker-compose up -d

# Run with monitoring stack
docker-compose -f docker-compose.monitoring.yml up -d
```

### Kubernetes

```bash
# Apply all manifests
kubectl apply -k k8s/

# Check deployment
kubectl get pods -n noesis

# View logs
kubectl logs -f deployment/noesis-api -n noesis
```

### Environment Variables

```bash
# Required
FREE_ASTROLOGY_API_KEY=your_key_here
JWT_SECRET=your_secret_here

# Optional
REDIS_URL=redis://localhost:6379
POSTGRES_URL=postgres://user:pass@localhost/noesis
ALLOWED_ORIGINS=http://localhost:3000,http://localhost:5173
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_WINDOW_SECS=60
REQUEST_TIMEOUT_SECS=30
RUST_LOG=info,noesis_api=debug
LOG_FORMAT=json
```

---

## 📊 Metrics & Monitoring

### Prometheus Metrics

- `engine_calculation_duration_seconds` - Engine latency histogram
- `engine_calculation_total` - Total calculations counter
- `engine_calculation_errors_total` - Error counter
- `http_requests_total` - HTTP request counter
- `http_request_duration_seconds` - HTTP latency histogram

### Grafana Dashboards

1. **API Overview** - Request rates, latency, error rates
2. **Engine Performance** - Per-engine latency, throughput
3. **Cache Performance** - Hit rates, evictions, memory usage

### Alerting Rules

- High error rate (>5% for 5 minutes)
- High latency (p95 >1s for 5 minutes)
- Low cache hit rate (<80% for 10 minutes)
- Pod restarts (>3 in 10 minutes)

---

## 📝 Documentation

### Key Documents

| Document | Location | Purpose |
|----------|----------|---------|
| **Architecture Overview** | `.context/architecture.md` | System design |
| **Engine Specs** | `.context/engines/*.md` | 16 engine specifications |
| **API Reference** | `docs/api/` | Endpoint documentation |
| **Deployment Guide** | `docs/deployment/` | K8s + Docker setup |
| **Troubleshooting** | `docs/troubleshooting.md` | Common issues |
| **Wave 1 Retrospective** | `docs/WAVE_1_RETROSPECTIVE.md` | Phase 1-4 summary |
| **FreeAstrology Integration** | `.context/reports/implementations/FREE_ASTROLOGY_API_INTEGRATION_SUMMARY.md` | API integration plan |
| **Migration Guide** | `crates/noesis-vedic-api/MIGRATION.md` | Native→API migration |

---

## 🔧 Development

### Build Commands

```bash
# Build workspace
cargo build --workspace

# Build release
cargo build --workspace --release

# Run server
cargo run --bin noesis-server

# Run with env file
cargo run --bin noesis-server -- --env .env

# Run TypeScript engines
cd ts-engines && bun run src/server/index.ts
```

### Code Quality

```bash
# Format
cargo fmt --all

# Lint
cargo clippy --workspace -- -D warnings

# Check
cargo check --workspace

# Audit dependencies
cargo audit
```

---

## 📈 Project Status

### Wave 1 (API Infrastructure + Core Engines) ✅
- **Tasks:** 91/91 (100%)
- **Duration:** ~60 minutes (parallel agents)
- **Engines:** Human Design, Gene Keys, Vimshottari

### Wave 2 (TypeScript Engines + Workflows + Production) ✅
- **Tasks:** 96/96 (100%)
- **Duration:** ~12 hours (parallel agents)
- **Engines:** Tarot, I-Ching, Enneagram, Sacred Geometry, Sigil Forge
- **Workflows:** 6 synthesis workflows
- **Infrastructure:** Docker, K8s, CI/CD, monitoring

### FreeAstrologyAPI Integration ⚠️
- **Tasks:** 92/120 (77%)
- **Status:** Phase 10 in progress
- **Remaining:** 8 remaining tasks in Phase 10 outside FAPI-093-110

### Total Progress
- **Tasks:** 279/307 (91%)
- **Engines:** 14/14 (100%)
- **Workflows:** 6/6 (100%)
- **Infrastructure:** Production-ready

---

## 🎯 Next Steps

### Immediate (Phase 10 completion)
1. ✅ Complete API response mocks (FAPI-093)
2. ✅ Integration tests (FAPI-094)
3. ✅ JHora validation (FAPI-095)
4. ✅ Reference validation (FAPI-096, FAPI-097)
5. ✅ Fallback to native (FAPI-098)
6. ✅ Metrics and monitoring (FAPI-099)
7. ✅ Rate limit handling (FAPI-105)
8. ✅ Batch request optimization (FAPI-106)
9. ✅ API versioning support (FAPI-107)
10. ✅ Migration guide (FAPI-108)
11. ✅ Full test suite (FAPI-110)

### Short-term (Phase 11)
- Daily Panchang notifications
- Planetary hour alarms
- Dasha change alerts
- Festival calendar
- Eclipse predictions

### Long-term
- Sacred Geometry visual generation
- Sigil Forge visual output
- Biofield hardware integration (PIP)
- Face Reading MediaPipe integration
- Mobile app development

---

## 📞 Contact & Resources

- **Repository:** https://github.com/tryambakam/noesis
- **API Docs:** https://freeastrologyapi.com/api-docs
- **License:** MIT
- **Team:** Tryambakam Noesis Team

---

**Last Updated:** 2026-02-08  
**Index Version:** 1.0  
**Codebase Version:** 2.0.0
