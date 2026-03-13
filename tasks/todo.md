# Task Plan — Admin Dashboard Wave 1 Bootstrap

---

# Task Plan — noesis-api Error Handling Audit for Issue #49

## Checklist
- [x] Load relevant workflow context and existing task/lesson notes.
- [x] Inspect `crates/noesis-api/src/lib.rs` for shared API error helpers and response types.
- [x] Inspect `crates/noesis-api/src/handlers/*.rs` and classify each handler by error-handling pattern.
- [x] Record which handlers use `engine_error_to_response` / `ApiError` versus ad-hoc `StatusCode + Json(ErrorResponse)`.
- [x] Conclude whether issue `#49` can close via documentation only or still requires code.

## Notes
- Scope is audit only for handler error handling patterns in `noesis-api`.
- Do not modify API behavior in this pass unless the inspection reveals a trivial factual correction is required for the audit itself.

## Review (fill after execution)
- `crates/noesis-api/src/handlers/auth.rs`, `users.rs`, and `admin.rs` are consistent on the standardized path:
  - all public handlers return `Result<Response, ApiError>`
  - `ApiError` delegates to `engine_error_to_response`
- `crates/noesis-api/src/lib.rs` is mixed:
  - direct `engine_error_to_response` use in:
    - `calculate_handler`
    - `workflow_execute_handler`
    - `legacy_panchanga_handler`
    - `legacy_ghati_current_handler`
  - ad-hoc `(StatusCode, Json(ErrorResponse))` construction in:
    - `validate_handler`
    - `engine_info_handler`
    - `workflow_info_handler`
    - `list_readings_handler`
    - `get_reading_handler`
    - `readings_stats_handler`
  - special non-`ErrorResponse` outlier:
    - `metrics_handler` returns plain-text `500`
- Audit conclusion:
  - issue `#49` is not documentation-only if the intended outcome is consistent handler error handling across `noesis-api`
  - the inconsistency is real in current code, concentrated in `lib.rs`
  - documentation can describe the current split, but closing the issue cleanly still requires code to standardize the remaining ad-hoc handlers

---

# Task Plan — Astrology Hygiene Verification Audit

## Checklist
- [x] Review current hygiene-related docs and task/lesson context.
- [ ] Verify whether FreeAstrologyAPI provider wiring is actually active in runtime code paths.
- [ ] Verify whether the claimed engine fixes/tests exist in the repo and match the transcript.
- [ ] Run fresh targeted verification commands for the affected engines/API tests.
- [ ] Record confirmed findings, contradictions, and remaining gaps in the review section.

## Notes
- Scope is verification only for the current hygiene changes and provider/runtime behavior.
- Do not mutate engine logic in this pass unless verification reveals a safe, necessary correction.

## Review (fill after execution)
- Trusted screenshot reference for `1991-08-13 13:31 Bangalore` shows:
  - `Tithi = Chaturthi upto 10:04 PM`
  - `Nakshatra = Uttara Phalguni upto 01:52 PM`
  - `Nakshatra Pada = 4` at the birth moment (`pada 3` ended at `08:05 AM`)
  - `Yoga = Siddha upto 07:48 PM`
  - `Karana = Vishti upto 10:04 PM` (`Vanija` ended at `10:48 AM`)
  - `Weekday = Mangalavara`
  - `Paksha = Shukla Paksha`
  - `Moon sign = Kanya`
  - `Sun sign = Karka`
  - `Lahiri Ayanamsha = 23.746632`
- Native Selemene Panchanga at the same moment returned:
  - `tithi_name = Chaturthi (Shukla)`
  - `nakshatra_name = Uttara Phalguni`
  - `nakshatra_value = 11.987291238292826` -> `pada 4`
  - `yoga_name = Siddha`
  - `karana_name = Vishti`
  - `vara_name = Mangalavara (Tuesday)`
  - `solar_longitude = 116.37350004069478` -> `Karka`
  - `lunar_longitude = 159.83054984390435` -> `Kanya`
  - `julian_day = 2448481.834027778`
- Comparison verdict:
  - native Panchanga matches the trusted screenshot on every high-signal birth field we extracted:
    - `tithi`
    - `nakshatra`
    - `nakshatra pada`
    - `yoga`
    - `karana`
    - `vara`
    - `moon sign`
    - `sun sign`
  - we added regression coverage for this exact birth moment in `engine-panchanga`.
- Verification:
  - `cargo test -p engine-panchanga test_birth_reference_1991_08_13_bangalore_matches_trusted_screenshot -- --nocapture`
  - `cargo test -p engine-panchanga -- --nocapture`
- Residual nuance:
  - the engine does not currently expose ayanamsha as an output field.
  - internal sidereal conversion still uses a simplified Lahiri constant (`23.72`) rather than the screenshot's `23.746632`.
  - despite that, the birth-time Panchanga classifications match the trusted reference for this canonical input.

---

# Task Plan — Panchanga Provider Attribution Check

## Checklist
- [x] Verify which runtime engines are currently routed through FreeAstrologyAPI.
- [x] Capture raw FreeAstrologyAPI Panchanga response behavior for the canonical birth input.
- [x] Capture Selemene `/api/v1/engines/panchanga/calculate` response for the same input.
- [x] Capture native Panchanga response for the same input.
- [x] Compare provider behavior vs mapped API response vs native response and isolate the divergence point.
- [x] Record conclusion and recommended next action in the review section.

## Notes
- Scope is surgical: provider attribution and correctness for Panchanga first.
- Do not change engine math or broaden provider wiring in this pass.

## Review (fill after execution)
- Canonical input used:
  - `1991-08-13`
  - `13:31`
  - `12.9340, 77.6214`
  - `Asia/Kolkata`
- Direct live provider probes:
  - Old configured repo host `https://json.freeastrologyapi.com/panchang` with `x-api-key` returned `403 MissingAuthenticationToken`.
  - Current public host `https://astro-api-1qnc.onrender.com/api/v1/vedic/panchang` with `Authorization: Bearer ...` returned `401 Missing x-api-key header`.
  - Current public host with `x-api-key` returned `403 Invalid API Key` for the supplied key.
- Selemene API route in `VEDIC_ENGINE_PROVIDER=api` mode returned:
  - `_selemene_execution.route = fallback`
  - `provider_attempted = true`
  - `fallback_used = true`
  - `backend = native-rust`
  - Panchanga values:
    - `tithi_name = Tritiya (Shukla)`
    - `nakshatra_name = Uttara Phalguni`
    - `yoga_name = Siddha`
    - `karana_name = Balava`
    - `vara_name = Mangalavara (Tuesday)`
    - `solar_longitude = 117.54499786675959`
    - `lunar_longitude = 153.10711914058018`
    - `julian_day = 2448481.834027778`
- Selemene API route in `VEDIC_ENGINE_PROVIDER=native` mode returned the exact same Panchanga values, confirming the `api`-mode result came from native fallback rather than provider mapping.
- Divergence point isolated:
  - the failure is upstream of astrology mapping.
  - initial `noesis-vedic-api` client used Bearer auth and the wrong Panchanga path.
  - patched client now uses `x-api-key` and `complete-panchang`, with contract coverage:
    - `cargo test -p noesis-vedic-api --test client_tests test_get_panchang_uses_x_api_key_header_and_complete_endpoint -- --nocapture`
    - `cargo test -p noesis-api --test vedic_provider_route_tests -- --nocapture`
  - after the client contract fix, direct raw provider call still returns `403 AccessDenied` on `https://json.freeastrologyapi.com/complete-panchang` for the supplied key.
  - Selemene `api` mode with fallback disabled now returns `500 BRIDGE_ERROR`, which correctly surfaces the provider failure instead of silently producing astrology output.
  - Selemene `api` mode with fallback enabled returns the same native Panchanga output as `native` mode, confirming live provider mapping still does not execute because provider authorization blocks first.
- Recommended next action:
  - provider contract is now corrected for Panchanga.
  - next unblocker is provider authorization: verify the active API key/account entitlement for `complete-panchang`, or switch to a provider/endpoint that will return a live Panchanga payload for comparison.
  - additional entitlement probes against official docs endpoints all failed from this environment with the supplied key:
    - `POST /complete-panchang` -> `403 AccessDenied`
    - `POST /tithi-durations` -> `403 Forbidden`
    - `POST /nakshatra-durations` -> `403 Forbidden`
    - `POST /yoga-durations` -> `403 Forbidden`
    - `POST /geo-details` -> `403 Forbidden`
    - `POST /aayanam` -> `403 Forbidden`
  - control probes on `geo-details` with no key and with a random key also returned `403 Forbidden`, so the provider is not giving a usable success path or a differentiating auth error from this environment.
  - viability conclusion: FreeAstrologyAPI is not currently usable as a live Panchanga source in this setup until key/account access is repaired.

---

# Task Plan — Panchanga Native Baseline + Replacement Provider Comparison

## Checklist
- [ ] Lock the canonical Panchanga fixture for:
  - `1991-08-13`
  - `13:31`
  - `12.9340, 77.6214`
  - `Asia/Kolkata`
- [ ] Capture native Selemene Panchanga output for the canonical fixture.
- [ ] Capture a trusted external reference Panchanga result for the same fixture.
- [ ] Compare native output vs trusted reference on:
  - `tithi`
  - `nakshatra`
  - `yoga`
  - `karana`
  - `vara`
  - `solar_longitude`
  - `lunar_longitude`
  - `julian_day` if available
- [ ] Decide whether native Panchanga is trustworthy enough to serve as the interim source of truth.
- [ ] Only after native validation, test replacement provider candidates against the same canonical fixture.
- [ ] Record the comparison matrix and decision in the review section.

## Notes
- Freeze FreeAstrologyAPI for Panchanga investigation until provider access is repaired.
- Native engine validation comes before any replacement provider integration work.
- Keep this wave evidence-driven and fixture-based.

