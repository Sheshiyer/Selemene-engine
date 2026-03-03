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

---

# Task Plan — Admin Dashboard Login Incident Hotfix

## Checklist
- [x] Process GitHub issues/PR context to isolate likely incident vectors.
- [x] Confirm runtime route behavior for admin-web domains (`/` vs `/admin/login`).
- [x] Implement root-path redirect safety for Vercel deployment.
- [x] Implement explicit API base URL misconfiguration detection in frontend API client.
- [x] Improve protected layout error surfacing for non-401 session failures.
- [x] Update admin-web runbook docs for env misconfiguration behavior.
- [x] Run verification gates (`typecheck`, `lint`, `build`) for `apps/admin-web`.

## Notes
- Scope focused on production login UX and diagnosability, with minimal code/config touch.
- Avoid backend auth behavior changes without fresh incident evidence.

## Review (fill after execution)
- Added root redirect in `apps/admin-web/next.config.mjs` (`/` -> `/admin/login`, `basePath: false`) so Next.js serves it correctly on Vercel.
- Added runtime safety checks in `apps/admin-web/src/lib/api.ts`:
  - fail-fast `ADMIN_ENV_MISCONFIG` when deployed frontend points API base URL to localhost,
  - fail-fast `API_UNREACHABLE` with actionable error text for network/CORS failures.
- Updated protected session shell in `apps/admin-web/app/(protected)/layout.tsx` to surface actionable API error messages for non-401 failures.
- Updated `apps/admin-web/README.md` with root redirect and env-misconfig behavior note.
- Verification:
  - `npm --prefix apps/admin-web run typecheck`
  - `npm --prefix apps/admin-web run lint`
  - `npm --prefix apps/admin-web run build`
  - all passed.

---

# Task Plan — Admin Dashboard Parallel Hardening Pack

## Checklist
- [x] Add backend fallback for API key list/get queries when optional metadata columns are missing in DB.
- [x] Add backend fallback for API key create/rotate writes when optional metadata columns are missing in DB.
- [x] Add admin deployment smoke script for frontend+backend route sanity checks.
- [x] Refresh admin Vercel deployment runbook with current project/domain and root-directory guidance.
- [x] Run verification (`cargo check -p noesis-data`, `cargo check -p noesis-api`, script smoke checks).

## Notes
- Goal: complete high-impact reliability tasks that can land quickly in parallel tracks (backend resilience + deploy observability + docs).
- Scope intentionally avoids large feature work (audit/system modules, full RBAC redesign, billing integration).

## Review (fill after execution)
- Updated `crates/noesis-data/src/repositories/admin_repository.rs`:
  - Added graceful fallback paths for API key list/count/get/create/rotate when `api_keys.name` / `api_keys.key_prefix` columns are missing.
  - Added legacy query/insert helpers and `missing_api_keys_optional_columns()` detector (`Postgres 42703`) to avoid hard 500s on schema drift.
- Added `scripts/smoke_admin_web.sh`:
  - Validates `/admin/login` frontend reachability and backend `/health/live`, `/api/v1/admin/session` (401), `/api/v1/admin/api-keys` (401).
  - Requires explicit `ADMIN_WEB_URL` and `API_BASE_URL` inputs.
- Refreshed `docs/deployment/VERCEL_ADMIN_WEB.md`:
  - Corrected project/env examples, clarified root-directory modes for monorepo vs app-subdir deploys, and documented migration dependency for API keys metadata columns.
- Verification:
  - `cargo check -p noesis-data` ✅
  - `cargo check -p noesis-api` ✅
  - `ADMIN_WEB_URL=https://144.tryambakam.space API_BASE_URL=https://selemene-engine-production.up.railway.app bash scripts/smoke_admin_web.sh` ✅

---

# Task Plan — Admin System/Audit Completion + Post-Deploy Smoke CI

