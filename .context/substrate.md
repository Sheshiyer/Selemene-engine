# Tryambakam Noesis - Project Substrate

> **Documentation as Code as Context** - Entry point for AI-assisted development

**Project:** Tryambakam Noesis  
**Current State:** Selemene Engine (single Panchanga engine in Rust)  
**Vision:** 16-engine consciousness computing platform for self-consciousness development  
**Status:** Late development → Transformation phase  
**Date:** 2026-01-30

---

## 🎯 What is Tryambakam Noesis?

Tryambakam Noesis is a **self-consciousness development platform** built on 16 complementary engines that serve as **Decision Mirrors** - they reflect your patterns back to you so you can **witness** them, not prescribe what to do.

### Core Philosophy

**Self-consciousness ≠ Consciousness**

- **Consciousness**: The field itself, awareness as ground of being (cannot be developed)
- **Self-consciousness**: The witness, "I am aware that I am aware" (CAN be developed)

**Noesis develops self-consciousness through:**
1. **WITNESS** - Observe patterns without identifying with them
2. **UNDERSTAND** - See patterns clearly across multiple systems
3. **AUTHOR** - Consciously respond vs. unconsciously react

---

## 🏗️ Current Architecture (Selemene Engine)

### What Exists Now

**Single-crate Rust project** providing astronomical calculation services:

```
selemene-engine/
├── src/
│   ├── api/           # Axum HTTP server (routes, handlers, middleware)
│   ├── cache/         # L1 (LRU), L2 (Redis), L3 (disk) - 3-layer cache
│   ├── auth/          # JWT + API key authentication
│   ├── metrics/       # Prometheus monitoring
│   ├── engines/       # Calculation engines:
│   │   ├── calculation_orchestrator.rs  # Coordinates all calculations
│   │   ├── hybrid_backend.rs            # Native vs Swiss Ephemeris routing
│   │   ├── native_solar.rs              # VSOP87 solar engine
│   │   ├── native_lunar.rs              # ELP-2000 lunar engine
│   │   └── panchanga_calculator.rs      # Vedic Panchanga
│   ├── time/          # Ghati time system, realtime tracking
│   └── models/        # Request/response types, errors
├── data/
│   ├── ephemeris/     # Swiss Ephemeris data files
│   └── wisdom-docs/   # 35 JSON files of archetypal data (NEW)
└── Cargo.toml         # Single crate (soon to be workspace)
```

### Key Infrastructure (To Preserve)

✅ **Three-Layer Cache Architecture** - L1 (in-memory LRU) → L2 (Redis distributed) → L3 (disk precomputed)  
✅ **Hybrid Backend Strategy** - Native engines (VSOP87/ELP-2000) with Swiss Ephemeris fallback  
✅ **Orchestrator Pattern** - All calculations flow through `CalculationOrchestrator`  
✅ **Authentication System** - JWT + API keys with role-based access  
✅ **Metrics & Monitoring** - Prometheus instrumentation throughout  
✅ **Ghati Time System** - Hybrid 24-minute fixed intervals + solar adjustments

**Status:** Late development - architecture complete, core implementations pending

---

## 🔮 Target Architecture (Tryambakam Noesis)

### Vision: 16-Engine Consciousness Computing Platform

Transform Selemene into the **core backend for all 16 self-consciousness engines**:

```
Cargo Workspace Monorepo
├── crates/
│   ├── noesis-core/           # ConsciousnessEngine trait, shared types
│   ├── noesis-api/            # Axum HTTP (refactored from src/api/)
│   ├── noesis-cache/          # Multi-tier cache (from src/cache/)
│   ├── noesis-auth/           # Authentication (from src/auth/)
│   ├── noesis-metrics/        # Monitoring (from src/metrics/)
│   ├── noesis-orchestrator/   # Workflow engine, parallel execution
│   ├── noesis-bridge/         # TypeScript engine HTTP adapter
│   ├── noesis-witness/        # Witness Agent prompt injection
│   │
│   ├── engine-panchanga/      # REFACTORED: Existing Selemene
│   ├── engine-numerology/     # Pythagorean + Chaldean
│   ├── engine-human-design/   # 88° solar arc, 64 gates
│   ├── engine-biorhythm/      # 3 sine cycles (23/28/33 day)
│   ├── engine-vimshottari/    # 120-year dasha periods
│   ├── engine-gene-keys/      # Shadow-Gift-Siddhi transformation
│   ├── engine-vedic-clock/    # TCM organ clock + Vedic time
│   ├── engine-biofield/       # Biometric analysis (Rust compute)
│   └── engine-face-reading/   # MediaPipe mesh analysis (Rust compute)
│
├── ts-engines/                # TypeScript engines (Bun HTTP server)
│   ├── tarot/                 # Archetypal card readings
│   ├── i-ching/               # Wisdom transmission (64 hexagrams)
│   ├── enneagram/             # 9 personality types + integration
│   ├── sacred-geometry/       # Visual pattern generation
│   └── sigil-forge/           # Intention encoding
│
└── data/
    ├── ephemeris/             # Swiss Ephemeris (shared by HD, Gene Keys, Vimshottari)
    └── wisdom-docs/           # 35 JSON files - archetypal data corpus
```

### The Core Trait: ConsciousnessEngine

All 16 engines (11 Rust + 5 TypeScript) implement:

```rust
#[async_trait]
pub trait ConsciousnessEngine: Send + Sync {
    fn engine_id(&self) -> &str;
    fn engine_name(&self) -> &str;
    fn required_phase(&self) -> u8;  // 0-5 consciousness level access gating
    
    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError>;
    async fn validate(&self, output: &EngineOutput) -> Result<ValidationResult, EngineError>;
    fn cache_key(&self, input: &EngineInput) -> String;  // SHA-256 deterministic
}

pub struct EngineOutput {
    pub engine_id: String,
    pub result: Value,                // Engine-specific JSON
    pub witness_prompt: String,       // Self-inquiry question (CRITICAL)
    pub consciousness_level: u8,      // 0-5 depth indicator
    pub metadata: CalculationMetadata,
}
```

**Critical Innovation**: Every calculation includes a `witness_prompt` - a self-inquiry question that trains the observer capacity.

---

## 🧬 The 16 Self-Consciousness Engines

### Foundational (Pure Math, No Dependencies)
1. **Numerology** - Life path patterns (Pythagorean + Chaldean)
2. **Biorhythm** - Personal cycles (23/28/33 day sine waves)

### Systemic (Astronomical, Swiss Ephemeris Dependent)
3. **Human Design** - Energetic blueprint (88° solar arc, 64 gates)
4. **Gene Keys** - Shadow-Gift-Siddhi consciousness evolution
5. **Vimshottari Dasha** - 120-year planetary period timeline

### Temporal (Time Optimization)
6. **Panchanga** - Vedic 5-limb time system (Tithi, Vara, Nakshatra, Yoga, Karana)
7. **VedicClock-TCM** - Chrono-biological optimization (Vedic + Chinese Medicine)

### Archetypal (Symbolic, TypeScript)
8. **Tarot** - 78-card archetypal mirrors
9. **I-Ching** - 64 hexagram wisdom transmission

### Typological (Pattern Recognition)
10. **Enneagram** - 9 personality types + wings + integration paths

### Generative (Visual/Creative, TypeScript)
11. **Sacred Geometry** - Pattern language visualization
12. **Sigil Forge** - Intention encoding through symbol generation

### Diagnostic (Physical Analysis)
13. **Face Reading** - Vedic + TCM facial diagnosis
14. **Biofield** - Energetic field measurement (requires hardware)

---

## 🔗 6 Pre-defined Workflows

Multi-engine workflows that reveal core patterns through cross-validation:

1. **birth-blueprint** — Numerology + HD + Vimshottari (Who am I designed to be?)
2. **daily-practice** — Panchanga + VedicClock-TCM + Biorhythm (Optimal timing today)
3. **decision-support** — Tarot + I-Ching + HD transit (Guidance on this choice)
4. **self-inquiry** — Gene Keys + Enneagram + Witness prompts (Shadow work)
5. **creative-expression** — Sigil Forge + Sacred Geometry + Raaga (Generative)
6. **full-spectrum** — All 16 engines (Complete self-portrait)

---

## 💎 Wisdom Data Corpus

### 35 JSON Files of Archetypal Knowledge

Located in `data/wisdom-docs/`, extracted from WitnessOS:

| System | Files | Records | Purpose |
|--------|-------|---------|---------|
| Human Design | 12 | ~500+ | Gates, centers, channels, types, authorities, profiles |
| Astrology/Vimshottari | 4 | ~100 | Nakshatras, dashas, planets |
| Face Reading | 4 | ~200+ | Landmarks, TCM zones, Vedic zones |
| I Ching | 2 | 64 hexagrams | Complete with line interpretations |
| Tarot | 2 | 78 cards | Major + Minor Arcana (Rider-Waite) |
| Gene Keys | 1 | 64 keys | Shadow-Gift-Siddhi for each gate |
| Enneagram | 1 | 9 types | Full type descriptions + wings |
| TCM/Vedic Clock | 5 | ~150 | Organ clock, elements, practices |
| Sacred Geometry | 2 | ~50 symbols | Geometric templates |
| Biofield/Biorhythm | 3 | Algorithmic | Spatial/temporal algorithms |

**Key Principle**: Preserve archetypal depth - these are millennia of crystallized wisdom, not mere data structures.

---

## 🛠️ Tech Stack

### Backend (Rust)
- **Framework**: Axum (async HTTP)
- **Runtime**: Tokio (async executor)
- **Serialization**: Serde (JSON)
- **Database**: PostgreSQL (via SQLx)
- **Cache**: Redis (L2 distributed cache)
- **Ephemeris**: Swiss Ephemeris (via `swisseph` crate)
- **Auth**: JWT (via `jsonwebtoken`)
- **Metrics**: Prometheus

### Frontend Bridge (TypeScript)
- **Runtime**: Bun (fast JS runtime)
- **Framework**: Express or Hono (HTTP server)
- **Engine Count**: 5 (Tarot, I-Ching, Enneagram, Sacred Geometry, Sigil Forge)
- **Communication**: HTTP bridge to Rust API

### Infrastructure
- **Deployment**: Docker containers
- **Orchestration**: Kubernetes (production)
- **Monitoring**: Prometheus + Grafana
- **Logging**: Structured JSON logs

---

## 📍 Current Phase: Transformation

### What's Happening Now

**Migrating from**: Single-crate Selemene Engine  
**Migrating to**: Multi-crate Tryambakam Noesis workspace

**Transformation Plan** (from `.claude/crystalline-giggling-trinket.md`):

#### Phase 1: Foundation (4-6 weeks)
- Create `noesis-core` with `ConsciousnessEngine` trait
- Move cache/auth/metrics to dedicated crates
- Implement witness prompt system

#### Phase 2: First Engines (6-8 weeks)
- Refactor Panchanga into `engine-panchanga`
- Implement Numerology (prove the pattern)
- Implement Biorhythm (simple algorithmic)

#### Phase 3: Orchestration (8-10 weeks)
- Build `noesis-orchestrator` with parallel execution
- Refactor API to support `/engines/:id/calculate` and `/workflows/:id/execute`
- Implement TypeScript bridge for TS engines

#### Phase 4: Complex Engines (4-6 weeks)
- Human Design (astronomical accuracy critical)
- Gene Keys (HD-dependent)
- Vimshottari (Swiss Ephemeris intensive)

**See**: `.context/migration/transformation-roadmap.md` for detailed steps

---

## 🎓 Key Concepts & Glossary

### Technical Terms

