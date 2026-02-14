<p align="center">
  <video src="docs/assets/images/engines/logo-wide.mp4" autoplay loop muted playsinline width="560"></video>
</p>

<h1 align="center">Selemene Engine</h1>

<p align="center">
  <em>High-Performance Consciousness Calculation Engine</em><br>
  <sub>Part of the Tryambakam Noesis Project</sub>
</p>

<p align="center">
  <a href="https://selemene.tryambakam.space/health/live"><img src="https://img.shields.io/badge/status-live-1A1A2E?style=for-the-badge&labelColor=B8860B" alt="Live"></a>
  &nbsp;
  <img src="https://img.shields.io/badge/engines-16-1A1A2E?style=for-the-badge&labelColor=6B6B6B" alt="16 Engines">
  &nbsp;
  <img src="https://img.shields.io/badge/workflows-6-1A1A2E?style=for-the-badge&labelColor=6B6B6B" alt="6 Workflows">
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.75+-CE422B?style=flat-square&logo=rust&logoColor=white" alt="Rust">
  &nbsp;
  <img src="https://img.shields.io/badge/Axum-HTTP-1A1A2E?style=flat-square" alt="Axum">
  &nbsp;
  <img src="https://img.shields.io/badge/Supabase-Postgres-3ECF8E?style=flat-square&logo=supabase&logoColor=white" alt="Supabase">
  &nbsp;
  <img src="https://img.shields.io/badge/Railway-deployed-0B0D0E?style=flat-square&logo=railway&logoColor=white" alt="Railway">
  &nbsp;
  <img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="License">
</p>

<br>

<p align="center">
  <img src="docs/assets/images/2C-glass-logo-nanobananapro-v2.png" alt="Noesis Glass Logo" width="560">
</p>

<br>

> **High-performance astronomical and consciousness calculation engine.**
>
> 16 engines spanning Vedic astrology, planetary transits, numerology, Human Design, Gene Keys, and esoteric traditions. Built in Rust with sub-millisecond calculations.
>
> Selemene Engine powers the computational backend of **Tryambakam Noesis** — providing real-time calculations, multi-engine orchestration, and consciousness-calibrated witness prompts.

<br>

## ✦ Quick Start

<table>
<tr>
<td width="33%" align="center">
<a href="docs/API_QUICKSTART.md"><strong>📖 API Quickstart</strong></a><br>
<sub>Zero to first call in 5 minutes</sub>
</td>
<td width="33%" align="center">
<a href="scripts/explore-api.sh"><strong>🔮 Terminal Explorer</strong></a><br>
<sub>Interactive CLI for every engine</sub>
</td>
<td width="33%" align="center">
<a href="https://selemene.tryambakam.space/api/docs"><strong>📜 Swagger UI</strong></a><br>
<sub>Full API documentation</sub>
</td>
</tr>
</table>

<br>

<details>
<summary><strong>First Call in 30 Seconds</strong></summary>

