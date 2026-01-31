# Plan: Transform Selemene Engine into Tryambakam Noesis Core Backend

## Context

Refactor the existing Selemene Engine (single Panchanga engine in Rust) into the **core backend for all 13 Tryambakam Noesis self-consciousness engines**. The existing Selemene codebase has solid infrastructure (Axum HTTP, 3-layer cache, auth, metrics) that becomes shared services. Each engine gets its own crate. 5 engines that are inherently visual/interactive stay in TypeScript and connect via HTTP bridge.

**Source documents:**
- `evolution-docs/foundational-research.md` — 13 engines as Decision Mirrors, 6 workflows
- `evolution-docs/core-engine-architecture.md` — engine specs, tier access, witness capacity
- `evolution-docs/technical-learnings.md` — HD astronomical accuracy, deterministic caching
- `evolution-docs/self-consciousness-framework.md` — 5 levels, Witness→Understand→Author
- `evolution-docs/IMPLEMENTATION-ROADMAP.md` — 4-phase build order, engine dependencies

**User choices:** Full architecture plan, Rust core + TypeScript edges, refactor existing then expand.

---

## Architecture: Cargo Workspace Monorepo

```
selemene-engine/
├── Cargo.toml                    # [workspace] root
├── crates/
│   ├── noesis-core/              # ConsciousnessEngine trait, shared types
│   ├── noesis-api/               # Axum HTTP server, routing, middleware
│   ├── noesis-cache/             # L1/L2/L3 cache (from existing cache/)
│   ├── noesis-auth/              # JWT + API keys (from existing auth/)
│   ├── noesis-metrics/           # Prometheus (from existing metrics/)
│   ├── noesis-orchestrator/      # Workflow engine, parallel execution
│   ├── noesis-bridge/            # TypeScript engine HTTP adapter
│   ├── noesis-witness/           # Witness Agent prompt injection
│   │
│   ├── engine-panchanga/         # EXISTING — Selemene Panchanga (refactored)
│   ├── engine-numerology/        # Pythagorean + Chaldean
│   ├── engine-human-design/      # 88° solar arc, 64 gates, Swiss Ephemeris
│   ├── engine-biorhythm/         # 3 sine cycles (23/28/33 day)
│   ├── engine-vimshottari/       # 120-year dasha, 27 nakshatras
│   ├── engine-gene-keys/         # Shadow-Gift-Siddhi, 3 sequences
│   ├── engine-vedic-clock/       # TCM organ clock + Vedic time
│   ├── engine-biofield/          # Biometric analysis (Rust compute core)
│   └── engine-face-reading/      # MediaPipe mesh analysis (Rust compute)
│
├── ts-engines/                   # TypeScript engines (Bun)
│   ├── tarot/
│   ├── i-ching/
│   ├── enneagram/
│   ├── sacred-geometry/
│   └── sigil-forge/
│
├── src/                          # TEMPORARY — existing code, migrates to crates/
└── ephe/                         # Swiss Ephemeris data files
```

---

## Core Trait: ConsciousnessEngine

All 13 engines implement this trait (Rust natively, TypeScript via BridgeEngine adapter):

```rust
// crates/noesis-core/src/lib.rs

#[async_trait]
pub trait ConsciousnessEngine: Send + Sync {
    fn engine_id(&self) -> &str;
    fn engine_name(&self) -> &str;
    fn required_phase(&self) -> u8;  // 0-5 access gating

    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError>;
    async fn validate(&self, output: &EngineOutput) -> Result<ValidationResult, EngineError>;
    fn cache_key(&self, input: &EngineInput) -> String;  // SHA-256 deterministic
}

pub struct EngineInput {
    pub birth_data: Option<BirthData>,
    pub current_time: DateTime<Utc>,
    pub location: Option<Coordinates>,
    pub precision: Precision,
    pub options: HashMap<String, Value>,
}

pub struct EngineOutput {
    pub engine_id: String,
    pub result: Value,                    // Engine-specific JSON
    pub witness_prompt: String,           // Self-inquiry question
    pub consciousness_level: u8,          // 0-5
    pub metadata: CalculationMetadata,
}
```

---

## Existing Code → Crate Mapping

