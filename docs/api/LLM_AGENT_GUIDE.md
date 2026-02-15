# Purpose

Provide a compact, deterministic API guide for LLMs and agents (Claude Code, OpenClaw, Cursor) to call Noesis safely and consistently.

# Current State

- Base URL: `https://selemene.tryambakam.space`
- Auth: `X-API-Key` or `Authorization: Bearer <jwt>`
- Engines: `POST /api/v1/engines/{engine_id}/calculate`
- Workflows: `POST /api/v1/workflows/{workflow_id}/execute`
- Shared request schema: `EngineInput`
- API key is a unique user identity; if `birth_data` is present, user profile is auto-populated.

# Required Actions

1. Always send auth headers.
2. Use the `EngineInput` schema exactly as shown.
3. Prefer `precision: "Standard"` unless a user asks otherwise.
4. When unsure, include only `birth_data` and omit `options`.
5. Handle non-200 responses using the standard error schema.

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