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

![Bridge engine icon set](../assets/images/engines/5D-3-bridge-engine-icons-recraft-v2.png)

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
      "spread_type": "three_card",
      "question": "What should I focus on this month?",
      "reversed_enabled": true
    }
  }'
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