| Existing File | Target Crate | Action |
|---------------|-------------|--------|
| `src/api/routes.rs`, `handlers.rs`, `middleware.rs`, `mod.rs` | `noesis-api` | Move + expand routes |
| `src/cache/l1_cache.rs`, `l2_cache.rs`, `l3_cache.rs`, `mod.rs` | `noesis-cache` | Move as-is |
| `src/auth/mod.rs` | `noesis-auth` | Move as-is |
| `src/metrics/mod.rs` | `noesis-metrics` | Move as-is |
| `src/models/mod.rs` | `noesis-core` | Refactor into EngineInput/EngineOutput |
| `src/engines/calculation_orchestrator.rs` | `noesis-orchestrator` | Refactor into WorkflowOrchestrator |
| `src/engines/hybrid_backend.rs` | `noesis-orchestrator` | Merge into orchestrator |
| `src/engines/native_solar.rs` | `engine-panchanga` | Move |
| `src/engines/native_lunar.rs` | `engine-panchanga` | Move |
| `src/engines/swiss_ephemeris.rs` | `noesis-core` (shared) | Shared by HD, Gene Keys, Vimshottari |
| `src/engines/panchanga_calculator.rs` | `engine-panchanga` | Move |
| `src/engines/validation.rs` | `noesis-core` | Generalize |
| `src/simple.rs` | `engine-panchanga` | Move |
| `src/main.rs` | `noesis-api` | Refactor into workspace binary |
| `src/lib.rs` | `noesis-core` | Refactor |

---

## API Route Structure

```
/api/v1/
├── engines/
│   ├── :engine_id/calculate      POST  — Single engine calculation
│   ├── :engine_id/validate       POST  — Validate engine output
│   └── :engine_id/info           GET   — Engine metadata
├── workflows/
│   ├── :workflow_id/execute      POST  — Multi-engine workflow
│   └── :workflow_id/info         GET   — Workflow metadata
├── panchanga/                    POST  — Legacy Selemene endpoint (preserved)
├── ghati/...                           — Existing Ghati routes (preserved)
├── ghati-panchanga/...                 — Existing integration routes (preserved)
├── health                        GET
├── status                        GET
└── metrics                       GET
```

**6 Pre-defined Workflows** (from foundational-research.md):
1. `birth-blueprint` — Numerology + HD + Gene Keys
2. `daily-practice` — Panchanga + VedicClock-TCM + Biorhythm
3. `decision-support` — Tarot + I-Ching + HD transit
4. `self-inquiry` — Gene Keys + Enneagram + Witness prompts
5. `creative-expression` — Sigil Forge + Sacred Geometry + Raaga
6. `full-spectrum` — All 13 engines

---

## TypeScript Bridge

5 engines stay in TypeScript (Tarot, I-Ching, Enneagram, Sacred Geometry, Sigil Forge) because they are inherently interactive/visual. They run as a Bun HTTP server on port 3001.

```rust
// crates/noesis-bridge/src/lib.rs
pub struct BridgeEngine {
    engine_id: String,
    base_url: String,  // http://localhost:3001
    client: reqwest::Client,
}

#[async_trait]
impl ConsciousnessEngine for BridgeEngine {
    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        let resp = self.client
            .post(format!("{}/engines/{}/calculate", self.base_url, self.engine_id))
            .json(&input)
            .send().await?;
        Ok(resp.json().await?)
    }
}
```

---

## Witness Agent Integration

Every `EngineOutput` includes a `witness_prompt` — a self-inquiry question generated from the calculation results. The `noesis-witness` crate provides prompt templates per consciousness level:

- Level 0 (Dormant): Observational prompts ("Notice what you feel when...")
- Level 1 (Glimpsing): Reflective prompts ("What pattern do you see in...")
- Level 2 (Practicing): Inquiry prompts ("Who is the one observing...")
- Level 3 (Integrated): Authorship prompts ("How might you choose to...")
- Level 4-5 (Embodied): Open prompts ("What wants to emerge?")

---

## Build Order (by dependency)

### Phase 1: Foundation
1. **`noesis-core`** — ConsciousnessEngine trait, EngineInput/Output, shared types, Swiss Ephemeris wrapper
2. **`noesis-cache`** — Move existing L1/L2/L3 cache code
3. **`noesis-auth`** — Move existing auth code
4. **`noesis-metrics`** — Move existing metrics code
5. **`noesis-witness`** — Witness prompt templates

### Phase 2: First Engines (simplest, prove the pattern)
6. **`engine-panchanga`** — Refactor existing Selemene into ConsciousnessEngine
7. **`engine-numerology`** — Pure math, no external deps
8. **`engine-biorhythm`** — Pure math (3 sine curves)

### Phase 3: Orchestration + API
9. **`noesis-orchestrator`** — WorkflowOrchestrator with parallel execution
10. **`noesis-api`** — Refactor existing Axum routes + add engine/workflow routes
11. **`noesis-bridge`** — TypeScript adapter

### Phase 4: Complex Engines (depend on Swiss Ephemeris)
12. **`engine-human-design`** — 88° solar arc, gates, channels, centers
13. **`engine-gene-keys`** — Shadow-Gift-Siddhi, 3 sequences
14. **`engine-vimshottari`** — 120-year dasha timeline

### Phase 5: Specialized Engines
15. **`engine-vedic-clock`** — TCM organ clock + Panchanga temporal
16. **`engine-biofield`** — Rust compute core for biometric analysis
17. **`engine-face-reading`** — MediaPipe mesh Rust bindings

