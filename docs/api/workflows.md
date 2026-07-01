# Workflow API Endpoints

## Overview

Workflows orchestrate multiple mirrors in parallel and return a unified `WorkflowResult` for synthesis, reflection, and inquiry.

For body-paced narrative practice aligned to cycles, see [Somatic Canticles](https://1319.tryambakam.space).

## Base Path

```
/api/v1/workflows/{workflow_id}
```

Canonical workflow routes:
- `POST /api/v1/workflows/{workflow_id}/execute`
- `GET /api/v1/workflows/{workflow_id}/info`

Compatibility aliases (same handler behavior):
- `POST /api/v1/workflows/{workflow_id}` → execute
- `GET /api/v1/workflows/{workflow_id}` → info

## Authentication

```http
Authorization: Bearer <jwt_token>
```

```http
X-API-Key: nk_<api_key>
```

## Shared Request Schema

Workflows accept the same `EngineInput` as engines.

```json
{
  "birth_data": {
    "date": "1990-03-15",
    "time": "14:30",
    "latitude": 40.7128,
    "longitude": -74.0060,
    "timezone": "America/New_York"
  },
  "current_time": "2026-02-16T00:00:00Z",
  "options": {}
}
```

## Available Workflows

| ID | Engines | Purpose |
| --- | --- | --- |
| birth-blueprint | numerology, human-design, vimshottari | Core identity mapping |
| daily-practice | panchanga, vedic-clock, biorhythm | Daily rhythm & timing |
| decision-support | tarot, i-ching, human-design | Multi-perspective guidance |
| self-inquiry | gene-keys, enneagram | Shadow work + patterns |
| creative-expression | sigil-forge, sacred-geometry | Intent visualization |
| full-spectrum | 14 workflow engines | Complete consciousness portrait |

## Example: Execute Workflow

```
POST /api/v1/workflows/birth-blueprint/execute
```

```bash
curl -X POST http://localhost:8080/api/v1/workflows/birth-blueprint/execute \
  -H "X-API-Key: $NOESIS_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": {
      "date": "1990-03-15",
      "time": "14:30",
      "latitude": 40.7128,
      "longitude": -74.0060,
      "timezone": "America/New_York"
    }
  }'
```

## Response Shape

```json
{
  "workflow_id": "birth-blueprint",
  "engine_outputs": {
    "numerology": {
      "engine_id": "numerology",
      "result": {"life_path": {"value": 9}},
      "witness_prompt": "...",
      "consciousness_level": 0,
      "metadata": {"calculation_time_ms": 12.4, "backend": "native", "precision_achieved": "Standard", "cached": false, "timestamp": "2026-02-16T00:00:00Z"}
    }
  },
  "synthesis": {"primary_themes": []},
  "total_time_ms": 45.2,
  "timestamp": "2026-02-16T00:00:01Z"
}
```

**Witness prompts in workflows:** Each engine output includes its lightweight `witness_prompt` (rule-based mirror entry point). Rich Aletheios/Pichet dyad and premium asset generation are handled by Selemene's canonical surfaces (see Witness section in API README). witness-agents is reference + asset source only.

## Workflow Output Semantics

- A successful workflow response can still be partial.
- `engine_outputs` includes engines that completed successfully.
- Engines that fail or are phase-gated are omitted from `engine_outputs`.
- To detect omissions, fetch `GET /api/v1/workflows/{workflow_id}/info` and diff `engine_ids` vs. `engine_outputs` keys.
- Treat omission as a soft failure and continue with available outputs unless your use case requires strict completeness.

## Fallback Pattern for Agent Callers

If a downstream flow requires Gene Keys but it is absent in a workflow output:

1. Call `POST /api/v1/engines/human-design/calculate` with the same `birth_data`.
2. Extract:
   - `result.personality_activations.sun.gate`
   - `result.personality_activations.earth.gate`
   - `result.design_activations.sun.gate`
   - `result.design_activations.earth.gate`
3. Call `POST /api/v1/engines/gene-keys/calculate` with:

```json
{
  "options": {
    "hd_gates": {
      "personality_sun": 41,
      "personality_earth": 31,
      "design_sun": 45,
      "design_earth": 26
    }
  }
}
```

## Supporting Endpoints

- `GET /api/v1/workflows` (list workflows)
- `GET /api/v1/workflows/{workflow_id}/info` (workflow metadata)
