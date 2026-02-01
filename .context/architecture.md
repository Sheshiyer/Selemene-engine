# Selemene Engine - Architecture Documentation

## System Overview

Selemene Engine is a high-performance consciousness calculation platform combining 14 engines across Rust and TypeScript runtimes, orchestrated through a unified API layer.

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              Client Layer                                        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐                  │
│  │   Web Client    │  │   Mobile App    │  │   CLI Tools     │                  │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘                  │
│           │                    │                    │                            │
│           └────────────────────┼────────────────────┘                            │
│                                ▼                                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│                         Load Balancer / Gateway                                  │
│                         (nginx / cloud LB)                                       │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  ┌────────────────────────────────────────────────────────────────────────────┐ │
│  │                        Noesis API (Axum - Port 8080)                       │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │ │
│  │  │    Routes    │  │  Middleware  │  │   Handlers   │  │   Health     │   │ │
│  │  │  /api/v1/*   │  │  Auth/Rate   │  │  REST/WS     │  │   Probes     │   │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘   │ │
│  └─────────────────────────────────┬──────────────────────────────────────────┘ │
│                                    │                                             │
│  ┌─────────────────────────────────▼──────────────────────────────────────────┐ │
│  │                    Workflow Orchestrator (noesis-orchestrator)              │ │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │ │
│  │  │                        Engine Registry                                │  │ │
│  │  │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────────────┐ │  │ │
│  │  │  │ Rust       │ │ Rust       │ │ Rust       │ │ TypeScript Bridge  │ │  │ │
│  │  │  │ Engines    │ │ Engines    │ │ Engines    │ │ (noesis-bridge)    │ │  │ │
│  │  │  │ (9 total)  │ │            │ │            │ │ → 5 TS Engines     │ │  │ │
│  │  │  └────────────┘ └────────────┘ └────────────┘ └────────────────────┘ │  │ │
│  │  └──────────────────────────────────────────────────────────────────────┘  │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │ │
│  │  │  Workflow    │  │  Synthesis   │  │  Caching     │  │  Witness     │   │ │
│  │  │  Executor    │  │  Layer       │  │  Layer       │  │  Prompts     │   │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘   │ │
│  └────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                  │
│  ┌───────────────────────────────┐  ┌───────────────────────────────────────┐   │
│  │     Rust Engine Crates        │  │      TypeScript Engines (Port 3001)   │   │
│  │  ┌─────────┐ ┌─────────┐     │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐  │   │
│  │  │ human-  │ │ gene-   │     │  │  │  tarot  │ │ i-ching │ │enneagram│  │   │
│  │  │ design  │ │ keys    │     │  │  └─────────┘ └─────────┘ └─────────┘  │   │
│  │  └─────────┘ └─────────┘     │  │  ┌─────────────────┐ ┌─────────────┐  │   │
│  │  ┌─────────┐ ┌─────────┐     │  │  │ sacred-geometry │ │ sigil-forge │  │   │
│  │  │vimshot- │ │panchanga│     │  │  └─────────────────┘ └─────────────┘  │   │
│  │  │ tari    │ │         │     │  └───────────────────────────────────────┘   │
│  │  └─────────┘ └─────────┘     │                                              │
│  │  ┌─────────┐ ┌─────────┐     │                                              │
│  │  │numerol- │ │biorhythm│     │                                              │
│  │  │ogy      │ │         │     │                                              │
│  │  └─────────┘ └─────────┘     │                                              │
│  │  ┌─────────┐ ┌─────────┐     │                                              │
│  │  │ vedic-  │ │biofield │     │                                              │
│  │  │ clock   │ │         │     │                                              │
│  │  └─────────┘ └─────────┘     │                                              │
│  │  ┌─────────────────────┐     │                                              │
│  │  │    face-reading     │     │                                              │
│  │  └─────────────────────┘     │                                              │
│  └───────────────────────────────┘                                              │
│                                                                                  │
├─────────────────────────────────────────────────────────────────────────────────┤
│                              Data Layer                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │    Redis     │  │  PostgreSQL  │  │  Ephemeris   │  │ Wisdom Data  │         │
│  │  (L2 Cache)  │  │  (Metadata)  │  │  Files       │  │    Files     │         │
│  │  Port 6379   │  │  Port 5432   │  │  /data/eph   │  │  /data/wis   │         │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘         │
├─────────────────────────────────────────────────────────────────────────────────┤
│                           Observability Stack                                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │  Prometheus  │  │   Grafana    │  │    Jaeger    │  │     Loki     │         │
│  │  (Metrics)   │  │ (Dashboards) │  │  (Tracing)   │  │   (Logs)     │         │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘         │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## Component Descriptions

### API Layer (noesis-api)

The HTTP interface built with Axum framework:

| Component | Description |
|-----------|-------------|
| **Routes** | RESTful endpoints under `/api/v1/` for engines, workflows, ghati, panchanga |
| **Middleware** | JWT/API key authentication, rate limiting, CORS, request tracing |
| **Handlers** | Request validation, parameter extraction, response formatting |
| **Health** | `/health`, `/ready`, `/metrics` endpoints for orchestration |

### Workflow Orchestrator (noesis-orchestrator)

Coordinates multi-engine execution:

| Component | Description |
|-----------|-------------|
| **Engine Registry** | Thread-safe storage of `Arc<dyn ConsciousnessEngine>` trait objects |
| **Workflow Executor** | Parallel execution via `futures::join_all` |
| **Synthesis Layer** | Cross-engine theme detection and insight generation |
| **Caching** | TTL-based workflow result caching |

### TypeScript Bridge (noesis-bridge)

HTTP bridge to TypeScript engines:

| Component | Description |
|-----------|-------------|
| **BridgeManager** | Connection pool and health checks to TS server |
| **BridgeEngine** | Implements `ConsciousnessEngine` trait via HTTP |
| **Serialization** | JSON request/response conversion |

### Engine Categories

| Category | Engines | Runtime | Nature |
|----------|---------|---------|--------|
| **Natal** | human-design, gene-keys, numerology, enneagram | Rust/TS | Fixed patterns from birth |
| **Temporal** | panchanga, vedic-clock, biorhythm, vimshottari | Rust | Time-based cycles |
| **Archetypal** | tarot, i-ching | TypeScript | Symbolic/oracular guidance |
| **Somatic** | biofield, face-reading | Rust | Body-based patterns |
| **Creative** | sacred-geometry, sigil-forge | TypeScript | Generative/visual output |

### Cache Architecture

Three-tier caching system:

```
┌─────────────────────────────────────────────────────────────────┐
│                        Request                                   │
│                           │                                      │
│                           ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ L1 Cache (In-Memory LRU via DashMap)                        ││
│  │ • Size: ~256MB configurable                                 ││
│  │ • Latency: <1ms                                             ││
│  │ • TTL: 1 hour default                                       ││
│  └─────────────────────────┬───────────────────────────────────┘│
│                     MISS   │                                     │
│                           ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ L2 Cache (Redis)                                            ││
│  │ • Size: ~1GB configurable                                   ││
│  │ • Latency: <10ms                                            ││
│  │ • TTL: 24 hours default                                     ││
│  │ • Distributed across pods                                   ││
│  └─────────────────────────┬───────────────────────────────────┘│
│                     MISS   │                                     │
│                           ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ L3 Cache (Disk - Precomputed)                               ││
│  │ • Common date calculations                                  ││
│  │ • Fixed ephemeris lookups                                   ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

## Data Flow Patterns

### Single Engine Calculation

```
Client Request
      │
      ▼
┌─────────────────┐
│  API Handler    │ ── validate request
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Auth Middleware│ ── check JWT/API key
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Rate Limiter    │ ── enforce tier limits
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Orchestrator    │ ── route to engine
└────────┬────────┘
         │
         ▼
┌─────────────────┐     ┌─────────────┐
│ Engine Registry │────►│ Cache Check │
└────────┬────────┘     └──────┬──────┘
         │                     │
         │ cache miss          │ cache hit
         ▼                     │
┌─────────────────┐            │
│ Engine.calculate│            │
└────────┬────────┘            │
         │                     │
         ▼                     │
┌─────────────────┐            │
│ Cache Store     │◄───────────┘
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Response Format │
└────────┬────────┘
         │
         ▼
    Client Response
```

### Workflow Execution (Parallel)

```
Workflow Request
      │
      ▼
┌─────────────────────────────────────────────────────────────┐
│                 WorkflowOrchestrator                         │
│                                                              │
│    ┌───────────────────────────────────────────────────┐    │
│    │            futures::join_all                       │    │
│    │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ │    │
│    │  │Engine 1 │ │Engine 2 │ │Engine 3 │ │Engine N │ │    │
│    │  │(async)  │ │(async)  │ │(async)  │ │(async)  │ │    │
│    │  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ │    │
│    │       │           │           │           │       │    │
│    │       └───────────┴───────────┴───────────┘       │    │
│    │                         │                          │    │
│    └─────────────────────────┼──────────────────────────┘    │
│                              ▼                               │
│    ┌────────────────────────────────────────────────────┐   │
│    │                   Synthesizer                       │   │
│    │  • Theme detection (3+ engine agreement)            │   │
│    │  • Alignment/tension analysis                       │   │
│    │  • Witness prompt generation                        │   │
│    └────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
         │
         ▼
   WorkflowResult
```

### TypeScript Engine Bridge Flow

```
Rust Orchestrator
      │
      ▼
┌─────────────────┐
│ BridgeEngine    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ HTTP POST       │───────────────────────────┐
│ localhost:3001  │                           │
└─────────────────┘                           │
                                              ▼
                              ┌───────────────────────────────┐
                              │   TypeScript Server (Bun)     │
                              │  ┌─────────────────────────┐  │
                              │  │    Express Router       │  │
                              │  └───────────┬─────────────┘  │
                              │              │                 │
                              │              ▼                 │
                              │  ┌─────────────────────────┐  │
                              │  │   Engine Implementation  │  │
                              │  │   (tarot/i-ching/etc)   │  │
                              │  └───────────┬─────────────┘  │
                              │              │                 │
                              │              ▼                 │
                              │  ┌─────────────────────────┐  │
                              │  │    JSON Response        │  │
                              │  └─────────────────────────┘  │
                              └───────────────────────────────┘
                                              │
         ┌────────────────────────────────────┘
         │
         ▼
┌─────────────────┐
│ Deserialize to  │
│ EngineOutput    │
└────────┬────────┘
         │
         ▼
   Rust Orchestrator
```

## Technology Stack

### Rust Components

| Component | Technology | Purpose |
|-----------|------------|---------|
| HTTP Server | Axum 0.7 | REST API, WebSocket |
| Async Runtime | Tokio | Concurrent execution |
| Serialization | Serde | JSON handling |
| Cache | DashMap + Redis | L1/L2 caching |
| Tracing | tracing + OpenTelemetry | Distributed tracing |
| Metrics | Prometheus | Metrics collection |
| Ephemeris | Swiss Ephemeris (swerust) | Astronomical calculations |

### TypeScript Components

| Component | Technology | Purpose |
|-----------|------------|---------|
| Runtime | Bun | Fast JS execution |
| HTTP Server | Express | REST API |
| Type Safety | TypeScript | Static typing |
| Build | biome | Linting/formatting |

### Infrastructure

| Component | Technology | Purpose |
|-----------|------------|---------|
| Containerization | Docker | Consistent deployment |
| Orchestration | Kubernetes | Production scaling |
| Cache | Redis 7 | Distributed caching |
| Database | PostgreSQL 16 | Metadata storage |
| Reverse Proxy | nginx / Traefik | Load balancing |

### Observability

| Component | Technology | Purpose |
|-----------|------------|---------|
| Metrics | Prometheus | Time-series metrics |
| Dashboards | Grafana | Visualization |
| Tracing | Jaeger | Distributed tracing |
| Logs | Loki | Log aggregation |

## Security Architecture

### Authentication Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    Request Header                            │
│    Authorization: Bearer <JWT>  OR  X-API-Key: <key>        │
└─────────────────────────────┬───────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Auth Middleware                           │
│  1. Extract token/key from headers                          │
│  2. Validate JWT signature OR lookup API key                │
│  3. Extract claims (user_id, tier, phase)                   │
│  4. Attach to request context                               │
└─────────────────────────────┬───────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Phase Gating                              │
│  • Each engine has required_phase (0-5)                     │
│  • User's consciousness_level from JWT claims               │
│  • Access denied if user_phase < required_phase             │
└─────────────────────────────────────────────────────────────┘
```

### Rate Limiting Tiers

| Tier | Requests/min | Burst | Engines |
|------|--------------|-------|---------|
| Free | 60 | 10 | Phase 0-1 |
| Premium | 1000 | 100 | Phase 0-3 |
| Enterprise | 10000 | 500 | Phase 0-5 |

## Directory Structure

```
selemene-engine/
├── src/                       # Main API binary
│   ├── api/                   # HTTP handlers, routes, middleware
│   ├── cache/                 # L1/L2/L3 cache implementations
│   ├── config/                # Environment configuration
│   ├── engines/               # Legacy calculation engines
│   ├── metrics/               # Prometheus metrics
│   ├── models/                # Shared data types
│   ├── time/                  # Ghati time calculations
│   └── main.rs               
├── crates/                    # Rust workspace crates
│   ├── engine-human-design/   # Human Design calculations
│   ├── engine-gene-keys/      # Gene Keys system
│   ├── engine-vimshottari/    # Vedic dasha system
│   ├── engine-panchanga/      # Vedic almanac
│   ├── engine-numerology/     # Numerology calculations
│   ├── engine-biorhythm/      # Biorhythm cycles
│   ├── engine-vedic-clock/    # Vedic time/dosha
│   ├── engine-biofield/       # Energy field analysis (stub)
│   ├── engine-face-reading/   # Face reading analysis (stub)
│   ├── noesis-api/            # API crate (reusable)
│   ├── noesis-auth/           # Authentication logic
│   ├── noesis-bridge/         # TypeScript engine bridge
│   ├── noesis-cache/          # Caching primitives
│   ├── noesis-core/           # Core traits and types
│   ├── noesis-metrics/        # Metrics primitives
│   ├── noesis-orchestrator/   # Workflow orchestration
│   └── noesis-witness/        # Witness prompt generation
├── ts-engines/                # TypeScript engines
│   ├── src/
│   │   ├── engines/
│   │   │   ├── tarot/
│   │   │   ├── i-ching/
│   │   │   ├── enneagram/
│   │   │   ├── sacred-geometry/
│   │   │   └── sigil-forge/
│   │   └── server/
│   └── package.json
├── data/                      # Runtime data
│   ├── ephemeris/             # Swiss Ephemeris files
│   └── wisdom-docs/           # Wisdom text data
├── k8s/                       # Kubernetes manifests
├── monitoring/                # Prometheus/Grafana configs
├── docs/                      # Documentation
└── tests/                     # Integration tests
```

## Performance Characteristics

| Metric | Target | Achieved |
|--------|--------|----------|
| Single engine calculation | <100ms | ~1-50ms |
| Full workflow (14 engines) | <2s | ~50ms (parallel) |
| Cache hit (L1) | <1ms | <0.5ms |
| Cache hit (L2/Redis) | <10ms | <5ms |
| TypeScript bridge overhead | <20ms | ~10-15ms |

## Deployment Patterns

### Development

```bash
# Rust API
cargo run

# TypeScript engines (separate terminal)
cd ts-engines && bun run dev
```

### Docker Compose

```bash
docker-compose up -d
```

### Kubernetes

```bash
kubectl apply -k k8s/
```

---

**Last Updated**: 2026-01
**Architecture Version**: 2.0.0 (Wave 2)
