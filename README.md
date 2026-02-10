<p align="center">
  <strong>Tryambakam Noesis</strong><br>
  <em>The Three-Eyed Witness</em>
</p>

<p align="center">
  <a href="https://github.com/Sheshiyer/Selemene-engine/actions"><img src="https://img.shields.io/github/actions/workflow/status/Sheshiyer/Selemene-engine/test.yml?style=flat-square&label=tests" alt="Tests"></a>
  &nbsp;
  <img src="https://img.shields.io/badge/Rust-1.75+-CE422B?style=flat-square&logo=rust&logoColor=white" alt="Rust">
  &nbsp;
  <img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="License">
  &nbsp;
  <a href="https://selemene-engine-production.up.railway.app/health/live"><img src="https://img.shields.io/badge/status-live-brightgreen?style=flat-square" alt="Live"></a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/engines-8-orange?style=flat-square" alt="Engines">
  &nbsp;
  <img src="https://img.shields.io/badge/workflows-6-blueviolet?style=flat-square" alt="Workflows">
  &nbsp;
  <img src="https://img.shields.io/badge/Axum-HTTP-teal?style=flat-square" alt="Axum">
  &nbsp;
  <img src="https://img.shields.io/badge/Supabase-Postgres-3ECF8E?style=flat-square&logo=supabase&logoColor=white" alt="Supabase">
  &nbsp;
  <img src="https://img.shields.io/badge/Railway-deployed-0B0D0E?style=flat-square&logo=railway&logoColor=white" alt="Railway">
</p>

---

A computational mirror for self-inquiry. 8 engines rooted in Vedic, numerological, and consciousness traditions — not to predict your future, but to reflect patterns worth witnessing.

**What makes this different**: Every response includes a *witness prompt* — a question calibrated to your consciousness level. You don't get answers. You get better questions.

---

## Get Started

**[API Quickstart Guide](docs/API_QUICKSTART.md)** — Zero to first API call in 5 minutes.

**[Interactive Explorer](scripts/explore-api.sh)** — Terminal-based menu for exploring every engine.

