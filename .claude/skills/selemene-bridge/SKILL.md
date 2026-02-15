---
name: selemene-bridge
description: "Selemene Engine Bridge: Query 16 consciousness engines and 6 workflows for witnessing cosmic patterns. Non-prescriptive mirror — generates inquiry, not advice."
tools:
  - name: selemene_health
    description: "Check Selemene engine health and loaded engines/workflows"
    parameters:
      type: object
      properties: {}
  - name: selemene_list_engines
    description: "List all available consciousness engines with metadata"
    parameters:
      type: object
      properties: {}
  - name: selemene_engine_info
    description: "Get detailed metadata for a specific engine"
    parameters:
      type: object
      properties:
        engine_id:
          type: string
          description: "Engine ID: panchanga, numerology, biorhythm, human-design, gene-keys, vimshottari, biofield, vedic-clock, face-reading, nadabrahman, transits, tarot, i-ching, enneagram, sacred-geometry, sigil-forge"
      required: [engine_id]
  - name: selemene_calculate
    description: "Run a calculation on a specific consciousness engine"
    parameters:
      type: object
      properties:
        engine_id:
          type: string
          description: "Engine ID to calculate with"
        birth_data:
          type: object
          description: "Birth data object with name, date, time, latitude, longitude, timezone"
      required: [engine_id, birth_data]
  - name: selemene_workflow_execute
    description: "Execute a multi-engine workflow (birth-blueprint, daily-practice, decision-support, self-inquiry, creative-expression, full-spectrum)"
    parameters:
      type: object
      properties:
        workflow_id:
          type: string
          description: "Workflow ID to execute"
        birth_data:
          type: object
          description: "Birth data object with name, date, time, latitude, longitude, timezone"
      required: [workflow_id, birth_data]
---

# Selemene Engine Bridge

> Non-prescriptive consciousness mirror — generates inquiry prompts, not advice.

## Production API

- **Base URL**: `https://selemene.tryambakam.space`
- **Auth Header**: `X-API-Key: nk_WSgs4eA9bhVa1GNrcOIUw6Vj9VqoxBxs`
- **All 16 engines and 6 workflows are live.**

## How to Call the API

Every engine and workflow takes the same `birth_data` object:

```json
{
  "birth_data": {
    "name": "Person Name",
    "date": "YYYY-MM-DD",
    "time": "HH:MM",
    "latitude": 12.9716,
    "longitude": 77.5946,
    "timezone": "Asia/Kolkata"
  }
}
```

### Single Engine Calculation

```bash
curl -s -X POST https://selemene.tryambakam.space/api/v1/engines/{engine_id}/calculate \
  -H "X-API-Key: nk_WSgs4eA9bhVa1GNrcOIUw6Vj9VqoxBxs" \
  -H "Content-Type: application/json" \
  -d '{"birth_data":{"name":"Test","date":"1991-08-13","time":"13:31","latitude":12.9716,"longitude":77.5946,"timezone":"Asia/Kolkata"}}'
```

Replace `{engine_id}` with any of the 16 engine IDs listed below.

### Workflow Execution (multi-engine)

```bash
curl -s -X POST https://selemene.tryambakam.space/api/v1/workflows/{workflow_id}/execute \
  -H "X-API-Key: nk_WSgs4eA9bhVa1GNrcOIUw6Vj9VqoxBxs" \
  -H "Content-Type: application/json" \
  -d '{"birth_data":{"name":"Test","date":"1991-08-13","time":"13:31","latitude":12.9716,"longitude":77.5946,"timezone":"Asia/Kolkata"}}'
```

Replace `{workflow_id}` with any of the 6 workflow IDs listed below.

### Read-Only Endpoints

```bash
# Health check
curl -s https://selemene.tryambakam.space/health/live

# List all engines
curl -s https://selemene.tryambakam.space/api/v1/engines \
  -H "X-API-Key: nk_WSgs4eA9bhVa1GNrcOIUw6Vj9VqoxBxs"

# Get engine info
curl -s https://selemene.tryambakam.space/api/v1/engines/{engine_id}/info \
  -H "X-API-Key: nk_WSgs4eA9bhVa1GNrcOIUw6Vj9VqoxBxs"

# List all workflows
curl -s https://selemene.tryambakam.space/api/v1/workflows \
  -H "X-API-Key: nk_WSgs4eA9bhVa1GNrcOIUw6Vj9VqoxBxs"
```

## Response Format

Every engine returns this structure:

