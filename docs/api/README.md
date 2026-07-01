# Noesis API Documentation

## Overview

Noesis provides 16 symbolic mirrors and multi-engine workflows through a unified REST API.
All engines share a common request shape (`EngineInput`) and return a normalized `EngineOutput` for reflection and inquiry.

Not prediction. Reflection. Inquiry. Witness.

## Quick Links

- **Live OpenAPI Spec** (v3.3.0): [https://selemene.tryambakam.space/api/openapi.json](https://selemene.tryambakam.space/api/openapi.json)
- **Swagger UI**: [https://selemene.tryambakam.space/api/docs](https://selemene.tryambakam.space/api/docs)
- **OpenAPI YAML** (local ref): [openapi.yaml](./openapi.yaml)
- **Engines**: [engines.md](./engines.md)
- **Workflows**: [workflows.md](./workflows.md)
- **Authentication**: [authentication.md](./authentication.md)
- **Billing (User)**: [billing.md](./billing.md)
- **Admin Analytics**: [admin-analytics.md](./admin-analytics.md)
- **Admin Reconciliation**: [admin-reconcile.md](./admin-reconcile.md)
- **LLM and Agent Guide**: [LLM_AGENT_GUIDE.md](./LLM_AGENT_GUIDE.md)
- **OpenClaw Integration**: [OPENCLAW_INTEGRATION.md](./OPENCLAW_INTEGRATION.md)
- **MCP Integration**: [MCP_INTEGRATION.md](./MCP_INTEGRATION.md)

## Base URLs

| Environment | URL | Description |
| --- | --- | --- |
| Local | `http://localhost:8080` | Rust API server |
| Production | `https://selemene.tryambakam.space` | Production deployment |

## Interactive API Explorer

- Swagger UI: `GET /api/docs`
- OpenAPI JSON: `GET /api/openapi.json`

## Authentication

Use **one** of the following headers:

```http
Authorization: Bearer <jwt_token>
```

```http
X-API-Key: nk_<api_key>
```

## Shared Request Schema

All engine and workflow execution endpoints accept `EngineInput`:

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
- `precision` is one of: `Standard`, `High`, `Extreme`.
- `options` is engine-specific.

## Core Endpoints

### Health

- `GET /health`
- `GET /health/live`
- `GET /health/ready`
- `GET /ready`
- `GET /metrics` (Prometheus)
- `GET /api/v1/status` (list engines and workflows; auth required)

### Engines

- `GET /api/v1/engines`
- `POST /api/v1/engines/{engine_id}/calculate`
- `POST /api/v1/engines/{engine_id}/validate`
- `GET /api/v1/engines/{engine_id}/info`

### Workflows

- `GET /api/v1/workflows`
- `GET /api/v1/workflows/{workflow_id}` (compat alias for `.../info`)
- `POST /api/v1/workflows/{workflow_id}` (compat alias for `.../execute`)
- `POST /api/v1/workflows/{workflow_id}/execute`
- `GET /api/v1/workflows/{workflow_id}/info`

Workflow output contract (v3.3.0):
- `workflow_id` and `total_time_ms` are always present on success.
- `engine_outputs` contains successful engine results only.
- `reading_id`, `reading_url`, `created_at`, `subject`, `evidence` are top-level reading-object fields.
- `witness_layer` contains: `title`, `summary`, `convergences`, `frictions`, `practice`, `question`.
- For strict completeness checks, compare `engine_outputs` keys to `GET /api/v1/workflows/{workflow_id}/info.engine_ids`.

### Admin (Users & Keys)

Requires `admin` or `platform-admin` role. See [admin-analytics.md](./admin-analytics.md) for analytics endpoints.

- `GET /api/v1/admin/session` — effective permissions for current token
- `GET /api/v1/admin/users` — list all users (`admin:users:list`)
- `PATCH /api/v1/admin/users/{user_id}/state` — activate/deactivate user
- `PATCH /api/v1/admin/users/{user_id}/tier` — change user tier
- `PUT /api/v1/admin/users/{user_id}/roles` — assign roles
- `GET /api/v1/admin/api-keys` — list all API keys (`admin:keys:list`)
- `POST /api/v1/admin/api-keys/{key_id}/revoke` — revoke key
- `POST /api/v1/admin/api-keys/{key_id}/rotate` — rotate key
- `GET /api/v1/admin/audit-events` — audit log (`admin:audit:list`)
- `GET /api/v1/admin/system/health` — system health (`admin:system:read`)
- `GET /api/v1/admin/system/cache` — cache stats
- `GET /api/v1/admin/system/services` — service statuses

### Admin Billing

Requires `billing-admin` role. Full reference: [admin-analytics.md](./admin-analytics.md).

- `GET /api/v1/admin/billing/overview` — MRR estimate, subscription counts
- `GET /api/v1/admin/billing/subscriptions` — paginated subscription list
- `POST /api/v1/admin/billing/subscriptions/{id}/cancel` — force-cancel
- `GET /api/v1/admin/billing/webhook-events` — processed webhook log
- `GET /api/v1/admin/billing/reconcile/drift` — billing drift report
- `POST /api/v1/admin/billing/reconcile/run` — trigger reconciliation
- `GET /api/v1/admin/billing/plans` — plan catalog
- `GET /api/v1/admin/usage/summary` — usage across all users
- `GET /api/v1/admin/analytics/summary` — aggregated analytics

### Billing (User-facing)

Full reference: [billing.md](./billing.md).

- `GET /api/v1/billing/balance` — credit balance and tier
- `GET /api/v1/billing/subscription` — subscription status
- `POST /api/v1/billing/checkout` — initiate plan upgrade (Dodo Payments)
- `GET /api/v1/billing/portal` — Dodo customer portal redirect

### Biofield

- `GET /api/v1/biofield/readings` — list biofield readings
- `GET /api/v1/biofield/readings/{reading_id}` — single reading
- `POST /api/v1/biofield/readings/{reading_id}/reprocess` — reprocess
- `GET /api/v1/biofield/baselines` — user baselines
- `POST /api/v1/biofield/exports` — create export
- `GET /api/v1/biofield/sessions` — PIP camera sessions
- `POST /api/v1/biofield/sessions/{session_id}/captures` — capture frame
- `POST /api/v1/biofield/sessions/{session_id}/close` — close session

### Witness

- `POST /api/v1/witness/interpret` — generate rich witness dyad interpretation (Aletheios + Pichet voices + synthesis) from engine outputs. Selemene is the canonical service for rich Aletheios/Pichet dyad interpretation.
  - Input: `{ engine_outputs: [...], context?: {...} }`
  - Output: reading-object with `witness_layer` fields (fixed 6-field dyad contract)
- Individual engine outputs continue to include a lightweight `witness_prompt` — this remains the rule-based, non-prescriptive mirror entry point. Rich persona voices and multi-pass synthesis live in the dyad path.
- Additive premium asset surfaces (e.g. `POST /api/v1/assets/generate` and SDK `generatePremiumAsset`) provide access to multi-pass integrated reading generation and source-pack artifacts. These are additive and do not affect existing contracts.

**Retirement note:** witness-agents is now reference + asset source only (personas, mode documents, historical batches). All live rich dyad and premium asset traffic uses Selemene endpoints and SDK surfaces. The non-prescriptive "mirror" philosophy is preserved.

### User Profile

- `GET /api/v1/users/me` — current user profile
- `GET /api/v1/users/me/usage` — personal usage stats

### Auth

- `POST /api/v1/auth/register`
- `POST /api/v1/auth/login`
- `POST /api/v1/auth/forgot-password`
- `POST /api/v1/auth/reset-password`
- `POST /api/v1/auth/change-password`
- `GET /api/v1/auth/discord/authorize` — Discord OAuth initiation
- `GET /api/v1/auth/discord/callback` — Discord OAuth callback

### Readings History

- `GET /api/v1/readings` — list saved readings for authenticated user
- `GET /api/v1/readings/{reading_id}` — single reading (full reading-object)
- `GET /api/v1/readings/stats` — reading statistics

### Legacy (Deprecated)

- `POST /api/legacy/panchanga/calculate`
- `GET /api/legacy/ghati/current`

## Example: Engine Calculation

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

## Error Format

```json
{
  "error": "Invalid or expired API key",
  "error_code": "UNAUTHORIZED",
  "details": {"auth_method": "api_key"}
}
```

## Rate Limiting

Rate limits are enforced server-side and vary by tier and deployment configuration.
If you receive `429`, slow down and retry with backoff.
