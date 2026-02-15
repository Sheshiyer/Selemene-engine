# Purpose

Provide a concrete, OpenClaw-ready integration guide for calling Noesis APIs from OpenClaw agents and tools.

# Current State

- Base URL: `https://selemene.tryambakam.space`
- Auth header: `X-API-Key: nk_<api_key>`
- Engine endpoint: `POST /api/v1/engines/{engine_id}/calculate`
- Workflow endpoint: `POST /api/v1/workflows/{workflow_id}/execute`
- Shared request schema: `EngineInput` (see below)
- API key is a unique user identity; `birth_data` auto-populates the user profile.

# Required Actions

1. Store the API key in OpenClaw credentials:
   - Add `NOESIS_API_KEY` to `~/.openclaw/credentials/<profile>.env`.
2. Ensure the OpenClaw runtime loads that profile env (preferred):
   - `openclaw-seed env --root "$(pwd)" --profile <profile>`
3. Use the Noesis base URL and headers in tool calls:
   - Header: `X-API-Key: $NOESIS_API_KEY`
   - Header: `Content-Type: application/json`
4. Use the `EngineInput` JSON body for engines and workflows.
5. Handle errors using the standard error schema.

# Verification

1. `GET /health/live` returns `{"status":"ok"...}`.
2. `GET /api/v1/engines` returns 16 engines.
3. `POST /api/v1/engines/numerology/calculate` succeeds with `engine_id`, `result`, `witness_prompt`.

# EngineInput

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

# OpenClaw Tool Call Example

Use a standard HTTP tool or a custom wrapper. Example request:

```http
POST /api/v1/engines/numerology/calculate
Host: selemene.tryambakam.space
X-API-Key: nk_<api_key>
Content-Type: application/json

{"birth_data":{"name":"Test User","date":"1991-08-13","time":"13:31","latitude":12.9716,"longitude":77.5946,"timezone":"Asia/Kolkata"}}
```

# Minimal cURL (for tool verification)

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

# Error Schema

```json
{
  "error": "Invalid or expired API key",
  "error_code": "UNAUTHORIZED",
  "details": {"auth_method": "api_key"}
}
```