```json
{
  "engine_id": "panchanga",
  "result": { ... },
  "witness_prompt": "A reflective question — the core output for consciousness work",
  "consciousness_level": 0,
  "metadata": {
    "calculation_time_ms": 0.02,
    "backend": "native-rust",
    "precision_achieved": "Standard",
    "cached": false,
    "timestamp": "2026-02-15T20:45:15Z",
    "engine_version": "0.1.0"
  }
}
```

Workflow responses wrap multiple engine outputs:

```json
{
  "workflow_id": "daily-practice",
  "engine_outputs": {
    "panchanga": { ... },
    "vedic-clock": { ... },
    "biorhythm": { ... }
  },
  "synthesis": { ... },
  "metadata": { ... }
}
```

## 16 Engines

| Engine ID | Backend | What It Calculates |
|-----------|---------|-------------------|
| `panchanga` | Rust | Vedic calendar — tithi, nakshatra, yoga, karana, vara |
| `numerology` | Rust | Life path, expression, soul urge (Pythagorean + Chaldean) |
| `biorhythm` | Rust | Physical, emotional, intellectual cycles from birth date |
| `human-design` | Rust | Bodygraph — type, authority, profile, defined centers, gates |
| `gene-keys` | Rust | Shadow-Gift-Siddhi activation sequences from HD gates |
| `vimshottari` | Rust | 120-year Vedic dasha periods — current mahadasha/antardasha |
| `biofield` | Rust | Chakra resonance and energy field analysis from birth data |
| `vedic-clock` | Rust | Current muhurta, hora, ghati — Vedic time divisions |
| `face-reading` | Rust | Physiognomy patterns from birth data |
| `nadabrahman` | Rust | Sound consciousness — nada frequencies from birth data |
| `transits` | Rust | Current planetary transits, aspects, Sade Sati status |
| `tarot` | TypeScript | Three-card spread — past/present/future with arcana |
| `i-ching` | TypeScript | Hexagram casting with changing lines and interpretation |
| `enneagram` | TypeScript | Enneagram type analysis with wing and integration lines |
| `sacred-geometry` | TypeScript | Sacred geometry patterns from birth numerology |
| `sigil-forge` | TypeScript | Sigil creation from intention and birth data |

## 6 Workflows

| Workflow ID | Engines Used | Purpose |
|-------------|-------------|---------|
| `birth-blueprint` | numerology, human-design, gene-keys | Core identity mapping |
| `daily-practice` | panchanga, vedic-clock, biorhythm | Daily rhythm awareness |
| `decision-support` | tarot, i-ching, human-design | Multi-system decision mirrors |
| `self-inquiry` | gene-keys, enneagram | Deep self-consciousness exploration |
| `creative-expression` | sigil-forge, sacred-geometry | Creative and aesthetic exploration |
| `full-spectrum` | All 16 engines | Complete self-portrait |

## Using the Python Bridge Script

The bridge script at `.claude/skills/selemene-bridge/scripts/bridge.py` wraps the API for CLI use:

```bash
# Set the environment (production)
export SELEMENE_URL="https://selemene.tryambakam.space"
export SELEMENE_API_KEY="nk_WSgs4eA9bhVa1GNrcOIUw6Vj9VqoxBxs"

# Health check
python .claude/skills/selemene-bridge/scripts/bridge.py selemene_health

# List engines
python .claude/skills/selemene-bridge/scripts/bridge.py selemene_list_engines

# Calculate with an engine
python .claude/skills/selemene-bridge/scripts/bridge.py selemene_calculate '{
  "engine_id": "panchanga",
  "birth_data": {
    "name": "Test",
    "date": "1991-08-13",
    "time": "13:31",
    "latitude": 12.9716,
    "longitude": 77.5946,
    "timezone": "Asia/Kolkata"
  }
}'

# Execute a workflow
python .claude/skills/selemene-bridge/scripts/bridge.py selemene_workflow_execute '{
  "workflow_id": "daily-practice",
  "birth_data": {
    "name": "Test",
    "date": "1991-08-13",
    "time": "13:31",
    "latitude": 12.9716,
    "longitude": 77.5946,
    "timezone": "Asia/Kolkata"
  }
}'
```

## Witness Prompt Philosophy

The `witness_prompt` in each response is the most important output. It is:
- A **question**, never an answer
- A **mirror**, not a prescription
- Designed to help the user **notice patterns**, not follow advice

When presenting results to users, lead with the witness prompt. The numerical/symbolic data supports the inquiry — it doesn't replace it.
