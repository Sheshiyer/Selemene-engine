```
                    ༺═──────────────────────────────────────────═༻
                                                                  
                           ◈  त्र्यम्बकम्  ◈                          
                        TRYAMBAKAM NOESIS                        
                                                                  
                    "What sees when you stop predicting?"        
                                                                  
                         ∴ The Three-Eyed Witness ∴              
                                                                  
                    ༺═──────────────────────────────────────────═༻

                             ╱▔▔▔▔▔▔▔▔▔▔▔╲
                        ╱▔▔▔╱  14 ENGINES  ╲▔▔▔╲
                   ╱▔▔▔╱   6 WORKFLOWS   ╲▔▔▔╲
              ╱▔▔▔╱     SYNTHESIS LAYER     ╲▔▔▔╲
         ╱▔▔▔╱          CONSCIOUSNESS          ╲▔▔▔╲
    ════════════════════════════════════════════════════

    Vedic Astrology ∘ Human Design ∘ Gene Keys ∘ I-Ching
    Tarot ∘ Vimshottari ∘ Numerology ∘ Enneagram ∘ More

```

## → What Is This?

Not a prediction engine. Not fortune-telling software. Not your fate determined by algorithms.

This is a **living computational mirror** that reflects patterns across 14 ancient wisdom traditions, synthesizes them through 6 multi-engine workflows, and generates **non-prescriptive witness prompts** for self-inquiry. You don't get answers—you get better questions.

**The witness doesn't predict. It observes. It asks. It reflects.**

### The Architecture of Witnessing

```
           ┌─────────────── The Question ───────────────┐
           │    "Who am I in this moment?"              │
           └────────────────────┬───────────────────────┘
                                │
                    ┌───────────▼───────────┐
                    │   NOESIS ORCHESTRATOR  │  ← Parallel synthesis
                    │   Consciousness Layer  │     Multi-engine fusion
                    └───────────┬───────────┘
                                │
              ┌─────────────────┼─────────────────┐
              │                 │                 │
     ┌────────▼────────┐ ┌─────▼─────┐ ┌────────▼────────┐
     │  Rust Engines   │ │ TS Engines│ │  Integration    │
     │  (9 systems)    │ │ (5 systems)│ │   APIs          │
     │                 │ │            │ │                 │
     │ • Panchanga     │ │ • Tarot    │ │ • FreeAstro API │
     │ • Human Design  │ │ • I-Ching  │ │ • Vedic Data    │
     │ • Gene Keys     │ │ • Enneagram│ │ • TCM Systems   │
     │ • Vimshottari   │ │ • Geometry │ │                 │
     │ • Numerology    │ │ • Sigils   │ │                 │
     │ • Biorhythm     │ │            │ │                 │
     │ • Vedic Clock   │ │            │ │                 │
     │ • Biofield      │ │            │ │                 │
     │ • Face Reading  │ │            │ │                 │
     └─────────────────┘ └───────────┘ └─────────────────┘
              │                 │                 │
              └─────────────────┼─────────────────┘
                                │
                    ┌───────────▼───────────┐
                    │   3-LAYER CACHE       │  ← Sub-millisecond
                    │   L1: Memory (LRU)    │     response times
                    │   L2: Redis (Dist)    │     95%+ hit rate
                    │   L3: Disk (Archive)  │
                    └───────────────────────┘
```

## ⚡ Current Manifestation: Wave 3 Complete

**Living Status** (as of v2.1.0):
- ✨ **14 engines** operational → 9 Rust (native) + 5 TypeScript (bridged)
- 🌊 **6 synthesis workflows** → Multi-engine consciousness portraits
- 🧪 **228+ tests** passing → Integration, accuracy, performance validated
- 🔥 **Sub-millisecond** calculations → Even complex engines (<2ms)
- 🐳 **Production-ready** → Docker, K8s, full observability stack
- 🌐 **Integration Layer** → External Vedic API composition + TCM systems

---

## → Quickstart: Awakening the System