## Review (fill after execution)
- External reference used in this pass:
  - pasted Drik-style Bengaluru day Panchang for `March 10, 2026`
  - daily context anchored around sunrise `06:30`
  - updated screenshot clarified that `12:21 PM` refers to the `Anuradha` **pada** transition, while core nakshatra remains `Anuradha` until `07:05 PM`
- Native Selemene checks run at:
  - `2026-03-10 06:30 Asia/Kolkata`
  - `2026-03-10 12:20/12:21 Asia/Kolkata`
  - Bengaluru center and user coordinates produced effectively the same result for core fields
- Native engine matches the reference on coarse fields:
  - `vara = Mangalavara (Tuesday)`
  - `nakshatra = Anuradha` at day start
  - `nakshatra = Anuradha` still valid at `12:21 PM`
  - `lunar_longitude = 220.7894` at sunrise -> moon in `Vrishchika`, `Anuradha`, roughly pada 3
  - `lunar_longitude = 224.0012` at `12:21 PM` -> moon still in `Anuradha`, now roughly pada 4
  - `solar_longitude = 324.0117` at sunrise -> sun in `Kumbha`
  - `tithi = Saptami (Krishna)` matches the screenshot's `Saptami upto 01:54 AM, Mar 11`
  - `yoga` progression also matches:
    - sunrise: native `Harshana`
    - noon: native `Vajra`
    - screenshot: `Harshana upto 08:21 AM`, then `Vajra`
- Root cause isolated and fixed in this pass:
  - `engine-panchanga` used a placeholder modulo-based karana mapping instead of the real 60 half-tithi cycle.
  - `engine-panchanga` also used rough mean solar/lunar longitudes, which flipped the March 10, 2026 Bengaluru karana transition too early.
  - surgical fix applied:
    - replaced karana derivation with the standard 60-slot half-tithi sequence
    - changed `compute_panchanga()` to prefer local Swiss Ephemeris Sun/Moon positions via `engine-human-design`, with the previous mean-longitude math retained only as fallback
    - normalized internal karana naming from `Garaja` to `Gara` to match the rest of the system
- Verification after fix:
  - `cargo test -p engine-panchanga test_karana_regression_march_10_2026_day_sequence -- --nocapture`
  - `cargo test -p engine-panchanga -- --nocapture`
  - `cargo test -p noesis-api --test engine_consistency_tests -- --nocapture`
- Current confidence decision:
  - the March 10, 2026 reference now matches on the previously broken karana sequence:
    - `Vishti` at sunrise
    - still `Vishti` at `12:21 PM`
    - `Bava` after the documented `12:40 PM` transition (regression asserted at `12:41 PM`)
  - native Panchanga is now materially stronger for day-level validation and no longer blocked on the karana mismatch.
  - canonical birth-fixture validation against a trusted external Panchanga source is still pending before declaring native Panchanga the final baseline for all cases.

---

# Task Plan — Panchanga Birth-Time Validation Against Trusted Screenshot

## Checklist
- [x] Extract the key Panchanga fields from the trusted birth-day screenshots for:
  - `1991-08-13`
  - `13:31`
  - `Bangalore, India`
  - `Asia/Kolkata`
- [x] Run native Selemene Panchanga for the same birth moment.
- [x] Compare native output vs screenshot on:
  - `tithi`
  - `nakshatra`
  - `nakshatra pada`
  - `yoga`
  - `karana`
  - `vara`
  - `moon sign`
  - `sun sign`
  - `ayanamsha`
- [x] Record the exact matches/mismatches and confidence decision.

## Notes
- This pass is validation only.
- Use the screenshot as the trusted external reference for the exact birth moment.
- Keep the March 10, 2026 day-Panchang fix separate from the birth-time verdict.

## Review (fill after execution)
- Trusted screenshot reference for `1991-08-13 13:31 Bangalore` shows:
  - `Tithi = Chaturthi upto 10:04 PM`
  - `Nakshatra = Uttara Phalguni upto 01:52 PM`
  - `Nakshatra Pada = 4`
  - `Yoga = Siddha upto 07:48 PM`
  - `Karana = Vishti upto 10:04 PM`
  - `Weekday = Mangalavara`
  - `Moon sign = Kanya`
  - `Sun sign = Karka`
  - `Lahiri Ayanamsha = 23.746632`
- Native Selemene Panchanga at the same moment returned:
  - `tithi_name = Chaturthi (Shukla)`
  - `nakshatra_name = Uttara Phalguni`
  - `nakshatra_value = 11.987291238292826` -> `pada 4`
  - `yoga_name = Siddha`
  - `karana_name = Vishti`
  - `vara_name = Mangalavara (Tuesday)`
  - `solar_longitude = 116.37350004069478` -> `Karka`
  - `lunar_longitude = 159.83054984390435` -> `Kanya`
  - `julian_day = 2448481.834027778`
- Decision:
  - native Panchanga matches the trusted screenshot on all high-signal birth fields for the canonical input.
  - regression coverage added in `engine-panchanga` for this exact birth-time fixture.

---

# Task Plan — Panchanga Provider Runtime Decommission

## Checklist
- [x] Remove FreeAstrologyAPI runtime dispatch from `noesis-api` for Panchanga.
- [x] Remove provider mapping/runtime dependency hooks that are no longer reachable.
- [x] Scrub provider reporting fields from active `/api/v1/status` and `/ready` responses.
- [x] Add route-level tests proving Panchanga remains native even if old provider env vars are still set.
- [x] Verify no provider trace fields remain in active Panchanga route responses.

## Notes
- Scope is runtime cleanup only in `noesis-api`.
- Keep the `noesis-vedic-api` crate in the repo; remove only the active API-path association.
- Do not touch unrelated engines or legacy docs in this pass.

## Review (fill after execution)
- Runtime cleanup completed in `noesis-api`:
  - removed the provider dispatch branch from `calculate_handler`
  - removed provider-specific Panchanga mapping/helpers from `src/lib.rs`
  - removed the active `noesis-vedic-api` dependency from `crates/noesis-api/Cargo.toml`
  - removed provider/status fields from active `/api/v1/status` and `/ready` responses
- Trace verification after cleanup:
  - `panchanga` route stays native even when `FREE_ASTROLOGY_API_*` and `VEDIC_ENGINE_PROVIDER=api` are set
  - response no longer contains `provider` or `_selemene_execution` trace fields
  - `/api/v1/status` and `/ready` no longer report provider configuration fields
- Verification:
  - `cargo test -p engine-panchanga -- --nocapture`
  - `cargo test -p noesis-api --test vedic_provider_route_tests -- --nocapture`
  - `cargo test -p noesis-api --test engine_consistency_tests -- --nocapture`

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

---

# Task Plan — Selemene Engine Hygiene Verification (2026-03-09)

## Checklist
- [x] Review the existing hygiene report, task log, and current worktree changes.
- [x] Verify whether `noesis-api` currently routes Vedic engines through `noesis-vedic-api` / FreeAstrologyAPI.
- [x] Run the targeted regression tests cited in the hygiene report.
- [x] Run at least one broader validation suite to check whether the "fixed" claims hold beyond the canonical fixture.
- [x] Record the verified state and the main contradictions/gaps.

## Notes
- Scope is verification only. No engine logic changes in this pass.
- Focus question: whether current runtime uses FreeAstrologyAPI, and whether the pasted hygiene transcript accurately reflects the repo's present state.

## Review
- Verified provider wiring in `crates/noesis-api`:
  - `noesis-api` now depends on `noesis-vedic-api`.
  - `VEDIC_ENGINE_PROVIDER` defaults to `api`.
  - `POST /api/v1/engines/:engine_id/calculate` uses provider dispatch for `panchanga` and `vimshottari` only, with fallback controlled by `VEDIC_ENGINE_FALLBACK_ENABLED`.
  - `human-design`, `gene-keys`, `transits`, and `vedic-clock` remain native in provider mode.
  - The legacy Panchanga route still bypasses provider routing and uses the native orchestrator path.
- Verified targeted regression tests pass:
  - `cargo test -p engine-panchanga`
  - `cargo test -p engine-vimshottari`
  - `cargo test -p engine-vedic-clock`
  - `cargo test -p noesis-bridge`
  - `cargo test -p engine-human-design test_canonical_profile_regression -- --nocapture`
  - `cargo test -p engine-gene-keys test_gk_birth_mode_derives_from_hd_engine -- --nocapture`
  - `cargo test -p engine-gene-keys -- --nocapture`
  - `cargo test -p noesis-api provider_mode_tests -- --nocapture`
  - `cargo test -p noesis-api --test engine_consistency_tests -- --nocapture`
- Verified important gap:
  - `cargo test -p engine-human-design -- --nocapture` still fails its broader `reference_validation_tests` suite badly:
    - Sun/Earth validation: 0.0%
    - Type validation: 0.0%
    - Authority validation: 25.0%
    - Profile validation: 75.0%
    - Centers validation: 0.0%
    - Channels validation: 0.0%
- Conclusion:
  - The pasted transcript is only partially accurate.
  - It is accurate that targeted canonical fixes/tests were added.
  - It is no longer accurate to say provider env vars are unused in runtime for all Vedic paths.
  - It overstates the Human Design fix status; the canonical fixture passes, but the broader reference suite is still failing.
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

# Task Plan — Engine Hygiene Issue Triage (2026-03-08)

## Checklist
- [x] Consolidate engine hygiene challenges from latest validation run.
- [x] Create parent tracking issue for coordinated remediation.
- [x] Create child issues for each validated challenge domain.
- [x] Link all child issues back to parent tracker.
- [x] Create report with challenge matrix + issue links.

## Review
- Parent tracking issue created: #501
- Child issues created: #502, #503, #504, #505, #506, #507
- Report added: `docs/planning/selemene-engine-hygiene-2026-03-08.md`
- Recommended fix order documented in the report.