- **Ghati**: Vedic time unit (1 day = 60 ghatis, 1 ghati = 24 minutes)
- **Panchanga**: 5 limbs of Vedic time (Tithi, Vara, Nakshatra, Yoga, Karana)
- **Backend Strategy**: Native (VSOP87/ELP) vs Swiss Ephemeris fallback routing
- **Precision Level**: Standard/High/Extreme calculation accuracy modes
- **Cache Key**: SHA-256 hash of normalized input (deterministic)
- **Orchestrator Pattern**: All calculations flow through central coordinator

### Mystical/Archetypal Terms

- **Shadow-Gift-Siddhi**: Gene Keys consciousness progression (unconscious → present → transcendent)
- **Witness Prompt**: Self-inquiry question generated from calculation results
- **Archetypal Data**: JSON wisdom files (not "content" - crystallized millennia of insight)
- **Consciousness Level**: 0-5 scale (Dormant → Glimpsing → Practicing → Integrated → Embodied)
- **Decision Mirror**: Engine showing pattern, not prescribing action

---

## 📚 Documentation Structure

This `.context/` folder follows the Substrate Methodology:

```
.context/
├── substrate.md              # THIS FILE - Project overview
├── ai-rules.md               # Hard constraints (MUST follow)
├── anti-patterns.md          # Forbidden approaches (NEVER do)
├── glossary.md               # Technical + mystical terminology
│
├── architecture/
│   ├── overview.md           # Workspace structure
│   ├── trait-system.md       # ConsciousnessEngine trait design
│   ├── backend-routing.md    # Hybrid backend strategy
│   └── cache-layers.md       # L1/L2/L3 cache architecture
│
├── engines/
│   ├── engine-design.md      # Template for new engines
│   ├── panchanga.md          # Astronomical engine specifics
│   ├── human-design.md       # HD calculation details
│   └── [other engines]/      # Per-engine documentation
│
├── wisdom-data/
│   ├── overview.md           # 35 JSON files catalog
│   ├── loading-patterns.md   # Serde + lazy_static patterns
│   └── semantic-meanings.md  # Archetypal significance
│
├── api/
│   ├── endpoints.md          # Route documentation
│   └── authentication.md     # JWT + API key patterns
│
├── consciousness/
│   ├── witness-prompts.md    # Self-inquiry generation
│   └── levels.md             # 0-5 consciousness framework
│
└── migration/
    ├── transformation-roadmap.md  # Phase-by-phase plan
    └── current-progress.md        # What's done, what's next
```

---

## 🚀 Quick Start for AI Assistants

When asked to implement features, always:

1. **Read** `.context/ai-rules.md` for hard constraints
2. **Check** `.context/anti-patterns.md` for what NOT to do
3. **Reference** relevant domain docs (architecture/, engines/, wisdom-data/)
4. **Preserve** the orchestrator pattern - never bypass it
5. **Include** witness prompts in all engine outputs
6. **Maintain** archetypal depth from wisdom-docs

---

## 🔗 External References

- **Source Evolution Docs**: `/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/evolution-docs/`
  - `foundational-research.md` - Core philosophy
  - `core-engine-architecture.md` - 16 engines detailed
  - `self-consciousness-framework.md` - 5-level consciousness model
  - `technical-learnings.md` - HD astronomical accuracy, VedicClock-TCM integration
  - `IMPLEMENTATION-ROADMAP.md` - Build sequence with timelines

- **Current Project Docs**:
  - `selemene_architecture.md` - Existing Selemene deep-dive
  - `GHATI_CALCULATION_STANDARDS.md` - Ghati time system specification
  - `.claude/crystalline-giggling-trinket.md` - Active transformation plan

---

## 📞 Development Workflow

### For New Features
1. Create feature branch
2. Reference `.context/` for patterns
3. Implement following trait contracts
4. Add tests (unit + integration)
5. Update `.context/` if patterns evolve
6. PR with documentation updates

### For Bug Fixes
1. Reproduce issue
2. Check `.context/anti-patterns.md` - is it a known violation?
3. Fix following existing patterns
4. Add regression test
5. Update `.context/` if pattern discovered

---

**Last Updated**: 2026-01-30  
**Document Version**: 1.0.0  
**Status**: 🟢 Active - Transformation phase
