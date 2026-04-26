# Purpose

Provide a deterministic MCP-facing integration contract for Noesis engines and workflows.

# Base Contract

- Base URL: `https://selemene.tryambakam.space`
- Auth:
  - API key: `X-API-Key: nk_<api_key>`
  - JWT: `Authorization: Bearer <jwt>`
- Content type: `application/json`
- Shared payload: `EngineInput`

# Recommended MCP Tool Surface

Expose these tool-level actions from your MCP server:

1. `noesis_list_engines`
   - `GET /api/v1/engines`
2. `noesis_calculate_engine`
   - `POST /api/v1/engines/{engine_id}/calculate`
3. `noesis_validate_engine`
   - `POST /api/v1/engines/{engine_id}/validate`
4. `noesis_list_workflows`
   - `GET /api/v1/workflows`
5. `noesis_workflow_info`
   - Prefer `GET /api/v1/workflows/{workflow_id}/info`
   - Compat alias: `GET /api/v1/workflows/{workflow_id}`
6. `noesis_execute_workflow`
   - Prefer `POST /api/v1/workflows/{workflow_id}/execute`
   - Compat alias: `POST /api/v1/workflows/{workflow_id}`

# Output Semantics

- Engine calls return normalized `EngineOutput`.
- Workflow calls return `WorkflowResult`.
- `WorkflowResult.engine_outputs` may be partial:
  - successful engine outputs are present
  - failed/phase-gated engines are omitted

Strict-mode client behavior:

1. fetch expected engines via workflow info
2. diff against `engine_outputs` keys
3. run targeted fallback calls for missing engines

# Fallback Patterns

## Gene Keys Fallback (birth_data -> hd_gates)

If `POST /api/v1/engines/gene-keys/calculate` with `birth_data` returns `500 CALCULATION_ERROR`:

1. Call Human Design:
   - `POST /api/v1/engines/human-design/calculate`
2. Extract gates:
   - `result.personality_activations.sun.gate`
   - `result.personality_activations.earth.gate`
   - `result.design_activations.sun.gate`
   - `result.design_activations.earth.gate`
3. Retry Gene Keys with `options.hd_gates`.

## Sigil Forge Input Compatibility

For `sigil-forge`, intention can be passed as:
- `options.question`
- `options.intention`
- `options.intent`
- `options.intent_text`

## Numerology Validation

`numerology` requires:
- `birth_data`
- `birth_data.name`

Missing required fields return `422 VALIDATION_ERROR`.

# Bridge Engines (TS Sidecar)

Bridged engine IDs:
- `tarot`
- `i-ching`
- `enneagram`
- `sacred-geometry`
- `sigil-forge`

All are called through the same Noesis API endpoint:

```http
POST /api/v1/engines/{engine_id}/calculate
```

No MCP-side direct call to the TS sidecar is required.

# Minimal MCP Request Example

```json
{
  "engine_id": "daily-practice",
  "input": {
    "birth_data": {
      "date": "1991-08-13",
      "time": "13:31",
      "latitude": 12.9716,
      "longitude": 77.5946,
      "timezone": "Asia/Kolkata"
    },
    "options": {}
  }
}
```

Map to:

```http
POST /api/v1/workflows/daily-practice/execute
```