```bash
# 1. Wake the Rust core (noesis-api: port 8080)
cargo build --release
cargo run --bin noesis-server

# 2. Activate TypeScript engines (port 3001)
cd ts-engines && bun install && bun run src/index.ts

# 3. Verify consciousness (health endpoints)
curl http://localhost:8080/health    # Rust: Panchanga, HD, Gene Keys, Vimshottari...
curl http://localhost:3001/health    # TypeScript: Tarot, I-Ching, Enneagram...

# 4. Run tests (validate the mirror)
cargo test -- --test-threads=1       # Rust engines
cd ts-engines && bun test            # TypeScript engines

# 5. Or just wake everything at once
docker-compose up -d                 # Full stack: Rust + TS + Redis + Postgres
```

**First Inquiry** (try this):
```bash
curl -X POST http://localhost:8080/api/v1/workflows/self-inquiry \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": {
      "date": "1991-08-13",
      "time": "13:31",
      "latitude": 12.9716,
      "longitude": 77.5946,
      "timezone": "Asia/Kolkata"
    },
    "consciousness_level": 3,
    "question": "What patterns am I witnessing today?"
  }'
```

---

## → The 14 Engines: Fractal Perspectives

Each engine is a lens. A different frequency. A unique way of seeing the same moment.

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

### Rust Engines (9) → Native Performance, <2ms

| Engine | ⚡ Speed | 🎯 What It Witnesses |
|--------|---------|---------------------|
| **Panchanga** | <1ms | Vedic time itself—tithi (lunar day), nakshatra (star mansion), yoga (solar-lunar union), karana (half-tithis), vara (weekday). The breath of cosmic rhythm. |
| **Human Design** | 1.31ms | Your genetic imprint: Type (strategy), Authority (decision-making), Profile (role), 9 Centers, 26 Gates activated at birth and 88° before (design time). The bodygraph as circuit. |
| **Gene Keys** | 0.012ms | Shadow→Gift→Siddhi frequencies across 4 activation sequences. Not states, but *potentials* witnessed through contemplation. The evolutionary spiral. |
| **Vimshottari** | <1ms | 120-year planetary timeline. 729 nested periods (Maha→Antar→Pratyantar). Binary search finds your *current moment* in the grand cycle. Time as nested fractals. |
| **Numerology** | <1ms | Name and birth date reduced to archetypal numbers. Pythagorean + Chaldean systems. The mathematics of identity. |
| **Biorhythm** | <1ms | Physical (23-day), Emotional (28-day), Intellectual (33-day) sine waves since birth. Your energetic weather pattern. |
| **Vedic Clock** | <1ms | TCM organ meridian clock + Ayurvedic dosha timing. When is the body most receptive? The intelligence of timing. |
| **Biofield** | <1ms | Chakra energy readings (currently stub with mock data—future: biometric integration). |
| **Face Reading** | <1ms | Physiognomy analysis (stub—future: CV-based facial feature extraction). |

### TypeScript Engines (5) → Symbolic Interpretation