## Follow-up Execution — #502 + #503
- [x] Implemented sidereal conversion in panchanga output path.
- [x] Fixed vimshottari timezone conversion (local birth time -> UTC).
- [x] Added canonical regression tests for Uttara Phalguni resolution.
- [x] Added cross-engine parity test in noesis-api.
- [x] Verified with:
  - `cargo test -p engine-panchanga`
  - `cargo test -p engine-vimshottari`
  - `cargo test -p noesis-api --test engine_consistency_tests`

## Follow-up Execution — #504 + #505
- [x] Corrected HD gate-sequence transposition causing design-side canonical drift (23/43 vs 8/14).
- [x] Added canonical HD regression for incarnation-cross gate anchors.
- [x] Added GK birth-mode contract test proving HD-derived activation sequence mapping.
- [x] Added HD↔GK canonical parity test in `noesis-api` consistency suite.
- [x] Verified with:
  - `cargo test -p engine-human-design test_canonical_profile_regression`
  - `cargo test -p engine-gene-keys test_gk_birth_mode_derives_from_hd_engine`
  - `cargo test -p noesis-api --test engine_consistency_tests`

## Follow-up Execution — #507 + provider wiring discovery
- [x] Fixed Vedic-clock timezone fallback to `birth_data.timezone` when explicit offset missing.
- [x] Added/updated tests for timezone fallback behavior.
- [x] Ran verification: `cargo test -p engine-vedic-clock`.
- [x] Created provider wiring issue #508 to track `api/native` runtime selection gap for Vedic engines.

## Follow-up Execution — #506 + #508
- [x] Added sigil-forge pre-validation in bridge adapter to avoid malformed-input bridge 500s.
- [x] Mapped bridge 4xx responses to validation errors (422 path).
- [x] Added provider-mode visibility to readiness/status payloads in noesis-api.
- [x] Added provider-mode mapping tests.
- [x] Verified with:
  - `cargo test -p noesis-bridge`
  - `cargo test -p noesis-api provider_mode_tests`

## Follow-up Execution — #508 true switching pass
- [x] Added noesis-vedic-api runtime dependency in noesis-api.
- [x] Implemented provider dispatch for `panchanga` and `vimshottari` in calculate handler.
- [x] Added fallback-to-native behavior when provider fails and fallback is enabled.
- [x] Updated status/readiness effective provider mode semantics.
- [x] Verified with `cargo test -p noesis-api provider_mode_tests -- --nocapture`.
- [x] Added provider-routing helper tests for supported engine gating.
- [x] Added fallback-flag env parsing test coverage.
- [x] Added provider-output mapping contract tests for panchanga and vimshottari.
- [x] Re-verified with `cargo test -p noesis-api provider_mode_tests -- --nocapture` (7 passing).

---

# Task Plan — Next Wave Synthesis: Astrology API Correctness (2026-03-10)

## Discovery Summary
- Planning depth: lean
- Delivery mode: hardening
- CI/CD expectation: basic + targeted regression gates
- Release model: phased rollout
- Quality bar: prove route selection, prove provider-vs-native source, then fix calculation/mapping defects
- Team topology: solo-owner friendly

