# Migrating from Noesis API v2 to v3

This guide covers every breaking change between the v2 (Selemene) and v3 (Noesis) API
surfaces so you can update your integration with confidence.

---

## Table of Contents

1. [Base URL](#base-url)
2. [Authentication](#authentication)
3. [Request Schema — `EngineInput`](#request-schema--engineinput)
4. [Engine ID Changes](#engine-id-changes)
5. [Workflow Changes](#workflow-changes)
6. [Response Schema Changes](#response-schema-changes)
7. [Error Response Format](#error-response-format)
8. [Input Validation](#input-validation)
9. [Removed Endpoints](#removed-endpoints)
10. [New Endpoints in v3](#new-endpoints-in-v3)
11. [Quick-Reference Diff Table](#quick-reference-diff-table)

---

## Base URL

| Version | Base URL |
|---------|----------|
| v2 | `https://selemene.tryambakam.space/api/v2` |
| v3 | `https://selemene.tryambakam.space/api/v1` |

> **Note:** The v3 path segment is `/api/v1` (not `/api/v3`). The version is expressed in the
> product line, not the URL segment.

---

## Authentication

### v2 — Bearer + API key mixed headers
```
Authorization: Bearer <jwt>
X-API-Key: <api-key>
```
In v2 both headers were accepted but treated independently with separate code paths.

### v3 — Single `Authorization` header, unified identity
```
Authorization: Bearer <jwt>        # Issued via POST /auth/login
Authorization: Bearer <api-key>    # API keys are treated as first-class bearer tokens
```

**Breaking changes:**
- `X-API-Key` header is **no longer supported**. API keys must be passed in `Authorization: Bearer`.
- API keys now act as full user identities. The response includes the same `AuthUser` fields as
  JWT-authenticated requests.
- JWT tokens are valid for **24 hours**. Refresh by calling `POST /auth/login` again.
- Required JWT claims: `exp`, `iat`, `sub`. Tokens missing any of these are rejected.

---

## Request Schema — `EngineInput`

### v2 schema (flat)
```json
{
  "date": "1990-06-15",
  "time": "14:30",
  "latitude": 28.6139,
  "longitude": 77.2090,
  "timezone": "Asia/Kolkata",
  "options": {}
}
```

### v3 schema (nested `birth_data`)
```json
{
  "birth_data": {
    "date": "1990-06-15",
    "time": "14:30",
    "latitude": 28.6139,
    "longitude": 77.2090,
    "timezone": "Asia/Kolkata"
  },
  "options": {}
}
```

**All birth-related fields are now nested under `birth_data`.** Top-level flat fields are ignored.

### Validated bounds (new in v3)

| Field | Constraint | Error |
|-------|-----------|-------|
| `birth_data.latitude` | `[-90, 90]` | 422 VALIDATION_ERROR |
| `birth_data.longitude` | `[-180, 180]` | 422 VALIDATION_ERROR |
| `birth_data.date` | `YYYY-MM-DD` | 422 VALIDATION_ERROR |
| `birth_data.time` | `HH:MM` or `HH:MM:SS` | 422 VALIDATION_ERROR |
| `options` map size | ≤ 64 keys | 422 VALIDATION_ERROR |

---

## Engine ID Changes

| v2 engine ID | v3 engine ID | Notes |
|---|---|---|
| `vedic-chart` | `panchanga` | Renamed; now includes tithi, nakshatra, yoga, karana |
| `hd-bodygraph` | `human-design` | |
| `gene-keys` | `gene-keys` | Unchanged |
| `dasha` | `vimshottari` | Renamed to match standard term |
| `numerology` | `numerology` | Unchanged |
| `biorhythm` | `biorhythm` | Now includes aesthetic + spiritual secondary cycles |
| `organ-clock` | `vedic-clock` | Renamed |
| `biofield` | `biofield` | Unchanged |
| `face-reading` | `face-reading` | Unchanged |
| `sound` | `nadabrahman` | Renamed |
| `transits` | `transits` | Unchanged |
| `tarot` *(TS)* | `tarot` | Unchanged (TS sidecar) |
| `i-ching` *(TS)* | `i-ching` | Unchanged (TS sidecar) |
| `enneagram` *(TS)* | `enneagram` | Unchanged (TS sidecar) |
| `sacred-geometry` *(TS)* | `sacred-geometry` | Unchanged (TS sidecar) |
| `sigil-forge` *(TS)* | `sigil-forge` | Unchanged (TS sidecar) |

Use `GET /api/v1/engines` to get the authoritative list at runtime.

---

## Workflow Changes

### Renamed workflows

| v2 workflow ID | v3 workflow ID |
|---|---|
| `birth-chart` | `birth-blueprint` |
| `daily` | `daily-practice` |
| `decision` | `decision-support` |
| `inquiry` | `self-inquiry` |
| `creative` | `creative-expression` |
| `full` | `full-spectrum` |

### `daily-practice` now includes `transits`

The `daily-practice` workflow previously ran 3 engines (`panchanga`, `vedic-clock`, `biorhythm`).
In v3 it runs 4 — `transits` is added as the fourth engine.

If you were post-processing `daily-practice` results by index, update your code to handle the
additional `transits` result.

### `full-spectrum` passes `partner_birth_date` to `biorhythm`

When `options.partner_birth_date` is set in a `full-spectrum` request, the biorhythm engine now
receives it and returns a `compatibility` block in its output.

---

## Response Schema Changes

### Engine output envelope

#### v2
```json
{
  "engine_id": "vedic-chart",
  "result": { ... },
  "calculated_at": "2026-04-30T12:00:00Z"
}
```

#### v3
```json
{
  "engine_id": "panchanga",
  "output": { ... },
  "witness_prompt": "You stand at the junction of Bharani and Krittika...",
  "consciousness_level": 1,
  "calculation_time_ms": 4.7,
  "calculated_at": "2026-04-30T12:00:00Z"
}
```

Key changes:
- `result` → `output`
- `witness_prompt` added (consciousness calibration text)
- `consciousness_level` added (integer tier 1–7)
- `calculation_time_ms` added

### Biorhythm output — secondary cycles (new in v3)

```json
{
  "output": {
    "cycles": {
      "physical":      { "value": 0.72, "percentage": 72, "phase": "ascending" },
      "emotional":     { "value": -0.31, "percentage": -31, "phase": "descending" },
      "intellectual":  { "value": 0.95, "percentage": 95, "phase": "peak" },
      "aesthetic":     { "value": 0.44, "percentage": 44, "phase": "ascending" },
      "spiritual":     { "value": -0.61, "percentage": -61, "phase": "descending" }
    },
    "critical_days": [],
    "compatibility": {
      "physical": 88,
      "emotional": 73,
      "intellectual": 91,
      "overall": 84
    }
  }
}
```

`aesthetic` and `spiritual` are new in v3. `compatibility` only appears when `partner_birth_date`
is supplied in `options`.

---

## Error Response Format

#### v2
```json
{
  "error": "not_found",
  "message": "Engine vedic-chart not found"
}
```

#### v3
```json
{
  "status": 404,
  "error_code": "ENGINE_NOT_FOUND",
  "message": "Engine 'vedic-chart' not found. Use GET /engines for the engine list.",
  "error": "ENGINE_NOT_FOUND",
  "trace_id": "01HZ..."
}
```

Key changes:
- `status` field (numeric HTTP status) added
- `error_code` is now SCREAMING_SNAKE_CASE
- `trace_id` added for support requests
- Legacy `error` field retained (same value as `error_code`) for backward compatibility

---

## Input Validation

v3 validates `EngineInput` at the API boundary before the request reaches any engine. Invalid
requests receive a `422 Unprocessable Entity` with `error_code: "VALIDATION_ERROR"` before any
compute is performed.

Previously in v2, some invalid inputs were silently coerced or produced engine-specific error
messages; now they fail fast with a structured error.

---

## Removed Endpoints

| Endpoint | Reason |
|---|---|
| `POST /api/v2/calculate` | Replaced by `POST /api/v1/engines/:engine_id/calculate` |
| `POST /api/v2/chart` | Merged into `panchanga` engine |
| `GET /api/v2/status` | Replaced by `GET /health/live` and `GET /health/ready` |
| `POST /api/v2/workflow` | Replaced by `POST /api/v1/workflows/:workflow_id` |

---

## New Endpoints in v3

| Endpoint | Description |
|---|---|
| `GET /api/v1/engines` | List all engines with metadata |
| `GET /api/v1/engines/:id` | Engine info (description, phase gate, input schema) |
| `POST /api/v1/engines/:id/validate` | Validate engine output (no auth required) |
| `GET /api/v1/workflows` | List all workflows |
| `GET /api/v1/workflows/:id` | Workflow info |
| `GET /health/live` | Liveness probe |
| `GET /health/ready` | Readiness probe with dependency status |
| `GET /metrics` | Prometheus metrics (internal only) |
| `GET /admin/ephemeris/checksums` | Ephemeris data integrity checksums |

---

## Quick-Reference Diff Table

| What | v2 | v3 |
|------|----|----|
| Auth header | `X-API-Key` or `Authorization: Bearer` | `Authorization: Bearer` only |
| Birth fields | Top-level flat fields | Nested under `birth_data` |
| Engine result key | `result` | `output` |
| Error format | `{ error, message }` | `{ status, error_code, message, trace_id }` |
| Engine ID: Vedic calendar | `vedic-chart` | `panchanga` |
| Engine ID: Dasha | `dasha` | `vimshottari` |
| Engine ID: Organ clock | `organ-clock` | `vedic-clock` |
| Engine ID: Sound | `sound` | `nadabrahman` |
| Workflow ID: Daily | `daily` | `daily-practice` |
| Workflow ID: Full | `full` | `full-spectrum` |
| `daily-practice` engines | 3 | 4 (+transits) |
| Biorhythm secondary cycles | None | `aesthetic`, `spiritual` |
| Input validation | Engine-level | API boundary (422 before engine) |