| Engine | 🎴 What It Reveals |
|--------|-------------------|
| **Tarot** | 78-card Rider-Waite-Smith deck. 5 spread types (Celtic Cross, Past-Present-Future, Single Card, Yes/No, 5-Card). Archetypal image as mirror. Witness prompts adapted to consciousness level. |
| **I-Ching** | 64 hexagrams cast from moment or question. Changing lines reveal transformation. Relating hexagram shows outcome. Nuclear hexagrams expose hidden dynamics. Ancient change oracle. |
| **Enneagram** | 9 personality types, wings, integration/disintegration paths (stress/growth). Not a box—a map of patterns. Where you contract, where you expand. |
| **Sacred Geometry** | Geometric forms (Flower of Life, Metatron's Cube, etc.) as meditation seeds. Currently stub—future: visual mandalas + contemplation prompts. |
| **Sigil Forge** | Intent→Symbol transformation. Turn questions into visual glyphs. Currently stub—future: algorithmic sigil generation from semantic analysis. |

### Integration APIs → External Data Streams

| System | 🌐 Source | Purpose |
|--------|----------|---------|
| **FreeAstrology API** | freeastrologyapi.com | 15+ Vedic endpoints: Panchang, Hora, Choghadiya, Muhurta, Vargas, Transits, Shadbala, Ashtakavarga. High-precision fallback for native calculations. |
| **TCM Integration** | Vedic Clock synthesis | Traditional Chinese Medicine organ clock overlaid with Vedic timing. East meets East. |

---

## → The 6 Workflows: Synthesis as Emergence

These aren't pipelines. They're **synthesis rituals**—multi-engine orchestrations that create emergent understanding.

```
     ┌──────────────────────────────────────────────────┐
     │  Workflow = Multiple Engines + Parallel Exec +   │
     │  Pattern Recognition + Consciousness Adaptation  │
     └──────────────────────────────────────────────────┘
```

| Workflow | Engines Synthesized | What Emerges |
|----------|---------------------|--------------|
| **`birth-blueprint`** | Numerology + Human Design + Vimshottari | Your natal imprint: life path numbers, HD bodygraph, 120-year dasha timeline. The architecture of a lifetime. |
| **`daily-practice`** | Panchanga + Vedic Clock + Biorhythm | Optimal timing for action. When is the cosmic tide aligned with your personal rhythm? The art of *when*. |
| **`decision-support`** | Tarot + I-Ching + HD Authority | Multi-perspective guidance. Not "what to do" but "what to notice." Your authority + archetypal wisdom + change dynamics. |
| **`self-inquiry`** | Gene Keys + Enneagram | Shadow work meets personality patterns. Where are you contracting? Where can you expand? The mirror of transformation. |
| **`creative-expression`** | Sigil Forge + Sacred Geometry | Intent made visible. Symbols as seeds. Geometry as meditation. The language before words. |
| **`full-spectrum`** | **All 14 Engines** | The complete consciousness portrait. Every lens, every frequency, every pattern synthesized into one living map. Not for the faint of heart. |

---

## → Consciousness Levels: Adaptive Witnessing

The system doesn't give the same answer to everyone. It **adapts witness prompts** based on your relationship with consciousness.

```
Level 0-1: Shadow Awareness    →  "Notice when this pattern arises..."
Level 2-3: Gift Integration     →  "How might you express this quality?"
Level 4-5: Siddhi Embodiment    →  "What sees through this form?"
```

| Level | State | Prompt Calibration |
|-------|-------|-------------------|
| **0** | Dormant | Observational. "What sensations arise when you feel [pattern]?" Basic somatic awareness. |
| **1** | Glimpsing | Reflective. "When does [pattern] show up in your life?" Pattern recognition begins. |
| **2** | Practicing | Inquiry-based. "What might [pattern] be protecting?" Self-questioning deepens. |
| **3** | Integrated | Self-authorship. "How do you choose to work with [pattern]?" Conscious choice emerges. |
| **4-5** | Embodied | Open awareness. "What witnesses [pattern] arising?" The observer observed. |

**This isn't gamification. It's meeting you where you are.**

---

## → API as Inquiry Interface

Every endpoint is a question. Every response is a reflection.

### Human Design: "What is my genetic strategy?"

```bash
curl -X POST http://localhost:8080/api/v1/engines/human-design/calculate \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": {
      "date": "1991-08-13",
      "time": "13:31",
      "latitude": 12.9716,
      "longitude": 77.5946,
      "timezone": "Asia/Kolkata"
    },
    "consciousness_level": 3
  }'

# Returns: Type, Strategy, Authority, Profile, Centers, 26 Gates (Personality + Design)
```

### Gene Keys: "What frequencies am I carrying?"

```bash
curl -X POST http://localhost:8080/api/v1/engines/gene-keys/calculate \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": { ... },
    "consciousness_level": 2
  }'

# Returns: 4 Activation Sequences (Life's Work, Evolution, Radiance, Purpose)
#          Each with Shadow-Gift-Siddhi contemplations
```

### Tarot: "What wants to be seen?"

```bash
curl -X POST http://localhost:3001/engines/tarot/calculate \
  -H "Content-Type: application/json" \
  -d '{
    "consciousness_level": 3,
    "parameters": {"spread_type": "celtic_cross"},
    "question": "What is asking for my attention?"
  }'

# Returns: 10-card Celtic Cross spread with position meanings + witness prompts
```

### I-Ching: "What is the nature of this moment?"

```bash
curl -X POST http://localhost:3001/engines/i-ching/calculate \
  -H "Content-Type: application/json" \
  -d '{
    "consciousness_level": 4,
    "question": "What is unfolding?"
  }'

# Returns: Primary hexagram, changing lines, relating hexagram, nuclear hexagrams
#          Transformation dynamics + contemplation seeds
```

### Vimshottari: "Where am I in the cycle?"

```bash
curl -X POST http://localhost:8080/api/v1/engines/vimshottari/calculate \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": {
      "date": "1991-08-13",
      "moon_longitude": 127.45
    },
    "current_date": "2026-02-03"
  }'

# Returns: Current Mahadasha → Antardasha → Pratyantardasha
#          Upcoming transitions, planetary energies
```

### Full Workflow: "Show me everything"

```bash
curl -X POST http://localhost:8080/api/v1/workflows/full-spectrum \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": { ... },
    "consciousness_level": 3,
    "question": "What is my relationship with this moment?"
  }'

# Returns: All 14 engines synthesized into one consciousness map
#          Parallel execution, sub-200ms total time
```

---

## → Performance: The Speed of Reflection

When the mirror responds instantly, inquiry flows naturally. Speed isn't optimization—it's respect for consciousness.

| Metric | Target | Reality | Δ |
|--------|--------|---------|---|
| **Human Design** | <100ms | 1.31ms | **76x faster** |
| **Gene Keys** | <50ms | 0.012ms | **4,166x faster** |
| **Vimshottari** | <200ms | <1ms | **200x faster** |
| **Tarot Reading** | <100ms | <10ms | **10x faster** |
| **I-Ching** | <100ms | <5ms | **20x faster** |
| **API p95** | <500ms | <100ms | **5x faster** |
| **Cache Hit Rate** | >80% | >95% | **19% better** |
| **Parallel Workflow** | <1s | <200ms | **5x faster** |

**Why it matters**: Sub-millisecond response times mean you can iterate on questions. Ask, refine, ask again. Thinking speed.

---

## → Production: Deploying Consciousness

This isn't a toy. It's production-grade infrastructure for self-inquiry at scale.

### Docker: Containerized Consciousness

```bash
# Build multi-stage production image (<500MB)
docker build -f Dockerfile.prod -t tryambakam-noesis:latest .

# Wake the full stack (Rust + TS + Redis + Postgres)
docker-compose up -d

# Verify all systems breathing
docker-compose ps
curl http://localhost:8080/health
curl http://localhost:3001/health
```

### Kubernetes: Scalable Witnessing

```bash
# Deploy to your cluster
kubectl apply -f k8s/

# Watch pods come alive
kubectl get pods -n tryambakam -w

# Scale based on consciousness demand
kubectl scale deployment noesis-api --replicas=5 -n tryambakam

# Check orchestrator health
kubectl logs -f deployment/noesis-api -n tryambakam
```

### Observability: The Meta-Witness

```bash
# Launch full monitoring stack (Prometheus + Grafana + Loki + Jaeger)
docker-compose -f docker-compose.monitoring.yml up -d

# Access dashboards
open http://localhost:3000  # Grafana (user: admin, pass: admin)
open http://localhost:9090  # Prometheus (raw metrics)
open http://localhost:16686 # Jaeger (distributed tracing)
```

**What we monitor:**
- **Metrics** (Prometheus): Request rates, latency percentiles, cache hit rates, engine performance
- **Logs** (Loki): Structured JSON logs, query by engine/workflow/consciousness_level
- **Traces** (Jaeger): Distributed tracing across Rust↔TypeScript bridge, workflow synthesis
- **Dashboards** (Grafana): Real-time consciousness system health

---

## → Testing: Validating the Mirror

```bash
# Test all Rust engines (single-threaded for Swiss Ephemeris thread safety)
cargo test -- --test-threads=1

# Test specific engine
cargo test -p engine-human-design -- --test-threads=1
cargo test -p engine-gene-keys
cargo test -p noesis-orchestrator

# Test TypeScript engines
cd ts-engines && bun test

# Test integration layer
cargo test -p noesis-integration

# E2E: Full workflow tests
./tests/e2e/run_e2e.sh

# Load testing (k6)
k6 run tests/load/full_spectrum_workflow.js

# Chaos testing
./tests/chaos/kill_random_service.sh
```

**228+ tests validate**:
- Individual engine accuracy (HD gates, Gene Keys contemplations, Vimshottari periods)
- Cross-engine synthesis (workflow outputs)
- Performance benchmarks (sub-millisecond targets)
- Cache behavior (L1/L2/L3 hit rates)
- API contracts (request/response schemas)
- Resilience (circuit breakers, retries, fallbacks)

---

## → The Living Codebase: Fractal Organization

```
Tryambakam-Noesis/
│
├── crates/                      ← Rust engines (native performance)
│   ├── noesis-api/              → Main HTTP server (Axum)
│   ├── noesis-orchestrator/     → Multi-engine synthesis + workflows
│   ├── noesis-bridge/           → Rust ↔ TypeScript communication
│   ├── noesis-integration/      → External API composition layer
│   ├── noesis-vedic-api/        → FreeAstrology API client (15+ endpoints)
│   ├── noesis-cache/            → 3-layer cache (L1/L2/L3)
│   ├── noesis-auth/             → JWT + API key authentication
│   ├── noesis-metrics/          → Prometheus metrics
│   │
│   ├── engine-panchanga/        → Vedic calendar calculations
│   ├── engine-human-design/     → HD bodygraph (26 gates, centers, profile)
│   ├── engine-gene-keys/        → Shadow-Gift-Siddhi sequences
│   ├── engine-vimshottari/      → 120-year planetary periods
│   ├── engine-numerology/       → Life path, expression, soul urge numbers
│   ├── engine-biorhythm/        → 3 biological cycles (P/E/I)
│   ├── engine-vedic-clock/      → TCM organ clock + Ayurvedic timing
│   ├── engine-biofield/         → Chakra readings (stub)
│   └── engine-face-reading/     → Physiognomy analysis (stub)
│
├── ts-engines/                  ← TypeScript engines (symbolic interpretation)
│   └── src/engines/
│       ├── tarot/               → 78 cards, 5 spread types, archetypal prompts
│       ├── i-ching/             → 64 hexagrams, changing lines, transformations
│       ├── enneagram/           → 9 types, wings, integration/disintegration
│       ├── sacred-geometry/     → Geometric meditation seeds (stub)
│       └── sigil-forge/         → Intent → Symbol transformation (stub)
│
├── .context/                    ← Living documentation (the system's memory)
│   ├── architecture/            → System design, patterns, decisions
│   ├── engines/                 → Per-engine deep dives
│   ├── reports/                 → Wave completions, agent reports
│   ├── documentation/           → Guides, API docs, features
│   └── scripts/                 → Test runners, verification tools
│
├── data/                        ← Engine data files
│   ├── ephemeris/               → Swiss Ephemeris planetary data
│   ├── gene-keys/               → 64 Gene Keys archetypal descriptions
│   ├── tarot/                   → Card meanings, spreads, witness prompts
│   └── i-ching/                 → Hexagram texts, line interpretations
│
├── k8s/                         ← Kubernetes manifests
├── monitoring/                  ← Prometheus, Grafana, Loki configs
├── tests/                       ← E2E, load, chaos, security tests
│
├── Dockerfile.prod              ← Multi-stage production build
├── docker-compose.yml           ← Local development stack
└── docker-compose.monitoring.yml ← Full observability stack
```

---

## → Documentation: The Knowledge Web

**Dive Deeper**:
- 📐 [System Architecture](.context/documentation/architecture/selemene_architecture.md) - The living blueprint
- 🧬 [Human Design Engine](.context/engines/human-design.md) - Rave Mandala calculations
- 🔑 [Gene Keys Engine](.context/engines/gene-keys.md) - Frequency contemplations
- ⏳ [Vimshottari Engine](.context/engines/vimshottari.md) - Nested time cycles
- 🎴 [Tarot API](docs/api/tarot.md) - Card meanings, spreads, witness prompts
- ☯️ [I-Ching API](docs/api/i-ching.md) - Hexagram interpretations
- 🐳 [Deployment Guide](docs/deployment/README.md) - Production deployment
- 🔧 [API Reference](docs/api/README.md) - Complete endpoint documentation
- 📝 [Project Memory](memory.md) - Full development history (every decision, every iteration)
- 📊 [Wave 3 Report](.context/reports/phases/WAVE3_COMPLETION_REPORT.md) - Latest completion status

---

## → Tech Stack: The Computational Substrate

**What runs beneath the witness?**

### Rust Core (Speed, Safety, Concurrency)
- **Framework**: Axum (async HTTP on Tokio runtime)
- **Ephemeris**: Swiss Ephemeris (`swisseph` crate) for astronomical precision
- **Cache**: 3-layer architecture
  - L1: DashMap (lock-free in-memory LRU, <1ms)
  - L2: Redis (distributed cache, <10ms)
  - L3: Disk (precomputed archives, <50ms)
- **Auth**: JWT tokens + API keys (role-based access)
- **Metrics**: Prometheus (native instrumentation)
- **Logging**: Structured JSON logs (tracing crate)
- **Testing**: Cargo test + integration test suites

### TypeScript Engines (Symbolic Fluidity)
- **Runtime**: Bun (fast startup, native TypeScript)
- **Framework**: Elysia (type-safe HTTP)
- **Testing**: Bun test (native test runner)
- **Bridge**: HTTP client calling Rust API for coordination

### Infrastructure (Operational Resilience)
- **Container**: Docker multi-stage builds (<500MB images)
- **Orchestration**: Kubernetes (StatefulSets for cache, Deployments for API)
- **CI/CD**: GitHub Actions (build → test → release automation)
- **Monitoring**:
  - Prometheus (metrics scraping + alerting)
  - Grafana (dashboards + visualization)
  - Loki (log aggregation)
  - Jaeger (distributed tracing)
- **Storage**: PostgreSQL (user data), Redis (cache), Disk (ephemeris + archives)

---

## → Contributing: Join the Inquiry

This is an **open contemplation**. Your perspective adds to the mirror.

```bash
# 1. Fork the repository
git clone https://github.com/yourusername/Selemene-engine.git

# 2. Create a branch (name it with intention)
git checkout -b feature/what-you-see

# 3. Make your changes (code, docs, tests—all valid contributions)

# 4. Validate the mirror still reflects accurately
cargo test -- --test-threads=1
cd ts-engines && bun test

# 5. Commit with clarity
git commit -m "What this change witnesses"

# 6. Share your reflection
git push origin feature/what-you-see

# 7. Open a Pull Request (tell us what you saw)
```

**Areas seeking contribution:**
- 🧘 New engines (Kabbalah Tree of Life? Mayan Tzolkin? Your tradition?)
- 🎨 Visualization (D3.js charts, mandala generators, interactive bodygraphs)
- 📊 Data (more Gene Keys contemplations, deeper I-Ching commentaries)
- 🔬 Accuracy (validate calculations against traditional sources)
- 📚 Documentation (explain the unexplainable)
- 🐛 Bug reports (what breaks the reflection?)

---

## → License & Acknowledgment

**MIT License** - See [LICENSE](LICENSE)

This work stands on the shoulders of:
- **Ra Uru Hu** (Human Design System)
- **Richard Rudd** (Gene Keys transmission)
- **B.V. Raman** (Vedic astrology systematization)
- **Maharishi Parashara** (Vimshottari Dasha system)
- **Wilhelm/Baynes** (I-Ching translation)
- **Arthur Edward Waite & Pamela Colman Smith** (Rider-Waite Tarot)
- **Oscar Ichazo & Claudio Naranjo** (Enneagram psychology)
- **Swiss Ephemeris Team** (astronomical calculations)
- **All who witness without claiming to predict**

---

```
                    ༺═──────────────────────────────────────────═༻
                                                                  
                    "The three eyes see: past, present, future.
                     But the witness sees all three as now."
                                                                  
                           ∴ Tryambakam ∴                         
                        The Three-Eyed One                        
                                                                  
                    Not prediction. Reflection. Inquiry. Witness.
                                                                  
                    ༺═──────────────────────────────────────────═༻

                              Made with ∞ 
                         for consciousness explorers
                              everywhere

                     v2.1.0 "Vedic Bridge" • Feb 2026
```