### Phase 6: TypeScript Engines
18. **`ts-engines/tarot`**, `i-ching`, `enneagram`, `sacred-geometry`, `sigil-forge`

---

## Critical Files to Modify

### Workspace Setup
- `Cargo.toml` — Convert to `[workspace]` with members list
- Create `crates/` directory structure (17 crate directories)
- Each crate gets `Cargo.toml` + `src/lib.rs`

### Migration (move existing code)
- `src/cache/*` → `crates/noesis-cache/src/`
- `src/auth/*` → `crates/noesis-auth/src/`
- `src/metrics/*` → `crates/noesis-metrics/src/`
- `src/models/mod.rs` → `crates/noesis-core/src/models.rs` (refactored)
- `src/engines/native_solar.rs` → `crates/engine-panchanga/src/`
- `src/engines/native_lunar.rs` → `crates/engine-panchanga/src/`
- `src/engines/panchanga_calculator.rs` → `crates/engine-panchanga/src/`
- `src/engines/swiss_ephemeris.rs` → `crates/noesis-core/src/ephemeris.rs`
- `src/api/*` → `crates/noesis-api/src/`
- `src/main.rs` → `crates/noesis-api/src/main.rs`

### New Files (core trait + types)
- `crates/noesis-core/src/lib.rs` — ConsciousnessEngine trait
- `crates/noesis-core/src/types.rs` — EngineInput, EngineOutput, BirthData
- `crates/noesis-core/src/error.rs` — Unified EngineError
- `crates/noesis-orchestrator/src/lib.rs` — WorkflowOrchestrator
- `crates/noesis-bridge/src/lib.rs` — BridgeEngine
- `crates/noesis-witness/src/lib.rs` — Witness prompt templates

---

## Execution Plan

### Step 1: Create workspace structure
- Convert root `Cargo.toml` to workspace
- Create all crate directories with `Cargo.toml` and stub `src/lib.rs`
- Ensure `cargo check` passes with empty crates

### Step 2: Implement noesis-core
- Define `ConsciousnessEngine` trait
- Define `EngineInput`, `EngineOutput`, `BirthData`, `Coordinates`, `Precision`
- Define unified `EngineError`
- Move Swiss Ephemeris wrapper as shared service

### Step 3: Move shared services
- Move cache code → `noesis-cache`
- Move auth code → `noesis-auth`
- Move metrics code → `noesis-metrics`
- Update all `use` paths, ensure compilation

### Step 4: Refactor engine-panchanga
- Move solar/lunar/panchanga code into `engine-panchanga`
- Implement `ConsciousnessEngine` trait for Panchanga
- Add witness prompt generation
- Verify existing functionality preserved

### Step 5: Build orchestrator + API
- Implement `WorkflowOrchestrator` with `tokio::join!` parallel execution
- Refactor Axum routes to include `/engines/:id/calculate` and `/workflows/:id/execute`
- Preserve all existing Panchanga/Ghati endpoints
- Wire orchestrator into API

### Step 6: Implement simple engines
- `engine-numerology` — Pythagorean + Chaldean name/date reduction
- `engine-biorhythm` — 3 sine cycles from birth date

### Step 7: Implement bridge + TS engines
- `noesis-bridge` — HTTP adapter implementing ConsciousnessEngine
- Scaffold `ts-engines/` with Bun HTTP server
- Implement Tarot engine as first TS engine

### Step 8: Complex engines (Swiss Ephemeris dependent)
- `engine-human-design` — port from evolution-docs specs
- `engine-gene-keys` — port from evolution-docs specs
- `engine-vimshottari` — port from evolution-docs specs

### Step 9: Specialized engines
- `engine-vedic-clock`, `engine-biofield`, `engine-face-reading`
- Remaining TS engines: I-Ching, Enneagram, Sacred Geometry, Sigil Forge

---

## Verification

After each step:
1. `cargo check --workspace` — all crates compile
2. `cargo test --workspace` — existing tests pass
3. `cargo run -p noesis-api` — server starts, health endpoint responds
4. After Step 4: `curl -X POST http://localhost:8080/api/v1/panchanga` returns valid Panchanga
5. After Step 5: `curl -X POST http://localhost:8080/api/v1/engines/panchanga/calculate` works
6. After Step 6: Numerology and Biorhythm engines respond
7. End-to-end: `curl -X POST http://localhost:8080/api/v1/workflows/birth-blueprint/execute` runs 3 engines in parallel

---

## Documentation (deferred)

The original doc restructuring plan (4 files → evolution-docs pattern) will be executed AFTER the workspace refactor is complete, since the codebase will have changed significantly. The new docs will cover the multi-engine architecture rather than just Panchanga.
