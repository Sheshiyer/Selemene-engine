# Engine API Endpoints

## Overview

Engine endpoints provide direct access to individual symbolic mirrors.
All engines use the shared `EngineInput` schema and return `EngineOutput` for reflection and inquiry.

## Base Path

```
/api/v1/engines/{engine_id}/calculate
```

## Authentication

Use one of the following headers:

```http
Authorization: Bearer <jwt_token>
```

```http
X-API-Key: nk_<api_key>
```

## Shared Request Schema

```json
{
  "birth_data": {
    "name": "Optional Name",
    "date": "1990-01-01",
    "time": "14:30",
    "latitude": 12.9716,
    "longitude": 77.5946,
    "timezone": "Asia/Kolkata"
  },
  "current_time": "2026-02-16T00:00:00Z",
  "location": {"latitude": 12.9716, "longitude": 77.5946},
  "precision": "Standard",
  "options": {}
}
```

Notes:
- `birth_data` is optional, but required for birth-chart engines.
- `birth_data.name` is required for numerology.
- `current_time` defaults to now if omitted.
- `precision` is one of: `Standard`, `High`, `Extreme`.
- `options` is engine-specific.

## Available Engines

### Rust Engines (Native)

| Engine ID | Name | Required Phase |
|-----------|------|----------------|
| human-design | Human Design | 1 |
| gene-keys | Gene Keys | 2 |
| vimshottari | Vimshottari Dasha | 2 |
| panchanga | Panchanga | 0 |
| numerology | Numerology | 0 |
| biorhythm | Biorhythm | 0 |
| vedic-clock | Vedic Clock | 0 |
| biofield | Biofield | 1 |
| face-reading | Face Reading | 1 |
| nadabrahman | Nadabrahman | 0 |
| transits | Transits | 0 |

### TypeScript Engines (Bridged)

| Engine ID | Name | Required Phase |
|-----------|------|----------------|
| tarot | Tarot | 0 |
| i-ching | I-Ching | 0 |
| enneagram | Enneagram | 1 |
| sacred-geometry | Sacred Geometry | 0 |
| sigil-forge | Sigil Forge | 1 |

Sigil Forge compatibility aliases:
- `options.question`
- `options.intention`
- `options.intent`
- `options.intent_text`

---

## Example: Numerology

```
POST /api/v1/engines/numerology/calculate
```

```bash
curl -X POST http://localhost:8080/api/v1/engines/numerology/calculate \
  -H "X-API-Key: $NOESIS_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": {
      "date": "1990-03-15",
      "name": "John Michael Smith"
    }
  }'
```

## Example: Panchanga

```
POST /api/v1/engines/panchanga/calculate
```

```bash
curl -X POST http://localhost:8080/api/v1/engines/panchanga/calculate \
  -H "X-API-Key: $NOESIS_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": {
      "date": "2025-01-15",
      "latitude": 28.6139,
      "longitude": 77.2090,
      "timezone": "Asia/Kolkata"
    }
  }'
```

## Example: Tarot

```
POST /api/v1/engines/tarot/calculate
```

```bash
curl -X POST http://localhost:8080/api/v1/engines/tarot/calculate \
  -H "X-API-Key: $NOESIS_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "options": {
      "spread": "horseshoe",
      "question": "What should I focus on this month?",
      "reversed_enabled": true
    }
  }'
```

Supported tarot spread values:
- `single_card`
- `three_card`
- `celtic_cross`
- `horseshoe`
- `relationship`
- `career`
- `yes_no`

Compatibility aliases accepted by the engine:
- `options.spread` (preferred)
- `options.spread_type` (legacy)
- `options.spreadType` (legacy)

Example tarot response payload:

```json
{
  "engine_id": "tarot",
  "result": {
    "spread": {
      "type": "yes_no",
      "name": "Yes or No",
      "description": "A focused one-card spread for binary clarity in the present moment",
      "card_count": 1,
      "available_types": ["single_card", "three_card", "celtic_cross", "horseshoe", "relationship", "career", "yes_no"]
    },
    "question": "Should I proceed with this partnership?",
    "positions": [
      {
        "position": 0,
        "name": "Answer Card",
        "meaning": "The card polarity and orientation indicate a yes/no tendency",
        "card": {
          "id": "m00",
          "name": "The Fool",
          "arcana": "major",
          "number": 0,
          "isReversed": false,
          "interpretation": {
            "meaning": "Beginnings and unguarded trust",
            "keywords": ["beginnings", "trust"]
          }
        }
      }
    ],
    "cards": [
      {
        "position": 0,
        "name": "Answer Card",
        "cardName": "The Fool"
      }
    ],
    "decision": {
      "answer": "yes",
      "confidence": 0.74,
      "rationale": "The Fool appeared upright, suggesting a supportive current for this direction."
    }
  },
  "witness_prompts": [
    {
      "prompt": "What makes this yes feel alive in your body?"
    }
  ],
  "calculated_at": "2026-04-24T12:00:00Z",
  "processing_time_ms": 14
}
```

## Errors

All engine errors use a consistent schema:

```json
{
  "error": "Access denied: requires phase 2, current phase 1",
  "error_code": "PHASE_ACCESS_DENIED",
  "details": {
    "required_phase": 2,
    "current_phase": 1
  }
}
```
