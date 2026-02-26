# Noesis API Documentation

## Overview

Noesis provides 16 symbolic mirrors and multi-engine workflows through a unified REST API.
All engines share a common request shape (`EngineInput`) and return a normalized `EngineOutput` for reflection and inquiry.

Not prediction. Reflection. Inquiry. Witness.

## Quick Links

- **OpenAPI Spec**: [openapi.yaml](./openapi.yaml)
- **Engines**: [engines.md](./engines.md)
- **Workflows**: [workflows.md](./workflows.md)
- **Authentication**: [authentication.md](./authentication.md)
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

Workflow output contract:
- `workflow_id` and `total_time_ms` are always present on success.
- `engine_outputs` contains successful engine results only.
- For strict completeness checks, compare `engine_outputs` keys to `GET /api/v1/workflows/{workflow_id}/info.engine_ids`.

### User Profile

- `GET /api/v1/users/me`
- `PATCH /api/v1/users/me`

### Auth

- `POST /api/v1/auth/register`
- `POST /api/v1/auth/login`
- `POST /api/v1/auth/forgot-password`
- `POST /api/v1/auth/reset-password`
- `POST /api/v1/auth/change-password`

### Readings History

- `GET /api/v1/readings`
- `GET /api/v1/readings/{reading_id}`
- `GET /api/v1/readings/stats`

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
