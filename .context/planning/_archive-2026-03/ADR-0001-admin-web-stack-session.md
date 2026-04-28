# ADR-0001: Admin Web Stack and Session Strategy

- Date: 2026-02-25
- Status: Accepted
- Related issue: #14

## Context

The admin portal must ship quickly while integrating with an existing Rust API that already supports:

- `POST /api/v1/auth/login` (JWT issuance)
- `Authorization: Bearer <token>` and `X-API-Key` auth middleware
- evolving admin permission model (`admin:*`)

The roadmap also sets a hosting split:

- Admin frontend on Vercel
- API/engine backend on Railway

## Decision

1. Frontend stack
- Use Next.js App Router + TypeScript in `apps/admin-web`.
- Configure `basePath = "/admin"` so routes are stable (`/admin/login`, `/admin/dashboard`, etc.).
- Keep UI scope to operator dashboard flows (keys, users, history sync, analytics, system, audit).

2. Hosting model
- Deploy `apps/admin-web` to Vercel.
- Keep `/api/v1/*` services on Railway.
- Configure frontend backend origin via `NEXT_PUBLIC_API_BASE_URL`.

3. Session strategy (Wave 1)
- Use bearer token session bootstrap:
  - Login via `POST /api/v1/auth/login`
  - Persist token client-side for session continuity
  - Validate/token-hydrate via `GET /api/v1/admin/session`
- Introduce `GET /api/v1/admin/session` as canonical session envelope for admin UI:
  - `user_id`, `email`, `tier`, `permissions`, `roles`, `has_admin_access`

4. Authorization model
- Use permission keys in `admin:<domain>:<action>` format.
- Frontend route guards are UX-only.
- Backend remains source of truth for access checks.

## Consequences

Positive:
- Fast implementation path with minimal backend disruption.
- Vercel deployment is isolated from Railway runtime rollout risk.
- Frontend can evolve independently while backend admin APIs are implemented wave-by-wave.

Tradeoffs:
- Client-side token persistence is transitional and should move to hardened session transport.
- Current login path may not yet emit full admin permissions until RBAC tables and role assignment are complete.

## Follow-up

- Implement RBAC-backed admin claims issuance.
- Add step-up auth for sensitive actions (key rotation/revoke, role mutation).
- Replace transitional session handling with hardened production session strategy once backend role management lands.
