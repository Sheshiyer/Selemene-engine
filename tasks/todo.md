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

---

# Task Plan — Admin Dashboard Wave 2 Live Admin Operations

## Checklist
- [x] Add backend admin APIs for:
  - [x] users list + status/tier updates
  - [x] API keys list/create/revoke/rotate
  - [x] history sync users/devices/events projections
  - [x] analytics summary/timeseries/breakdown/top-consumers for dashboard charts
- [x] Wire backend routes/OpenAPI/CORS/AppState dependencies for the new admin endpoints.
- [x] Add backend tests for all new admin handlers and critical repository queries.
- [x] Replace frontend placeholders on `/admin/users`, `/admin/api-keys`, `/admin/history-sync`, `/admin/analytics`, and `/admin/dashboard` with live API-driven tables/charts.
- [x] Add frontend typed API contracts for all new admin endpoints.
- [x] Run verification gates:
  - [x] `cargo fmt --all`
  - [x] `cargo test -p noesis-api --lib -- --nocapture`
  - [x] `npm --prefix apps/admin-web run typecheck`
  - [x] `npm --prefix apps/admin-web run lint`
  - [x] `npm --prefix apps/admin-web run build`
- [x] Configure/administer Vercel for `apps/admin-web` with production envs targeting Railway API.
- [x] Run deployed smoke tests on Vercel against Railway API and capture endpoint/UI proof.

## Notes
- Roles/permissions persistence will use `api_keys.permissions` as current source of truth in this wave unless a dedicated user-roles schema is introduced.
- History sync views will be derived from current `usage_logs` + `readings` schema in this wave.
- Keep scope to merge-ready, non-breaking changes for the existing API surface.

## Review (fill after execution)
- Added backend data layer:
  - New `AdminRepository` at `crates/noesis-data/src/repositories/admin_repository.rs`
  - Exported via `crates/noesis-data/src/repositories/mod.rs`
  - Covers users, key lifecycle, history-sync projections, analytics summaries/timeseries/breakdowns/top-consumers
- Expanded API surface:
  - Implemented full `/api/v1/admin/*` handlers in `crates/noesis-api/src/handlers/admin.rs`
  - Wired routes/OpenAPI/AppState in `crates/noesis-api/src/lib.rs`
  - Enabled CORS methods for `PATCH` and `PUT`
  - Added `admin_repository` to app state (prod and lazy builders)
- Frontend live integration:
  - Replaced placeholders on users/api-keys/history-sync/analytics/dashboard pages
  - Added typed API contracts and client methods in `apps/admin-web/src/types/admin.ts` + `apps/admin-web/src/lib/api.ts`
  - Added charting (`recharts`) and table/action UI styles
- Deployment:
  - Created + linked Vercel project `sheshiyers-projects/selemene-admin-dashboard`
  - Added env vars (`NEXT_PUBLIC_API_BASE_URL`, `NEXT_PUBLIC_ADMIN_DEV_MODE`) for production/preview/development
  - Added `apps/admin-web/vercel.json` to pin Next.js framework build
  - Production alias live at `https://selemene-admin-dashboard.vercel.app`
- Smoke validation:
  - Frontend deploy routes return `200` (`/admin/login`, `/admin/dashboard`)
  - Railway health endpoint returns `200`
  - New Railway admin endpoints currently return `404` (backend deployment to Railway still pending)
  - Railway CORS still reports `GET,POST,OPTIONS` on live service (new CORS settings not yet deployed)

---

# Task Plan — Issue #15 Migration Reconciliation

## Checklist
- [x] Replace stale Supabase example/partial migrations with canonical-aligned migration set.
- [x] Ensure Supabase migration order mirrors root `migrations/001..006`.
- [x] Add a migration drift verification script for root-vs-supabase parity.
- [x] Add/update minimal docs for the reconciliation workflow.
- [x] Run verification (`bash scripts/check_migration_drift.sh`) and capture results.
- [x] Prepare PR skeleton linked to issue `#15`.

## Notes
- Scope is issue `#15` only (Wave 1 data foundation migration reconciliation).
- Keep code changes minimal and non-destructive outside migration assets and docs.

## Review (fill after execution)
- Replaced non-canonical Supabase migration files:
  - removed `20260208000001_example_users_table.sql`
  - replaced legacy `20260210220000_readings_table.sql`
  - added canonical mirrors for root `001..006` with timestamped Supabase names
- Added `scripts/check_migration_drift.sh`:
  - enforces one-to-one root-to-supabase mirror mapping
  - validates content parity via SHA-256 checks
  - fails on missing/multiple/mismatched/extra files
- Added `supabase/README.md` documenting canonical mirror policy and drift-check command.
- Verification:
  - `bash scripts/check_migration_drift.sh` passed.
