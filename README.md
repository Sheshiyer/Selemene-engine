# Tryambakam Noesis - Consciousness Engine Backend

A high-performance Rust backend for 13 self-consciousness engines providing Decision Mirrors for human development. Combines ancient wisdom traditions with modern computation to generate personalized, non-prescriptive self-inquiry prompts based on consciousness level (0-5).

## Status: Wave 1 Complete

- 6 engines operational (Panchanga, Numerology, Biorhythm, Human Design, Gene Keys, Vimshottari)
- 100+ tests passing across all engines and infrastructure
- Production-ready API with authentication, caching, metrics
- Sub-millisecond calculations for all complex engines

## Quick Start

```bash
# Build
cargo build --release

# Run server
cargo run --bin noesis-server

# Run all tests
cargo test --workspace

# API docs (after starting server)
open http://localhost:8080/api/docs
```

## Architecture

```
crates/
  noesis-core/           Shared ConsciousnessEngine trait, types, errors
  noesis-api/            Axum HTTP server (26 integration tests)
  noesis-cache/          3-layer cache (L1 memory, L2 Redis, L3 disk)
  noesis-auth/           JWT + API key + consciousness level gating
  noesis-metrics/        Prometheus instrumentation
  noesis-orchestrator/   Parallel engine execution + workflow registry

  engine-panchanga/      Phase 0 - Vedic calendar calculations
  engine-numerology/     Phase 0 - Pythagorean + Chaldean reduction
  engine-biorhythm/      Phase 0 - 3 biological sine cycles
  engine-human-design/   Phase 1 - 88-degree solar arc bodygraph (100% accuracy validated)
  engine-gene-keys/      Phase 2 - Shadow-Gift-Siddhi transformation framework
  engine-vimshottari/    Phase 2 - 120-year planetary period timeline
```

## Engines

| Engine | Phase | Performance | Key Feature |
|--------|-------|-------------|-------------|
| Panchanga | 0 | Legacy | Vedic calendar, tithi, nakshatra |
| Numerology | 0 | <1ms | Pythagorean + Chaldean name/date reduction |
| Biorhythm | 0 | <1ms | Physical (23d), Emotional (28d), Intellectual (33d) |
| Human Design | 1 | 1.31ms | 26 planetary activations, 100% accuracy vs professional software |
| Gene Keys | 2 | 0.012ms | 4 activation sequences, Shadow-Gift-Siddhi triplets |
| Vimshottari | 2 | <1ms | 729 nested periods, binary search current detection |

## API

All engines accessible via unified REST API:

```
POST /api/v1/engines/:engine_id/calculate    Single engine calculation
GET  /api/v1/engines/:engine_id/info         Engine metadata
GET  /api/v1/engines                         List all engines
POST /api/v1/workflows/:workflow_id/execute  Multi-engine workflow
GET  /health                                 Health check
GET  /ready                                  Readiness probe
GET  /metrics                                Prometheus metrics
GET  /api/docs                               Swagger UI
```

Authentication: JWT bearer token or X-API-Key header. Consciousness level gating enforced at orchestrator.

## Performance

| Metric | Target | Achieved |
|--------|--------|----------|
| HD calculation | <100ms | 1.31ms (76x faster) |
| Gene Keys | <50ms | 0.012ms (4166x faster) |
| Vimshottari | <200ms | <1ms (200x faster) |
| API p95 | <500ms | <100ms |
| Cache hit rate | >80% | >95% |

## Consciousness Levels

The system adapts witness prompts based on user consciousness level:

| Level | Name | Prompt Style |
|-------|------|-------------|
| 0 | Dormant | Observational |
| 1 | Glimpsing | Reflective |
| 2 | Practicing | Inquiry-based |
| 3 | Integrated | Self-authorship |
| 4-5 | Embodied | Open awareness |

## Documentation

- [System Architecture](.context/architecture/system-overview.md) - Wave 1 implementation overview
- [Architecture (Original)](.context/architecture/overview.md) - Full target architecture
- [Human Design Engine](.context/engines/human-design.md) - HD calculation details and validation
- [Gene Keys Engine](.context/engines/gene-keys.md) - Shadow-Gift-Siddhi framework
- [Vimshottari Engine](.context/engines/vimshottari.md) - 120-year dasha timeline
- [Wave 1 Retrospective](docs/WAVE_1_RETROSPECTIVE.md) - Completion summary and learnings
- [Project Memory](memory.md) - Full development history

## Tech Stack

- **Language**: Rust 2021 edition
- **Web Framework**: Axum (async HTTP)
- **Serialization**: Serde (JSON)
- **Async Runtime**: Tokio
- **Ephemeris**: Swiss Ephemeris (via swerust)
- **Cache**: DashMap (L1), Redis (L2), Disk (L3)
- **Metrics**: Prometheus
- **Auth**: JWT (jsonwebtoken) + API keys
- **Docs**: utoipa + utoipa-swagger-ui (OpenAPI 3.0)

## Next: Wave 2

TypeScript engines (Tarot, I-Ching, Enneagram, Sacred Geometry, Sigil Forge), specialized Rust engines (Vedic Clock, Biofield, Face Reading), workflow synthesis, production deployment.

---

*From Selemene (Vedic astrology) to Tryambakam Noesis (multi-engine consciousness) - bridging ancient wisdom with modern computation.*
