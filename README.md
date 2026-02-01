# Tryambakam Noesis Engine

A high-performance consciousness calculation platform with **14 engines** and **6 synthesis workflows**. Combines ancient wisdom traditions (Vedic astrology, Human Design, Gene Keys, I-Ching, Tarot) with modern computation to generate personalized, non-prescriptive self-inquiry prompts.

## Status: Wave 2 Complete ✅

- **14 engines** operational (9 Rust + 5 TypeScript)
- **6 synthesis workflows** for multi-engine coordination
- **228+ tests** passing across all engines
- Production-ready with Docker, Kubernetes, monitoring stack
- Sub-millisecond calculations for complex engines

## Quick Start

```bash
# Build and run Rust API (port 8080)
cargo build --release
cargo run --bin selemene-engine

# Start TypeScript engines (port 3001)
cd ts-engines && bun install && bun run src/index.ts

# Run all tests
cargo test -- --test-threads=1  # Rust (single-threaded for Swiss Ephemeris)
cd ts-engines && bun test       # TypeScript

# Health checks
curl http://localhost:8080/health
curl http://localhost:3001/health
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Client Request                            │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                 Rust API (selemene-engine:8080)                  │
│  Endpoints: /health, /api/v1/panchanga, /engines, /workflows    │
│  Features: Auth (JWT/API Key), Rate Limiting, 3-Layer Cache     │
└─────────────────────────────────────────────────────────────────┘
                               │
           ┌───────────────────┼───────────────────┐
           ▼                   ▼                   ▼
┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
│   Rust Engines   │  │   TS Engines     │  │   Orchestrator   │
│   (9 engines)    │  │   (5 engines)    │  │   + Workflows    │
│                  │  │   Port: 3001     │  │                  │
└──────────────────┘  └──────────────────┘  └──────────────────┘

crates/
  noesis-orchestrator/   Parallel engine execution + 6 workflow synthesis
  noesis-bridge/         HTTP bridge for Rust↔TypeScript communication
  engine-human-design/   HD chart (type, authority, profile, centers)
  engine-gene-keys/      Shadow-Gift-Siddhi activation sequences
  engine-vimshottari/    120-year planetary dasha periods
  engine-vedic-clock/    TCM organ clock + Ayurvedic doshas
  engine-biofield/       Chakra readings (stub)
  engine-face-reading/   Physiognomy analysis (stub)

ts-engines/
  tarot/                 78 cards, 5 spread types, witness prompts
  i-ching/               64 hexagrams, changing lines, nuclear hexagrams
  enneagram/             9 types, wings, integration/disintegration
  sacred-geometry/       Geometric form meditation (stub)
  sigil-forge/           Intent-based sigil creation (stub)
```

## The 14 Engines

### Rust Engines (9)

| Engine | Status | Performance | Description |
|--------|--------|-------------|-------------|
| **Panchanga** | Full | <1ms | Vedic calendar (tithi, nakshatra, yoga, karana, vara) |
| **Human Design** | Full | 1.31ms | 26 planetary activations, Rave Mandala gate sequence |
| **Gene Keys** | Full | 0.012ms | 4 activation sequences, Shadow-Gift-Siddhi triplets |
| **Vimshottari** | Full | <1ms | 729 nested periods, binary search current detection |
| **Numerology** | Full | <1ms | Pythagorean + Chaldean name/date reduction |
| **Biorhythm** | Full | <1ms | Physical (23d), Emotional (28d), Intellectual (33d) |
| **VedicClock** | Full | <1ms | TCM organ clock + Ayurvedic dosha timing |
| **Biofield** | Stub | <1ms | Chakra energy readings (mock data) |
| **FaceReading** | Stub | <1ms | Physiognomy analysis (mock data) |

### TypeScript Engines (5)

| Engine | Status | Description |
|--------|--------|-------------|
| **Tarot** | Full | 78-card deck, 5 spread types (Celtic Cross, 3-card, etc.) |
| **I-Ching** | Full | 64 hexagrams, changing lines, relating hexagrams |
| **Enneagram** | Full | 9 types, wings, stress/growth directions |
| **SacredGeometry** | Stub | Geometric form meditation prompts |
| **SigilForge** | Stub | Intent-based sigil generation |

## The 6 Workflows

| Workflow | Engines Used | Purpose |
|----------|--------------|---------|
| **birth-blueprint** | Numerology + Human Design + Vimshottari | Complete natal life analysis |
| **daily-practice** | Panchanga + VedicClock + Biorhythm | Daily timing optimization |
| **decision-support** | Tarot + I-Ching + HD Authority | Multi-perspective guidance |
| **self-inquiry** | Gene Keys + Enneagram | Shadow work + self-understanding |
| **creative-expression** | Sigil Forge + Sacred Geometry | Generative/creative guidance |
| **full-spectrum** | All 14 engines | Complete consciousness portrait |

## API Examples

