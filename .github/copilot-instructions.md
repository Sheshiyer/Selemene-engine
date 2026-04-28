# Noesis Engine — AI Coding Agent Instructions

## Project Overview
Noesis (formerly Selemene Engine) is a consciousness calculation platform with **16 engines** (11 Rust + 5 TypeScript), deployed on **Railway** with **Supabase** PostgreSQL for auth/DB. It serves a RESTful API at `https://selemene.tryambakam.space`.

**Current Status**: Production — all 16 engines live and calculating.

## Architecture

### Engine System
All engines implement the `ConsciousnessEngine` trait in [crates/noesis-core/src/lib.rs](../crates/noesis-core/src/lib.rs). The orchestrator in [crates/noesis-orchestrator/](../crates/noesis-orchestrator/) manages parallel execution and workflow synthesis.

**11 Rust Engines** (in `crates/engine-*/`):
- `engine-panchanga` — Vedic calendar (tithi, nakshatra, yoga, karana)
- `engine-human-design` — HD bodygraph (64 gates, 9 centers, profile)
- `engine-gene-keys` — Shadow-Gift-Siddhi activation sequences
- `engine-vimshottari` — 120-year nested dasha periods
- `engine-numerology` — Pythagorean + Chaldean number systems
- `engine-biorhythm` — 3 biological cycles (Physical/Emotional/Intellectual)
- `engine-vedic-clock` — TCM organ clock + Ayurvedic timing
- `engine-biofield` — Chakra & biofield analysis from birth data
- `engine-face-reading` — Physiognomy analysis
- `engine-nadabrahman` — Sound consciousness engine
- `engine-transits` — Planetary transits, aspects & Sade Sati

**5 TypeScript Engines** (in `ts-engines/`, deployed as Railway sidecar on port 3001):
- Tarot, I-Ching, Enneagram, Sacred Geometry, Sigil Forge
- Bridged to Rust via `BridgeEngine` in [crates/noesis-bridge/](../crates/noesis-bridge/)

### Infrastructure
- **Deployment**: Railway (Rust API on port 8080, TS sidecar on port 3001)
- **Database/Auth**: Supabase PostgreSQL
- **Cache**: Redis (L1 in-memory + L2 Redis)
- **API URL**: `https://selemene.tryambakam.space`
- **Binary**: `noesis-server` (defined in [crates/noesis-api/Cargo.toml](../crates/noesis-api/Cargo.toml))

### Crate Structure
```
crates/
  noesis-api/             Axum HTTP server (binary: noesis-server)
  noesis-orchestrator/    Multi-engine parallel execution + workflow synthesis
  noesis-auth/            JWT + API key auth (Supabase Postgres)
  noesis-cache/           Cache layer (L1 memory, L2 Redis)
  noesis-core/            Shared traits (ConsciousnessEngine, EngineInput/Output)
  noesis-bridge/          TS engine bridge (HTTP → ConsciousnessEngine trait)
  noesis-data/            Data loading and management
  noesis-metrics/         Prometheus instrumentation
  noesis-vedic-api/       FreeAstrologyAPI client (15+ Vedic endpoints)
  noesis-western-api/     Western astrology API client
  noesis-witness/         Witness prompt generation & consciousness calibration
  noesis-integration/     Integration tests & cross-engine coordination
  engine-*/               Individual engine crates (11 Rust engines)
```

### Orchestrator Pattern
`CalculationOrchestrator` in [crates/noesis-orchestrator/](../crates/noesis-orchestrator/) coordinates all calculations:
1. Request validation
2. Engine selection and parallel execution
3. Workflow synthesis (combining multiple engine results)
4. Result post-processing with witness prompts

All calculation flows go through the orchestrator — never call engines directly from API handlers.

## Error Handling
Use `EngineError` enum from [crates/noesis-core/src/error.rs](../crates/noesis-core/src/error.rs). Never use `anyhow` or generic errors in public APIs. Pattern:
```rust
Err(EngineError::CalculationError(format!("specific context: {}", detail)))
```

## Development Workflows

### Building and Testing
```bash
cargo build                              # Dev build
cargo test --test panchanga_tests        # Specific test suite
cargo test --test accuracy_tests         # Accuracy validation
cargo bench                              # Performance benchmarks
cargo run --example standalone_panchanga_demo
```

### Running Locally
```bash
# Start the Rust API server (port 8080)
cargo run --bin noesis-server

# Start TS engines sidecar (port 3001) — in ts-engines/ directory
cd ts-engines && bun run src/index.ts

# Or use Docker Compose for everything
docker-compose up
```

### API Routes
Routes in [crates/noesis-api/src/](../crates/noesis-api/src/) follow RESTful conventions:
```
POST /api/v1/engines/{engine_id}/calculate  — Run single engine
POST /api/v1/workflows/{workflow_id}        — Run multi-engine workflow
GET  /api/v1/engines                        — List all engines
GET  /health/live                           — Liveness probe
GET  /health/ready                          — Readiness probe
```

## Testing Philosophy
- **Unit tests**: In individual crate `src/` files
- **Integration tests**: In [crates/noesis-integration/](../crates/noesis-integration/) and `tests/`
- **Accuracy tests**: In `tests/validation/` comparing engine outputs
- **Performance tests**: In `benches/`
- **E2E tests**: In `tests/e2e/` for full API workflow testing

## External Dependencies
- **Supabase**: PostgreSQL for auth/users/API keys
- **Redis**: Required for cache in production (optional for local dev)
- **FreeAstrologyAPI**: External API for Vedic calculations (via `noesis-vedic-api`)
- **Ephemeris data**: In `data/ephemeris/` for planetary calculations

## Documentation Sources
- API Quickstart: [docs/API_QUICKSTART.md](../docs/API_QUICKSTART.md)
- Engine details: [docs/ENGINES.md](../docs/ENGINES.md)
- Deployment: [docs/deployment/RAILWAY.md](../docs/deployment/RAILWAY.md)
- API reference: [docs/api/README.md](../docs/api/README.md)

## When Making Changes

1. **Adding a new engine**: Create `crates/engine-{name}/`, implement `ConsciousnessEngine` trait, register in orchestrator, expose via API
2. **Modifying cache behavior**: Update in [crates/noesis-cache/](../crates/noesis-cache/)
3. **Adding API endpoints**: Define in [crates/noesis-api/src/](../crates/noesis-api/src/), add to API docs
4. **Performance optimization**: Benchmark with `cargo bench`, validate accuracy with `tests/validation/`

Always preserve the orchestrator pattern — calculations flow through `CalculationOrchestrator`, not directly from handlers to engines.

## Agent Dispatch Contract

When you are invoked on a PR whose head ref matches `agent/issue-<N>-*`:

1. Your full task contract is in `.agent-tasks/issue-<N>.md`. Read it before writing any code.
2. Implement against the Deliverable + Acceptance Criteria in that file.
3. Run `cargo build && cargo test` and confirm green before pushing the final commit.
4. Delete `.agent-tasks/issue-<N>.md` in your final commit.
5. Convert the PR from draft to ready-for-review.
6. Post a PR comment: `agent: done -- tests passing, ready for review`.

Branch naming and the task-file marker are load-bearing — `agent-merge-lane.yml` and `agent-post-merge.yml` both depend on them. Do not rename the branch or the task file.
