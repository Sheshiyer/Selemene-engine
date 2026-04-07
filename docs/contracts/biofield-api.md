# Biofield API Contract

Date: 2026-04-05
Status: frozen for Phase 1 implementation

## Purpose

This contract defines the public Noesis API surface for the web-first biofield product.

This is the only browser-visible backend contract. The Python sidecar remains private and is called only by Noesis API.

## Authentication

All biofield routes are authenticated.

Accepted auth modes:

- bearer JWT
- existing supported Noesis API-key paths only if explicitly enabled later

Phase 1 assumption:

- user-facing biofield-web uses bearer JWT

All read and write operations are user-scoped.

## Route Namespace

Frozen namespace:

- `POST /api/v1/biofield/sessions`
- `POST /api/v1/biofield/sessions/:session_id/close`
- `GET /api/v1/biofield/sessions/:session_id`
- `POST /api/v1/biofield/sessions/:session_id/captures`
- `GET /api/v1/biofield/readings`
- `GET /api/v1/biofield/readings/:reading_id`
- `POST /api/v1/biofield/readings/:reading_id/reprocess`
- `GET /api/v1/biofield/baselines`
- `POST /api/v1/biofield/baselines`
- `POST /api/v1/biofield/exports`

Phase 1 shipping routes:

- `POST /api/v1/biofield/sessions`
- `POST /api/v1/biofield/sessions/:session_id/close`
- `GET /api/v1/biofield/sessions/:session_id`
- `POST /api/v1/biofield/sessions/:session_id/captures`
- `GET /api/v1/biofield/readings`
- `GET /api/v1/biofield/readings/:reading_id`

## Resource Shapes

### Session

```json
{
  "id": "uuid",
  "status": "active",
  "started_at": "2026-04-05T12:00:00Z",
  "closed_at": null,
  "client_device_id": "optional-string",
  "viewer_version": "optional-string"
}
```

### Reading Summary

```json
{
  "reading_id": "uuid",
  "session_id": "uuid",
  "engine_id": "biofield-capture",
  "created_at": "2026-04-05T12:05:00Z",
  "quality": {
    "sufficient_quality": true
  },
  "artifact": {
    "kind": "source-image",
    "mime_type": "image/jpeg"
  }
}
```

### Reading Detail

```json
{
  "reading_id": "uuid",
  "session_id": "uuid",
  "engine_id": "biofield-capture",
  "created_at": "2026-04-05T12:05:00Z",
  "input": {},
  "result": {},
  "quality": {},
  "artifacts": []
}
```

## Endpoint Contract Details

### `POST /api/v1/biofield/sessions`

Purpose:

- create a new active session for the authenticated user

Request body:

```json
{
  "client_device_id": "optional-string",
  "viewer_version": "optional-string",
  "context": {
    "platform": "optional-string"
  }
}
```

Response:

- `201 Created`
- returns session resource

### `POST /api/v1/biofield/sessions/:session_id/close`

Purpose:

- explicitly close an active session

Request body:

```json
{
  "reason": "optional-string"
}
```

Response:

- `200 OK`
- returns updated session resource

### `GET /api/v1/biofield/sessions/:session_id`

Purpose:

- fetch one user-owned session

Response:

- `200 OK`
- returns session resource

### `POST /api/v1/biofield/sessions/:session_id/captures`

Purpose:

- upload a capture image and request deeper analysis

Transport:

- `multipart/form-data`

Required multipart fields:

- `image` or `file`

Optional form fields:

- `algorithms`
- `options`
- `capture_metadata`

Success response:

- `201 Created`
- returns:
  - persisted reading ID
  - normalized analysis payload
  - artifact metadata

Failure classes:

- `400` invalid multipart or malformed JSON options
- `401` unauthenticated
- `403` cross-user access or forbidden scope
- `404` session not found
- `409` session not active
- `413` payload too large
- `422` invalid image or quality rejection
- `502` or `503` sidecar unavailable

### `GET /api/v1/biofield/readings`

Purpose:

- list user-owned persisted biofield capture readings

Supported query params:

- `limit`
- `offset`
- `session_id` optional later

Response:

- `200 OK`
- returns paginated reading summaries

### `GET /api/v1/biofield/readings/:reading_id`

Purpose:

- fetch one persisted biofield reading with detail payload

Response:

- `200 OK`
- returns reading detail

### Deferred But Frozen

`POST /api/v1/biofield/readings/:reading_id/reprocess`

- explicit rerun on stored capture artifact

`GET/POST /api/v1/biofield/baselines`

- baseline lifecycle

`POST /api/v1/biofield/exports`

- export request initiation

## Persistence Mapping

- successful capture analysis must create:
  - one `readings` row with `engine_id = "biofield-capture"`
  - one or more linked `biofield_capture_artifacts` rows
- session lifecycle is tracked separately in `biofield_sessions`

## Error Envelope

Biofield routes should use the standard Noesis error envelope.

Required machine-readable error codes for Phase 1:

- `BIOFIELD_SESSION_NOT_FOUND`
- `BIOFIELD_SESSION_NOT_ACTIVE`
- `BIOFIELD_CAPTURE_INVALID`
- `BIOFIELD_CAPTURE_TOO_LARGE`
- `BIOFIELD_CAPTURE_REJECTED_QUALITY`
- `BIOFIELD_ANALYSIS_UNAVAILABLE`

## Stable Invariants

- browser cannot access the sidecar directly
- session ownership is always enforced server-side
- persisted reading IDs are the canonical references used by history and detail routes
- Phase 1 routes may expand fields but must not rename the route namespace or core resource nouns