```bash
# Set your API key
export NOESIS_API_KEY="nk_your_key_here"

# Ask the mirror a question
curl -s -X POST https://selemene.tryambakam.space/api/v1/engines/numerology/calculate \
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

</details>

<br>

## ✦ The 16 Engines

16 calculation engines spanning Vedic, Western, and esoteric systems. **Sub-millisecond performance** for all calculations.

<br>

### Rust Engines (11)

<table>
<tr>
<td align="center" width="20%">
<strong>Panchanga</strong><br>
<sub>Tithi · Nakshatra · Yoga · Karana</sub><br>
<code>date, lat/lng</code>
</td>
<td align="center" width="20%">
<strong>Human Design</strong><br>
<sub>Type · Centers · Gates · Profile</sub><br>
<code>date, time, lat/lng</code>
</td>
<td align="center" width="20%">
<strong>Gene Keys</strong><br>
<sub>Shadow · Gift · Siddhi</sub><br>
<code>date, time, lat/lng</code>
</td>
<td align="center" width="20%">
<strong>Vimshottari</strong><br>
<sub>120-Year Dasha Periods</sub><br>
<code>date, time, lat/lng</code>
</td>
<td align="center" width="20%">
<strong>Numerology</strong><br>
<sub>Life Path · Expression</sub><br>
<code>date, name</code>
</td>
</tr>
<tr><td colspan="5"><br></td></tr>
<tr>
<td align="center" width="20%">
<strong>Biorhythm</strong><br>
<sub>Physical · Emotional · Intellectual</sub><br>
<code>date</code>
</td>
<td align="center" width="20%">
<strong>Vedic Clock</strong><br>
<sub>TCM Meridians · Doshas</sub><br>
<code>current_time</code>
</td>
<td align="center" width="20%">
<strong>Biofield</strong><br>
<sub>Vedic Chakra · Birth-Data Analysis</sub><br>
<code>date, time, lat/lng</code>
</td>
<td align="center" width="20%">
<strong>Face Reading</strong><br>
<sub>Physiognomy Analysis</sub><br>
<code>image_data</code>
</td>
<td align="center" width="20%">
<strong>Nadabrahman</strong><br>
<sub>Sound Consciousness</sub><br>
<code>audio_data</code>
</td>
</tr>
<tr><td colspan="5"><br></td></tr>
<tr>
<td align="center" width="20%">
<strong>Transits</strong><br>
<sub>Planetary Transits · Sade Sati</sub><br>
<code>date, time, lat/lng</code>
</td>
<td align="center" width="20%"></td>
<td align="center" width="20%"></td>
<td align="center" width="20%"></td>
<td align="center" width="20%"></td>
</tr>
</table>

<br>

### TypeScript Engines (5)

> **Note:** These engines run in a separate `ts-engines` service. The API attempts to connect to them at startup (with 30s retry logic). If the service is unreachable, these endpoints will be unavailable.

<table>
<tr>
<td align="center" width="20%">
<strong>Tarot</strong><br>
<sub>78-Card System</sub><br>
<code>date, question</code>
</td>
<td align="center" width="20%">
<strong>I Ching</strong><br>
<sub>64 Hexagrams</sub><br>
<code>question, method</code>
</td>
<td align="center" width="20%">
<strong>Enneagram</strong><br>
<sub>9 Types · Wings · Tritypes</sub><br>
<code>date, name</code>
</td>
<td align="center" width="20%">
<strong>Sacred Geometry</strong><br>
<sub>Platonic Solids · Patterns</sub><br>
<code>parameters</code>
</td>
<td align="center" width="20%">
<strong>Sigil Forge</strong><br>
<sub>Intent Manifestation</sub><br>
<code>intent_text</code>
</td>
</tr>
</table>

<br>

<p align="center">
  <a href="docs/ENGINES.md"><strong>→ View Detailed Engine Documentation</strong></a>
</p>

<br>

<details>
<summary><strong>Request Format & Examples</strong></summary>

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
BASE="https://selemene.tryambakam.space/api/v1"

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

</details>

<br>

---

## ✦ The 6 Workflows

Workflows synthesize multiple engines into emergent understanding. Not pipelines — *synthesis rituals*.

<br>

<table>
<tr>
<td width="50%">

**`birth-blueprint`** — *Numerology + Human Design + Vimshottari*
> Your natal imprint. Life path numbers, bodygraph, 120-year timeline.

**`daily-practice`** — *Panchanga + Vedic Clock + Biorhythm*
> Optimal timing. Cosmic tide aligned with personal rhythm.

**`decision-support`** — *Tarot + I-Ching + HD Authority*
> Multi-perspective guidance. Not "what to do" but "what to notice."

</td>
<td width="50%">

**`self-inquiry`** — *Gene Keys + Enneagram*
> Shadow work meets personality. Where you contract, where you expand.

**`creative-expression`** — *Sigil Forge + Sacred Geometry*
> Intent made visible. Symbols as seeds, geometry as meditation.

**`full-spectrum`** — *All 16 Engines*
> Complete consciousness portrait. Every lens, every frequency.

</td>
</tr>
</table>

<sub>*Workflows referencing TypeScript engines (Tarot, I-Ching, Enneagram, Sacred Geometry, Sigil Forge) require the TS engines server. See [Bridge CLI](bridges/cli/README.md) for setup.*</sub>

<details>
<summary><strong>Execute a Workflow</strong></summary>

```bash
curl -s -X POST $BASE/workflows/birth-blueprint/execute \
  -H "X-API-Key: $NOESIS_API_KEY" -H "Content-Type: application/json" \
  -d '{"birth_data":{"date":"1991-08-13","time":"13:31","latitude":12.9716,"longitude":77.5946,"timezone":"Asia/Kolkata"}}'