**[Swagger UI](https://selemene-engine-production.up.railway.app/api/docs)** — Full interactive API documentation.

### First Call in 30 Seconds

```bash
# Set your API key
export NOESIS_API_KEY="nk_your_key_here"

# Ask the mirror a question
curl -s -X POST https://selemene-engine-production.up.railway.app/api/v1/engines/numerology/calculate \
  -H "X-API-Key: $NOESIS_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": {
      "name": "Your Name",
      "date": "1991-08-13",
      "time": "13:31",
      "latitude": 12.9716,
      "longitude": 77.5946,
      "timezone": "Asia/Kolkata"
    }
  }' | python3 -m json.tool
```

```json
{
    "engine_id": "numerology",
    "result": {
        "life_path": { "value": 5, "meaning": "Freedom, change, adventure" },
        "expression": { "value": 1, "meaning": "Leadership, independence, pioneering" },
        "soul_urge": { "value": 5, "meaning": "Freedom, change, adventure" }
    },
    "witness_prompt": "What patterns arise when freedom meets discipline?",
    "consciousness_level": 0,
    "metadata": { "calculation_time_ms": 0, "backend": "native" }
}
```

---

## The 8 Engines

Each engine is a different lens on the same moment. All return sub-millisecond calculations.

| Engine | What It Witnesses | Key Input |
|--------|-------------------|-----------|
| **Panchanga** | Vedic time — tithi, nakshatra, yoga, karana, vara. The breath of cosmic rhythm. | `date`, lat/lng, timezone |
| **Human Design** | Your bodygraph — type, strategy, authority, profile, 9 centers, 26 gates. Genetic imprint as circuit. | `date`, `time`, lat/lng, timezone |
| **Gene Keys** | Shadow, Gift, Siddhi frequencies across 4 activation sequences. Potentials witnessed through contemplation. | `date`, `time`, lat/lng, timezone |
| **Vimshottari** | 120-year planetary timeline. 729 nested dasha periods. Your current moment in the grand cycle. | `date`, `time`, lat/lng, timezone |
| **Numerology** | Life path, expression, soul urge, personality — Pythagorean + Chaldean systems. | `date`, `name` |
| **Biorhythm** | Physical (23d), emotional (28d), intellectual (33d) sine waves. Your energetic weather. | `date` |
| **Vedic Clock** | TCM organ meridian clock + Ayurvedic dosha timing. The intelligence of *when*. | `current_time` (auto) |
| **Biofield** | Chakra energy readings. The subtle body as data. | `date` |

### Request Format

All engines accept the same `EngineInput` shape:

```jsonc
{
    "birth_data": {
        "name": "string",              // Used by numerology
        "date": "YYYY-MM-DD",          // Required
        "time": "HH:MM",              // Required for HD, gene-keys, vimshottari
        "latitude": 12.9716,           // Decimal degrees
        "longitude": 77.5946,          // Decimal degrees
        "timezone": "Asia/Kolkata"     // IANA timezone
    },
    "precision": "standard"            // "standard" | "high" | "extreme"
}
```

### Try Each Engine

```bash
BASE="https://selemene-engine-production.up.railway.app/api/v1"

# Numerology — needs name + date
curl -s -X POST $BASE/engines/numerology/calculate \
  -H "X-API-Key: $NOESIS_API_KEY" -H "Content-Type: application/json" \
  -d '{"birth_data":{"name":"Test","date":"1991-08-13","latitude":12.97,"longitude":77.59,"timezone":"Asia/Kolkata"}}'

# Biorhythm — just a birth date
curl -s -X POST $BASE/engines/biorhythm/calculate \
  -H "X-API-Key: $NOESIS_API_KEY" -H "Content-Type: application/json" \
  -d '{"birth_data":{"date":"1991-08-13","latitude":12.97,"longitude":77.59,"timezone":"Asia/Kolkata"}}'

# Human Design — needs exact birth time
curl -s -X POST $BASE/engines/human-design/calculate \
  -H "X-API-Key: $NOESIS_API_KEY" -H "Content-Type: application/json" \
  -d '{"birth_data":{"date":"1991-08-13","time":"13:31","latitude":12.9716,"longitude":77.5946,"timezone":"Asia/Kolkata"}}'

# Vedic Clock — uses current time, no birth data needed
curl -s -X POST $BASE/engines/vedic-clock/calculate \
  -H "X-API-Key: $NOESIS_API_KEY" -H "Content-Type: application/json" \
  -d '{}'
```

---

## The 6 Workflows

Workflows synthesize multiple engines into emergent understanding. Not pipelines — *synthesis rituals*.

| Workflow | Engines | What Emerges |
|----------|---------|--------------|
| **`birth-blueprint`** | Numerology + Human Design + Vimshottari | Your natal imprint — life path numbers, bodygraph, 120-year timeline |
| **`daily-practice`** | Panchanga + Vedic Clock + Biorhythm | Optimal timing — cosmic tide aligned with personal rhythm |
| **`decision-support`** | Tarot + I-Ching + HD Authority | Multi-perspective guidance — not "what to do" but "what to notice" |
| **`self-inquiry`** | Gene Keys + Enneagram | Shadow work meets personality patterns — where you contract, where you expand |
| **`creative-expression`** | Sigil Forge + Sacred Geometry | Intent made visible — symbols as seeds, geometry as meditation |
| **`full-spectrum`** | All 14 Engines | Complete consciousness portrait — every lens, every frequency |

*Workflows referencing TypeScript engines (Tarot, I-Ching, Enneagram, Sacred Geometry, Sigil Forge) require the TS engines server. See [Bridge CLI](bridges/cli/README.md) for setup.*

```bash
# Execute a workflow
curl -s -X POST $BASE/workflows/birth-blueprint/execute \
  -H "X-API-Key: $NOESIS_API_KEY" -H "Content-Type: application/json" \
  -d '{"birth_data":{"date":"1991-08-13","time":"13:31","latitude":12.9716,"longitude":77.5946,"timezone":"Asia/Kolkata"}}'
```

---

## Consciousness Levels

The system adapts witness prompts based on your relationship with awareness.

| Level | State | Prompt Calibration |
|-------|-------|-------------------|
| **0** | Dormant | Observational. *"What sensations arise when you feel this pattern?"* |
| **1** | Glimpsing | Reflective. *"When does this pattern show up in your life?"* |
| **2** | Practicing | Inquiry-based. *"What might this pattern be protecting?"* |
| **3** | Integrated | Self-authorship. *"How do you choose to work with this pattern?"* |
| **4-5** | Embodied | Open awareness. *"What witnesses this pattern arising?"* |

This isn't gamification. It's meeting you where you are.

---

## Authentication

| Method | Header | Use Case |
|--------|--------|----------|
| API Key | `X-API-Key: nk_...` | Server-to-server, scripts, CLI |
| JWT | `Authorization: Bearer <token>` | User sessions (login flow) |

### Tiers

| Tier | Rate Limit | Access |
|------|-----------|--------|
| `free` | 60 req/min | Basic engines |
| `premium` | 1,000 req/min | All engines + batch |
| `enterprise` | 10,000 req/min | Everything + admin |

### Seed API Keys

```bash
DATABASE_URL="your-postgres-url" \
  cargo run --package noesis-auth --features postgres --example seed_api_keys
```

Creates an admin user (`admin@tryambakam.com`) + 5 API keys across tiers. Keys print once and cannot be recovered.

---

## Architecture

```
crates/
  noesis-api/             Axum HTTP server (the binary you deploy)
  noesis-orchestrator/    Multi-engine parallel execution + workflow synthesis
  noesis-auth/            JWT + API key auth (Postgres-backed)
  noesis-cache/           3-layer cache (L1 memory, L2 Redis, L3 disk)
  noesis-metrics/         Prometheus instrumentation
  noesis-core/            Shared types (EngineInput, EngineOutput, BirthData)
  noesis-vedic-api/       FreeAstrologyAPI client (15+ Vedic endpoints)
  engine-panchanga/       Vedic calendar calculations
  engine-human-design/    HD bodygraph (26 gates, 9 centers, profile)
  engine-gene-keys/       Shadow-Gift-Siddhi activation sequences
  engine-vimshottari/     120-year nested dasha periods
  engine-numerology/      Pythagorean + Chaldean number systems
  engine-biorhythm/       3 biological cycles (P/E/I)
  engine-vedic-clock/     TCM organ clock + Ayurvedic timing
  engine-biofield/        Chakra readings
```

### Performance

| Engine | Calculation Time |
|--------|-----------------|
| Gene Keys | 0.012ms |
| Panchanga | <1ms |
| Numerology | <1ms |
| Biorhythm | <1ms |
| Vimshottari | <1ms |
| Human Design | 1.31ms |
| API p95 | <100ms |
| Workflow (parallel) | <200ms |

---

## Production Stack

| Component | Technology |
|-----------|------------|
| **Compute** | [Railway](https://railway.app) — Docker, <100MB runtime |
| **Database** | [Supabase](https://supabase.com) PostgreSQL (ap-south-1) |
| **Cache** | Railway Redis (L2) + in-memory LRU (L1) |
| **Errors** | [Sentry](https://sentry.io) (10% trace sampling) |
| **Metrics** | Prometheus (`/metrics`) |
| **Docs** | Swagger UI (`/api/docs`) |

### Endpoints

| Path | Auth | Purpose |
|------|------|---------|
| `/health/live` | No | Liveness probe — always 200 |
| `/health/ready` | No | Readiness — checks DB, Redis, orchestrator |
| `/api/docs` | No | Swagger UI |
| `/metrics` | No | Prometheus metrics |
| `/api/v1/engines` | Yes | List engines |
| `/api/v1/engines/:id/calculate` | Yes | Run calculation |
| `/api/v1/engines/:id/info` | Yes | Engine metadata |
| `/api/v1/workflows` | Yes | List workflows |
| `/api/v1/workflows/:id/execute` | Yes | Execute workflow |

---

## Local Development

```bash
# Build and run
cargo build --release
cargo run --bin noesis-server

# Run tests
cargo test -- --test-threads=1

# Docker
docker-compose up -d
```

### Environment

Copy `.env.example` to `.env` and fill in values. Required:
- `RUST_ENV` — `development` or `production`
- `JWT_SECRET` — signing key for JWT tokens
- `DATABASE_URL` — Postgres connection string (optional — runs degraded without DB)

See [`.env.example`](.env.example) for the full list.

---

## Documentation

| Guide | Purpose |
|-------|---------|
| **[API Quickstart](docs/API_QUICKSTART.md)** | Zero to first call |
| **[Swagger UI](https://selemene-engine-production.up.railway.app/api/docs)** | Interactive API explorer |
| **[Terminal Explorer](scripts/explore-api.sh)** | CLI-based API exploration |
| **[Agent Bridge CLI](bridges/cli/README.md)** | `npx @selemene/bridge init` — generate tool defs for Claude, OpenAI, LangChain |
| **[Architecture](.context/documentation/architecture/selemene_architecture.md)** | System design |
| **[Deployment](docs/deployment/README.md)** | Production deployment guide |

---

## Acknowledgments

This work stands on the shoulders of:

- **Maharishi Parashara** — Vimshottari Dasha system
- **Ra Uru Hu** — Human Design System
- **Richard Rudd** — Gene Keys transmission
- **B.V. Raman** — Vedic astrology systematization
- **Swiss Ephemeris Team** — astronomical calculations

---

<p align="center">
  <strong>MIT License</strong><br>
  <em>Not prediction. Reflection. Inquiry. Witness.</em>
</p>