## Checklist
- [x] Implement backend admin system endpoints: `/api/v1/admin/system/health`, `/services`, `/workflows`, `/cache` with permission gating.
- [x] Implement backend audit endpoints: `/api/v1/admin/audit-events`, `/api/v1/admin/audit-events/{event_id}`, `/api/v1/admin/audit-events/actions` with filters/pagination.
- [x] Wire new admin routes + OpenAPI schemas for system/audit responses.
- [x] Add frontend typed contracts + API client methods for system/audit endpoints.
- [x] Replace `/admin/system` and `/admin/audit` placeholders with live API-driven views (loading/empty/error states).
- [x] Add GitHub Actions post-deploy smoke check step using `scripts/smoke_admin_web.sh` with env-driven URLs.
- [x] Run verification gates (`cargo fmt`, `cargo check -p noesis-data`, `cargo check -p noesis-api`, `npm --prefix apps/admin-web run typecheck`, `npm --prefix apps/admin-web run lint`, smoke script sanity).

## Notes
- Favor read-only operational visibility and forensic observability without introducing destructive admin actions.
- Reuse existing `usage_logs` as immutable event substrate for audit MVP.
- Keep CI smoke workflow env-driven and skip-safe when URLs are not configured.

## Review (fill after execution)
- Backend admin system + audit API implementation:
  - Added system endpoints in `crates/noesis-api/src/handlers/admin.rs`:
    - `GET /api/v1/admin/system/health`
    - `GET /api/v1/admin/system/services`
    - `GET /api/v1/admin/system/workflows`
    - `GET /api/v1/admin/system/cache`
  - Added audit endpoints:
    - `GET /api/v1/admin/audit-events`
    - `GET /api/v1/admin/audit-events/{event_id}`
    - `GET /api/v1/admin/audit-events/actions`
  - Added OpenAPI path/component wiring and route registration in `crates/noesis-api/src/lib.rs`.
- Data layer updates in `crates/noesis-data/src/repositories/admin_repository.rs`:
  - Added runtime `ping()` for DB service checks.
  - Added workflow snapshots (`system_workflow_snapshots`) for system workflows view.
  - Added audit query methods (`list/count/get events`, `list actions`) over immutable `usage_logs` substrate.
- Frontend wiring in `apps/admin-web`:
  - Added typed contracts in `src/types/admin.ts` for system/audit payloads.
  - Added API client methods in `src/lib/api.ts` for all new system/audit endpoints.
  - Replaced placeholders with live pages:
    - `app/(protected)/system/page.tsx`
    - `app/(protected)/audit/page.tsx`
- CI/CD post-deploy smoke integration:
  - Updated `.github/workflows/deploy.yaml` with `admin-smoke` job calling `scripts/smoke_admin_web.sh` when `ADMIN_WEB_URL` + `API_BASE_URL` variables are configured.
- Verification:
  - `cargo fmt --all` ✅
  - `cargo check -p noesis-data` ✅
  - `cargo check -p noesis-api` ✅
  - `npm --prefix apps/admin-web run typecheck` ✅
  - `npm --prefix apps/admin-web run lint` ✅
  - `npm --prefix apps/admin-web run build` ✅
  - `ADMIN_WEB_URL=https://144.tryambakam.space API_BASE_URL=https://selemene-engine-production.up.railway.app bash scripts/smoke_admin_web.sh` ✅
- Live production status (pre-deploy check): newly added endpoints currently return `404` on Railway and require backend deployment of this branch before UI pages can consume them.

---

# Task Plan — Admin UX P0 Parallel Fast-Track

## Checklist
- [x] Create GitHub issues before implementation for the P0 tracks.
- [x] Implement URL-synced state for primary admin list surfaces.
- [x] Implement auto-refresh + last-updated indicators for operational pages.
- [x] Implement copy/export QoL utilities on key admin tables.
- [x] Standardize severity/status visual mapping with shared helper.
- [x] Run verification (`typecheck`, `lint`, `build`, smoke script sanity).

## Notes
- Issues created first (as requested):
  - #482 URL-synced table state
  - #483 Auto-refresh + staleness indicator
  - #484 Copy/Export utilities
  - #485 Severity visual language standardization
  - #486 P0 umbrella tracker