```

</details>

<br>

---

## ✦ Consciousness Levels

The system adapts witness prompts based on your relationship with awareness.

<br>

<p align="center">
  <img src="docs/assets/images/5A-heritage-engraving-recraft-v2.png" alt="Heritage Seal" width="280">
</p>

<br>

| Level | State | Prompt Calibration |
|:-----:|-------|-------------------|
| **0** | Dormant | *"What sensations arise when you feel this pattern?"* |
| **1** | Glimpsing | *"When does this pattern show up in your life?"* |
| **2** | Practicing | *"What might this pattern be protecting?"* |
| **3** | Integrated | *"How do you choose to work with this pattern?"* |
| **4-5** | Embodied | *"What witnesses this pattern arising?"* |

<sub>This isn't gamification. It's meeting you where you are.</sub>

<br>

---

## ✦ Authentication

| Method | Header | Use Case |
|--------|--------|----------|
| API Key | `X-API-Key: nk_...` | Server-to-server, scripts, CLI |
| JWT | `Authorization: Bearer <token>` | User sessions |

<details>
<summary><strong>Rate Limits & Tiers</strong></summary>

| Tier | Rate Limit | Access |
|------|-----------|--------|
| `free` | 60 req/min | Basic engines |
| `premium` | 1,000 req/min | All engines + batch |
| `enterprise` | 10,000 req/min | Everything + admin |

**Seed API Keys:**
```bash
DATABASE_URL="your-postgres-url" \
  cargo run --package noesis-auth --features postgres --example seed_api_keys
```

Creates admin user (`admin@tryambakam.com`) + 5 API keys. Keys print once.

</details>

<br>

---

## ✦ Architecture

<p align="center">
  <img src="docs/assets/images/noesis-architecture.png" alt="Architecture" width="720">
</p>

<br>

<p align="center">
  <img src="docs/assets/images/noesis-identifiers.png" alt="Identity System" width="680">
</p>

<details>
<summary><strong>Crate Structure</strong></summary>

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
  engine-biofield/        Vedic birth-data-driven chakra & biofield analysis
  engine-face-reading/    Physiognomy analysis (stub)
  engine-nadabrahman/     Sound consciousness (stub)
  engine-transits/        Planetary transits, aspects & Sade Sati
```

</details>

<br>

### Performance

| Engine | Time | | Engine | Time |
|--------|------|-|--------|------|
| Gene Keys | 0.012ms | | Vimshottari | <1ms |
| Panchanga | <1ms | | Human Design | 1.31ms |
| Numerology | <1ms | | **API p95** | <100ms |
| Biorhythm | <1ms | | **Workflow** | <200ms |

<br>

---

## ✦ Production Stack

<table>
<tr>
<td width="50%">