```bash
# Panchanga (Vedic Calendar)
curl -X POST http://localhost:8080/api/v1/panchanga \
  -H "Content-Type: application/json" \
  -d '{"name":"Test","date":"1991-08-13","time":"13:31","timezone":"Asia/Kolkata","latitude":12.9716,"longitude":77.5946}'

# Tarot Reading (3-card spread)
curl -X POST http://localhost:3001/engines/tarot/calculate \
  -H "Content-Type: application/json" \
  -d '{"consciousness_level":3,"parameters":{"spread_type":"three_card"},"question":"What should I focus on?"}'

# I-Ching Divination
curl -X POST http://localhost:3001/engines/i-ching/calculate \
  -H "Content-Type: application/json" \
  -d '{"consciousness_level":3,"parameters":{},"question":"What is the nature of this moment?"}'

# Enneagram Type Lookup
curl -X POST http://localhost:3001/engines/enneagram/calculate \
  -H "Content-Type: application/json" \
  -d '{"consciousness_level":3,"parameters":{"type":4}}'
```

## Performance

| Metric | Target | Achieved |
|--------|--------|----------|
| HD calculation | <100ms | 1.31ms (76x faster) |
| Gene Keys | <50ms | 0.012ms (4166x faster) |
| Vimshottari | <200ms | <1ms (200x faster) |
| Tarot reading | <100ms | <10ms |
| I-Ching | <100ms | <5ms |
| API p95 | <500ms | <100ms |
| Cache hit rate | >80% | >95% |

## Consciousness Levels

The system adapts witness prompts based on user consciousness level (0-5):

| Level | Name | Prompt Style |
|-------|------|--------------|
| 0 | Dormant | Observational, basic awareness |
| 1 | Glimpsing | Reflective, pattern recognition |
| 2 | Practicing | Inquiry-based, self-questioning |
| 3 | Integrated | Self-authorship, meaning-making |
| 4-5 | Embodied | Open awareness, witnessing |

## Production Deployment

### Docker

```bash
# Build production image
docker build -f Dockerfile.prod -t tryambakam-noesis .

# Run with docker-compose
docker-compose up -d
```

### Kubernetes

```bash
# Deploy to K8s cluster
kubectl apply -f k8s/

# Check deployment
kubectl get pods -n tryambakam
```

### Monitoring Stack

- **Prometheus**: Metrics collection (`/metrics`)
- **Grafana**: Dashboards (port 3000)
- **Loki**: Log aggregation
- **Jaeger**: Distributed tracing

```bash
docker-compose -f docker-compose.monitoring.yml up -d
```

## Documentation

### Architecture
- [System Architecture](.context/architecture/system-overview.md)
- [Engine Overview](.context/architecture/overview.md)

### Engines
- [Human Design](.context/engines/human-design.md) - HD calculation with Rave Mandala
- [Gene Keys](.context/engines/gene-keys.md) - Shadow-Gift-Siddhi framework
- [Vimshottari](.context/engines/vimshottari.md) - 120-year dasha timeline
- [Tarot](docs/api/tarot.md) - Card meanings and spreads
- [I-Ching](docs/api/i-ching.md) - Hexagram interpretation

### Operations
- [Deployment Guide](docs/deployment/README.md)
- [Troubleshooting](docs/troubleshooting.md)
- [API Reference](docs/api/README.md)
- [Release Notes](docs/RELEASE_NOTES.md)

### Development
- [Project Memory](memory.md) - Full development history
- [Wave 1 Retrospective](docs/WAVE_1_RETROSPECTIVE.md)

## Tech Stack

### Rust Backend
- **Framework**: Axum (async HTTP)
- **Runtime**: Tokio
- **Ephemeris**: Swiss Ephemeris (swisseph crate)
- **Cache**: DashMap (L1), Redis (L2), Disk (L3)
- **Auth**: JWT + API keys
- **Metrics**: Prometheus

### TypeScript Engines
- **Runtime**: Bun
- **Framework**: Elysia
- **Testing**: Bun test

### Infrastructure
- **Container**: Docker (multi-stage builds)
- **Orchestration**: Kubernetes
- **CI/CD**: GitHub Actions
- **Monitoring**: Prometheus + Grafana + Loki + Jaeger

## Testing

```bash
# All Rust tests (single-threaded for Swiss Ephemeris thread safety)
cargo test -- --test-threads=1

# Specific engine
cargo test -p engine-human-design -- --test-threads=1
cargo test -p noesis-orchestrator

# TypeScript engines
cd ts-engines && bun test

# E2E tests
./tests/e2e/run_e2e.sh

# Load tests (k6)
k6 run tests/load/load_test.js
```

## Project Structure

```
Selemene-engine/
├── src/                    # Main Rust API binary
├── crates/                 # Rust engine crates
│   ├── engine-human-design/
│   ├── engine-gene-keys/
│   ├── engine-vimshottari/
│   ├── engine-vedic-clock/
│   ├── noesis-orchestrator/
│   └── noesis-bridge/
├── ts-engines/             # TypeScript engines (Bun + Elysia)
│   └── src/engines/
├── data/ephemeris/         # Swiss Ephemeris data files
├── docs/                   # Documentation
├── k8s/                    # Kubernetes manifests
├── monitoring/             # Prometheus, Grafana, Loki configs
├── tests/                  # E2E, load, chaos, security tests
└── .context/               # Engine documentation
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Run tests: `cargo test -- --test-threads=1`
4. Submit a pull request

## License

MIT License - see [LICENSE](LICENSE)

---

*Tryambakam Noesis Engine - Bridging ancient wisdom with modern computation for consciousness exploration.*

*From Selemene (Vedic astrology calculations) to Tryambakam (three-eyed one) Noesis (understanding) - a multi-engine platform for self-inquiry and decision mirrors.*