- Scope focused on frontend/operator QoL, intentionally no new backend contracts in this pass.

## Review (fill after execution)
- Added reusable admin utilities:
  - `apps/admin-web/src/lib/url-query.ts`
  - `apps/admin-web/src/lib/export.ts`
  - `apps/admin-web/src/lib/status.ts`
- Applied URL-synced state:
  - `users` (`query`, `tier`, `state`)
  - `api-keys` (`query`, `tier`, `status`)
  - `audit` (`actor`, `action`, `result`, `from`, `to`, `refresh`)
  - `system` (`window_hours`, `refresh`)
  - plus refresh sync on `dashboard` + `analytics`
- Added auto-refresh + last-updated badges:
  - `dashboard`, `analytics`, `system`, `audit`
- Added copy/export QoL:
  - Users: copy user ID, export CSV/JSON
  - API Keys: copy key/user IDs, export CSV/JSON
  - Audit: copy actor/request/event IDs, export CSV/JSON
- Standardized status rendering with shared `statusPillClass()` helper:
  - `system`, `audit`, `history-sync`, `api-keys`, `users`
- Added small shared UI affordance:
  - `.link-btn` style in `apps/admin-web/app/globals.css`
- Verification:
  - `npm --prefix apps/admin-web run typecheck` ✅
  - `npm --prefix apps/admin-web run lint` ✅
  - `npm --prefix apps/admin-web run build` ✅
  - `ADMIN_WEB_URL=https://144.tryambakam.space API_BASE_URL=https://selemene-engine-production.up.railway.app bash scripts/smoke_admin_web.sh` ✅
- Deployment verification pass:
  - `https://144.tryambakam.space/admin/*` routes return expected 200/307 ✅
  - Railway `health/live` and existing admin session/api-keys routes return expected 200/401 ✅
  - Newly added `/api/v1/admin/system/*` and `/api/v1/admin/audit-events*` still return 404 in production (pending branch merge/deploy) ⚠️
- Issue lifecycle updates:
  - Closed as completed: `#482`, `#483`, `#484`, `#485`.
  - Kept umbrella `#486` open pending merge/deploy confirmation.

---

# Task Plan — v3.0.0 Launch Polish + Issue Status Alignment

## Checklist
- [x] Re-verify production admin endpoint availability for system/audit routes.
- [x] Draft launch-day operational runbook with step-by-step commands and rollback drill.
- [x] Draft launch-gate checklist with evidence placeholders + sign-off section.
- [x] Draft final release execution checklist (tag, registries, deploy, smoke).
- [x] Update launch issues `#486`, `#477`, `#478`, `#479` with evidence-backed status comments.
- [x] Close fully completed issue(s) and leave precise remaining actions on open issues.

## Notes
- Scope limited to launch polish/docs + GitHub issue hygiene for the v3.0.0 launch track.
- Do not force-close issues lacking objective evidence for acceptance criteria.

## Review (fill after execution)
- Production re-verification completed:
  - `https://selemene-engine-production.up.railway.app/health/live` -> `200`
  - `https://selemene-engine-production.up.railway.app/api/v1/admin/session` -> `401`
  - `https://selemene-engine-production.up.railway.app/api/v1/admin/api-keys` -> `401`
  - `https://selemene-engine-production.up.railway.app/api/v1/admin/system/*` -> `401` (not `404`)
  - `https://selemene-engine-production.up.railway.app/api/v1/admin/audit-events*` -> `401` (not `404`)
  - Admin web smoke script passed against production URLs.
- Added launch polish docs:
  - `docs/launch/v3.0.0-launch-day-runbook.md`
  - `docs/launch/v3.0.0-launch-gate-checklist.md`
  - `docs/launch/v3.0.0-release-execution-checklist.md`
- Updated GitHub issues with status comments:
  - `#477` runbook readiness + remaining close conditions
  - `#478` launch gate evidence/sign-off status
  - `#479` release execution sequencing + remaining evidence