| Component | Technology |
|-----------|------------|
| **Compute** | [Railway](https://railway.app) |
| **Database** | [Supabase](https://supabase.com) PostgreSQL |
| **Cache** | Redis (L2) + LRU (L1) |

</td>
<td width="50%">

| Component | Technology |
|-----------|------------|
| **Errors** | [Sentry](https://sentry.io) |
| **Metrics** | Prometheus (`/metrics`) |
| **Docs** | Swagger UI (`/api/docs`) |

</td>
</tr>
</table>

<details>
<summary><strong>All Endpoints</strong></summary>

| Path | Auth | Purpose |
|------|:----:|---------|
| `/health/live` | ✗ | Liveness probe |
| `/health/ready` | ✗ | Readiness check |
| `/api/docs` | ✗ | Swagger UI |
| `/metrics` | ✗ | Prometheus metrics |
| `/api/v1/engines` | ✓ | List engines |
| `/api/v1/engines/:id/calculate` | ✓ | Run calculation |
| `/api/v1/engines/:id/info` | ✓ | Engine metadata |
| `/api/v1/workflows` | ✓ | List workflows |
| `/api/v1/workflows/:id/execute` | ✓ | Execute workflow |

</details>

<br>

---

## ✦ Physical Embodiments

Beyond code, Noesis manifests in ritual objects and somatic practices.

<br>

<p align="center">
  <img src="docs/assets/images/4A-ritual-blend-catalog-layout-nanobananapro-v2.png" alt="Ritual Catalog" width="720">
</p>

<br>

<table>
<tr>
<td width="50%" align="center">
  <img src="docs/assets/images/3A-ritual-kit-nanobananapro-v2.png" alt="Ritual Kit" width="100%">
  <br><sub><strong>Ritual Kit</strong> — Essential tools for embodied practice</sub>
</td>
<td width="50%" align="center">
  <img src="docs/assets/images/3B-somatic-book-nanobananapro-v2.png" alt="Somatic Book" width="100%">
  <br><sub><strong>Somatic Grimoire</strong> — Knowledge meets sensation</sub>
</td>
</tr>
<tr><td colspan="2"><br></td></tr>
<tr>
<td width="50%" align="center">
  <img src="docs/assets/images/3C-essential-oil-bottle-nanobananapro-v2.png" alt="Essential Oils" width="100%">
  <br><sub><strong>Clarity Elixir</strong> — Aromatic anchors for inquiry</sub>
</td>
<td width="50%" align="center">
  <img src="docs/assets/images/4B-ritual-object-flat-lay-nanobananapro-v2.png" alt="Flat Lay" width="100%">
  <br><sub><strong>Sacred Objects</strong> — Daily practice artifacts</sub>
</td>
</tr>
</table>

<br>

---

## ✦ Local Development

```bash
cargo build --release && cargo run --bin noesis-server   # Build & run
cargo test -- --test-threads=1                           # Run tests
docker-compose up -d                                     # Docker
```

<details>
<summary><strong>Environment Setup</strong></summary>

Copy `.env.example` to `.env`. Required variables:
- `RUST_ENV` — `development` or `production`
- `JWT_SECRET` — signing key for JWT tokens
- `DATABASE_URL` — Postgres connection (optional — runs degraded without)

See [`.env.example`](.env.example) for full list.

</details>

<br>

---

## ✦ Documentation

| | |
|---|---|
| **[API Quickstart](docs/API_QUICKSTART.md)** | Zero to first call |
| **[Swagger UI](https://selemene.tryambakam.space/api/docs)** | Interactive explorer |
| **[Terminal Explorer](scripts/explore-api.sh)** | CLI exploration |
| **[Agent Bridge](bridges/cli/README.md)** | Claude, OpenAI, LangChain tool defs |
| **[Architecture](.context/documentation/architecture/selemene_architecture.md)** | System design |
| **[Deployment](docs/deployment/README.md)** | Production guide |

<br>

---

## ✦ Brand Identity

<p align="center">
  <img src="docs/assets/images/2A-brand-kit-bento-nanobananapro-v1.png" alt="Brand Kit" width="720">
</p>

<br>

<p align="center">
  <img src="docs/assets/images/5B-campaign-visual-identity-grid-nanobananapro-v2.png" alt="Visual Identity System" width="720">
</p>

<br>

---

## ✦ Acknowledgments

<sub>This work stands on the shoulders of **Maharishi Parashara** (Vimshottari Dasha), **Ra Uru Hu** (Human Design), **Richard Rudd** (Gene Keys), **B.V. Raman** (Vedic astrology), and the **Swiss Ephemeris Team**.</sub>

<br>

---

<br>

<p align="center">
  <img src="docs/assets/images/2B-wax-seal-nanobananapro-v2.png" alt="Seal" width="120">
</p>

<p align="center">
  <strong>MIT License</strong><br>
  <sub>Not prediction. Reflection. Inquiry. Witness.</sub>
</p>

<p align="center">
  <sub>
    <a href="docs/API_QUICKSTART.md">Quickstart</a> · 
    <a href="https://selemene.tryambakam.space/api/docs">API Docs</a> · 
    <a href="bridges/cli/README.md">Agent Bridge</a>
  </sub>
</p>
