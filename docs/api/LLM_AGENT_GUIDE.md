# Purpose

Provide a compact, deterministic API guide for LLMs and agents (Claude Code, OpenClaw, Cursor) to interface with Noesis as a reflection system: return patterns to witness, not prescriptions to follow.

# Current State

- Base URL: `https://selemene.tryambakam.space`
- Auth:
  - `X-API-Key: nk_<api_key>` for API key auth
  - `Authorization: Bearer <jwt>` for JWT auth
- Engines: `POST /api/v1/engines/{engine_id}/calculate`
- Workflows:
  - Canonical: `POST /api/v1/workflows/{workflow_id}/execute`, `GET /api/v1/workflows/{workflow_id}/info`
  - Compatibility aliases: `POST/GET /api/v1/workflows/{workflow_id}`
- Shared request schema: `EngineInput`
- API key is a unique user identity; if `birth_data` is present, user profile is auto-populated.

# Required Actions

1. Always send auth headers.
2. Use the `EngineInput` schema exactly as shown.
3. Prefer `precision: "Standard"` unless a user asks otherwise.
4. When unsure, include only `birth_data` and omit `options`.
5. Handle non-200 responses using the standard error schema.
6. For workflow calls, treat `engine_outputs` as partial-by-design and handle missing engines gracefully.

# Verification

1. `GET /health/live` should return status `ok`.
2. `GET /api/v1/engines` should return 16 engines.
3. A test call to `numerology` with `birth_data` should return `engine_id`, `result`, and `witness_prompt`.

# API Schema Summary

## Headers

```http
X-API-Key: nk_<api_key>
```

```http
Authorization: Bearer <jwt_token>
```

## EngineInput

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
- `current_time` defaults to now if omitted.
- `precision` values: `Standard`, `High`, `Extreme`.

# Minimal Engine Example

```
POST /api/v1/engines/numerology/calculate
```

```bash
curl -s -X POST https://selemene.tryambakam.space/api/v1/engines/numerology/calculate \
  -H "X-API-Key: $NOESIS_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": {
      "name": "Test User",
      "date": "1991-08-13",
      "time": "13:31",
      "latitude": 12.9716,
      "longitude": 77.5946,
      "timezone": "Asia/Kolkata"
    }
  }'
```

# Minimal Workflow Example

```
POST /api/v1/workflows/birth-blueprint/execute
```

```bash
curl -s -X POST https://selemene.tryambakam.space/api/v1/workflows/birth-blueprint/execute \
  -H "X-API-Key: $NOESIS_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": {
      "date": "1991-08-13",
      "time": "13:31",
      "latitude": 12.9716,
      "longitude": 77.5946,
      "timezone": "Asia/Kolkata"
    }
  }'
```

# Error Schema

```json
{
  "error": "Invalid or expired API key",
  "error_code": "UNAUTHORIZED",
  "details": {"auth_method": "api_key"}
}
```

# Engine Compatibility and Fallbacks

- Numerology requires `birth_data.name`; missing name returns `422 VALIDATION_ERROR`.
- Sigil Forge intention aliases accepted in `options`: `question`, `intention`, `intent`, `intent_text`.
- Gene Keys fallback:
  1. First try `POST /api/v1/engines/gene-keys/calculate` with `birth_data`.
  2. On `500 CALCULATION_ERROR`, call Human Design, extract Sun/Earth gates, retry Gene Keys with `options.hd_gates`.

# Reading-Object Contract (v3.3.0)

Workflow responses now include a reading-object at the top level. Agents should surface `witness_layer.question` to the user as the primary reflection prompt.

```json
{
  "workflow_id": "daily-practice",
  "reading_id": "uuid",
  "reading_url": "https://noesis.tryambakam.space/readings/uuid",
  "created_at": "ISO-8601",
  "subject": "birth-data-summary",
  "evidence": ["panchanga", "vedic-clock", "biorhythm"],
  "witness_layer": {
    "title": "...",
    "summary": "...",
    "convergences": ["..."],
    "frictions": ["..."],
    "practice": "...",
    "question": "..."
  },
  "engine_outputs": {...}
}
```

Always present `witness_layer.question` as the closing reflection. Do not interpret it — only witness it.

# Hermes Integration (Planned)

Hermes agent integration is in progress. Tool definitions will be available at `bridges/hermes/` once finalized. Use the Bridge CLI for now: `bridges/cli/`.