- Closed completed issue:
  - `#486` (admin production blocker resolved)
- Release preflight blockers documented on launch issues:
  - Production `/health/live` currently returns `version: 0.1.0` (not `3.0.0`)
  - Highest git tag present is `v2.4.0` (no `v3.0.0` tag yet)
  - Launch gate evidence/sign-off remains incomplete for `#478`

---

# Task Plan — v3.0.0 Version Alignment Pass

## Checklist
- [x] Align Rust workspace + crate package versions from `0.1.0` to `3.0.0`.
- [x] Align API-reported runtime/openapi version strings to `3.0.0`.
- [x] Align JS package metadata versions (`apps/admin-web`, `bridges/cli`) to `3.0.0`.
- [x] Align Python service versions and health/openapi test expectations to `3.0.0`.
- [x] Run targeted verification across Rust, Python, and admin-web typecheck.

## Notes
- Scope is version consistency only; no release tag/publish/deploy executed in this pass.
- `@selemene/sdk` package artifact is still not present in-repo and remains a separate release-track item.

## Review (fill after execution)
- Updated version metadata to `3.0.0` across:
  - `Cargo.toml` workspace and all crate package manifests
  - `crates/noesis-api` OpenAPI info + health endpoint version value
  - `apps/admin-web/package.json` + lockfile top-level package version
  - `bridges/cli/package.json` + CLI self-reported version
  - `python-services` pyproject + service health/openapi/test version values
  - `Dockerfile.prod` OCI image version label
- Verification:
  - `cargo check -p noesis-api` ✅
  - `pytest -q tests/test_biofield_health.py tests/test_mediapipe_health.py` ✅
  - `npm run typecheck` (apps/admin-web) ✅

---

# Task Plan — Merge + Deploy + Release Tag Execution

## Checklist
- [x] Merge version-alignment PR into `main`.
- [x] Verify production deploy result and runtime version.
- [x] Push release tag `v3.0.0`.
- [x] Verify GitHub release publication.
- [x] Re-run production admin smoke checks.
- [x] Update launch issues with current evidence and blockers.

## Notes
- Kept launch issues open where acceptance criteria still require external registry evidence/sign-offs.

## Review (fill after execution)
- Merged PR `#488` into `main`.
- Pushed tag `v3.0.0` and confirmed GitHub release published.
- Production health now returns `version: 3.0.0` on `/health/live`.
- Admin smoke checks passed against production URLs.
- Confirmed unresolved registry blockers:
  - crates `noesis-core` and `noesis-sdk` not found on crates.io
  - npm package `@selemene/sdk` not found on npm
- Posted status/evidence updates to issues `#477`, `#478`, and `#479`.

---

# Task Plan — Crates Publish Commanding + `@selemene/sdk` Package Target

## Checklist
- [x] Prepare exact command order for crates.io + npm publishing.
- [x] Update crate metadata needed for crates.io readiness (`noesis-core`, `noesis-sdk`).
- [x] Add crate README files for publish metadata completeness.
- [x] Locate/scaffold intended `@selemene/sdk` package target in repo.
- [x] Verify new package builds/typechecks and validate publish preconditions.

## Notes
- `noesis-sdk` dry-run requires `noesis-core@3.0.0` to be indexed first; this is now explicitly encoded in command order.

## Review (fill after execution)
- Added publish command runbook: `docs/launch/v3.0.0-registry-publish-commands.md`
- Updated workspace/crate metadata for crates publication readiness:
  - `Cargo.toml` workspace package now includes repository/homepage
  - `crates/noesis-core/Cargo.toml` includes readme/keywords/categories + workspace metadata
  - `crates/noesis-sdk/Cargo.toml` includes readme/keywords/categories + `noesis-core` versioned dependency
- Added crate README files:
  - `crates/noesis-core/README.md`
  - `crates/noesis-sdk/README.md`
- Scaffolded npm package target at:
  - `packages/noesis-sdk-ts` (published name `@selemene/sdk`)
  - includes `package.json`, tsconfig, typed `NoesisClient`, README
