# Selemene Engine - AI Coding Agent Instructions

## Project Overview
Selemene Engine is a high-performance astronomical calculation engine for Vedic astrology and Panchanga calculations, built in Rust with an Axum HTTP API. The project combines native calculation engines (VSOP87 solar, ELP-2000 lunar) with Swiss Ephemeris as a fallback.

**Current Status**: Late development phase - core architecture complete, core calculation implementations pending.

## Architecture Principles

### Hybrid Backend System
The engine uses intelligent routing between three calculation backends:
- **Native engines** (preferred): `NativeSolarEngine` and `NativeLunarEngine` in [src/engines/](../src/engines/)
- **Swiss Ephemeris** (fallback): `SwissEphemerisEngine` for validation and backup
- **Validated mode**: Cross-checks results between native and Swiss Ephemeris

Backend selection happens in `HybridBackend::select_backend()` based on `BackendRoutingStrategy` enum values.

### Three-Layer Cache Architecture
Cache lookups cascade through three tiers defined in [src/cache/](../src/cache/):
1. **L1** (in-memory LRU): ~256MB, <1ms access via `dashmap::DashMap`
2. **L2** (Redis): ~1GB, <10ms access for distributed caching
3. **L3** (disk): Precomputed results for common queries

Cache keys use `CacheKey` struct with date, coordinates, precision, and backend identifier.

### Orchestrator Pattern
`CalculationOrchestrator` in [src/engines/calculation_orchestrator.rs](../src/engines/calculation_orchestrator.rs) coordinates all calculations:
1. Request validation
2. Backend selection via `HybridBackend`
3. Calculation execution (native/swiss/validated)
4. Result post-processing

All calculation flows go through the orchestrator - never bypass it.

## Critical Implementation Details

### Ghati Time System
Ghati is a Vedic time unit where 1 day = 60 ghatis, 1 ghati = 24 minutes. Implementation in [src/time/ghati_calculator.rs](../src/time/ghati_calculator.rs) uses the **Hybrid System** (see [GHATI_CALCULATION_STANDARDS.md](../GHATI_CALCULATION_STANDARDS.md)):
- Fixed 24-minute intervals as base
- Solar time adjustments based on longitude
- Critical for Panchanga timing precision

### Precision Levels
`PrecisionLevel` enum in [src/models/mod.rs](../src/models/mod.rs):
- `Standard`: Fast calculations for real-time use
- `High`: Increased accuracy for detailed analysis
- `Extreme`: Maximum precision for research

Precision affects both calculation method selection and cache key generation.

### Error Handling
Use `EngineError` enum from [src/models/mod.rs](../src/models/mod.rs) for all errors. Never use `anyhow` or generic errors in public APIs. Pattern:
```rust
Err(EngineError::CalculationError(format!("specific context: {}", detail)))
```

## Development Workflows

### Building and Testing
```bash
# Development build
cargo build

# Run tests (specific test suites)
cargo test --test panchanga_tests
cargo test --test accuracy_tests

# Run benchmarks
cargo bench
# Or use: ./scripts/benchmark.sh

# Run example
cargo run --example standalone_panchanga_demo
```

### Running Locally
```bash
# Start the API server (port 8080)
cargo run --bin selemene-engine

# Or with release optimizations
cargo run --release
```

Runtime configuration is read from environment variables via the config module in [src/config/](../src/config/).

## Code Patterns and Conventions

### Async/Await Usage
All calculation methods are `async fn` using `tokio` runtime. Pattern:
```rust
pub async fn calculate_panchanga(&self, request: PanchangaRequest) 
    -> Result<PanchangaResult, EngineError>
```

### Module Organization
- `src/engines/`: All calculation engines and orchestration
- `src/api/`: HTTP API layer (routes, handlers, middleware)
- `src/cache/`: Multi-tier caching (L1/L2/L3)
- `src/time/`: Ghati calculations and time conversions
- `src/models/`: Request/response types and errors
- `src/auth/`: JWT and API key authentication

### API Route Pattern
Routes in [src/api/routes.rs](../src/api/routes.rs) follow RESTful conventions:
```rust
.route("/panchanga/calculate", post(handlers::calculate_panchanga))
.route("/panchanga/batch", post(handlers::calculate_batch_panchanga))
```

Many Ghati and real-time routes are stubbed with TODO comments - implementations pending.

### Parallel Processing
Use `rayon` for CPU-bound parallel calculations in `CalculationOrchestrator::calculate_range_parallel()`. Pattern processes requests in chunks to avoid thread pool exhaustion.

## Known Limitations and TODOs

1. **Swiss Ephemeris initialization**: Currently synchronous in `CalculationOrchestrator::new()`, needs async refactor
2. **Native engine implementations**: Solar/lunar engines have placeholder logic in [src/engines/native_solar.rs](../src/engines/native_solar.rs) and [native_lunar.rs](../src/engines/native_lunar.rs)
3. **Ghati API handlers**: Stubbed in routes.rs, need implementation in `src/api/ghati_handlers.rs`
4. **Real-time tracking**: Architecture defined in [src/time/realtime_tracker.rs](../src/time/realtime_tracker.rs) but handlers incomplete
5. **Cache invalidation**: L2/L3 cache invalidation strategies not fully implemented

## Testing Philosophy

- **Unit tests**: In individual module files for pure functions
- **Integration tests**: In `tests/integration/` for engine coordination
- **Accuracy tests**: In `tests/validation/accuracy_tests.rs` comparing backends
- **Performance tests**: In `tests/performance/benchmark_tests.rs` and `benches/`

Run specific suites as needed rather than entire test suite during active development.

## External Dependencies

- **Swiss Ephemeris**: Requires ephemeris data files in `data/ephemeris/` (configured via `EngineConfig.swiss_ephemeris_path`)
- **Redis**: Required for L2 cache in production (optional for local dev)

## Documentation Sources

- Architecture deep-dive: [selemene_architecture.md](../selemene_architecture.md)
- Project status: [PROJECT_SUMMARY.md](../PROJECT_SUMMARY.md)
- Codebase overview: [CODEBASE_SUMMARY.md](../CODEBASE_SUMMARY.md)
- Ghati standards: [GHATI_CALCULATION_STANDARDS.md](../GHATI_CALCULATION_STANDARDS.md)
- API reference: [docs/api/README.md](../docs/api/README.md)

## When Making Changes

1. **Adding calculation features**: Implement in appropriate engine, wire through orchestrator, expose via API handler
2. **Modifying cache behavior**: Update all three cache layers consistently
3. **Changing time calculations**: Reference Ghati calculation standards, update integration points in `src/time/panchanga_integration.rs`
4. **Adding API endpoints**: Define in routes.rs, implement handler, add to API docs
5. **Performance optimization**: Benchmark before/after using `cargo bench`, validate accuracy with cross-backend validation

Always preserve the orchestrator pattern - calculations should flow through `CalculationOrchestrator`, not directly call engines.
