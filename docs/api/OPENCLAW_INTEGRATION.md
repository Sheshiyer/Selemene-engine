# Purpose

Provide a concrete, OpenClaw-ready integration guide for calling Noesis APIs from OpenClaw agents and tools as reflective interfaces: return patterns to witness, not prescriptions to follow.

# Current State

- Base URL: `https://selemene.tryambakam.space`
- Auth header: `X-API-Key: nk_<api_key>`
- Engine endpoint: `POST /api/v1/engines/{engine_id}/calculate`
- Workflow endpoints:
  - Canonical: `POST /api/v1/workflows/{workflow_id}/execute`, `GET /api/v1/workflows/{workflow_id}/info`
  - Compatibility aliases: `POST/GET /api/v1/workflows/{workflow_id}`
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
   - Do not send API keys in `Authorization: Bearer`; Bearer is JWT-only.
4. Use the `EngineInput` JSON body for engines and workflows.
5. Handle errors using the standard error schema.

# Verification

1. `GET /health/live` returns `{"status":"ok"...}`.
2. `GET /api/v1/engines` returns 16 engines.
3. `POST /api/v1/engines/numerology/calculate` succeeds with `engine_id`, `result`, `witness_prompt`.

# Runtime Contract for OpenClaw Agents

- `workflow_id` + `total_time_ms` are stable on successful workflow execution.
- `engine_outputs` may be partial; missing engines are omitted (not null-filled).
- To enforce strict completeness:
  1. call `GET /api/v1/workflows/{workflow_id}/info`
  2. compare `engine_ids` with `engine_outputs` keys
  3. run fallback calls for missing engines as needed.

# Bridge Engine Inputs (TS sidecar)

For bridged engines (`tarot`, `i-ching`, `enneagram`, `sacred-geometry`, `sigil-forge`):
- Pass engine options via `EngineInput.options`.
- For `sigil-forge`, intention aliases are supported:
  - `options.question`
  - `options.intention`
  - `options.intent`
  - `options.intent_text`

# Gene Keys Fallback Flow

If `gene-keys` birth-data mode fails in a workflow or direct call:

1. Call Human Design:
   - `POST /api/v1/engines/human-design/calculate`
2. Extract gates from response:
   - `result.personality_activations.sun.gate`
   - `result.personality_activations.earth.gate`
   - `result.design_activations.sun.gate`
   - `result.design_activations.earth.gate`
3. Retry Gene Keys in `hd_gates` mode:

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