- Verification:
  - `cargo publish --dry-run --allow-dirty -p noesis-core` ✅
  - `cargo publish --dry-run --allow-dirty -p noesis-sdk` ⛔ (expected until `noesis-core` is actually published/indexed)
  - `npm run typecheck` + `npm run build` in `packages/noesis-sdk-ts` ✅

---

# Task Plan — Wave W2 Usage Dashboard API Endpoints (#456)

## Checklist
- [x] Add `GET /api/v1/users/me/usage` endpoint with authenticated self-service usage summary.
- [x] Add `GET /api/v1/admin/usage/summary` endpoint with admin-gated global usage summary.
- [x] Extend usage repository with daily/monthly aggregates, engine breakdown, and top-users queries.
- [x] Wire routes and OpenAPI path/schema registrations.
- [x] Add integration tests for auth and response shape validation.
- [x] Run verification gates for noesis-api.

## Notes
- Kept scope strictly to issue #456 deliverable and acceptance criteria.
- Reused existing admin permission model (`admin:analytics:read`) for admin endpoint gating.

## Review (fill after execution)
- Implemented user endpoint in `crates/noesis-api/src/handlers/users.rs`:
  - `GET /api/v1/users/me/usage`
  - returns daily/monthly usage counts and per-engine breakdown.
- Implemented admin endpoint in `crates/noesis-api/src/handlers/admin.rs`:
  - `GET /api/v1/admin/usage/summary`
  - returns daily/monthly platform usage, engine breakdown, and top users.
- Added repository methods in `crates/noesis-data/src/repositories/usage_repository.rs`:
  - `user_usage_summary`
  - `user_engine_breakdown`
  - `admin_usage_summary`
  - `admin_engine_breakdown`
  - `admin_top_users`
- Wired OpenAPI and routes in `crates/noesis-api/src/lib.rs`.
- Added integration tests in `crates/noesis-api/tests/usage_analytics_tests.rs`.
- Verification:
  - `cargo fmt --all` ✅
  - `cargo check -p noesis-api` ✅
  - `cargo test -p noesis-api --test usage_analytics_tests -- --nocapture` ✅

---

# Task Plan — Wave W2 Admin Usage Analytics Dashboard (#458)

## Checklist
- [x] Extend usage summary API payload for dashboard data (`daily_requests`, `tier_distribution`).
- [x] Add date-range query support (`range_days`) to `/api/v1/admin/usage/summary`.
- [x] Wire frontend API client/types for new usage summary contract.
- [x] Update admin analytics page to use `/api/v1/admin/usage/summary`.
- [x] Add daily request chart, engine popularity pie chart, tier distribution pie chart, top 10 users table.
- [x] Add date range selector and hover-enabled chart interactions.
- [x] Run backend and frontend verification gates.

## Notes
- Kept existing analytics endpoints untouched for backward compatibility.
- Dashboard now uses usage-summary endpoint as primary source for usage views.

## Review (fill after execution)
- Backend (`noesis-api` + `noesis-data`):
  - Extended admin usage response with:
    - `daily_requests` (time series)
    - `tier_distribution`
  - Added `range_days` query support on `/api/v1/admin/usage/summary`
  - Added repository methods:
    - `admin_daily_series`
    - `admin_tier_distribution`
  - Updated OpenAPI schema registration for new response entities.
- Frontend (`apps/admin-web`):
  - Added new API client method: `getAdminUsageSummary`
  - Added types: `AdminUsageSummaryResponse` and nested usage chart/table types
  - Reworked `/analytics` page to include:
    - daily requests bar chart
    - engine popularity pie chart
    - tier distribution pie chart
    - top 10 users table
    - date range selector + auto-refresh
- Verification:
  - `cargo fmt --all` ✅
  - `cargo check -p noesis-api` ✅
  - `cargo test -p noesis-api --test usage_analytics_tests -- --nocapture` ✅
  - `npm --prefix apps/admin-web run typecheck` ✅
  - `npm --prefix apps/admin-web run lint` ✅
  - `npm --prefix apps/admin-web run build` ✅