## Assumptions
- The immediate problem is not "add more engines" but "make current panchanga/vimshottari answers attributable and trustworthy."
- The existing issue set (#501-#508) remains the right container; the next wave should refine/advance those issues rather than open a fresh parallel tracker unless scope expands.
- Human Design/Gene Keys should not be treated as solved while the broader HD reference suite is still failing.

## Proposed next waves

### Wave 1 — Provider Truth and Attribution
- [ ] Add end-to-end API tests for `POST /api/v1/engines/panchanga/calculate` and `POST /api/v1/engines/vimshottari/calculate` that assert actual runtime path selection under:
  - `VEDIC_ENGINE_PROVIDER=api`
  - `VEDIC_ENGINE_PROVIDER=native`
  - provider failure + fallback enabled
  - provider failure + fallback disabled
- [ ] Add explicit response metadata proving which path served the result:
  - provider direct
  - native direct
  - native fallback after provider failure
- [ ] Add a reproducible comparison harness/script for canonical birth input:
  - raw `noesis-vedic-api` payload
  - mapped `noesis-api` response
  - native engine response
- [ ] Decide and document whether the legacy Panchanga route should:
  - remain native and be clearly documented as such, or
  - be routed through the same provider-aware dispatcher

### Wave 2 — Provider Contract Hardening (#502, #503, #508)
- [ ] Add golden fixtures from known-good provider responses for canonical birth data.
- [ ] Add mapping completeness tests ensuring provider payload fields are preserved or intentionally transformed.
- [ ] Add negative tests for timezone, ayanamsa, and moon-longitude drift against the canonical birth fixture.
- [ ] Add a one-command validation script that reports:
  - configured provider mode
  - effective route used
  - backend label
  - canonical nakshatra / dasha summary
- [ ] Update stale docs/report language that still says provider routing is unused.

### Wave 3 — Runtime Behavior in Real Environments
- [ ] Add readiness/status checks that surface whether provider mode is configured but unusable due to missing key/network.
- [ ] Add production/local smoke steps for:
  - `FREE_ASTROLOGY_API_KEY` present
  - provider reachable
  - fallback triggered/not triggered
- [ ] Verify Railway/local env parity for:
  - `FREE_ASTROLOGY_API_KEY`
  - `VEDIC_ENGINE_PROVIDER`
  - `VEDIC_ENGINE_FALLBACK_ENABLED`
- [ ] Add log assertions or structured tracing around provider invocation/fallback transitions.

### Wave 4 — Remaining Native Engine Correctness (#504, #505, #507)
- [ ] Re-scope #504 around the failing broad reference suite, not the passing canonical fixture.
- [ ] Build a reduced reference chart pack for HD/GK triage so fixes can land incrementally.
- [ ] Keep GK aligned to HD outputs, but do not close HD until full reference validation improves materially.
- [ ] Close #507 only after route-level behavior is verified through API integration, not just unit tests.

## Recommended execution order
1. #508 first: prove route selection and fallback semantics end-to-end.
2. #502 + #503 second: use the new harness to determine whether remaining drift is provider payload, mapper logic, or native fallback confusion.
3. Docs/report cleanup third: remove contradictory statements so future debugging is not polluted by stale assumptions.
4. #504 + #505 after that: broad HD/GK correctness is still a separate native-engine track.

## Review
- Current blocker is attribution ambiguity, not lack of more targeted math tests.
- Once route attribution is explicit, we can answer "FreeAstrologyAPI is wrong" vs "our bridge is wrong" with evidence instead of inference.

## Follow-up Execution — Wave 1 attribution hardening
- [x] Added explicit execution attribution for `panchanga` and `vimshottari` responses in `crates/noesis-api/src/lib.rs`.
  - `result._selemene_execution.route` is now one of:
    - `provider`
    - `native`
    - `fallback`
  - Attribution also records:
    - `requested_provider`
    - `provider_attempted`
    - `fallback_used`
    - resolved `backend`
- [x] Added isolated route-level provider tests in `crates/noesis-api/tests/vedic_provider_route_tests.rs` using `wiremock`.
- [x] Added `wiremock` + `noesis-vedic-api` mocks support to `crates/noesis-api/Cargo.toml` dev-dependencies.
- [x] Verified route behavior end-to-end:
  - provider direct success for `panchanga`
  - provider direct success for `vimshottari`
  - native direct mode for `panchanga`
  - provider failure + fallback enabled for `panchanga`
  - provider failure + fallback disabled for `panchanga`
- [x] Verification commands:
  - `cargo test -p noesis-api --test vedic_provider_route_tests -- --nocapture`
  - `cargo test -p noesis-api provider_mode_tests -- --nocapture`

## Follow-up Execution — Vimshottari provider de-association (surgical)
- [x] Remove only `vimshottari` from the FreeAstrologyAPI runtime path.
- [x] Keep `panchanga` provider wiring unchanged.
- [x] Preserve response schema and native `vimshottari` behavior.
- [x] Update route/provider tests so `vimshottari` is asserted native even when `VEDIC_ENGINE_PROVIDER=api`.
- [x] Re-verify targeted `noesis-api` and `engine-vimshottari` tests.

### Constraints
- Do not remove `noesis-vedic-api` globally.
- Do not change `panchanga` routing in this pass.
- Do not touch HD/GK/native engine math in this pass.

### Review
- `crates/noesis-api/src/lib.rs`
  - `effective_vedic_engine_modes_for("api")` now reports only `panchanga` as `api`.
  - `use_api_provider_for_engine(...)` now returns `true` only for `panchanga`.
  - Removed provider-side `vimshottari` mapping/dispatch from the runtime path.
- `crates/noesis-api/tests/vedic_provider_route_tests.rs`
  - Replaced provider-success expectation for `vimshottari` with native-route assertion under `VEDIC_ENGINE_PROVIDER=api`.
  - Added assertion that no provider request is made for `vimshottari`.
- `crates/noesis-api/src/lib.rs` provider-mode tests
  - Updated expected mode map and provider gating assertions.
  - Removed obsolete provider-vimshottari mapping contract test.
- Verification:
  - `cargo test -p engine-vimshottari`
  - `cargo test -p noesis-api provider_mode_tests -- --nocapture`
  - `cargo test -p noesis-api --test vedic_provider_route_tests -- --nocapture`
  - `cargo test -p noesis-api --test engine_consistency_tests -- --nocapture`

---

# Task Plan — Transits Engine Validation

## Checklist
- [ ] Inspect the native `transits` engine path, inputs, and output contract.
- [ ] Lock a fixed transit timestamp for comparison so validation is deterministic.
- [ ] Reproduce the current transits output for the canonical birth data.
- [ ] Compare high-signal fields against trusted expectations:
  - natal Moon sign / longitude basis
  - transit Saturn sign
  - Sade Sati status
  - a few major transit-to-natal aspects
- [ ] Isolate whether the first defect is:
  - birth timezone handling
  - tropical vs sidereal basis mismatch
  - aspect logic
  - output/schema drift
- [ ] Add regression coverage and implement the smallest correct fix if a real defect is confirmed.

## Notes
- Scope is native `engine-transits` only. FreeAstrologyAPI is no longer in the active runtime path here.
- Use the canonical birth fixture:
  - `1991-08-13`
  - `13:31`
  - `12.9340, 77.6214`
  - `Asia/Kolkata`
- Use a fixed transit timestamp for verification, not a moving `now`.

## Review (fill after execution)
- Early code review findings before patching:
  - `crates/engine-transits/src/engine.rs` parses birth date/time as UTC and ignores `birth_data.timezone`.
  - `crates/engine-transits/src/ephemeris.rs` uses raw Swiss Ephemeris tropical longitudes.
  - `crates/engine-transits/src/sade_sati.rs` computes Sade Sati from those sign values, which is conceptually Vedic and therefore likely requires sidereal Moon/Saturn signs.
  - Existing tests cover helper logic and output shape, but not canonical birth-time correctness, timezone handling, or sidereal-vs-tropical assumptions.
- Fixed in this pass:
  - `crates/engine-transits/src/engine.rs`
    - `parse_birth_datetime()` now parses `birth_data.timezone` as an IANA timezone and converts local birth time to UTC before natal calculations.
  - `crates/engine-transits/src/ephemeris.rs`
    - converted Swiss Ephemeris tropical longitudes to sidereal using the same approximate Lahiri offset (`23.72`) already used by native Panchanga/Vimshottari.
- Trusted references used:
  - birth screenshot for `1991-08-13 13:31 Bangalore`
    - natal Sun sign `Karka` / `Cancer`
    - natal Moon sign `Kanya` / `Virgo`
  - Bengaluru day Panchang screenshot for `2026-03-10 06:30 Asia/Kolkata`
    - transit Sun sign `Kumbha` / `Aquarius`
    - transit Moon sign `Vrishchika` / `Scorpio`
- New regression coverage:
  - `test_parse_birth_datetime_respects_birth_timezone`
    - proved the old bug first (`1991-08-13 13:31 Asia/Kolkata` was being treated as `1991-08-13T13:31:00Z`)
    - now asserts the correct UTC conversion: `1991-08-13T08:01:00Z`
  - `test_birth_and_transit_signs_match_trusted_references`
    - asserts natal `Sun = Cancer`, `Moon = Virgo`
    - asserts transit `Sun = Aquarius`, `Moon = Scorpio` for `2026-03-10T01:00:00Z` (`06:30 IST`)
- Verification:
  - `cargo test -p engine-transits test_parse_birth_datetime_respects_birth_timezone -- --nocapture`
  - `cargo test -p engine-transits test_birth_and_transit_signs_match_trusted_references -- --nocapture`
  - `cargo test -p engine-transits -- --nocapture`
  - `cargo test -p noesis-api provider_mode_tests -- --nocapture`
- Current verdict:
  - native `transits` had a real foundational defect.
  - the engine now uses timezone-correct natal timestamps and sidereal sign assignment aligned with the rest of the native Vedic stack.
  - aspect math was left unchanged because applying the same sidereal offset to natal and transit longitudes preserves the inter-planet angular relationships used by the aspect detector.
- Residual risk:
  - we have not yet externally validated a full list of transit-to-natal aspects or Sade Sati phase against a second trusted transit source.
  - docs still describe the engine as `9 Navagraha`, while the runtime output includes outer planets plus Rahu/Ketu.
- Additional natal reference fixtures queued from user-provided AstroSage PDFs:
- Additional natal reference fixtures validated from user-provided AstroSage PDFs:
  - `Natesh Aiyer`
    - `1960-11-20 17:15 Bangalore`
    - `Sun = Scorpio`, `Nakshatra = Anuradha pada 1`
    - `Moon = Scorpio`, `Nakshatra = Jyeshtha pada 3`
    - `Asc = Aries`
  - `Anitha Nateshan`
    - `1965-06-01 00:35 Bangalore`
    - `Sun = Taurus`, `Nakshatra = Rohini pada 3`
    - `Moon = Taurus`, `Nakshatra = Mrigashira pada 2`
    - `Asc = Aquarius`
- Regression coverage added for both PDF fixtures in `crates/engine-transits/src/engine.rs`:
  - `test_natesh_aiyer_natal_positions_match_pdf_reference`
  - `test_anitha_nateshan_natal_positions_match_pdf_reference`
- One fixture (`Anitha Nateshan`) initially exposed a residual defect:
  - fixed-offset ayanamsha (`23.72`) was too imprecise across years and placed the natal Sun in `Rohini pada 2` instead of the trusted `Rohini pada 3`.
- Surgical refinement applied:
  - `crates/engine-transits/src/ephemeris.rs` now gets the date-correct Lahiri ayanamsha from Swiss Ephemeris via `swe_set_sid_mode(Lahiri)` + `swe_get_ayanamsa_ut(...)` instead of subtracting a fixed constant.
- Expanded verification:
  - `cargo test -p engine-transits test_natesh_aiyer_natal_positions_match_pdf_reference -- --nocapture`
  - `cargo test -p engine-transits test_anitha_nateshan_natal_positions_match_pdf_reference -- --nocapture`
  - `cargo test -p engine-transits -- --nocapture`
  - `cargo test -p noesis-api provider_mode_tests -- --nocapture`
- Updated confidence:
  - native `transits` now matches three independent natal reference fixtures for Sun/Moon sign and nakshatra/pada-sensitive sidereal placement:
    - your canonical 1991 birth chart
    - `Natesh Aiyer` 1960 chart
    - `Anitha Nateshan` 1965 chart
  - this materially raises confidence in the natal baseline used for transit analysis.

---

# Task Plan — Docs + Commit + Push

## Checklist
- [ ] Update user-facing docs to reflect the current native-first Vedic runtime and transits fixes.
- [ ] Update release-facing notes with the March 2026 engine hygiene corrections.
- [ ] Add a short correction note to the hygiene planning report so it no longer claims provider routing is active.
- [ ] Re-run targeted verification before committing.
- [ ] Commit the validated changes and push to a remote branch so existing workflows can trigger safely.

## Notes
- Current repo state is detached `HEAD`; do not commit directly without creating a branch first.
- `deploy.yaml` auto-deploys only on `main` or tags.
- `test.yml` runs only on `main`, `develop`, and pull requests.
- Safe path: create a `codex/...` branch, push it, then open/merge through the normal repo flow if deployment to `main` is desired.

## Review (fill after execution)
- Branch created from detached `HEAD`:
  - `codex/engine-hygiene-native-runtime-docs`
- User-facing docs updated:
  - `README.md`
  - `docs/ENGINES.md`
  - `docs/PROJECT_OVERVIEW.md`
  - `docs/RELEASE_NOTES.md`
  - `docs/MIGRATION_TO_FREE_ASTROLOGY_API.md`
  - `docs/planning/selemene-engine-hygiene-2026-03-08.md` (historical-note correction)
- Commit created:
  - `c379346d`
  - `fix(vedic): restore native runtime and validate transits baselines`
- Branch pushed:
  - `origin/codex/engine-hygiene-native-runtime-docs`
- Pull request opened:
  - `#509`
  - `https://github.com/Sheshiyer/Selemene-engine/pull/509`
- Verification run before commit:
  - `cargo test -p engine-panchanga -- --nocapture`
  - `cargo test -p engine-vimshottari -- --nocapture`
  - `cargo test -p engine-transits -- --nocapture`
  - `cargo test -p noesis-api --test engine_consistency_tests -- --nocapture`
  - `cargo test -p noesis-api --test vedic_provider_route_tests -- --nocapture`
  - `cargo test -p noesis-api provider_mode_tests -- --nocapture`
  - `cargo test -p noesis-bridge -- --nocapture`
  - `cargo test -p engine-vedic-clock -- --nocapture`
  - `cargo test -p engine-human-design test_canonical_profile_regression -- --nocapture`
  - `cargo test -p engine-gene-keys test_gk_birth_mode_derives_from_hd_engine -- --nocapture`
  - `cargo test -p noesis-vedic-api --test client_tests test_get_panchang_uses_x_api_key_header_and_complete_endpoint -- --nocapture`
  - `cargo test -p noesis-vedic-api --test panchang_tests -- --nocapture`
- Workflow status at handoff:
  - PR exists and is the correct route for `test.yml` to trigger.
  - At the time of verification, only `Supabase Preview` had appeared and was `skipped`; no GitHub Actions test run had shown up yet.

---

# Task Plan — Wave 1 / Wave 2 Backlog Review

## Checklist
- [x] Pull all open GitHub issues carrying `wave:W1` or `wave:W2`.
- [x] Group the open Wave 1 / Wave 2 inventory by milestone.
- [x] Compare the largest open clusters against current repo state.
- [x] Identify realistic finish-now clusters versus clusters that should be deferred.
- [x] Convert the finish-now clusters into an execution order once approved.

## Notes
- Raw open Wave 1 / Wave 2 backlog size on `2026-03-11`: `267` issues.
- Counts by milestone:
  - `P1-Stabilization`: `46` open (`W1=21`, `W2=25`)
  - `P2-Workflow-Hardening`: `53` open (`W1=24`, `W2=29`)
  - `P3-Bridge-Reliability`: `31` open (`W1=16`, `W2=15`)
  - `P4-Performance-Observability`: `49` open (`W1=28`, `W2=21`)
  - `P5-Release-Readiness`: `46` open (`W1=23`, `W2=23`)
  - `v2.2.0-Specialized-Engines`: `42` open (`W1=29`, `W2=13`)
- There are no open `wave:W1` or `wave:W2` issues left in `v3.0.0-Platform-Launch`.
- These issue sets are not one coherent “current sprint”; many are old taskmaster leaves that were never closed or re-scoped after architecture drift.

## Review (fill after execution)
- Evidence collected from GitHub:
  - Open issue inventory pulled with `gh issue list --state open --limit 500 --json ...`
  - Roadmap reference checked in `.github/projects/CONSCIOUSNESS_ROADMAP.md`
- Current-code evidence that some old Wave 1 / Wave 2 issues are partially or fully stale:
  - `crates/noesis-api/src/lib.rs` already exposes `/health/live`, `/health/ready`, and `/ready`
  - `crates/noesis-api/tests/error_handling_tests.rs` and `crates/noesis-api/src/lib.rs` already implement structured `error_code` responses and broad mapping tests
  - `crates/noesis-orchestrator/src/lib.rs`, `crates/noesis-orchestrator/src/workflow/registry.rs`, and `crates/noesis-orchestrator/tests/*` already cover workflow registry and phase gating
  - `scripts/smoke_admin_web.sh` and `.github/workflows/deploy.yaml` already provide part of the release-smoke infrastructure
- Current-code evidence that other clusters are still materially unfinished:
  - `crates/noesis-orchestrator/src/workflow/executor.rs` still contains `TODO: Implement other synthesizers`
  - `docs/PROJECT_OVERVIEW.md` explicitly notes that only `birth-blueprint` and `daily-practice` have fully implemented synthesis, while `decision-support`, `self-inquiry`, `creative-expression`, and `full-spectrum` remain sparse
  - `crates/engine-biofield/src` does not yet contain `meridian_analysis`, `aura_layers`, or `healing_recommendations` structures implied by the `v2.2.0` Wave 1 / Wave 2 issues
  - `crates/engine-numerology/src` and `crates/engine-biorhythm/src` do not yet show the planned personal-cycle / secondary-rhythm feature set from `v2.2.0-Specialized-Engines`
  - `crates/noesis-api/src/lib.rs` still uses `engine_error_to_response` inline and does not contain a dedicated `ErrorMapper` module
  - repository-wide search shows no `CacheKeyBuilder` implementation; the P1 cache-key migration cluster is still open in substance

### Finish-Now Clusters
- `P2-W1 health/readiness surface`
  - Closest to done because the health base already exists.
  - Candidate issues:
    - `#115` `Add /health/live liveness endpoint to TS server`
    - `#116` `Add /health/ready readiness endpoint with engine checks`
    - `#121` `Add /health/engines endpoint listing per-engine status`
    - `#122` `Wire sidecar health into Rust /health/ready endpoint`
    - `#120` `Test sidecar probe responses under partial engine failure`
  - Practical read: `/health/live` and `/health/ready` exist on the Rust side already, so the remaining work is to make sidecar/engine health explicit and then close the stale split issues.

- `P1 error-mapping cleanup`
  - Large parts already exist, but the shape is scattered.
  - Candidate issues:
    - `#45` through `#49`
    - `#50` through `#57`
  - Practical read: this is now mostly consolidation and verification, not greenfield implementation.

- `P5 release smoke consolidation`
  - Existing smoke/deploy plumbing means this is a realistic short wave.
  - Candidate issues:
    - `#291` through `#299`
    - optionally `#305` for Docker `HEALTHCHECK`
  - Practical read: extend the existing smoke scripts/workflow rather than invent a new release framework.

- `Issue hygiene / stale closure pass`
  - Several old audit/doc issues are likely closeable after short evidence notes instead of more code.
  - Candidate review issues:
    - `#29` through `#37`
    - `#45` through `#49`
    - `#106`
  - Practical read: these are “prove current state and close or re-scope” issues, not all new engineering work.

### Defer / Re-Scope Clusters
- `P2-W2 workflow contract suite` (`#123` through `#151`)
  - Do not treat as a quick finish.
  - The synthesis layer is still incomplete for four workflows, so schema fixtures and determinism work here will sprawl.

- `v2.2.0-Specialized-Engines` (`#361` through `#402`)
  - Not “barely finish now”.
  - These require real feature work across numerology, biorhythm, biofield, and workflow synthesis; current runtime state does not show those features landed.

- `P3 bridge retry/circuit-breaker reliability` (`#185` through `#199`, plus `#107` through `#114`)
  - Needs re-scoping before execution.
  - The active runtime path changed materially during the native-first Vedic cleanup, so some of this plan targets architecture that is no longer central.

- `P4 idempotency / auth soak / canary rollout` (`#256` through `#269`, `#314` through `#336`)
  - Operationally valuable, but not the fastest closure path.
  - These are infra-heavy and environment-dependent rather than current codebase cleanup.

### Recommended Near-Term Closure Sequence
- Wave A: `P2-W1` health/readiness cluster
- Wave B: `P1` error-mapping cleanup cluster
- Wave C: `P5-W1` smoke-test consolidation cluster
- After those three: stale-audit closure sweep across the old `P1` documentation/audit issues

### Wave A — Exact Issue Map
- `#115` `Add /health/live liveness endpoint to TS server`
  - Current state:
    - TS sidecar already has a coarse `/health` endpoint in `ts-engines/src/server/app.ts`.
    - Integration coverage for `/health` already exists in `ts-engines/tests/integration.test.ts`.
  - Missing:
    - exact `/health/live` route returning unconditional process liveness
    - sidecar docs / startup log update if needed
  - Decision:
    - implement, not close as-is
  - Expected touch set:
    - `ts-engines/src/server/app.ts`
    - `ts-engines/tests/integration.test.ts`

- `#116` `Add /health/ready readiness endpoint with engine checks`
  - Current state:
    - no `/health/ready` exists in the TS sidecar
    - `ConsciousnessEngine` interface does not expose `selfCheck`
  - Missing:
    - readiness route
    - lightweight per-engine health contract
    - aggregate status payload
  - Decision:
    - implement
  - Expected touch set:
    - `ts-engines/src/types/engine.ts`
    - `ts-engines/src/server/app.ts`
    - `ts-engines/src/server/registry.ts`
    - engine implementations only if a default self-check cannot be derived centrally
    - `ts-engines/tests/integration.test.ts`

- `#121` `Add /health/engines endpoint listing per-engine status`
  - Current state:
    - no per-engine health endpoint exists
  - Missing:
    - route exposing per-engine health objects with latency
    - shared health result type reused by `/health/ready`
  - Decision:
    - implement in the same batch as `#116`
  - Expected touch set:
    - `ts-engines/src/types/engine.ts`
    - `ts-engines/src/server/app.ts`
    - `ts-engines/tests/integration.test.ts`

- `#122` `Wire sidecar health into Rust /health/ready endpoint`
  - Current state:
    - Rust API already exposes `/health/ready` and `/ready` in `crates/noesis-api/src/lib.rs`
    - `BridgeManager::health_check()` exists in `crates/noesis-bridge/src/lib.rs`, but it only hits sidecar `/health` and returns coarse success/failure
    - `ReadinessResponse` currently reports only `redis`, `orchestrator`, and `overall_status`
  - Missing:
    - bridge/sidecar readiness field in API response
    - richer bridge health payload or a new manager method consuming sidecar `/health/ready` or `/health/engines`
    - readiness tests covering available/degraded bridge states
  - Decision:
    - implement after `#116` + `#121`
  - Expected touch set:
    - `crates/noesis-bridge/src/lib.rs`
    - `crates/noesis-api/src/lib.rs`
    - `crates/noesis-api/tests/...` readiness/route tests

- `#120` `Test sidecar probe responses under partial engine failure`
  - Current state:
    - only coarse `/health` tests exist on the TS side
  - Missing:
    - healthy / partial failure / all failed / recovery scenarios
    - either mocking hooks or test doubles for engine self-check state
  - Decision:
    - implement last in the Wave A batch, after the routes exist
  - Expected touch set:
    - `ts-engines/tests/integration.test.ts`
    - possibly a small test helper in `ts-engines/tests/`

### Wave A — Recommended Build Order
- 1. `#115`
  - cheap route split; gives clean liveness semantics
- 2. `#116` + `#121`
  - shared sidecar health model and routes in one TS-side PR slice
- 3. `#122`
  - consume sidecar readiness from Rust API and expose `bridge_status`
- 4. `#120`
  - lock the new behavior with state-transition tests

### Wave A — Closure Rule
- Do not close any of `#115`, `#116`, `#120`, `#121`, `#122` from review alone.
- `#115` is closest to stale, but still misses the exact route shape requested by the issue.
- The right move is one compact implementation wave that closes all five together with proof.

### Wave A — Execution Review
- Implemented TS-side health probe surface:
  - `ts-engines/src/types/engine.ts`
    - added `EngineHealthStatus`, `LivenessResponse`, `ReadinessResponse`, `EnginesHealthResponse`
    - added optional `selfCheck()` contract on `ConsciousnessEngine`
  - `ts-engines/src/server/registry.ts`
    - exported `EngineRegistry`
    - added `all()` accessor for per-engine health iteration
  - `ts-engines/src/server/app.ts`
    - `createServer()` now accepts an injected registry for isolated tests
    - added `/health/live`
    - added `/health/ready`
    - added `/health/engines`
    - implemented default self-check behavior for engines that do not yet define a custom health probe
  - `ts-engines/src/server/index.ts`
    - exports `EngineRegistry` for isolated test setup
- Added TS-side regression coverage:
  - `ts-engines/tests/health.test.ts`
    - unconditional liveness
    - healthy readiness
    - per-engine health payload
    - degraded readiness
    - recovery back to healthy
- Implemented Rust bridge-aware readiness:
  - `crates/noesis-bridge/src/lib.rs`
    - added `SidecarEngineHealth` and `SidecarReadinessStatus`
    - added `BridgeManager::readiness_status()`
  - `crates/noesis-api/src/lib.rs`
    - `AppState` now carries `bridge_manager`
    - `/ready` response now includes:
      - `bridge_status`
      - `bridge_engines`
      - `bridge_failed_engines`
    - overall readiness now factors bridge availability alongside cache + orchestrator
- Added Rust-side regression coverage:
  - `crates/noesis-api/tests/bridge_readiness_tests.rs`
    - bridge available path
    - bridge degraded path with failed-engine details
- Verification:
  - `bun test ts-engines/tests/health.test.ts ts-engines/tests/integration.test.ts`
  - `cargo test -p noesis-api --test bridge_readiness_tests --test vedic_provider_route_tests -- --nocapture`
  - `cargo test -p noesis-bridge -- --nocapture`
- Wave A status:
  - `#115`, `#116`, `#120`, `#121`, `#122` are now implemented in code and covered by automated tests.
  - Remaining administrative work is issue updates/closure, not more implementation for this cluster.

### Wave A — Commit and Closure
- [x] Re-run Wave A verification commands before commit/push.
- [x] Commit the Wave A readiness + health surface changes on the current branch.
- [x] Push `codex/engine-hygiene-native-runtime-docs`.
- [x] Comment on and close `#115`, `#116`, `#120`, `#121`, and `#122` with verification evidence.
- Closeout record:
  - commit: `d80eee79` `fix(health): implement Wave A sidecar readiness surface`
  - issue comments:
    - `#115`: `issuecomment-4038674085`
    - `#116`: `issuecomment-4038674114`
    - `#120`: `issuecomment-4038674087`
    - `#121`: `issuecomment-4038674205`
    - `#122`: `issuecomment-4038674089`
  - issue state:
    - `#115`, `#116`, `#120`, `#121`, `#122` closed on GitHub

### Wave B — P1 Error Mapping Review
- [x] Audit issue `#45` against the current `EngineError` taxonomy and HTTP mapping.
- [x] Audit issue `#46` against current engine crate error patterns and identify real inconsistencies.
- [x] Audit issues `#47` through `#57` against current `noesis-api` error response behavior and mark closable vs still-open.
- [x] Record the Wave B audit in a repo artifact that can be linked from GitHub issues.
- [x] Run fresh verification on the error-handling cluster before closing any issues.
- [x] Comment on and close only the Wave B issues that are fully satisfied by current code + audit evidence.
- Wave B audit artifact:
  - `docs/planning/wave-b-p1-error-mapping-audit-2026-03-11.md`
- Fresh verification:
  - `cargo test -p noesis-api --test error_handling_tests -- --nocapture`
  - `cargo test -p noesis-bridge bridge_engine -- --nocapture`
- Closable now:
  - `#45`
  - `#46`
  - `#49`
- Keep open:
  - `#47`
  - `#48`
  - `#50` through `#57`
- Closeout record:
  - audit commit: `0878b1a2` `docs(api): audit Wave B error mapping cluster`
  - issue comments:
    - `#45`: `issuecomment-4038733331`
    - `#46`: `issuecomment-4038733340`
    - `#49`: `issuecomment-4038733330`
  - issue state:
    - closed: `#45`, `#46`, `#49`
    - left open by audit: `#47`, `#48`, `#50`, `#51`, `#52`, `#53`, `#54`, `#55`, `#56`, `#57`

### Wave B — `#47` + `#50` Implementation
- [x] Add failing regression coverage for the expanded `ErrorResponse` contract:
  - `status`
  - `message`
  - `trace_id`
  - preserve legacy `error`
- [x] Extract the inline API error mapping into a dedicated `error_mapper` module.
- [x] Move the shared `ErrorResponse` schema into the new module and update OpenAPI exports.
- [x] Migrate `ApiError`, inline handler call sites, auth middleware, and rate-limit middleware to the new mapper/schema.
- [x] Remove `engine_error_to_response()` from `lib.rs` once all call sites are migrated.
- [x] Run fresh verification for:
  - `cargo test -p noesis-api --test error_handling_tests -- --nocapture`
  - `cargo test -p noesis-api --test rate_limit_tests -- --nocapture`
  - `cargo test -p noesis-api --test workflow_tests -- --nocapture`
  - `cargo test -p noesis-api error_mapper -- --nocapture`
  - `cargo build -p noesis-api`
  - `rg -n "engine_error_to_response" crates/noesis-api/src`
- Review:
  - `#47` is now closeable:
    - `ErrorResponse` lives in `crates/noesis-api/src/error_mapper.rs`
    - schema now includes `status`, `error_code`, `message`, `error`, `details`, and `trace_id`
    - response-shape assertions were extended in `crates/noesis-api/tests/error_handling_tests.rs` and `crates/noesis-api/tests/rate_limit_tests.rs`
  - `#50` is now closeable:
    - `engine_error_to_response()` was removed from `crates/noesis-api/src/lib.rs`
    - all migrated call sites now use `ErrorMapper::map()` or `ErrorMapper::response()`
    - `rg` confirms no source references remain
  - Keep open after this pass:
    - `#48`, `#51`, `#52`, `#53`, `#54`, `#55`, `#56`, `#57`
- Closeout record:
  - commit: `9b049161` `fix(api): add unified error mapper module`
  - push target: `origin/codex/engine-hygiene-native-runtime-docs`
  - issue comments:
    - `#47`: `issuecomment-4038797864`
    - `#50`: `issuecomment-4038799068`
  - issue state:
    - closed: `#47`, `#50`
    - still open: `#48`, `#51`, `#52`, `#53`, `#54`, `#55`, `#56`, `#57`

### Issue Reduction Sweep — 2026-03-11
- [x] Inventory the current open issue set and group it into closeable, implement-now, and defer buckets.
- [x] Review Wave 1 / Wave 2 clusters already touched in this branch and identify issues already satisfied by shipped code.
- [x] Close the issues that are already proven complete by code, tests, or merged workflow/docs updates.
- [x] Record a manageable next-issue set for the remaining open clusters.
- [x] Add a sweep review summary with counts, closures, and the reduced backlog shape.
- Review:
  - Open backlog at sweep start: `416`
  - Largest remaining milestone buckets:
    - `P5-Release-Readiness`: `70`
    - `P4-Performance-Observability`: `70`
    - `P2-Workflow-Hardening`: `65`
    - `P1-Stabilization`: `65`
    - `v2.2.0-Specialized-Engines`: `61`
    - `P3-Bridge-Reliability`: `52`
  - Closed in this sweep:
    - `#502`, `#503`, `#507` from engine-hygiene follow-up
  - Closed immediately before the sweep inventory:
    - `#47`, `#50` from Wave B error-mapping
  - Not closeable on evidence yet:
    - `#48`, `#51`, `#52`, `#53`, `#54`, `#55`, `#56`, `#57`
    - `#504`, `#505`, `#506`, `#508`
    - most of `P5`, `P4`, `P2`, and `P3` still require real implementation rather than admin cleanup
  - Reduced backlog shape after the three sweep closures:
    - live open count: `413`
  - Next manageable issue set:
    - `#52`, `#54`, `#55`, `#56` as the next small P1 error-mapping tranche
    - `#504`, `#505`, `#506`, `#508` as the remaining engine-hygiene tranche
    - workflow / registry candidates to audit next:
      - `#400`, `#401`, `#416`, `#417`, `#419`
  - Closeout record:
    - commits:
      - `9b049161` `fix(api): add unified error mapper module`
      - `805a4c35` `docs(tasks): record Wave B issue closures`
      - `79d47d69` `fix(vedic-clock): expose resolved timezone basis`
    - issue comments:
      - `#502`: `issuecomment-4039333979`
      - `#503`: `issuecomment-4039333989`
      - `#507`: `issuecomment-4039333996`

### Backlog Reduction Tranche — `#52`, `#54`, `#55`, `#56`
- [x] Add explicit `EngineError` exhaustiveness coverage around `ErrorMapper` for all current variants.
  - Added a compile-time exhaustive `match` helper and a variant coverage test in `crates/noesis-api/src/error_mapper.rs`.
- [x] Make request log trace correlation deterministic so error responses reuse the same request trace ID.
  - `request_logging_middleware()` now generates a request `trace_id`, injects it into the tracing span, and scopes it through `ErrorMapper::with_request_trace_id()` in `crates/noesis-api/src/middleware.rs`.
  - Added log-correlation proof in `crates/noesis-api/tests/error_trace_correlation_tests.rs`.
- [x] Add Insta snapshot coverage for all current `EngineError` response shapes with stable scrubbed fields.
  - Added `crates/noesis-api/tests/error_response_snapshot_tests.rs`.
  - Generated 12 snapshots in `crates/noesis-api/tests/snapshots/` for the current `EngineError` variant set.
- [x] Add Prometheus API error counter keyed by `error_code` and verify it increments via `/metrics`.
  - Added `noesis_api_errors_total{error_code=...}` in `crates/noesis-metrics/src/lib.rs`.
  - Wired `ErrorMapper` to increment it on every structured error response in `crates/noesis-api/src/error_mapper.rs`.
  - Added `/metrics` delta verification in `crates/noesis-api/tests/error_metrics_tests.rs`.
- [x] Run targeted verification for:
  - `cargo test -p noesis-api error_exhaustiveness -- --nocapture`
  - `cargo test -p noesis-api --test error_handling_tests -- --nocapture`
  - `INSTA_UPDATE=always cargo test -p noesis-api --test error_response_snapshot_tests -- --nocapture`
  - `cargo test -p noesis-api --test error_metrics_tests -- --nocapture`
  - `cargo test -p noesis-api --test error_trace_correlation_tests -- --nocapture`
  - `cargo test -p noesis-api --test error_metrics_tests --test error_trace_correlation_tests --test error_response_snapshot_tests -- --nocapture`

### Backlog Reduction Tranche — Review
- `#52` is now satisfied.
  - The `ErrorMapper` unit test will stop compiling if a new `EngineError` variant is added without updating the exhaustive `match`.
- `#54` is now satisfied.
  - Router-level integration test proves the JSON `trace_id` appears in captured request logs for the same failing request.
- `#55` is now satisfied.
  - Snapshot coverage now exists for every current `EngineError` response shape, using a fixed scoped `trace_id`.
- `#56` is now satisfied.
  - `/metrics` now exposes `noesis_api_errors_total`, and the integration test verifies per-`error_code` increments.

### Backlog Reduction Tranche — `#48`, `#51`, `#53`, `#57`
- [x] Re-audit `#48` against the current `BridgeError` enum and update the propagation document with all six variants plus API-layer outcomes.
  - Updated `docs/planning/wave-b-p1-error-mapping-audit-2026-03-11.md` with the live six-variant `BridgeError` table and API-layer status outcomes.
- [x] Normalize TS bridge `BridgeError` -> `EngineError` translation so the live bridge path uses the structured enum instead of ad-hoc strings.
  - `crates/noesis-bridge/src/lib.rs` now constructs `BridgeError` variants in the TS bridge path before converting them at the trait boundary.
- [x] Add unit coverage for each `BridgeError` variant preserving the expected context string and `EngineError` category.
  - Added coverage in `crates/noesis-bridge/src/error.rs`.
- [x] Add Sentry breadcrumb/event split in `ErrorMapper`: 4xx breadcrumb-only, 5xx breadcrumb + event capture with `error_code` and `trace_id`.
  - Implemented in `crates/noesis-api/src/error_mapper.rs`.
- [x] Add Sentry tests with a captured transport proving 5xx event capture and 4xx breadcrumb-only behavior.
  - Added unit coverage in `crates/noesis-api/src/error_mapper.rs` using Sentry’s captured-event test transport.
- [x] Decide `#57` from current contract evidence:
  - if `WorkflowResult` still intentionally returns `200` with partial `engine_outputs`, leave `#57` open and document why
  - only close it if the HTTP contract actually changes to `207 Multi-Status`
  - `WorkflowResult` in `crates/noesis-core/src/types.rs` still has only `engine_outputs`, `synthesis`, `total_time_ms`, and `timestamp`.
  - Current docs and tests still treat partial workflow success as `200 OK` with partial `engine_outputs`.
- [x] Run targeted verification for:
  - `cargo test -p noesis-bridge --lib -- --nocapture`
  - `cargo test -p noesis-api error_mapper -- --nocapture`
  - `cargo test -p noesis-api --test error_handling_tests -- --nocapture`

### Backlog Reduction Tranche — Review
- `#48` is now satisfied by the refreshed audit document and the live bridge translation path.
- `#51` is now satisfied by the breadcrumb/event split plus captured Sentry tests.
- `#53` is now satisfied by the structured `BridgeError` translation and unit coverage.
- `#57` remains open.
  - It still requires a deliberate API contract change from `200` partial workflow responses to `207 Multi-Status` with per-engine error details.

### Backlog Reduction Tranche — `#29`, `#30`, `#31`, `#32`, `#35`, `#36`, `#37`, `#106`
- [x] Create `docs/baseline/engine-matrix.json` covering the Rust engine crates and TypeScript sidecar engines with current versions/phases/source references.
- [x] Create a workflow parity artifact documenting the 6 canonical workflows, engine membership, required phases, and synthesis path.
- [x] Create an environment parity checklist documenting all `ApiConfig::from_env()` variables across local, CI, and Railway assumptions.
- [x] Add/update orchestrator routing invariant doc comments on the key orchestrator types for `#35`.
- [x] Add trait conformance coverage for the Rust engines that verifies non-empty identity metadata, bounded phase values, and deterministic cache keys.
- [x] Create a crate dependency graph artifact (Mermaid + JSON summary) from current workspace metadata.
- [x] Add the Redis degradation/fallback runbook for `#106`.
- [x] Run targeted verification for:
  - `cargo test -p noesis-orchestrator --test trait_conformance_tests -- --nocapture`
  - `cargo test -p noesis-orchestrator --test baseline_artifact_tests -- --nocapture`
  - `cd ts-engines && bun test tests/health.test.ts tests/baseline_registry.test.ts`
  - `cargo test -p noesis-api test_from_env_uses_server_host_alias -- --nocapture`
  - `cargo test -p noesis-api test_from_env_uses_server_port_alias -- --nocapture`
  - `cargo run -p noesis-api --bin validate_config -- --dry-run`
  - `cargo test -p engine-face-reading test_cache_key_without_seed -- --nocapture`
  - `cargo doc -p noesis-orchestrator --no-deps`
  - `rg -n "Routing invariant" target/doc/noesis_orchestrator -g '*.html'`
- [x] Leave `#33` and `#34` out of this tranche unless route inventory and orchestrator test-harness evidence fall out naturally from the work.

### Backlog Reduction Tranche — Review
- `#29` is now satisfied by `docs/baseline/engine-matrix.json` plus `baseline_artifact_tests.rs` version checks against workspace manifests.
- `#30` is now satisfied by the TS section of `docs/baseline/engine-matrix.json` plus `ts-engines/tests/baseline_registry.test.ts` proving the sidecar registers exactly 5 engines and reports healthy readiness.
- `#31` is now satisfied by `docs/baseline/workflow-parity.json` and the synchronized correction in `docs/api/workflows.md` from `gene-keys` to `vimshottari` for `birth-blueprint`.
- `#32` is now satisfied by `docs/baseline/env-parity.md`, `validate_config -- --dry-run`, the CI audit step in `.github/workflows/test.yml`, and `ApiConfig` compatibility aliases for `SERVER_HOST` / `SERVER_PORT`.
- `#35` is now satisfied by the routing invariant doc comments on `EngineRegistry`, `WorkflowOrchestrator`, and `WorkflowExecutor`, plus `cargo doc` / generated HTML grep evidence.
- `#36` is now satisfied by 11 engine-specific trait conformance tests in `crates/noesis-orchestrator/tests/trait_conformance_tests.rs`.
- `#36` also exposed and fixed a real bug: `engine-face-reading` used timestamp-based cache keys for unseeded inputs, which broke determinism.
- `#37` is now satisfied by `docs/baseline/dependency-graph.json`, `docs/baseline/dependency-graph.md`, and metadata-backed validation in `baseline_artifact_tests.rs`.
- `#106` is now satisfied by `docs/runbooks/redis-degradation.md`, grounded in current readiness fields (`redis`, `overall_status`, `redis_available`) and the actual L2 Redis warning messages.
- Closure record:
  - committed in `d56ac355` (`docs(baseline): land Wave B baseline audit tranche`)
  - pushed to `origin/codex/engine-hygiene-native-runtime-docs`
  - closed with evidence comments:
    - `#29`
    - `#30`
    - `#31`
    - `#32`
    - `#35`
    - `#36`
    - `#37`
    - `#106`

### Next Wave 2 Candidate — `#323`

- [x] Review open Wave 2 issues and select the smallest live candidate after the baseline tranche.
- [x] Expand the Redis degradation notes into the incident-style runbook requested by `#323`.
- [x] Verify the runbook matches the current Redis graceful-degradation behavior and readiness/admin surfaces.
- [x] Commit, push, and close `#323` with evidence if the acceptance criteria are satisfied.

#### `#323` Selection Note

- `#323` is the tightest Wave 2 issue because it overlaps the Redis degradation work already landed for `#106` but asks for a more specific incident runbook path and operator procedure.
- I am intentionally not branching into `#322` or `#324` yet; those require new API-down / TS-bridge incident procedures that are not already mostly captured.
- Evidence for `#323` now lives in:
  - `docs/runbooks/incident-redis-failure.md`
  - `docs/runbooks/redis-degradation.md` cross-link
  - `crates/noesis-cache/src/l2_cache.rs` Redis graceful-degradation log/fallback behavior
  - `crates/noesis-api/src/lib.rs` readiness fields
  - `crates/noesis-api/src/handlers/admin.rs` admin `redis_available` status
- Closure record:
  - committed in `527fc5ad` (`docs(runbooks): add Redis incident procedure for Wave 2`)
  - pushed to `origin/codex/engine-hygiene-native-runtime-docs`
  - closed with evidence comment: `#323`

### Next Wave 2 Candidate — `#324`

- [x] Review `#324` requirements against the current bridge health/readiness surface.
- [x] Write the TS sidecar bridge incident runbook at the requested path.
- [x] Verify the runbook against `BridgeManager::health_check()`, bridge readiness state, and the 11-Rust-engine fallback boundary.
- [x] Commit, push, and close `#324` if the runbook honestly satisfies the issue acceptance criteria.

#### `#324` Selection Note

- `#324` is the next tightest Wave 2 issue because the bridge readiness surface already exists:
  - `BridgeManager::health_check()`
  - `BridgeManager::readiness_status()`
  - `/ready` fields `bridge_status`, `bridge_engines`, and `bridge_failed_engines`
- Important constraint:
  - the current bridge does **not** expose a dedicated circuit breaker object or state machine
  - the runbook must document the operational equivalent using the existing `available` / `degraded` / unavailable bridge states rather than inventing a feature that is not shipped
- Verification used for `#324`:
  - `cargo test -p noesis-api --test bridge_readiness_tests -- --nocapture`
  - locked engine counts from `docs/baseline/engine-matrix.json` (`11` Rust, `5` TS)
  - source checks for:
    - `BridgeManager::health_check()`
    - `BridgeManager::readiness_status()`
    - `/ready` fields `bridge_status` and `bridge_failed_engines`
- Closure record:
  - committed in `2d5fadd0` (`docs(runbooks): add TS bridge incident procedure`)
  - pushed to `origin/codex/engine-hygiene-native-runtime-docs`
  - closed with evidence comment: `#324`

### Batch Attempt — `#326`, `#327`, `#328`

- [x] Inspect `#326`, `#327`, and `#328` as a shared non-critical docs batch.
- [x] Determine which issues are actually closeable from the current code surface.
- [x] Commit, push, and close only the issues honestly satisfied by the shared runbook/docs work.

#### Batch Review Note

- `#326` is **not** closeable yet:
  - the acceptance criteria ask for per-engine timeout settings and Sentry breadcrumb correlation at the orchestrator incident level
  - current orchestrator behavior does cover partial engine failures, but the issue asks for a richer timeout/observability runbook than we have today
- `#327` is **not** closeable yet:
  - it explicitly depends on the broader runbook set being complete
- `#328` is closeable if the TS bridge runbook explicitly covers the top five bridge failure modes:
  - timeout spike
  - sidecar crash loop
  - effective open-state bridge outage
  - TS engine memory leak / resource exhaustion
  - bridge schema mismatch
- Closure record:
  - runbook expanded to cover all five requested bridge failure modes
  - `#328` closed
  - `#326` and `#327` intentionally left open

### Next Non-Critical Closure Wave

- [x] Review open non-critical Wave 2 issues for a docs/runbook/policy batch that the current codebase can support honestly.
- [x] Implement the shared runbook batch for:
  - `#322` API down
  - `#325` DB pool exhaustion
  - `#326` workflow timeout / partial engine failure (supporting artifact even if the issue itself may stay open)
  - `#327` runbook index / quick reference
- [ ] Evaluate `#314` canary rollout policy as an optional fifth closure in the same wave.
- [x] Verify each candidate issue against current code/docs and close only the ones that are fully satisfied.

#### Wave Selection Note

- The next realistic closure wave is smaller than the ideal `10-15` issue target.
- The best current batch is the remaining non-critical runbook/docs surface because:
  - it shares the same touch set (`docs/runbooks/`, `docs/troubleshooting.md`, deployment/monitoring references)
  - it depends on existing readiness, Railway, auth, and DB-degraded-mode behavior that already exists in the repo
  - it avoids the heavier implementation risk in canary automation, idempotency, or specialized-engine roadmap items
- Current execution scope for this wave:
  - target closures: `#322`, `#325`, `#327`
  - supporting artifact: `#326` runbook document may be created to complete the runbook set, but `#326` itself is not assumed closable
  - deferred from this wave: `#329`, because current code/docs do not show `JWKS` support
- Closure record:
  - committed in `d6e91eb7` (`docs(runbooks): add API and DB incident wave`)
  - pushed to `origin/codex/engine-hygiene-native-runtime-docs`
  - closed:
    - `#322`
    - `#325`
    - `#327`
  - intentionally left open:
    - `#326`
    - `#329`

### Next Workflow Parity / Policy Wave

- [x] Confirm `.DS_Store` is already ignored and untrack the root workspace copy from git.
- [x] Review `#73`, `#74`, and `#314` against the current startup, CI, and monitoring surfaces.
- [x] Implement startup workflow registry parity check for `#73` using the actual 6 canonical workflow IDs currently shipped by `WorkflowOrchestrator::new()`.
- [x] Add a CI workflow parity gate for `#74` so drift fails the pipeline explicitly.
- [x] Add `docs/runbooks/canary-rollout-policy.md` for `#314` with thresholds grounded in the existing monitoring and deploy docs.
- [x] Verify targeted tests/docs, commit only the relevant files plus the `.DS_Store` untrack, and close only the issues fully satisfied by this wave.

#### Wave Selection Note

- This wave is preferable to broader docs batching because it is tied to code and CI surfaces we can prove:
  - `build_app_state()` / `build_app_state_lazy_db()`
  - `WorkflowOrchestrator::default_workflows()`
  - `.github/workflows/test.yml`
  - `docs/deployment/monitoring.md`
- Important constraint:
  - the canonical workflow set in the code today is:
    - `birth-blueprint`
    - `daily-practice`
    - `decision-support`
    - `self-inquiry`
    - `creative-expression`
    - `full-spectrum`
  - I will not close anything against older workflow names that are no longer the runtime baseline.
- Scope control:
  - `.DS_Store` cleanup is part of this wave because it reduces branch noise and should be committed with the next safe batch
  - `#314` is closeable as a shipped policy document, but the human review/signoff it mentions remains an operational follow-through item rather than an in-repo test

#### Verification

- `cargo test -p noesis-api workflow_parity -- --nocapture`
- `cargo run -p noesis-api --bin validate_workflow_parity`

#### Closure Record

- `.DS_Store` is now untracked while remaining ignored via `.gitignore`
- startup parity is logged from both:
  - `build_app_state()`
  - `build_app_state_lazy_db()`
- dedicated parity module and CLI landed in:
  - `crates/noesis-api/src/workflow_parity.rs`
  - `crates/noesis-api/src/bin/validate_workflow_parity.rs`
- CI parity gate landed in:
  - `.github/workflows/test.yml`
- canary policy landed in:
  - `docs/runbooks/canary-rollout-policy.md`

### Next Smoke / Deploy Wave

- [x] Review the `P5-W1` smoke/deploy cluster (`#291`-`#299`) against the current deploy workflow, auth requirements, and live API contract.
- [x] Implement a generic smoke runner script that emits structured JSON, exits non-zero on failure, and supports protected endpoint auth via `SMOKE_TEST_JWT` or `SMOKE_TEST_API_KEY`.
- [x] Add smoke checks for:
  - health live/ready
  - engines list
  - panchanga calculation
  - birth-blueprint workflow execution
  - metrics endpoint
  - TS bridge calculation through Rust
- [x] Add or align the minimal API readiness surface needed for the smoke runner contract (notably `postgres` readiness reporting).
- [x] Wire the smoke runner into `.github/workflows/deploy.yaml` as the post-deploy smoke gate.
- [x] Close only the issues whose acceptance criteria match the shipped contract exactly; keep contract-mismatch issues open.

#### Wave Selection Note

- The best closure yield now is the smoke/deploy batch because one implementation can plausibly satisfy:
  - `#291`
  - `#292`
  - `#293`
  - `#294`
  - `#295`
  - `#296`
  - `#298`
  - `#299`
- `#297` is questionable because its acceptance criteria mention `envelope_version`, which is not part of the current Rust `EngineOutput` contract. I will not close it unless the delivered behavior truly matches that requirement.

#### Smoke / Deploy Batch Review

- The runner now produces a structured JSON report on both the happy path and the failure path.
- Verified pass path with a mock target returning:
  - `/health/live`
  - `/health/ready`
  - `/api/v1/engines`
  - `/api/v1/engines/panchanga/calculate`
  - `/api/v1/workflows/birth-blueprint/execute`
  - `/metrics`
  - `/api/v1/engines/tarot/calculate`
- Verified failure path with `http://127.0.0.1:9`, which now exits `1` and still emits a complete failing JSON report.
- Honest closure set from this wave:
  - `#291`
  - `#292`
  - `#293`
  - `#294`
  - `#296`
  - `#298`
  - `#299`
- Closed on GitHub with commit-backed evidence comments after push of `0b68cee5`.
- Keep open:
  - `#295` because the public workflow contract is `engine_outputs`, not `engine_results`
  - `#297` because `EngineOutput` does not include `envelope_version`
  - `#305` because Docker health status is configured but auto-restart on unhealthy status is not proven by the current runtime/deploy setup
- Backlog count after this wave: `382` open issues.

#### Verification

- `cargo test -p noesis-api --test bridge_readiness_tests -- --nocapture`
- `bash -n scripts/smoke-test-runner.sh`
- `SMOKE_TEST_JWT=smoke-token bash scripts/smoke-test-runner.sh http://127.0.0.1:8770`
- `SMOKE_TEST_JWT=smoke-token bash scripts/smoke-test-runner.sh http://127.0.0.1:9`

### Next Contract / Proof Mini-Wave

- [x] Re-audit `#295`, `#297`, and `#305` against the live workflow, bridge, and Docker contracts.
- [x] Add a backward-compatible workflow response alias only if it can be done at the API boundary without disturbing the orchestrator/core types.
- [x] Add a bridge response envelope shim only if it can be done at the API boundary without mutating every native engine output producer.
- [x] Extend the smoke runner only if needed to satisfy `warn` semantics for the TS bridge check.
- [x] Re-check whether `#305` can be closed with real Docker proof; otherwise leave it open with no speculative closure.
- [x] Close only the issues whose acceptance criteria are actually satisfied after fresh verification.

#### Mini-Wave Note

- `#295` is currently blocked by `engine_results` vs `engine_outputs`.
- `#297` is currently blocked by missing `envelope_version` and missing `warn` handling in the smoke runner.
- `#305` remains a proof problem unless the current Docker/Compose behavior can be demonstrated end-to-end.

#### Mini-Wave Review

- `#295` is now satisfied via an API-boundary compatibility alias:
  - workflow execution responses now include both `engine_outputs` and `engine_results`
  - core/orchestrator `WorkflowResult` stayed unchanged
- `#297` is now satisfied via an API-boundary envelope shim plus smoke-runner warn semantics:
  - engine calculation responses now include `envelope_version: "1"`
  - the smoke runner now marks TS bridge outages as `warn` when the bridge returns a degraded error
- `#305` still stays open:
  - Docker and Docker Compose are available locally
  - the repo has a `HEALTHCHECK` in `Dockerfile.prod`
  - but the current Compose/restart model still does not prove auto-restart on unhealthy status, so closing it would be speculative
- Closed on GitHub after push of `d7b99088`:
  - `#295`
  - `#297`
- Backlog count after this mini-wave: `380` open issues.

#### Mini-Wave Verification

- `cargo test -p noesis-api --test integration_tests test_calculate_panchanga_success -- --nocapture`
- `cargo test -p noesis-api --test integration_tests test_workflow_execute_birth_blueprint_success -- --nocapture`
- `bash -n scripts/smoke-test-runner.sh`
- `SMOKE_TEST_JWT=smoke-token bash scripts/smoke-test-runner.sh http://127.0.0.1:8771`
  healthy mock with `engine_results` and `envelope_version`
- `SMOKE_TEST_JWT=smoke-token bash scripts/smoke-test-runner.sh http://127.0.0.1:8772`
  degraded bridge mock with `ts_bridge=warn`
- `docker --version`
- `docker compose version`
