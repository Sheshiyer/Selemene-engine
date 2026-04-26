# OpenClaw Integration (Noesis-Tuned)

Use this guide to connect OpenClaw agents to Selemene/Noesis as a reflection interface: return patterns to witness, not prescriptions to follow.

## Integration Surface

- Base URL: `https://selemene.tryambakam.space`
- Auth (preferred for OpenClaw): `X-API-Key: nk_<api_key>`
- Engine execute: `POST /api/v1/engines/{engine_id}/calculate`
- Workflow execute: `POST /api/v1/workflows/{workflow_id}/execute`
- Workflow info: `GET /api/v1/workflows/{workflow_id}/info`
- Compatibility aliases: `POST/GET /api/v1/workflows/{workflow_id}`

All 16 engines are called through the main API endpoint. TypeScript engines are sidecar-bridged internally and are transparent to OpenClaw callers.

## OpenClaw Skill Alignment

The local OpenClaw `noesis` skill requires:

- env var: `NOESIS_API_KEY`

Skill metadata contract (current local skill):
- `openclaw.requires.env = ["NOESIS_API_KEY"]`

Recommended credential setup:

1. Add `NOESIS_API_KEY` to `~/.openclaw/credentials/<profile>.env`
2. Load env for the profile before agent runtime
3. Ensure Noesis requests include:
   - `X-API-Key: $NOESIS_API_KEY`
   - `Content-Type: application/json`

Do not send API keys via `Authorization: Bearer`; Bearer is JWT-only.

## Bridge Tooling in This Repo

Use these bridge assets when generating OpenClaw-adjacent tool definitions:

- Bridge overview: `../../bridges/README.md`
- CLI bridge generator: `../../bridges/cli/README.md`
- LangChain/CrewAI tools: `../../bridges/langchain/README.md`

CLI workflow:

- `npx @selemene/bridge init` — interactive setup, framework selection, config creation
- `npx @selemene/bridge generate` — regenerate tools after API changes
- `npx @selemene/bridge check` — connectivity check
- `npx @selemene/bridge doctor` — full diagnostics

This bridge path keeps OpenClaw tool contracts synced with live OpenAPI specs.

## Engine + Workflow Coverage

### Engines (16)

Rust-native (11):
- `panchanga`, `human-design`, `gene-keys`, `vimshottari`, `numerology`, `biorhythm`, `vedic-clock`, `biofield`, `face-reading`, `nadabrahman`, `transits`

TS-bridged (5):
- `tarot`, `i-ching`, `enneagram`, `sacred-geometry`, `sigil-forge`

### Workflows (6)

- `birth-blueprint`
- `daily-practice`
- `decision-support`
- `self-inquiry`
- `creative-expression`
- `full-spectrum`

## Request Contract (`EngineInput`)

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
- `birth_data` is optional globally, but required for birth-chart engines
- `numerology` requires `birth_data.name`
- `current_time` defaults to now
- `precision`: `Standard` | `High` | `Extreme`

## OpenClaw Runtime Patterns

### Deterministic Tool Surface

Expose these operations in OpenClaw wrappers:

1. `noesis_list_engines`
2. `noesis_calculate_engine`
3. `noesis_list_workflows`
4. `noesis_workflow_info`
5. `noesis_execute_workflow`

### Partial Workflow Handling

`WorkflowResult.engine_outputs` can be partial by design:

1. fetch expected engine set via workflow info
2. diff expected vs returned outputs
3. run fallback single-engine calls for missing outputs

### Bridge Engine Option Rules

For bridged engines (`tarot`, `i-ching`, `enneagram`, `sacred-geometry`, `sigil-forge`), place per-engine values under `options`.

For `sigil-forge`, the following aliases are accepted:
- `options.question`
- `options.intention`
- `options.intent`
- `options.intent_text`

### Gene Keys Fallback

If direct `gene-keys` birth-data mode errors:

1. call `human-design`
2. extract Sun/Earth personality + design gates
3. retry `gene-keys` using `options.hd_gates`

## Verification Checklist (OpenClaw)

1. `GET /health/live` returns status `ok`
2. `GET /api/v1/engines` returns 16 engine IDs
3. `POST /api/v1/engines/numerology/calculate` returns `engine_id`, `result`, `witness_prompt`
4. `POST /api/v1/workflows/birth-blueprint/execute` returns `workflow_id` and `engine_outputs`

## Minimal Verification Calls

```bash
curl -s https://selemene.tryambakam.space/health/live

curl -s https://selemene.tryambakam.space/api/v1/engines \
  -H "X-API-Key: $NOESIS_API_KEY"

curl -s -X POST https://selemene.tryambakam.space/api/v1/engines/numerology/calculate \
  -H "X-API-Key: $NOESIS_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"birth_data":{"name":"Test User","date":"1991-08-13","time":"13:31","latitude":12.9716,"longitude":77.5946,"timezone":"Asia/Kolkata"}}'
```

## Error Schema

```json
{
  "error": "Invalid or expired API key",
  "error_code": "UNAUTHORIZED",
  "details": {"auth_method": "api_key"}
}
```