---

# Task Plan — Wave W2 Billing Event Hooks (#457)

## Checklist
- [x] Add `BillingEventEmitter` trait with required methods.
- [x] Add `NoopBillingEmitter` default that logs events at debug level.
- [x] Add `StripeWebhookEmitter` stub with JSON payload formatting helpers.
- [x] Emit usage billing events on each engine/workflow calculation attempt.
- [x] Emit quota-exceeded billing events on authenticated 429 rate-limit responses.
- [x] Add tests for Stripe payload format and billing hook emission wiring.
- [x] Run verification commands.

## Notes
- Billing hooks are non-blocking instrumentation only (no payment side effects).
- Existing API behavior remains unchanged; events are emitted in parallel to normal flow.

## Review (fill after execution)
- Added new module `crates/noesis-api/src/billing.rs`:
  - `BillingEventEmitter`
  - `NoopBillingEmitter`
  - `StripeWebhookEmitter`
  - global emitter setters and emit helpers
- Integrated usage event calls in:
  - `calculate_handler`
  - `workflow_execute_handler`
- Integrated quota-exceeded event calls in:
  - minute-limit 429 path in `rate_limit_middleware`
  - daily-limit 429 path in `rate_limit_middleware`
- Added integration test file:
  - `crates/noesis-api/tests/billing_hooks_tests.rs`
    - usage event emitted on calculation
    - quota event emitted on rate-limit exceed
    - Stripe payload format assertions
- Verification:
  - `cargo fmt --all` ✅
  - `cargo check -p noesis-api` ✅
  - `cargo check -p noesis-api --tests` ✅
  - `cargo test -p noesis-api --test billing_hooks_tests -- --nocapture` ⚠ blocked locally by Xcode license requirement on linker (`xcodebuild -license`)

---

# Task Plan — Wave W2 Per-Engine OpenAPI Schemas (#447)

## Checklist
- [x] Add 16 dedicated per-engine result schema structs with documented fields/examples.
- [x] Add OpenAPI union schema for `EngineOutput.result`.
- [x] Wire engine schema types into API OpenAPI component list.
- [x] Add OpenAPI regression tests for schema presence and result-field linkage.
- [x] Run compile/verification commands.

## Notes
- Kept runtime response shape unchanged (`result: serde_json::Value`), while improving OpenAPI typing via `schema(value_type = EngineResultData)`.
- Added explicit schema registration so `/api/openapi.json` reliably includes all 16 per-engine schemas.

## Review (fill after execution)
- Updated `crates/noesis-core/src/types.rs`:
  - Added 16 per-engine schema structs:
    - `PanchangaResultSchema`, `NumerologyResultSchema`, `BiorhythmResultSchema`, `HumanDesignResultSchema`, `GeneKeysResultSchema`, `VimshottariResultSchema`, `BiofieldResultSchema`, `VedicClockResultSchema`, `FaceReadingResultSchema`, `NadabrahmanResultSchema`, `TransitsResultSchema`, `EnneagramResultSchema`, `TarotResultSchema`, `IChingResultSchema`, `SacredGeometryResultSchema`, `SigilForgeResultSchema`
  - Added `EngineResultData` (untagged union enum)
  - Annotated `EngineOutput.result` with `#[schema(value_type = EngineResultData)]`
- Updated `crates/noesis-api/src/lib.rs`:
  - Imported and registered all engine schema types + `EngineResultData` in OpenAPI components.
- Added test file `crates/noesis-api/tests/openapi_schema_tests.rs`:
  - asserts 16 per-engine schemas exist in `/api/openapi.json`
  - asserts each has at least 3 documented fields
  - asserts `EngineOutput.result` references `EngineResultData`
- Verification:
  - `cargo fmt --all` ✅
  - `cargo check -p noesis-core --features openapi` ✅
  - `cargo check -p noesis-api` ✅
  - `cargo check -p noesis-api --tests` ✅
