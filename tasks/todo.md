# Task Plan — Admin Dashboard Wave 1 Bootstrap

## Checklist
- [x] Create `apps/admin-web` Next.js scaffold configured for Vercel deployment.
- [x] Implement admin route shell and placeholders for documented pages (`/admin/*` via `basePath`).
- [x] Implement login/session flow using current backend auth (`/api/v1/auth/login`) and token storage.
- [x] Add minimal backend endpoint `GET /api/v1/admin/session` for frontend session introspection.
- [x] Wire frontend API client + route guards to session permissions.
- [x] Add local run/deploy docs for admin dashboard (`apps/admin-web/README.md`).
- [x] Run targeted verification (`cargo test -p noesis-api` subset + frontend type/lint checks).

## Notes
- Scope is Wave 1 foundation only; no full admin CRUD implementation in this pass.
- Keep existing backend behavior stable; avoid broad auth refactors.

## Review (fill after execution)
- Implemented app scaffold at `apps/admin-web`:
  - Next.js 16 with `/admin` `basePath`
  - Protected route shell for `dashboard`, `users`, `api-keys`, `history-sync`, `analytics`, `system`, `audit`
  - Login page wired to `POST /api/v1/auth/login`
  - Session check wired to new `GET /api/v1/admin/session`
  - Permission guards with legacy alias compatibility (`admin:users`, `admin:analytics`)
  - Vercel runbook at `apps/admin-web/README.md`
- Added backend endpoint:
  - `crates/noesis-api/src/handlers/admin.rs`
  - Route registration at `/api/v1/admin/session`
  - OpenAPI path/component/tag integration
  - Unit tests for role derivation
- Updated root docs:
  - Production stack table now explicitly lists Vercel for admin frontend
  - Endpoint list includes `/api/v1/admin/session`
  - Added ADR: `docs/planning/ADR-0001-admin-web-stack-session.md`
- Verification:
  - `cargo fmt --all`
  - `cargo test -p noesis-api admin::tests -- --nocapture`
  - `cargo test -p noesis-api --lib -- --nocapture`
  - `npm --prefix apps/admin-web run typecheck`
  - `npm --prefix apps/admin-web run lint`
  - `npm --prefix apps/admin-web run build`
