# Workflow API Endpoints

## Overview

Workflows orchestrate multiple engines in parallel and return a unified `WorkflowResult`.

## Base Path

```
/api/v1/workflows/{workflow_id}/execute
```

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
| birth-blueprint | numerology, human-design, gene-keys | Core identity mapping |
| daily-practice | panchanga, vedic-clock, biorhythm | Daily rhythm & timing |
| decision-support | tarot, i-ching, human-design | Multi-perspective guidance |
| self-inquiry | gene-keys, enneagram | Shadow work + patterns |
| creative-expression | sigil-forge, sacred-geometry | Intent visualization |
| full-spectrum | all 16 engines | Complete consciousness portrait |

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

## Supporting Endpoints

- `GET /api/v1/workflows` (list workflows)
- `GET /api/v1/workflows/{workflow_id}/info` (workflow metadata)