# Task Plan — BF1 Completion (BF1-06 / BF1-07 / BF1-08)

## Checklist
- [x] Restore authoritative BF verification footing and record the DB/bootstrap state.
- [x] Implement BF1-06 reading history/detail APIs plus artifact linkage persistence.
- [x] Align the shared biofield domain + typed client contract for paginated history responses.
- [x] Implement BF1-07 real history and reading detail UI in `apps/biofield-web`.
- [x] Implement BF1-08 smoke/runbook proof for the finished Phase 1 biofield slice.
- [x] Run backend, frontend, smoke, and `git diff --check` verification.
- [x] Summarize exactly what landed, what was verified, and any remaining blockers.

## Notes
- Finish Phase 1 surgically on `bf1-web-first-tranche-1`; no unrelated cleanup.
- Local Postgres at `localhost:5432` was previously down, so DB-backed verification must be re-established before claiming full BF completion.

## Review (fill after execution)
- Scope:
  - completed the remaining BF1 Phase 1 surface on the dedicated BF1 branch/worktree
  - kept the changes limited to biofield backend/history/detail, shared client/domain seams, and smoke/runbook proof
- Verification footing / DB state:
  - confirmed local Docker was initially down, then launched Docker Desktop
  - confirmed the machine already had a healthy `noesis-postgres` container bound to `localhost:5432`
  - verified DB reachability with:
    - `PGPASSWORD=noesis_password psql 'postgresql://noesis_user@localhost:5432/noesis' -c 'select 1 as ok;'`
- BF1-06 backend outcome:
  - completed `GET /api/v1/biofield/readings` in:
    - `crates/noesis-api/src/handlers/biofield.rs`
    - now returns real user-scoped paginated reading summaries
  - completed `GET /api/v1/biofield/readings/:reading_id` in:
    - `crates/noesis-api/src/handlers/biofield.rs`
    - now returns real user-scoped reading detail payloads
  - upgraded capture persistence in:
    - `crates/noesis-api/src/handlers/biofield.rs`
    - now creates a source artifact row in `biofield_capture_artifacts`
    - keeps rejected/unlinked captures as artifact metadata rows
    - links successful captures to the persisted reading row
    - returns real artifact IDs + storage paths in capture/history/detail responses
  - removed the last stale placeholder schema usage in:
    - `crates/noesis-api/src/lib.rs`
  - extended router-level coverage in:
    - `crates/noesis-api/tests/biofield_capture_proxy.rs`
    - added assertions for:
      - artifact linkage on successful capture
      - unlinked artifact metadata on rejected quality capture
      - user-scoped history list
      - user-scoped reading detail
- Shared contract / package outcome:
  - added paginated history response typing in:
    - `packages/biofield-domain/src/index.ts`
  - aligned the typed client in:
    - `packages/biofield-api-client/src/biofield-client.ts`
    - `packages/biofield-api-client/src/biofield-client.test.ts`
    - `packages/biofield-api-client/tsconfig.json`
  - `listReadings(...)` now returns the real paginated BF1 response shape instead of a bare array
- BF1-07 frontend outcome:
  - replaced the placeholder history page with a real authenticated data-backed view in:
    - `apps/biofield-web/app/(protected)/history/page.tsx`
    - `apps/biofield-web/src/components/biofield-history-page.tsx`
  - replaced the placeholder reading detail page with a real authenticated detail view in:
    - `apps/biofield-web/app/(protected)/readings/[readingId]/page.tsx`
    - `apps/biofield-web/src/components/biofield-reading-detail-page.tsx`
  - tightened the viewer handoff in:
    - `apps/biofield-web/app/(protected)/viewer/page.tsx`
    - latest capture panel now links directly into reading detail and history
  - extended the web styles for the finished Phase 1 surfaces in:
    - `apps/biofield-web/app/globals.css`
- BF1-08 smoke / runbook outcome:
  - upgraded the smoke script in:
    - `scripts/smoke_biofield_web.sh`
    - it now:
      - self-registers a temporary user
      - logs in
      - creates a session
      - uploads a generated capture
      - verifies history and detail routes
      - closes the session
  - updated the operational proof in:
    - `docs/runbooks/biofield-capture-analysis.md`
  - important implementation note:
    - the sidecar stub quality gate is currently size-based, so the smoke script now generates a >100 KB PNG payload to deterministically pass the current quality threshold
- Verification:
  - `npm --prefix packages/biofield-api-client test` ✅
  - `npm --prefix packages/biofield-api-client run typecheck` ✅
  - `npm --prefix apps/biofield-web run typecheck` ✅
  - `npm --prefix apps/biofield-web run lint` ✅
  - `npm --prefix apps/biofield-web run build` ✅
  - `cargo test -p noesis-api --test biofield_handler_smoke -- --nocapture` ✅
  - `cargo test -p noesis-api --test biofield_capture_proxy --no-run` ✅
  - `DATABASE_URL=postgresql://noesis_user:noesis_password@localhost:5432/noesis cargo test -p noesis-api --test biofield_capture_proxy -- --nocapture` ✅
  - `DATABASE_URL=postgresql://noesis_user:noesis_password@localhost:5432/noesis cargo test -p noesis-api --test biofield_session_lifecycle -- --nocapture` ✅
  - `BIOFIELD_WEB_URL=http://127.0.0.1:3002 API_BASE_URL=http://127.0.0.1:8080 PYTHON_BIOFIELD_URL=http://127.0.0.1:8002 bash scripts/smoke_biofield_web.sh` ✅
  - `git diff --check` ✅
- Remaining blocker note:
  - no code blocker remains for BF1 Phase 1 in this worktree
  - the only operational wrinkle encountered was environment setup (Docker daemon + smoke image size threshold), both of which were resolved during verification

# Task Plan — BF1 Implementation Tranche 2

## Checklist
- [x] Continue safely on the dedicated BF1 branch/worktree and record tranche 2 in the tracker.
- [x] Implement BF1-04 / BWF-020 biofield-web auth + session bootstrap.
- [x] Implement BF1-05 / BWF-024-025 capture upload route and sidecar proxy in `noesis-api`.
- [x] Wire the viewer UI to create/close sessions and upload a capture through the typed API client.
- [x] Run frontend, Rust, and relevant Python verification plus `git diff --check`.
- [x] Summarize exactly what landed and what remains for BF1-06/BF1-07.

## Notes
- This tranche is intentionally limited to authenticated web shell + upload/proxy success.
- Persistence of readings/history/detail remains the next tranche unless a minimal capture response requires a tiny seam adjustment.

## Review (fill after execution)
- Scope:
  - continued on the dedicated BF1 worktree / branch from tranche 1
  - kept the tranche focused on BF1-04 frontend shell + BF1-05 capture route/proxy
- BF1-04 / BWF-020 outcome:
  - added `apps/biofield-web/src/lib/config.ts` with frontend-safe API base URL normalization
  - added `apps/biofield-web/src/lib/auth.ts` with localStorage-backed auth session storage + subscription helpers
  - added `apps/biofield-web/src/lib/api.ts` with:
    - `login(...)`
    - typed `BiofieldClient` construction for bearer-auth biofield calls
  - replaced the placeholder login page in:
    - `apps/biofield-web/app/(public)/login/page.tsx`
    - now supports real email/password login against `/api/v1/auth/login`
  - replaced the protected layout in:
    - `apps/biofield-web/app/(protected)/layout.tsx`
    - now acts as a real auth gate with sign-out support
  - replaced the viewer shell in:
    - `apps/biofield-web/app/(protected)/viewer/page.tsx`
    - now supports:
      - start session
      - close session
      - choose image file
      - upload capture
      - render returned analysis summary
  - extended `apps/biofield-web/app/globals.css` with the form, status, and detail styles needed for the new shell
- BF1-05 / BWF-024-025 outcome:
  - fixed the workspace source export in:
    - `packages/biofield-api-client/src/index.ts`
  - replaced the capture placeholder in:
    - `crates/noesis-api/src/handlers/biofield.rs`
    - new route now:
      - parses multipart form-data
      - accepts `image` or `file`
      - validates and parses `algorithms`, `options`, and `capture_metadata`
      - enforces session ownership and active-session checks
      - proxies the capture to the Python sidecar through `BiofieldClient`
      - normalizes quality rejection into `BIOFIELD_CAPTURE_REJECTED_QUALITY`
      - persists a minimal reading row so the response can return a stable `reading_id`
      - returns a typed capture response for the frontend viewer
  - added router-level BF1-05 coverage in:
    - `crates/noesis-api/tests/biofield_capture_proxy.rs`
    - covers:
      - missing image field
      - missing session
      - inactive session
      - successful sidecar proxy + reading persistence
      - quality rejection normalization
  - registered the new capture response schema in:
    - `crates/noesis-api/src/lib.rs`
- Verification:
  - `cargo test -p noesis-api --test biofield_handler_smoke -- --nocapture` ✅
  - `npm --prefix apps/biofield-web install` ✅
  - `npm --prefix apps/biofield-web run typecheck` ✅
  - `npm --prefix apps/biofield-web run lint` ✅
  - `npm --prefix apps/biofield-web run build` ✅
  - `git diff --check` ✅
  - attempted DB-backed verification for:
    - `DATABASE_URL=postgresql://noesis_user:noesis_password@localhost:5432/noesis cargo test -p noesis-api --test biofield_capture_proxy -- --nocapture`
    - `DATABASE_URL=postgresql://noesis_user:noesis_password@localhost:5432/noesis cargo test -p noesis-api --test biofield_session_lifecycle -- --nocapture`
  - current blocker:
    - local Postgres at `localhost:5432` is unavailable in this environment (`psql` returns connection refused), so those integration tests could not execute authoritatively
- Tranche boundary note:
  - a minimal reading row is now created during successful capture so the route can return a stable `reading_id`
  - full history/detail persistence surfaces and artifact linkage are still deferred to the next tranche
- Remaining next tranche:
  - BF1-06 persist reading/artifact linkage fully and expose history/detail APIs
  - BF1-07 render real history and reading detail in `biofield-web`
  - BF1-08 vertical-slice smoke/runbook proof

# Task Plan — BF1 Implementation Tranche 1

## Checklist
- [x] Create a clean BF1 implementation worktree/branch and verify it is isolated from unrelated dirty changes.
- [x] Record the user-correction lesson and this tranche in the repo trackers before editing code.
- [x] Implement BF1-02 / BWF-018-019 Python sidecar contract alignment.
- [x] Implement BF1-03 / BWF-023 authenticated biofield session lifecycle in `noesis-api`.
- [x] Run targeted Python + Rust verification and `git diff --check`.
- [x] Summarize exactly what landed and what remains for the next BF1 tranche.

## Notes
- This tranche intentionally focuses on the two highest-leverage backend blockers still left as stub/scaffold work:
  - Python sidecar contract parity
  - Rust biofield session create/get/close handlers
- Capture upload, reading persistence flow, and frontend bootstrap beyond placeholder state are explicitly deferred to the next tranche.

## Review (fill after execution)
- Scope:
  - isolated the BF1 tranche in the dedicated worktree/branch:
    - `.worktrees/bf1-web-first-tranche-1`
    - branch: `bf1-web-first-tranche-1`
  - recorded the user correction in `tasks/lessons.md` before implementation
- BF1-02 / BWF-018-019 outcome:
  - added `contract_version` and `analysis_version` to the shared Python biofield response model in:
    - `python-services/shared/models.py`
  - upgraded `python-services/biofield_cv_service/analyze.py` to:
    - emit `contract_version = "biofield-cv/v1"`
    - emit `analysis_version = "stub-metrics/v1"`
    - require the `quality_assessment` shape on successful responses
    - reject malformed `algorithms` JSON
    - reject non-array `algorithms`
    - reject unknown algorithm names
    - reject malformed / non-object `options`
    - reject malformed / non-object `capture_metadata`
  - expanded `python-services/tests/test_biofield_analyze.py` to cover:
    - required top-level contract fields
    - required quality-assessment fields
    - malformed/invalid payload behavior
    - retained deterministic/stub metric behavior
- BF1-03 / BWF-023 outcome:
  - added `close_session(...)` to `crates/noesis-data/src/repositories/biofield_repository.rs`
  - replaced the `501` placeholders in `crates/noesis-api/src/handlers/biofield.rs` for:
    - `POST /api/v1/biofield/sessions`
    - `GET /api/v1/biofield/sessions/:session_id`
    - `POST /api/v1/biofield/sessions/:session_id/close`
  - new handler behavior now includes:
    - authenticated user UUID parsing
    - repository-backed create/get/close flow
    - user ownership enforcement
    - `422` invalid UUID handling
    - `404 BIOFIELD_SESSION_NOT_FOUND`
    - `409 BIOFIELD_SESSION_NOT_ACTIVE`
    - `503 BIOFIELD_DB_UNAVAILABLE`
    - `500 BIOFIELD_DB_ERROR`
  - added focused router-level integration coverage in:
    - `crates/noesis-api/tests/biofield_session_lifecycle.rs`
- Verification:
  - `python3.11 -m venv <tmp> && pip install -e 'python-services[dev]' && pytest python-services/tests/test_biofield_analyze.py` ✅
  - `DATABASE_URL=postgresql://noesis_user:noesis_password@localhost:5432/noesis cargo test -p noesis-data biofield_repository -- --nocapture` ✅
  - `DATABASE_URL=postgresql://noesis_user:noesis_password@localhost:5432/noesis cargo test -p noesis-api --test biofield_session_lifecycle -- --nocapture` ✅
  - `cargo test -p noesis-api --test biofield_handler_smoke -- --nocapture` ✅
  - `git diff --check` ✅
- Remaining next tranche:
  - BF1-04 frontend auth/session bootstrap in `apps/biofield-web`
  - BF1-05 capture upload path + sidecar proxy
  - BF1-06 persistence of reading/artifact rows from successful capture flow

# Task Plan - Biofield Web-First Phase 1 Execution

## Checklist
- [x] BWF-001 Freeze biofield domain nouns and state model.
- [x] BWF-002 Freeze Noesis biofield route contract.
- [x] BWF-003 Freeze Python sidecar request and response contract.
- [x] BWF-004 Scaffold `packages/biofield-domain`.
- [x] BWF-005 Scaffold `packages/biofield-api-client`.
- [x] BWF-006 Scaffold `apps/biofield-web` as standalone Next app.
- [x] BWF-007 Add biofield env and config surfaces to `noesis-api`.
- [x] BWF-008 Create Noesis biofield sidecar client shell.
- [x] BWF-009 Add Phase 1 smoke script and local bootstrap notes.
- [x] BWF-010 Design biofield session and artifact schema.
- [x] BWF-011 Author root migration for biofield tables.
- [x] BWF-012 Mirror biofield migration into Supabase migration tree.
- [x] BWF-013 Add `noesis-data` biofield models.
- [x] BWF-014 Implement `biofield_repository` create and query primitives.
- [x] BWF-015 Wire `BiofieldRepository` into `noesis-api` app state creation and test harness surfaces.
- [x] BWF-016 Add the `noesis-api` biofield handler module skeleton.
- [x] BWF-017 Register Phase 1 biofield routes and OpenAPI surface.

## Notes
- Executing Phase 1 in batches from `docs/plans/2026-04-05-biofield-web-first-execution-plan.md`.
- Completed batches now cover BWF-001 through BWF-009, which closes the contract, scaffold, and API seam work for Wave 1.1.
- Wave 1.1 objective:
  - freeze the shared language and create the empty runtime shells before feature logic lands
- Wave 1.2 objective:
  - land the persistence seam before repository and route code starts depending on it

## Review (fill after execution)
- Batch 1 outcome:
  - added frozen contract docs for:
    - biofield domain nouns and lifecycle
    - public Noesis route namespace and resource shapes
    - private Python sidecar request, response, and versioning rules
  - these docs are now the reference for Phase 1 scaffolding and later implementation waves
- Batch 2 outcome:
  - scaffolded `packages/biofield-domain` with frozen types and package-local tests
  - scaffolded `packages/biofield-api-client` with the first typed route wrapper for the frozen biofield namespace
  - scaffolded `apps/biofield-web` as a standalone Next app with placeholder `login`, `viewer`, `history`, and `readings/[readingId]` routes
  - verified the new packages with install, test, typecheck, and build
  - verified the new app with install, typecheck, lint, and production build
  - build note:
    - Next emits a workspace-root warning because the repo has multiple lockfiles
    - the scaffold still builds correctly, and the warning should be revisited only if the repo adopts a clearer workspace strategy
- Batch 3 outcome:
  - added `PYTHON_BIOFIELD_URL` and `PYTHON_BIOFIELD_TIMEOUT_MS` to `noesis-api` config with defaults and validation
  - added a `BiofieldClient` wrapper in `crates/noesis-api` so the API layer can talk to the private Python biofield sidecar through the existing bridge client
  - added a first local bootstrap runbook plus `scripts/smoke_biofield_web.sh` for Phase 1 web/api/sidecar reachability checks
  - verified the new Rust surface with:
    - `cargo test -p noesis-api biofield -- --nocapture`
  - verified the smoke script syntax with:
    - `bash -n scripts/smoke_biofield_web.sh`
- Batch 4 outcome:
  - added a frozen persistence design in `docs/contracts/biofield-persistence.md`
  - added `migrations/017_biofield_sessions.sql` for:
    - `biofield_sessions`
    - `biofield_capture_artifacts`
    - nullable reading linkage through `biofield_capture_artifacts.reading_id`
  - mirrored the same SQL into `supabase/migrations/20260405000017_017_biofield_sessions.sql`
  - verified migration-tree sync with:
    - `cargo test -p noesis-data migration_017_exists_in_root_and_supabase -- --nocapture`
    - `diff -u migrations/017_biofield_sessions.sql supabase/migrations/20260405000017_017_biofield_sessions.sql`
  - verified local SQL apply/rollback with:
    - `PGPASSWORD=noesis_password PGCONNECT_TIMEOUT=2 psql "postgresql://noesis_user@localhost:5432/noesis" -v ON_ERROR_STOP=1 -c 'BEGIN' -f migrations/017_biofield_sessions.sql -c 'ROLLBACK'`
- Batch 5 outcome:
  - added `crates/noesis-data/src/models/biofield.rs` with:
    - session status constants
    - artifact kind constants
    - `BiofieldSession`
    - `NewBiofieldSession`
    - `BiofieldCaptureArtifact`
    - `NewBiofieldCaptureArtifact`
  - exported the new model and repository modules from `crates/noesis-data/src/models/mod.rs` and `crates/noesis-data/src/repositories/mod.rs`
  - added `crates/noesis-data/src/repositories/biofield_repository.rs` with primitives for:
    - session create
    - session get
    - artifact create
    - session artifact list
    - artifact-to-reading linkage
    - reading artifact list
  - added a narrow contract exposure test in `crates/noesis-data/tests/biofield_repository_contract.rs`
  - verified the model layer with:
    - `cargo test -p noesis-data models -- --nocapture`
  - verified the repository layer against local Postgres with:
    - `DATABASE_URL=postgresql://noesis_user:noesis_password@localhost:5432/noesis cargo test -p noesis-data biofield_repository -- --nocapture`
- Batch 6 outcome:
  - extended `crates/noesis-api/src/lib.rs` `AppState` so both eager and lazy builders now expose `biofield_repository` whenever `DATABASE_URL` is configured
  - added `crates/noesis-api/src/handlers/biofield.rs` and exported it from `crates/noesis-api/src/handlers/mod.rs`
  - registered the Phase 1 biofield namespace in the authenticated `/api/v1` router:
    - `POST /api/v1/biofield/sessions`
    - `POST /api/v1/biofield/sessions/:session_id/close`
    - `GET /api/v1/biofield/sessions/:session_id`
    - `POST /api/v1/biofield/sessions/:session_id/captures`
    - `GET /api/v1/biofield/readings`
    - `GET /api/v1/biofield/readings/:reading_id`
  - registered the same six routes in the generated OpenAPI document with placeholder schema types for:
    - session resource
    - reading summary/detail
    - list query/body DTOs
  - updated manual `AppState` test constructors so existing test helpers remain aligned with the new field set
  - verified the new API surface with:
    - `cargo test -p noesis-api --test test_harness -- --nocapture`
    - `cargo test -p noesis-api --test biofield_handler_smoke -- --nocapture`
    - `git diff --check -- crates/noesis-api/src/lib.rs crates/noesis-api/src/handlers/mod.rs crates/noesis-api/src/handlers/biofield.rs crates/noesis-api/tests/test_harness.rs crates/noesis-api/tests/common/test_harness.rs crates/noesis-api/tests/billing_hooks_tests.rs crates/noesis-api/tests/rate_limit_tests.rs crates/noesis-api/tests/biofield_handler_smoke.rs`

# Task Plan - Biofield Web-First Execution Planning

## Checklist
- [x] Inspect the existing app, package, data, and deployment structure the web-first biofield slice must fit into.
- [x] Freeze the web-first package, API, data, and sidecar boundaries at file and service granularity.
- [x] Write the execution-grade plan to `docs/plans/2026-04-05-biofield-web-first-execution-plan.md`.
- [x] Record the phase, wave, and swarm handoff in `tasks/todo.md`.

## Notes
- This takes the native integration architecture plan and turns it into an implementation-ready web-first delivery sequence.
- Constraints held constant:
  - dedicated user-facing app
  - web first, mobile second
  - Railway-native multi-service deployment
  - Supabase auth and data ownership
  - browser-local hot path
  - preserved Python analysis path
- Chosen execution shape:
  - `apps/biofield-web` as a standalone Next.js app
  - `packages/biofield-domain` and `packages/biofield-api-client` as shared TS packages
  - `crates/noesis-api` owns the biofield public API namespace
  - `crates/noesis-data` owns biofield session and artifact persistence
  - existing `python-services/biofield_cv_service` is upgraded instead of replaced
- Phase / wave / swarm handoff:
  - Phase 1 / Wave 1.1:
    - Swarm A - contract freeze
    - Swarm B - web and package scaffolds
    - Swarm C - API seam and smoke plumbing
  - Phase 1 / Wave 1.2:
    - Swarm D - data model and migrations
    - Swarm E - API route skeleton
    - Swarm F - Python contract alignment
  - Phase 1 / Wave 1.3:
    - Swarm G - viewer shell
    - Swarm H - capture and persistence flow
    - Swarm I - validation and docs

## Review (fill after execution)
- Planning outcome:
  - produced an execution-grade web-first plan with:
    - discovery summary,
    - assumptions and constraints,
    - exact file and service boundaries,
    - Phase 1 wave and swarm layout,
    - 80 schema-complete tasks with dependencies,
    - verification strategy,
    - GitHub sync guidance,
    - and risk plus fallback planning
- Most important execution choices:
  - avoid a repo-wide JS workspace migration in Phase 1
  - reuse `readings` for persisted biofield capture outputs with dedicated session and artifact tables around it
  - keep the browser hot path local
  - upgrade the existing Python biofield sidecar instead of creating a second service tree
  - keep `engine-biofield` stable while the capture-native product surface matures

# Task Plan — Biofield Viewer Native Integration Planning

## Checklist
- [x] Inspect the standalone BV-PIP folder structure, specs, and runtime assumptions.
- [x] Inspect Selemene engine, bridge, workflow, auth, and Railway deployment surfaces relevant to biofield integration.
- [x] Compare the architectural options for making the viewer native without losing Python calculation fidelity.
- [x] Write the phased integration plan to `docs/plans/2026-04-05-biofield-viewer-native-integration-plan.md`.
- [x] Confirm the frontend-hosting choice: dedicated user-facing web + mobile product surfaces.
- [x] Confirm delivery order: web first, mobile second.
- [x] Confirm the first execution slice before implementation begins.

## Notes
- User wants the biofield viewer to become a native part of Selemene Engine rather than remain a standalone BV-PIP project.
- Railway is the main compute stack; Supabase-backed auth and data ownership must remain intact.
- Key constraint:
  - preserve the proven BV-PIP calculation logic, especially the Python analysis modules, while integrating product, auth, persistence, and deployment under Selemene.
- Confirmed product direction:
  - dedicated user-facing biofield surfaces for both web and mobile
  - not `apps/admin-web`
  - web ships first; mobile follows on the same shared domain and API contracts

## Review (fill after execution)
- Planning outcome:
  - produced a detailed integration plan with:
    - discovery summary,
    - current-state findings,
    - architectural options and recommendation,
    - Railway-native target topology,
    - contract freeze,
    - migration phases,
    - 72 granular tasks across 6 phases,
    - verification gates,
    - and open product decisions
- Key architectural recommendation:
  - do not transplant the standalone BV-PIP app wholesale
  - keep the viewer native in Selemene, keep the browser hot path local, and preserve deeper Python analysis as a private Railway sidecar behind Noesis
  - keep current `engine-biofield` semantics stable in the first waves and introduce capture-derived somatic integration only after the reading domain is stable
- Execution handoff:
  - converted the architecture plan into the implementation-ready web-first sequence in:
    - `docs/plans/2026-04-05-biofield-web-first-execution-plan.md`

# Task Plan — Fix Discord OAuth Redirect URI Failure On Production Admin Domain

## Checklist
- [x] Confirm whether the Discord failure happens at authorize or callback time.
- [x] Identify whether the broken value comes from frontend override behavior or backend redirect validation.
- [x] Implement a shared callback-selection helper for production vs preview/local origins.
- [x] Wire the admin login and callback flow to the shared helper.
- [x] Update the admin auth docs to reflect the corrected redirect contract.
- [x] Run fresh verification for the frontend and targeted OAuth behavior.

## Notes
- User reported the admin dashboard Discord login was broken.
- Browser evidence shows Discord rejecting the request before consent with `Invalid OAuth2 redirect_uri`.
- The current production/custom admin domain should not send a dynamic callback override unless that exact domain is registered in the Discord app.
- The current same-origin override support is still needed for localhost and preview surfaces.

## Review (fill after execution)
- Root cause:
  - the production/custom admin domain was still sending a dynamic same-origin `redirect_uri` override to Discord
  - Discord rejected that value because it was not one of the exact callback URLs registered in the Discord app
  - the backend override validation was not the problem; the wrong production behavior was happening before the user ever completed Discord consent
- Changes made:
  - added `apps/admin-web/src/lib/discord-oauth.ts` to centralize callback override selection
  - stable production/custom domains now send no `redirect_uri` override and rely on the canonical backend `DISCORD_REDIRECT_URI`
  - preview/local origins (`localhost`, `127.0.0.1`, `*.vercel.app`, `*.railway.app`) still send same-origin callback overrides
  - updated both:
    - `apps/admin-web/app/(public)/login/login-client.tsx`
    - `apps/admin-web/app/(public)/login/discord-callback/discord-callback-client.tsx`
  - updated admin auth docs in:
    - `apps/admin-web/README.md`
    - `docs/AUTH_INTEGRATION.md`
- Verification:
  - `npm --prefix apps/admin-web run typecheck` ✅
  - `npm --prefix apps/admin-web run lint` ✅
  - `npm --prefix apps/admin-web run build` ✅
    - required a rerun with network access because `next/font` fetches Google Fonts during build
  - `cargo test -p noesis-api accepts_same_origin_preview_callback_uri -- --nocapture` ✅
  - `cargo test -p noesis-api rejects_cross_origin_callback_uri_override -- --nocapture` ✅

# Task Plan — Admin API Key Delete Endpoint Production Regression

## Checklist
- [x] Reproduce the delete-endpoint failure mode and distinguish code vs deployment.
- [x] Identify the root cause across the live API surface and git history.
- [x] Add automated coverage that asserts the delete route exists.
- [x] Update route docs/smoke checks so future releases verify the delete endpoint explicitly.
- [x] Run targeted verification and capture the resolution clearly.

## Notes
- User reported that the admin dashboard "delete" endpoint is not working.
- Live probe results:
  - `GET https://selemene-engine-production.up.railway.app/api/v1/admin/api-keys` -> `401`
  - `POST https://selemene-engine-production.up.railway.app/api/v1/admin/api-keys/<uuid>/rotate` -> `401`
  - `DELETE https://selemene-engine-production.up.railway.app/api/v1/admin/api-keys/<uuid>` -> `404`
- That pattern means the deployed API does not currently expose the delete route, because authenticated-gated siblings already return `401` when the route exists.
- Repo investigation shows the delete route exists in the current workspace branch, but `main` is currently at commit `08bc7a8a`, which predates commit `689c919c` (`feat(admin): upgrade api key management flow`) where the delete route was added.

## Review (fill after execution)
- Root cause:
  - this is a production deployment mismatch, not a frontend route-construction bug in the current branch
  - the live Railway API still behaves as if `DELETE /api/v1/admin/api-keys/:key_id` does not exist
  - current workspace code already defines that route in:
    - `crates/noesis-api/src/lib.rs`
    - `crates/noesis-api/src/handlers/admin.rs`
    - `crates/noesis-data/src/repositories/admin_repository.rs`
  - git history shows the delete-route feature landed in commit `689c919c` (`feat(admin): upgrade api key management flow`), while `main` currently points at `08bc7a8a`, which predates that change
- Changes made:
  - added an integration regression test for unauthenticated delete-route existence in `crates/noesis-api/tests/integration_tests.rs`
  - upgraded `scripts/smoke_admin_web.sh` so it can assert HTTP method-specific route existence and now checks `DELETE /api/v1/admin/api-keys/:key_id`
  - updated route docs in:
    - `docs/api/README.md`
    - `README.md`
    - `docs/deployment/VERCEL_ADMIN_WEB.md`
- Verification:
  - `bash -n scripts/smoke_admin_web.sh` ✅
  - `ADMIN_WEB_URL=https://144.tryambakam.space API_BASE_URL=https://selemene-engine-production.up.railway.app bash scripts/smoke_admin_web.sh` ❌
  - failure detail:
    - `DELETE https://selemene-engine-production.up.railway.app/api/v1/admin/api-keys/00000000-0000-0000-0000-000000000000 -> 404`
    - sibling admin routes still return `401`, confirming route absence on the deployed API
  - attempted Rust verification:
    - `cargo test -p noesis-api test_admin_api_key_delete_route_requires_auth -- --nocapture` ❌
    - `cargo test -p noesis-api --test integration_tests test_admin_api_key_delete_route_requires_auth -- --nocapture` ❌
    - both are currently blocked by unrelated dirty-worktree compile errors where `ApiConfig` initializers are missing `gateway_url` and `gateway_token` in:
      - `crates/noesis-api/src/config.rs`
      - `crates/noesis-api/tests/common/test_harness.rs`
- Resolution path:
  - merge or deploy a branch that includes commit `689c919c` (or equivalent delete-route changes) to the Railway API service
  - after deploy, rerun `scripts/smoke_admin_web.sh`; the new delete probe should flip from `404` to `401`

# Task Plan — Timed API Key Reveal In Admin Dashboard

## Checklist
- [x] Inspect the current API key modal and confirm whether existing secrets are retrievable.
- [x] Choose a safe reveal design that does not weaken backend secret storage.
- [x] Implement timed reveal/copy behavior for newly issued secrets in the admin modal.
- [x] Verify `apps/admin-web` typecheck, lint, and build.
- [x] Record the final behavior and constraints in the review section.

## Notes
- User asked for a "view api key" function that briefly shows the key and allows copying to clipboard.
- Current backend behavior only returns plaintext secrets on create/rotate. Persisted API keys are validated by hash, so existing secrets cannot be recovered from storage.
- Safe implementation target:
  - keep existing-key secrets unrecoverable,
  - improve the management modal so newly created/rotated secrets can be revealed briefly,
  - auto-hide after a countdown,
  - keep explicit copy-to-clipboard support.

## Review (fill after execution)
- Behavior:
  - newly created and rotated API keys now auto-enter a timed reveal state in the management modal
  - the full secret is visible for up to 15 seconds, with a live countdown in the action button
  - operators can still hide the secret immediately and use the copy-to-clipboard action while the secret is available
- Security constraint preserved:
  - existing API keys remain non-recoverable after issuance
  - no backend secret escrow or plaintext persistence was introduced
  - the modal now states this explicitly so operators understand why only prefixes are visible for older keys
- Files:
  - `apps/admin-web/app/(protected)/api-keys/page.tsx`
- Verification:
  - `npm --prefix apps/admin-web run typecheck` ✅
  - `npm --prefix apps/admin-web run lint` ✅
  - `npm --prefix apps/admin-web run build` ✅

# Task Plan — Dodo Payments Integration Planning

## Checklist
- [x] Review the current repo billing/auth/subscription surfaces.
- [x] Read the requested Dodo Payments docs and extract the relevant integration primitives.
- [x] Load the local `swarm-architect` guidance and shape the plan around phases, waves, and swarms.
- [x] Write the dated implementation plan to `docs/plans/2026-03-31-dodo-payments-integration-plan.md`.
- [ ] Confirm the plan direction before Phase P1 execution begins.

## Notes
- Goal: replace the current placeholder Stripe-shaped billing hooks with a production Dodo Payments integration plan.
- The repo already has `plan_catalog`, `billing_subscriptions`, and active-plan resolution logic, so the plan is built around extending those surfaces rather than replacing them wholesale.
- The full execution-grade plan lives in:
  - `docs/plans/2026-03-31-dodo-payments-integration-plan.md`

## Review (fill after execution)
- Planning outcome:
  - produced a Swarm Architect style plan with:
    - discovery summary,
    - contract freeze,
    - phase map,
    - detailed Phase 1 waves/swarms,
    - 84 granular tasks,
    - verification gates,
    - GitHub sync strategy,
    - worker split,
    - and rollout risks/fallbacks
- Key architectural recommendation:
  - keep `plan_catalog.code` as the canonical internal entitlement key
  - add first-class Dodo customer and webhook event persistence
  - make webhooks plus provider sync the source of truth for subscription state
  - keep UI strictly on backend-owned billing endpoints

# Task Plan — Dodo Payments Foundation Slice

## Checklist
- [x] Inspect the exact `noesis-api` / `noesis-data` lock-zone files for the first Dodo implementation wave.
- [x] Freeze the foundation contract in a repo-local doc.
- [x] Add root + Supabase migration files for Dodo customer and webhook persistence.
- [x] Expand local subscription status support for Dodo-ready lifecycle states.
- [x] Add Dodo config loading and validation scaffolding to `ApiConfig`.
- [x] Add a Dodo billing emitter scaffold with test coverage.
- [x] Run final formatting and targeted verification after all edits.

## Notes
- This slice is intentionally limited to backend/data groundwork.
- It does not yet implement checkout routes, portal routes, webhook handlers, or billing UI.
- Foundation contract doc:
  - `docs/contracts/dodo-payments-foundation.md`

## Review (fill after execution)
- Scope:
  - added a Dodo-ready schema foundation without changing entitlement behavior yet
  - added config support for Dodo env vars
  - added a Dodo billing emitter scaffold for provider-specific event formatting
- Files:
  - `migrations/016_dodo_billing_foundation.sql`
  - `supabase/migrations/20260331000016_016_dodo_billing_foundation.sql`
  - `docs/contracts/dodo-payments-foundation.md`
  - `crates/noesis-api/src/config.rs`
  - `crates/noesis-api/src/billing.rs`
- Verification target:
  - targeted `noesis-data` migration test
  - targeted `noesis-api` config tests
  - targeted `noesis-api` Dodo billing emitter test

# Task Plan — Admin Dashboard Wave 1 Bootstrap

---

# Task Plan — Fix Discord Login Regression After Admin UI Upgrades

## Checklist
- [x] Reproduce the Discord login regression in the current admin-web flow.
- [x] Isolate the root cause across the login page, callback handling, and API contract.
- [x] Add a failing regression test covering the broken Discord auth behavior.
- [x] Implement the minimal fix for the identified root cause.
- [x] Verify the fix with targeted tests and relevant app checks.

## Notes
- User reported that the new UI upgrades broke Discord login.
- Root-cause-first debugging only; no speculative auth-flow changes.

## Review (fill after execution)
- Scope:
  - fixed the admin-web Discord OAuth contract without changing credential login behavior
  - kept the fix constrained to callback URI selection + validation across frontend and API
- Reproduction:
  - verified the live custom domain login button still reached Discord
  - verified the alternate dashboard origin also sent users to Discord with a hard-coded production callback URI:
    - current origin: `https://enantiodromia-engine-dashboard.vercel.app/admin/login`
    - generated redirect URI: `https://144.tryambakam.space/admin/login/discord-callback`
  - this meant new dashboard origins could not complete OAuth on their own origin
- Root cause:
  - frontend always trusted the backend-configured `DISCORD_REDIRECT_URI`
  - backend always used the single configured redirect URI for both authorize and token exchange
  - redesign/new-origin surfaces therefore broke Discord login whenever the UI origin differed from the configured callback origin
- Fix:
  - backend now accepts an optional caller-provided Discord callback URI for authorize + callback exchange
  - backend validates that override against:
    - the current browser `Origin`
    - allowed admin callback paths only (`/admin/login/discord-callback`, `/admin/auth/discord/callback`)
    - `https`, or `http` on localhost only
  - login UI now passes the current origin callback URI
  - callback page now sends its exact current callback URI back during code exchange
  - updated admin-web runtime docs to reflect the same-origin callback contract
- Verification:
  - live browser reproduction showed the pre-fix hard-coded callback-origin behavior
  - `cargo test -p noesis-api accepts_same_origin_preview_callback_uri -- --nocapture`
  - `cargo test -p noesis-api rejects_cross_origin_callback_uri_override -- --nocapture`
  - `npm --prefix apps/admin-web run typecheck`
  - `npm --prefix apps/admin-web run lint`
  - `npm --prefix apps/admin-web run build`
  - build passed after rerunning with network access for `next/font` Google font fetches

---

# Task Plan — Audit Remaining GitHub Issues For Admin Dashboard

## Checklist
- [x] Confirm the GitHub repository and issue scope relevant to the admin dashboard.
- [x] Fetch the current open GitHub issues from the repository.
- [x] Separate admin-dashboard issues from unrelated repo issues if the repo contains mixed scopes.
- [x] Summarize the remaining issues clearly for the user.

## Notes
- User asked for "all the issues that are remaining" for the admin dashboard.
- Repo remote resolves to `Sheshiyer/Selemene-engine`.

## Review (fill after execution)
- Scope:
  - confirmed the relevant admin-dashboard tracker is the `ADR-01` through `ADR-34` issue series in `Sheshiyer/Selemene-engine`
  - excluded unrelated open roadmap issues from other repo areas
- Findings:
  - all 34 admin-dashboard ADR issues are still open on GitHub
  - the live issue range is `#515` through `#548`
  - open count by phase: `P1=8`, `P2=8`, `P3=10`, `P4=8`
  - open count by wave: `W1=14`, `W2=12`, `W3=8`
- Verification:
  - `gh issue list --repo Sheshiyer/Selemene-engine --state open --limit 200 --json number,title,url,labels --jq '.[] | select(.title | test("ADR-")) | [.number, .title, .url] | @tsv'`
  - `gh issue list --repo Sheshiyer/Selemene-engine --state all --limit 200 --json number,title,state --jq '[.[] | select(.title | test("ADR-"))] | {total: length, open: map(select(.state=="OPEN"))|length, closed: map(select(.state=="CLOSED"))|length}'`

---

# Task Plan — Admin Dashboard Redesign Wave 1 (P1 / W1)

## Checklist
- [x] Confirm executable Wave 1 scope from the redesign dependency graph.
- [x] Capture a source-of-truth baseline inventory and before evidence for the current admin surface.
- [x] Write the Wave 1 design artifact covering tokens, typography, tone, and ornament rules.
- [x] Land the Wave 1 token and typography system in `apps/admin-web`.
- [x] Add reusable visual grammar utilities that Wave 2 can build on without reinterpretation.
- [x] Verify `apps/admin-web` typecheck, lint, and build after the Wave 1 changes.
- [x] Record results in the review section.

## Notes
- User asked to "finish admin dashboard redesign issue wave 1".
- The only dependency-clean interpretation is `P1 / W1` (`ADR-01` through `ADR-04`).
- Later `W1` items in the master manifest depend on shell and primitive work from later waves and must not be claimed here.

## Review (fill after execution)
- Scope:
  - executed `P1 / W1` only (`ADR-01` through `ADR-04`)
  - did not claim later `W1` tasks because they depend on shell / primitive work from later waves
- Evidence:
  - added baseline screenshots:
    - `docs/assets/admin-dashboard-wave-1-baseline/login-desktop.png`
    - `docs/assets/admin-dashboard-wave-1-baseline/login-mobile.png`
  - captured protected-route baseline through source inventory in:
    - `docs/plans/2026-03-26-admin-dashboard-wave-1-design.md`
- Design system:
  - wired `Exo 2` as the display face in `apps/admin-web/app/layout.tsx`
  - replaced the old teal-led token set with the Tryambakam palette in `apps/admin-web/app/globals.css`
  - added shared visual grammar utilities and page-shell header treatment for Wave 2 reuse
- Verification:
  - `npm --prefix apps/admin-web run typecheck`
  - `npm --prefix apps/admin-web run lint`
  - `npm --prefix apps/admin-web run build`
  - post-change login renders were captured locally with headless Chrome to verify the updated surface loads

---

# Task Plan — Admin Dashboard Redesign Wave 2 (P1 / W2)

## Checklist
- [x] Confirm the exact Wave 2 deliverables from the dependency graph (`ADR-05`, `ADR-06`, `ADR-07`).
- [x] Rebuild the protected shell so every authenticated route inherits the new frame automatically.
- [x] Add shared surface and card primitives for page sections, metrics, and state panels.
- [x] Add reusable modal, drawer, and action rail primitives.
- [x] Apply the new primitives to at least the shell and one existing detail flow so they are not dead code.
- [x] Verify `apps/admin-web` typecheck, lint, and build after the Wave 2 changes.
- [x] Record results in the review section.

## Notes
- This pass is `P1 / W2` only.
- Scope covers shell and primitive infrastructure, not the later page redesign issues.
- The goal is to make later route-level redesigns compositional instead of one-off.

## Review (fill after execution)
- Scope:
  - executed `P1 / W2` only (`ADR-05`, `ADR-06`, `ADR-07`)
  - did not claim later route-level redesign waves because those depend on this shell/primitives layer
- Shell:
  - rebuilt `app/(protected)/layout.tsx` into a two-column shell with grouped navigation, route context, operator context, and shell-level action rail
  - added route metadata so every protected page inherits consistent eyebrow, title, and summary framing
  - replaced the old one-off loading / missing-session states with shared shell state cards
- Shared primitives:
  - added `apps/admin-web/src/components/admin-primitives.tsx` with:
    - `SurfaceCard`
    - `MetricSurface`
    - `ActionRail`
  - added `apps/admin-web/src/components/overlay-surface.tsx` with:
    - `ModalSurface`
    - `DrawerSurface`
  - refactored `apps/admin-web/src/components/page-shell.tsx` to use the shared surface primitive instead of a bespoke panel wrapper
- Primitive adoption:
  - updated `dashboard/page.tsx` to use shared metric and surface cards
  - updated `audit/page.tsx` to use shared metric cards, event ledger surface, and a drawer-based event detail view
  - updated `api-keys/page.tsx` to use shared metric cards, action rail, registry surface, and modal primitives for create/manage/confirm flows
  - extended `apps/admin-web/app/globals.css` with shell-v2, surface, action-rail, modal, and drawer styling needed for downstream waves
- Verification:
  - `npm --prefix apps/admin-web run typecheck`
  - `npm --prefix apps/admin-web run lint`
  - `npm --prefix apps/admin-web run build`
  - all three passed on the final post-fix state

---

# Task Plan — Admin Dashboard Redesign Wave 3 (P1 / W3)

## Checklist
- [x] Confirm the exact Wave 3 deliverable from the redesign plan (`ADR-08`).
- [x] Build shared loading, empty, error, and success state components for the admin app.
- [x] Add the supporting global CSS so state handling has one visual grammar instead of page-specific helpers.
- [x] Apply the new state system across protected admin pages so route-level loading and empty cases stop diverging.
- [x] Verify `apps/admin-web` typecheck, lint, and build after the Wave 3 changes.
- [x] Record results in the review section.

## Notes
- This pass is `P1 / W3` only.
- Scope is global state handling, not the later interaction primitives or page redesign issues.
- The goal is to eliminate ad-hoc `helper` paragraphs and one-off alert blocks before the page redesign waves begin.

## Review (fill after execution)
- Scope:
  - executed `P1 / W3` only (`ADR-08`)
  - did not claim later interaction, mobile, or page-redesign waves
- Shared state system:
  - added `apps/admin-web/src/components/admin-state.tsx`
  - introduced shared:
    - `StateBanner` for error/success feedback
    - `StatePanel` for loading and empty state surfaces
    - `TableEmptyStateRow` for empty table bodies
  - extended `apps/admin-web/app/globals.css` with reusable ADR-08 state styling
- Route adoption:
  - replaced ad-hoc loading/error/empty patterns across protected routes including:
    - `dashboard/page.tsx`
    - `analytics/page.tsx`
    - `system/page.tsx`
    - `history-sync/page.tsx`
    - `users/page.tsx`
    - `audit/page.tsx`
    - `api-keys/page.tsx`
  - standardized table-empty treatment so admin tables now share one empty grammar instead of inline helper paragraphs
  - standardized top-level success/error messaging so action feedback now uses one shared banner treatment
- Verification:
  - `npm --prefix apps/admin-web run typecheck`
  - `npm --prefix apps/admin-web run lint`
  - `npm --prefix apps/admin-web run build`
  - all three passed on the final Wave 3 state

---

# Task Plan — Admin Dashboard Overhaul Plan + GitHub Issue Sync

## Checklist
- [x] Gather redesign direction from the provided Tryambakam brand board and local HTML variant references.
- [x] Confirm current admin-web route inventory and shell scope.
- [x] Write a redesign plan doc with phases, waves, swarms, and issue mapping.
- [x] Generate a 30–40 issue manifest for GitHub synchronization.
- [x] Commit and push the planning artifacts.
- [x] Create GitHub issues from the manifest and record the results.

## Notes
- The redesign should shift the admin dashboard from low-taste utilitarian UI to a high-function management system.
- Visual direction should combine:
  - the warm Tryambakam brand board,
  - Exo 2 / Space Grotesk hierarchy,
  - bronze/teal/cream materials,
  - and the HUD / split-grid / drill-down energy from the provided HTML variants.
- The just-shipped API key management work is treated as completed baseline, not future scope.

## Review (fill after execution)
- Planning artifacts:
  - added `docs/plans/2026-03-25-admin-dashboard-overhaul-plan.md`
  - added `docs/plans/2026-03-25-admin-dashboard-overhaul-issues.json`
- Git:
  - pushed commit `e8622ab7` (`docs(admin): add dashboard overhaul plan`) to `origin/main`
- GitHub issue sync:
  - created 34 redesign issues from `ADR-01` through `ADR-34`
  - issue range: `#515` to `#548`
  - labels applied per issue:
    - `enhancement`
    - `roadmap`
    - `taskmaster`
    - phase label
    - wave label
    - area label
  - sync summary written to `/tmp/admin-dashboard-overhaul-issue-sync-summary.json`

---

# Task Plan — API Key Management Modal + Permanent Delete

## Checklist
- [x] Inspect current API keys UI, client contract, and backend admin API support.
- [x] Add backend admin API support for permanent API key deletion.
- [x] Extend frontend API client/types/permissions for delete support.
- [x] Redesign API key table into a row-clickable management surface with detail modal.
- [x] Add modal actions for reveal-once secret handling, rotate, revoke, and permanent delete.
- [x] Verify build/lint/typecheck for `apps/admin-web` and targeted backend tests if available.
- [x] Record results in the review section.

## Notes
- User wants the table upgraded into a management-level UI with a detail modal.
- Permanent delete is explicitly required, not a soft archive.
- Existing keys should not falsely promise full-secret reveal unless returned by create/rotate flows.

## Review (fill after execution)
- Backend:
  - added `DELETE /api/v1/admin/api-keys/:key_id` in `crates/noesis-api`
  - added repository-level hard delete in `crates/noesis-data`
  - introduced `admin:keys:delete` into admin role/permission normalization
- Frontend:
  - upgraded the API keys page from a passive table into a row-clickable management surface
  - added a wide detail modal with identity, lifecycle, permissions, secret-access section, and a danger zone
  - create/rotate now route the one-time secret into the management modal instead of a separate bare reveal modal
  - permanent delete is available via confirmation modal and requires the new permission
- Verification:
  - `npm --prefix apps/admin-web run typecheck`
  - `npm --prefix apps/admin-web run lint`
  - `npm --prefix apps/admin-web run build`
  - `cargo test -p noesis-api admin_role_includes_api_key_delete_permission -- --nocapture`
- Build/test status:
  - admin-web typecheck passed
  - admin-web lint passed
  - admin-web build passed
  - targeted noesis-api test passed

---

# Task Plan — Diagnose Vercel 404 After Admin Web Root Fix

## Checklist
- [x] Re-read deployment lessons and prior Vercel notes before further guidance.
- [ ] Verify the live responses for the custom domain and Vercel production alias.
- [ ] Inspect deployment docs and active repo config for any remaining Vercel requirements.
- [ ] Isolate whether the blocker is unsaved settings, missing redeploy, or wrong domain/project binding.
- [ ] Give the user the exact next action to clear the 404.

## Notes
- User changed Vercel settings but still sees Vercel `404: NOT_FOUND`.
- Prior advice must now be treated as incomplete until live deployment/domain state is proven.

## Review (fill after execution)
- Local cleanup:
  - fixed stale `ApiConfig` test fixtures by adding `gateway_url` and `gateway_token` to:
    - `crates/noesis-api/src/config.rs`
    - `crates/noesis-api/tests/common/test_harness.rs`
    - `crates/noesis-api/tests/rate_limit_tests.rs`
    - `crates/noesis-api/tests/billing_hooks_tests.rs`
  - normalized the docs relocation in git so most moved planning files now appear as explicit renames into `docs/plans/`
  - one launch checklist still shows as delete + add instead of rename, which implies content changed during the move rather than a byte-identical relocation
- Verification:
  - `npm --prefix apps/admin-web run typecheck` ✅
  - `cargo test -p noesis-api dodo_usage_payload_shape -- --nocapture` ✅
  - `cargo test -p noesis-api --test integration_tests test_admin_api_key_delete_route_requires_auth -- --nocapture` ✅
- GitHub issue review:
  - the repository currently has `387` open issues
  - range of open issue numbers: `#39` through `#548`
  - open issue distribution:
    - `136` issues below `#200`
    - `216` issues from `#200` to `#499`
    - `35` issues from `#500+`
  - the `#500+` range is almost entirely one admin-dashboard issue train:
    - `#515` through `#548` are all still open
    - `#501` is the only non-ADR open issue in that range
  - no open issues were returned for `label:bug` or `label:admin-dashboard`, so GitHub labeling is not currently supporting triage
- Review findings:
  - `#501` (`Engine hygiene follow-up`) looks stale as an umbrella
    - its scope overlaps strongly with merged PR `#509`
    - child issues `#502` through `#507` are no longer open, but `#501` remains open
    - recommendation: either close it with links to shipped work plus any residual follow-ups, or rewrite it to track only remaining hygiene regressions
  - the admin-dashboard ADR chain `#515`–`#548` is still fully open
    - this is useful as a phase map, but not as a truthful progress board
    - given recent shipped/admin work, several items likely need re-triage for `done`, `partial`, or `superseded`
  - older plan-generated issues remain open in bulk
    - representative example: `#39` asks for `orchestrator_routed` tracing, but a repo search did not find that symbol or clearly named implementation
    - recommendation: do not try to review these one-by-one from GitHub alone; first group them by active roadmap vs archival backlog, then bulk-close or migrate

---

# Task Plan — Trigger Fresh Vercel Deploy From `main`

## Checklist
- [x] Confirm local branch state is clean and tracking `origin/main`.
- [x] Update a low-risk file to create a fresh Git commit for Vercel branch deployment.
- [x] Commit the deploy-trigger change on `main`.
- [x] Push `main` to `origin`.
- [x] Verify `origin/main` advanced to the new commit and record the result.

## Notes
- User requested a README touch specifically to force a new deployment from the connected production branch.
- Keep the change content-neutral and avoid modifying application behavior.

## Review (fill after execution)
- Commit created on `main`: `31c2d1a5` (`chore: trigger vercel deploy from main`).
- Root `README.md` received a no-op HTML comment timestamp to force a new Git-based production deployment.
- Push succeeded:
  - `d5ac13b4..31c2d1a5  main -> main`
- Verification:
  - local `HEAD` = `31c2d1a5`
  - `origin/main` = `31c2d1a5`
  - `git status --short --branch` returned `## main...origin/main`, confirming a clean synchronized branch.

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

### Next 10-Issue Wave

- Selected batch:
  - `#33`
  - `#34`
  - `#39`
  - `#40`
  - `#41`
  - `#42`
  - `#43`
  - `#44`
  - `#326`
  - `#329`

#### Wave Rationale

- This is the next best coherent batch because it splits into two low-coupling tracks:
  - routing baseline / orchestrator-enforcement foundation
  - remaining failure-mode runbooks
- The routing issues should start with `#33` and `#34` because:
  - route inventory is the baseline artifact
  - the enforcement issues need shared test helpers/harnesses
- The runbook issues `#326` and `#329` can be handled as a sidecar documentation tranche once the foundation work is underway.

#### Execution Order

- [x] Select the next coherent 10-issue batch.
- [x] Implement foundation tranche:
  - `#33` route inventory baseline
  - `#34` orchestrator-enforcement test harness
- [x] Use that foundation to decide which of `#39`-`#44` are immediately closable vs need more code.
- [ ] Execute docs/runbook sidecar tranche:
  - `#326`
  - `#329`
- [ ] Close only the issues proven by fresh verification.

#### Foundation Tranche Review

- `#33` landed as a checked-in baseline artifact plus source-backed drift test:
  - `docs/baseline/api-route-inventory.json`
  - `crates/noesis-api/tests/common/route_inventory.rs`
  - `crates/noesis-api/tests/route_inventory_tests.rs`
- The route inventory baseline currently documents:
  - `40` unique `/api/v1` paths
  - `43` method/path entries
  - per-route auth requirement (`public` vs `bearer_or_api_key`)
  - whether the handler touches the orchestrator surface
- `#34` landed as a reusable route-level probe harness:
  - `crates/noesis-api/tests/common/test_harness.rs`
  - `crates/noesis-api/tests/test_harness.rs`
- The harness builds a custom `AppState` with probe engines, executes real HTTP requests through `create_router`, and asserts orchestrator delegation by checking sentinel outputs plus per-engine execution counters.

#### Foundation Verification

- `cargo test -p noesis-api route_inventory -- --nocapture`
  - passed
- `cargo test -p noesis-api test_harness -- --nocapture`
  - passed

#### Next Routing Mini-Wave

- The new harness makes this next batch practical without touching the dirty middleware file:
  - `#40` single-engine calculate routing enforcement
  - `#41` workflow execute routing enforcement
  - `#42` bridge-engine orchestrator-only enforcement
- `#39` stays separate for now because it requires `crates/noesis-api/src/middleware.rs`, which already has unrelated local modifications and should be handled deliberately.

#### Routing Enforcement Review

- Landed:
  - `crates/noesis-api/tests/routing_enforcement_tests.rs`
- `#40` evidence:
  - `11` single-engine calculate route tests now verify orchestration for:
    - `panchanga`
    - `numerology`
    - `biorhythm`
    - `human-design`
    - `gene-keys`
    - `vimshottari`
    - `biofield`
    - `vedic-clock`
    - `face-reading`
    - `nadabrahman`
    - `transits`
- `#41` evidence:
  - `6` workflow execute route tests now verify orchestration for:
    - `birth-blueprint`
    - `daily-practice`
    - `decision-support`
    - `creative-expression`
    - `self-inquiry`
    - `full-spectrum`
- `#42` evidence:
  - runtime routing test covers all bridge engine IDs:
    - `tarot`
    - `i-ching`
    - `enneagram`
    - `sacred-geometry`
    - `sigil-forge`
  - source audit verifies `crates/noesis-api/src/handlers/*.rs` do not import or call `BridgeManager` directly
  - handler-body audit verifies core engine/workflow handlers in `crates/noesis-api/src/lib.rs` do not touch `bridge_manager`

#### Routing Enforcement Verification

- `cargo test -p noesis-api routing_enforcement -- --nocapture`
  - passed
- `cargo test -p noesis-api workflow_routing -- --nocapture`
  - passed

#### Closure Readiness

- Closure-ready:
  - `#33`
  - `#34`
  - `#40`
  - `#41`
  - `#42`
- Intentionally left open:
  - `#39`
    - still requires request-tracing middleware changes in `crates/noesis-api/src/middleware.rs`
  - `#43`
  - `#44`
    - both still require new runtime assertion / bypass-count machinery, not just tests

#### GitHub Closure Result

- Closed on branch `codex/engine-hygiene-native-runtime-docs` after push of `b2b788d9`:
  - `#33`
  - `#34`
  - `#40`
  - `#41`
  - `#42`
- Backlog after this tranche:
  - open issues: `375`
  - closed issues: `104`

### Canary Automation Batch (`#315`, `#316`)

- [x] Audit current canary policy, deploy workflow, and script/test seams.
- [x] Implement `scripts/canary-health-score.sh`.
- [x] Add mocked verification for healthy and unhealthy Prometheus/Sentry responses.
- [x] Implement `scripts/canary-promote.sh` with dry-run mode and stage-by-stage decisions.
- [x] Add mocked verification for promotion and rollback decisions.
- [x] Update canary rollout docs with script usage.
- [x] Close only if the scripts and mocked verification satisfy the issue acceptance criteria.

Review:
- Added [scripts/canary-health-score.sh](/Volumes/madara/2026/witnessos/Selemene-engine/scripts/canary-health-score.sh) to score Prometheus error rate, Prometheus p95 latency, and Sentry critical count into a JSON `canary_healthy` decision.
- Added [scripts/canary-promote.sh](/Volumes/madara/2026/witnessos/Selemene-engine/scripts/canary-promote.sh) to walk canary stages `5 -> 25 -> 50 -> 100`, emit per-stage JSON decisions, and support dry-run promotion and rollback hooks.
- Added [scripts/test_canary_automation.sh](/Volumes/madara/2026/witnessos/Selemene-engine/scripts/test_canary_automation.sh) to prove healthy/failing health-score cases and dry-run/live promotion behavior with mocked dependencies.
- Updated [docs/runbooks/canary-rollout-policy.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/canary-rollout-policy.md) with the automation helper usage and hook contract.

Verification:
- `bash -n scripts/canary-health-score.sh scripts/canary-promote.sh scripts/test_canary_automation.sh`
- `bash scripts/test_canary_automation.sh`

### Canary Observability Batch (`#318`, `#319`)

- [x] Audit canary alert/dashboard requirements against current Prometheus labels, Grafana dashboards, and promotion script behavior.
- [x] Add `NoesisCanaryErrorDivergence` alert rule with an explicit canary-vs-stable label contract.
- [x] Add promtool-compatible rule tests for healthy and divergent canary scenarios.
- [x] Extend `scripts/canary-promote.sh` to post Grafana annotations for promotion and rollback events.
- [x] Add mocked verification for Grafana annotation payloads in the canary automation test harness.
- [x] Update Grafana dashboard/runbook docs so annotations are visible on the Selemene Engine dashboard.
- [x] Close only the issues whose acceptance criteria are satisfied by repo-visible changes and local verification.

Review:
- Added rollout-aware canary divergence alerting in [monitoring/prometheus/alerts/noesis-alerts.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/prometheus/alerts/noesis-alerts.yml) and documented the `rollout=stable|canary` scrape label contract in [monitoring/prometheus.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/prometheus.yml).
- Added promtool rule fixtures in [monitoring/prometheus/tests/noesis-canary-alerts.test.yml](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/prometheus/tests/noesis-canary-alerts.test.yml) with a local runner in [scripts/test_canary_alerts.sh](/Volumes/madara/2026/witnessos/Selemene-engine/scripts/test_canary_alerts.sh).
- Extended [scripts/canary-promote.sh](/Volumes/madara/2026/witnessos/Selemene-engine/scripts/canary-promote.sh) to post Grafana annotations for live promotion and rollback events, with mocked coverage in [scripts/test_canary_automation.sh](/Volumes/madara/2026/witnessos/Selemene-engine/scripts/test_canary_automation.sh).
- Updated the Selemene Engine Grafana dashboard and monitoring docs in [monitoring/grafana/dashboards/selemene-engine.json](/Volumes/madara/2026/witnessos/Selemene-engine/monitoring/grafana/dashboards/selemene-engine.json), [docs/runbooks/canary-rollout-policy.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/canary-rollout-policy.md), and [docs/deployment/monitoring.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/deployment/monitoring.md).
- Audited `#317` separately and kept it open because the repo still lacks a real canary slot and traffic-splitting infrastructure.

Verification:
- `bash -n scripts/canary-promote.sh scripts/test_canary_automation.sh scripts/test_canary_alerts.sh`
- `bash scripts/test_canary_automation.sh`
- `bash scripts/test_canary_alerts.sh`
- `jq empty monitoring/grafana/dashboards/selemene-engine.json`

### Failure-Mode Docs Batch (`#326`, `#330`)

- [x] Audit `#326`, `#330`, and nearby `#329` acceptance text against the current runtime.
- [x] Confirm `#326` is now closable because the orchestrator already has per-engine timeout controls and partial-failure handling.
- [x] Defer `#329` explicitly because its acceptance assumes JWKS rotation and Supabase auth downtime paths that do not exist in the current auth runtime.
- [x] Update the workflow timeout runbook so it reflects real timeout settings, partial engine failures, trace IDs, and Sentry breadcrumb correlation.
- [x] Add a rollback drill plan document with the three required scenarios and expected outcomes for `#330`.
- [x] Cross-link the new drill plan from the runbook index if needed.
- [x] Run doc/code-alignment verification and close only the issues fully satisfied by repo-visible artifacts.

Review:
- `#326` is materially closer to done than the current runbook suggests:
  - `crates/noesis-orchestrator/src/workflow/full_spectrum.rs` already exposes `FullSpectrumConfig.engine_timeout`
  - individual engines are wrapped in `tokio::time::timeout(...)`
  - timeout failures are preserved per-engine in `failed_engines`
  - orchestrator tests already prove:
    - partial success with one failing engine
    - success with all engines failing
    - timeout handling in full-spectrum workflow
- `#329` is not honest to close in this wave:
  - `crates/noesis-auth/src/lib.rs` implements JWT secret validation and API-key validation
  - there is no JWKS fetch/rotation path
  - there is no Supabase Auth runtime dependency to document as an incident mode
- This batch should therefore target:
  - `#326`
  - `#330`
  - while explicitly keeping `#329` open with the current blocker rationale

Verification:
- Updated [docs/runbooks/incident-workflow-timeout.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/incident-workflow-timeout.md) to distinguish:
  - standard `WorkflowOrchestrator::execute_workflow()` partial-result behavior
  - `FullSpectrumWorkflow` per-engine timeout handling via `FullSpectrumConfig.engine_timeout`
  - outer API `REQUEST_TIMEOUT_SECS` request-level `504` behavior
- Added [docs/drills/rollback-drill-plan.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/drills/rollback-drill-plan.md) with the three required drill scenarios:
  - broken environment variable deploy
  - crashing init code
  - canary elevated error rate
- Updated [docs/runbooks/README.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/README.md) with a drill-plan link.
- `#329` remains intentionally open because the current auth runtime in [crates/noesis-auth/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-auth/src/lib.rs) does not implement JWKS rotation or Supabase Auth downtime paths.

Verification:
- `cargo test -p noesis-orchestrator test_timeout_handling -- --nocapture`
  - passed
- `cargo test -p noesis-api --test workflow_execution_tests test_workflow_partial_failure_graceful_degradation -- --nocapture`
  - passed
- `cargo test -p noesis-api error_mapper -- --nocapture`
  - passed
- `rg -n "FullSpectrumConfig.engine_timeout|REQUEST_TIMEOUT_SECS|trace_id|api.error|NoesisCanaryErrorDivergence" docs/runbooks/incident-workflow-timeout.md docs/drills/rollback-drill-plan.md docs/runbooks/README.md`
  - passed

GitHub closure result:
- Closed on `codex/engine-hygiene-native-runtime-docs` after push of `089efd55`:
  - `#326`
  - `#330`
- Intentionally left open with blocker comment:
  - `#329`
- Backlog after this tranche:
  - open issues: `369`

### Release Docs Batch (`#355`, `#356`)

- [x] Audit the next non-canary, non-auth documentation issues for a coherent batch.
- [x] Select `#355` and `#356` as the next honest tranche.
- [x] Defer `#353` because it still needs stronger OpenAPI/runtime proof than this wave provides.
- [x] Defer `#354` because its acceptance requires a from-scratch Railway deploy walkthrough that is not yet proven in this wave.
- [ ] Create `docs/monitoring/README.md` that maps monitoring tools to the actual repo config files and explains alert flow from detection to notification.
- [ ] Create `docs/release/release-checklist-template.md` as a reusable markdown checklist based on the P5 release process.
- [ ] Add cross-links from existing docs where they improve discoverability.
- [ ] Run doc/config alignment verification and close only the issues fully satisfied by repo-visible artifacts.

Review:
- `#355` is a strong fit for this wave:
  - the monitoring config already exists in `monitoring/`
  - the alert rules, dashboard files, Alertmanager routing, Jaeger, Loki, and runbooks are all repo-visible
  - the missing artifact is the consolidated operator-facing README
- `#356` is also a strong fit:
  - the P5 process already exists across `docs/RELEASE_NOTES.md`, `docs/planning/p5-release-readiness-plan.json`, and the recent backlog work
  - the missing artifact is the reusable checklist template
- `#353` stays open for now:
  - it asks for updated OpenAPI spec and docs reflecting all P1-P5 endpoint changes
  - that needs stronger proof against actual generated server spec / merged spec output
- `#354` stays open for now:
  - `docs/deployment/RAILWAY.md` and `docs/deployment/VERCEL_ADMIN_WEB.md` exist
  - but the issue acceptance calls for a new-team-member Railway walkthrough from scratch, which we are not proving in this batch

Verification:
- pending

## Release Docs Batch (#355, #356)
- [x] Re-verify monitoring/release doc gaps against current repo state
- [x] Create docs/monitoring/README.md mapped to actual monitoring surfaces
- [x] Create docs/release/release-checklist-template.md
- [x] Cross-link new docs from existing deployment/release docs where appropriate
- [x] Verify referenced files/configs exist and docs align to repo state
- [x] Commit, push, and close #355 and #356 if acceptance criteria are satisfied

Release docs verification notes:
- `jq empty` passed for all repo-visible Grafana dashboard JSON files.
- `bash scripts/test_canary_alerts.sh` passed and confirmed the current Prometheus alert rules load and test cleanly.
- A link audit confirmed all absolute file links introduced in the new monitoring and release docs resolve to existing repo files.
- The monitoring README intentionally documents the Alertmanager webhook leg as configured-but-unverified because repo-visible webhook handlers were not found for the configured receiver URLs.
- Commit pushed: `659bd10d` (`docs(release): add monitoring and release references`)
- Closed issues:
  - `#355`
  - `#356`
- Backlog after this tranche:
  - open issues: `367`
  - closed issues: `112`

### v3 Product Docs Batch (`#472`, `#473`)

- [x] Audit `#472` and `#473` against current repo-visible product/docs/runtime surfaces.
- [x] Record the closure decision: pursue `#473`, keep `#472` open unless the live repo can honestly satisfy its migration acceptance.
- [x] Create `docs/contributing/engine-onboarding.md` covering trait implementation, crate scaffold, witness prompt guidance, synthesis/registration steps, and CI checklist.
- [x] Validate the guide by scaffolding a temporary dummy engine crate outside the repo and compiling it against local path dependencies.
- [x] Cross-link the onboarding guide from existing docs if useful.
- [x] Commit, push, and close only the issues truly satisfied by repo-visible artifacts and validation.

Review:
- `#472` currently appears blocked by repo reality: the issue expects migration steps for both Rust and TypeScript SDKs, but the repo only ships `noesis-sdk` (Rust) and explicitly notes that no TypeScript SDK package exists yet.
- `#473` is a strong fit for this wave: `ConsciousnessEngine` is stable in `crates/noesis-core`, orchestrator registration is straightforward in `crates/noesis-orchestrator` / `crates/noesis-api`, and trait conformance examples exist in tests.

Verification:
- Added [docs/contributing/engine-onboarding.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/contributing/engine-onboarding.md) and linked it from [README.md](/Volumes/madara/2026/witnessos/Selemene-engine/README.md).
- Validated the guide by scaffolding a temporary crate at `/tmp/selemene-engine-onboarding-VPuoES/engine-test-engine`.
- `cargo test` in that temporary crate passed with:
  - `tests::engine_compiles_and_produces_output`
  - `tests::engine_registers_with_orchestrator`
- Commit pushed: `2100999e` (`docs(contributing): add engine onboarding guide`)
- GitHub outcome:
  - closed `#473`
  - kept `#472` open with blocker comment because the repo still does not ship a TypeScript SDK package or the broader proven migration surface the issue assumes
- Backlog after this tranche:
  - open issues: `366`
  - closed issues: `113`

### v3 Launch Runbook Batch (`#477`)

- [x] Audit `#477` against existing release, runbook, rollback, and monitoring docs.
- [x] Generate an in-repo snapshot of current open GitHub issues for backlog review.
- [x] Draft a v3.0.0 launch-day operational runbook using repo-visible Railway, monitoring, smoke, and rollback surfaces.
- [x] Verify whether the issue can be closed honestly; if staging dry-run proof is missing, leave it open with a blocker note instead of forcing closure.

Review:
- `#477` is partially supported by existing artifacts: `docs/runbooks/README.md`, `docs/drills/rollback-drill-plan.md`, `docs/release/release-checklist-template.md`, `docs/monitoring/README.md`, and Railway verification scripts already exist.
- The likely blocker is the issue validation requirement: a rollback dry-run on staging completing in under 5 minutes is not automatically proven by repo contents alone.

Artifacts in progress:
- `docs/planning/open-issues-snapshot-2026-03-13.md`
- `docs/runbooks/launch-day-v3.0.0.md`

Verification / outcome:
- Added [docs/planning/open-issues-snapshot-2026-03-13.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/planning/open-issues-snapshot-2026-03-13.md) containing the full live open-issue list at the time of the audit.
- Added [docs/runbooks/launch-day-v3.0.0.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/launch-day-v3.0.0.md) and linked it from [docs/runbooks/README.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/README.md).
- Link audit passed for the new launch-day runbook and runbook index.
- Commit pushed: `36efa1a4` (`docs(runbooks): add launch-day draft and issue snapshot`)
- GitHub outcome:
  - kept `#477` open with blocker comment because staging rollback timing and live dashboard URL resolution remain unproven

## Infra/Auth Audit Batch (`#16`-`#20`)

- [x] Audit `#16` through `#20` against the current root migrations, Supabase migrations, and auth/admin runtime.
- [x] Confirm whether any of the schema issues are already satisfied by repo-visible migrations or runtime code.
- [x] Choose the smallest honest infra/auth tranche that fits the current runtime without touching unrelated dirty files.
- [x] Implement `#16` by adding canonical `user_roles` and `user_account_state` tables, backfill migration, and rolling-runtime support.
- [ ] Re-verify `#17` through `#20` after `#16` lands and decide the next schema batch.
- [x] Verify, commit, push, and update backlog counts / issue status.

Infra/auth audit notes:
- `#16` through `#20` are still genuinely open. The repo has adjacent auth/admin behavior, but no applied root or Supabase migrations for:
  - `user_roles`
  - `user_account_state`
  - `api_key_events`
  - history sync state / idempotency schema
  - plan catalog / billing subscriptions
  - usage partition maintenance function
- Existing runtime equivalents are partial and legacy-shaped:
  - role assignment currently persists in `user_profiles.preferences.admin_roles`
  - account state currently uses `users.locked_until`
  - API keys already have `name` / `key_prefix`, but no immutable lifecycle event table
- Selected first tranche: `#16`.
  This is the cleanest schema/auth step because the admin routes for state and roles already exist; we can add the missing tables, backfill from current data, and keep rollout-safe fallbacks so the live handlers do not break if migrations lag behind deploy.

Infra/auth implementation notes:
- Added root migration [migrations/010_user_roles_account_state.sql](/Volumes/madara/2026/witnessos/Selemene-engine/migrations/010_user_roles_account_state.sql) and matching Supabase migration [supabase/migrations/20260313000010_010_user_roles_account_state.sql](/Volumes/madara/2026/witnessos/Selemene-engine/supabase/migrations/20260313000010_010_user_roles_account_state.sql).
- Updated [crates/noesis-data/src/repositories/admin_repository.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-data/src/repositories/admin_repository.rs) so admin role/state reads now prefer the canonical tables while falling back to the legacy `user_profiles.preferences` and `users.locked_until` path when migrations are not yet applied.
- Kept the runtime blast radius low by continuing to mirror role assignments into `user_profiles.preferences` and account locks into `users.locked_until`, so auth and admin handlers remain compatible during rollout.

Verification:
- `cargo test -p noesis-data admin_repository -- --nocapture`
- `cargo build -p noesis-api`
- `cargo test -p noesis-api derives_platform_admin_role -- --nocapture`
- Commit pushed: `9e4f8fd8` (`feat(auth): add canonical admin role and state schema`)
- GitHub outcome:
  - closed `#16`
  - backlog now: `365` open / `114` closed

### API Key Audit Tranche (`#17`)

- [x] Add canonical lifecycle schema for API key event audit rows and actor-attribution columns.
- [x] Keep admin create/revoke/rotate handlers API-compatible while writing to the new schema.
- [x] Preserve rollout safety with repository fallbacks if the new columns/table are not yet migrated.
- [x] Commit, push, close `#17`, and refresh backlog counts.

API key audit implementation notes:
- Added root migration [migrations/011_api_key_events.sql](/Volumes/madara/2026/witnessos/Selemene-engine/migrations/011_api_key_events.sql) and matching Supabase migration [supabase/migrations/20260313000011_011_api_key_events.sql](/Volumes/madara/2026/witnessos/Selemene-engine/supabase/migrations/20260313000011_011_api_key_events.sql).
- Extended [crates/noesis-data/src/repositories/admin_repository.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-data/src/repositories/admin_repository.rs) so `create_api_key`, `revoke_api_key`, and `rotate_api_key` now target immutable `api_key_events` rows plus new metadata columns, with legacy fallbacks if the schema is not present yet.
- Updated [crates/noesis-api/src/handlers/admin.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/src/handlers/admin.rs) to pass actor attribution into key lifecycle mutations without changing the public route contract.

Verification:
- `cargo test -p noesis-data admin_repository -- --nocapture`
- `cargo build -p noesis-api`
- `cargo test -p noesis-api derives_platform_admin_role -- --nocapture`
- Commit pushed: `88a56d46` (`feat(auth): add api key lifecycle audit schema`)
- GitHub outcome:
  - closed `#17`
  - backlog now: `364` open / `115` closed

### Usage Partition Maintenance Tranche (`#20`)

- [x] Audit the current `usage_logs` partition surface and confirm only static 2026 partitions exist today.
- [x] Choose a real maintenance shape: DB function plus automated check script, instead of more hard-coded yearly partitions.
- [x] Add canonical root and Supabase migrations for the maintenance function.
- [x] Add an operational check script that runs the function and fails loudly on errors.
- [x] Add lightweight regression coverage for the new migration/script artifacts.
- [x] Verify, commit, push, close `#20`, and refresh backlog counts.

Usage partition maintenance notes:
- Added root migration [migrations/012_usage_partition_maintenance.sql](/Volumes/madara/2026/witnessos/Selemene-engine/migrations/012_usage_partition_maintenance.sql) and matching Supabase migration [supabase/migrations/20260313000012_012_usage_partition_maintenance.sql](/Volumes/madara/2026/witnessos/Selemene-engine/supabase/migrations/20260313000012_012_usage_partition_maintenance.sql).
- Added [scripts/check_usage_log_partitions.sh](/Volumes/madara/2026/witnessos/Selemene-engine/scripts/check_usage_log_partitions.sh), which calls `ensure_usage_log_partitions(...)` and exits nonzero with an alert message on failure.
- Extended [crates/noesis-data/src/repositories/admin_repository.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-data/src/repositories/admin_repository.rs) tests so the migration pair and script stay anchored in CI.

Verification:
- `cargo test -p noesis-data admin_repository -- --nocapture`
- `bash -n scripts/check_usage_log_partitions.sh`
- `bash scripts/check_usage_log_partitions.sh` without `DATABASE_URL` exits `1` and emits the alert path
- Commit pushed: `6fc290f8` (`feat(infra): add usage partition maintenance check`)
- GitHub outcome:
  - closed `#20`
  - backlog now: `363` open / `116` closed

### History Sync Schema Tranche (`#18`)

- [x] Audit `#18` against the current `readings` schema, history-sync routes, and repository/runtime code.
- [x] Add canonical root and Supabase migrations for `user_devices`, `history_sync_state`, and idempotent reading sync columns.
- [x] Finish minimal runtime wiring so existing reading persistence still compiles against the expanded `NewReading` builder.
- [x] Verify idempotent same-user `client_event_id` writes and cursor-delta queries via targeted repository tests/builds.
- [x] Commit, push, close `#18`, and refresh backlog counts.

History sync tranche notes:
- Issue `#18` is not already done. The current admin history-sync endpoints are synthetic over `readings` and `usage_logs`; there was no canonical `user_devices`, `history_sync_state`, `client_event_id`, or `sync_cursor` surface before this tranche.
- Added root migration [migrations/013_history_sync_schema.sql](/Volumes/madara/2026/witnessos/Selemene-engine/migrations/013_history_sync_schema.sql) and matching Supabase migration [supabase/migrations/20260313000013_013_history_sync_schema.sql](/Volumes/madara/2026/witnessos/Selemene-engine/supabase/migrations/20260313000013_013_history_sync_schema.sql).
- Extended [crates/noesis-data/src/models/reading.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-data/src/models/reading.rs) with `ReadingSyncRecord` and optional client/device sync metadata on `NewReading`.
- Extended [crates/noesis-data/src/repositories/readings_repository.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-data/src/repositories/readings_repository.rs) with fallback-aware idempotent save logic, cursor-delta listing, and migration-anchored tests.
- Updated the existing fire-and-forget persistence paths in [crates/noesis-api/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/src/lib.rs) to pass explicit `None` values for the new sync metadata so the runtime stays source-compatible until mobile/device sync starts sending those fields.

Verification:
- `cargo test -p noesis-data readings_repository -- --nocapture`
- `cargo build -p noesis-api`
- `DATABASE_URL='postgresql://noesis_user:noesis_password@localhost:5432/noesis' cargo test -p noesis-data save_reading_is_idempotent_when_client_event_id_is_present -- --nocapture`
- Canonical root migrations applied in order against local Postgres via `psql`, including [migrations/013_history_sync_schema.sql](/Volumes/madara/2026/witnessos/Selemene-engine/migrations/013_history_sync_schema.sql)
- Commit pushed: `ce621cf3` (`feat(sync): add history sync schema and idempotent reading writes`)
- GitHub outcome:
  - closed `#18`
  - backlog now: `362` open / `117` closed

### Plan Catalog And Billing Schema Tranche (`#19`)

- [x] Audit `#19` against the current `users.tier`, `api_keys.tier`, and repository/runtime tier handling.
- [x] Choose the smallest schema shape that eliminates active-plan ambiguity without rewriting auth or user APIs.
- [x] Add canonical root and Supabase migrations for plan catalog, billing subscriptions, and an unambiguous active-plan resolution view.
- [x] Sync the canonical plan/subscription rows from existing user creation and admin tier update paths with rollout-safe fallbacks.
- [x] Verify schema-level active-plan resolution and update GitHub status if the acceptance criteria are satisfied.

Plan catalog tranche notes:
- Current runtime still treats `users.tier` and `api_keys.tier` as the only tier sources. There is no canonical plan table, subscription table, or active-plan join surface yet.
- The acceptance on `#19` is stricter than a bare migration: we need a schema-level join path that resolves one active plan per user without duplicated tier ambiguity.
- Selected implementation shape:
  - `plan_catalog` table for canonical plans
  - `billing_subscriptions` table with provider identifiers and status constraints
  - single active-plan resolution view guarded by a unique active-subscription rule
  - minimal repository updates so new users and admin tier changes keep the canonical rows in sync while `users.tier` remains a compatibility field
- Added root migration [migrations/014_plan_catalog_billing_subscriptions.sql](/Volumes/madara/2026/witnessos/Selemene-engine/migrations/014_plan_catalog_billing_subscriptions.sql) and matching Supabase migration [supabase/migrations/20260313000014_014_plan_catalog_billing_subscriptions.sql](/Volumes/madara/2026/witnessos/Selemene-engine/supabase/migrations/20260313000014_014_plan_catalog_billing_subscriptions.sql).
- Updated [crates/noesis-data/src/repositories/user_repository.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-data/src/repositories/user_repository.rs) so new users attempt to create canonical active-plan rows and can resolve an active plan via `resolve_active_plan_code(...)`, with legacy fallbacks if the new schema is absent.
- Updated [crates/noesis-data/src/repositories/admin_repository.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-data/src/repositories/admin_repository.rs) so admin tier updates also resync the canonical active subscription when the new schema exists.

Verification:
- `cargo test -p noesis-data user_repository -- --nocapture`
- `cargo test -p noesis-data admin_repository -- --nocapture`
- `cargo build -p noesis-api`
- Docker Desktop restarted successfully; local Postgres was brought up from [docker-compose.yml](/Volumes/madara/2026/witnessos/Selemene-engine/docker-compose.yml)
- Canonical root migrations applied against local Postgres through [migrations/014_plan_catalog_billing_subscriptions.sql](/Volumes/madara/2026/witnessos/Selemene-engine/migrations/014_plan_catalog_billing_subscriptions.sql)
- `DATABASE_URL='postgresql://noesis_user:noesis_password@localhost:5432/noesis' cargo test -p noesis-data active_plan_resolution_stays_unambiguous_after_tier_update -- --nocapture`
- Commit pushed: `07300914` (`feat(billing): add canonical plan catalog schema`)
- GitHub outcome:
  - closed `#19`
  - backlog now: `361` open / `118` closed

### Orchestrator Routing Guard Tranche (`#43`, `#44`)

- [x] Audit `#43` and `#44` against the current orchestrator/runtime boundary and keep `#38` out of this tranche.
- [x] Add orchestrator-owned execution routing counters inside `noesis-orchestrator` so direct registry execution is observable.
- [x] Add debug/test phase-gate assertions so direct registry execution of an inaccessible engine panics in debug builds.
- [x] Route `WorkflowOrchestrator` and `WorkflowExecutor` engine execution through the guarded registry path.
- [x] Add targeted tests proving:
  - direct `EngineRegistry::execute(...)` increments bypass tracking,
  - routed orchestrator/workflow execution does not,
  - bypassing phase gating panics in debug tests.
- [x] Run targeted verification, then commit, push, and close any satisfied issues.

Orchestrator routing guard notes:
- Added routed-vs-direct execution counters and snapshots in [crates/noesis-orchestrator/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-orchestrator/src/lib.rs).
- `EngineRegistry::execute(...)` is now an explicitly observable bypass path: it increments `direct_execute_calls` and `bypass_count`, and in debug/test builds it panics if phase gating is bypassed for an inaccessible engine.
- `WorkflowOrchestrator::execute_engine(...)`, `WorkflowOrchestrator::execute_workflow(...)`, and [crates/noesis-orchestrator/src/workflow/executor.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-orchestrator/src/workflow/executor.rs) now route engine execution through the guarded registry path so legitimate execution increments only `orchestrated_execute_calls`.

Verification:
- `cargo test -p noesis-orchestrator registry_direct_execute_marks_bypass_count -- --nocapture`
- `cargo test -p noesis-orchestrator execute_workflow_records_routed_engine_executions -- --nocapture`
- `cargo test -p noesis-orchestrator execute_single_engine_success -- --nocapture`
- `cargo test -p noesis-orchestrator registry_direct_execute_phase_bypass_panics_in_debug -- --nocapture`
- `cargo test -p noesis-orchestrator -- --nocapture`

GitHub outcome:
- closed `#43`
- closed `#44`
- backlog now: `359` open / `120` closed

### Native Engine Visibility Guard Tranche (`#38`)

- [x] Audit `#38` against the current `noesis-api` dependency surface and confirm runtime engine crates are still imported directly there.
- [x] Move native engine registration behind a `noesis-orchestrator` helper so the API runtime no longer constructs engine crates itself.
- [x] Strip native engine crates from `noesis-api` runtime dependencies while preserving test/bench access through `dev-dependencies`.
- [x] Add a static regression test proving `noesis-api` runtime code and `[dependencies]` no longer reference native engine crates directly.
- [x] Run targeted verification, then commit, push, and close `#38`.

Native engine visibility guard notes:
- Added [WorkflowOrchestrator::register_native_runtime_engines(...)](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-orchestrator/src/lib.rs) so native engine construction now lives in `noesis-orchestrator` instead of `noesis-api`.
- Updated [crates/noesis-api/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/src/lib.rs) to build the runtime orchestrator through that helper and keep bridge registration local to the API crate.
- Removed native engine crates from `noesis-api` runtime `[dependencies]` in [crates/noesis-api/Cargo.toml](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/Cargo.toml), leaving only the subset needed for tests/benches under `[dev-dependencies]`.
- Added a static regression in [crates/noesis-api/tests/routing_enforcement_tests.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/tests/routing_enforcement_tests.rs) that fails if native engine crates reappear in `noesis-api` runtime dependencies or source imports.

Verification:
- `cargo build -p noesis-api`
- `cargo test -p noesis-orchestrator orchestrator_register_native_runtime_engines -- --nocapture`
- `cargo test -p noesis-api --test routing_enforcement_tests -- --nocapture`

GitHub outcome:
- closed `#38`
- backlog now: `358` open / `121` closed
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

---

# Task Plan — Wave W2 Workflow-Specific OpenAPI Schemas (#448)

## Checklist
- [x] Add workflow-specific typed synthesis schema models.
- [x] Add workflow-specific response schemas for all 6 workflows.
- [x] Add 6 workflow-specific OpenAPI endpoint definitions for execute routes.
- [x] Reuse shared execution logic to avoid behavioral drift.
- [x] Add OpenAPI regression tests for path presence and synthesis typing.
- [x] Run verification compile checks.

## Notes
- Runtime API surface unchanged (existing dynamic execute route still handles execution).
- Added static-path OpenAPI definitions for each workflow to satisfy per-workflow endpoint visibility in docs/Swagger.

## Review (fill after execution)
- Updated `crates/noesis-api/src/lib.rs`:
  - Added typed synthesis schemas:
    - `BirthBlueprintSynthesisSchema`, `DailyPracticeSynthesisSchema`, `DecisionSupportSynthesisSchema`, `SelfInquirySynthesisSchema`, `CreativeExpressionSynthesisSchema`, `FullSpectrumSynthesisSchema`
  - Added typed workflow result schemas (6):
    - `BirthBlueprintWorkflowResultSchema`, `DailyPracticeWorkflowResultSchema`, `DecisionSupportWorkflowResultSchema`, `SelfInquiryWorkflowResultSchema`, `CreativeExpressionWorkflowResultSchema`, `FullSpectrumWorkflowResultSchema`
  - Added 6 workflow-specific OpenAPI execute handlers/definitions:
    - `/api/v1/workflows/birth-blueprint/execute`
    - `/api/v1/workflows/daily-practice/execute`
    - `/api/v1/workflows/decision-support/execute`
    - `/api/v1/workflows/self-inquiry/execute`
    - `/api/v1/workflows/creative-expression/execute`
    - `/api/v1/workflows/full-spectrum/execute`
  - Refactored shared runtime logic into `execute_workflow_by_id(...)` and reused it.
- Added tests in `crates/noesis-api/tests/workflow_openapi_tests.rs`:
  - verifies 6 workflow execute paths exist in OpenAPI
  - verifies workflow result schemas reference typed synthesis schemas
- Verification:
  - `cargo fmt --all` ✅
  - `cargo check -p noesis-api` ✅
  - `cargo check -p noesis-api --tests` ✅

---

# Task Plan — Finish Remaining Wave W2 (#440 #441 #442 #449 #450 #451 #452)

## Checklist
- [x] #449 OpenAPI auth + rate-limit schema/docs hardening
- [x] #441 noesis-sdk retry/backoff + connection pool controls
- [x] #442 LocalProfile offline-first sync with deterministic conflict policy
- [x] #440 noesis-sdk API audit doc + rustdoc build verification
- [x] #450 Docusaurus-based developer portal scaffold under `docs/portal`
- [x] #451 Engine catalog pages for all 16 engines
- [x] #452 Workflow guide pages for all 6 workflows with synthesis semantics

## Review

### #449
- Enhanced OpenAPI `SecurityAddon` to:
  - keep JWT bearer + X-API-Key schemes documented
  - auto-insert standardized `429` response with rate-limit headers on all secured operations
- Added tests: `crates/noesis-api/tests/auth_rate_limit_openapi_tests.rs`

### #441
- Added retry/pooling config to `noesis-sdk`:
  - `Config`: `max_retries`, `backoff_ms`, `pool_max_idle_per_host`
  - env overrides: `NOESIS_MAX_RETRIES`, `NOESIS_BACKOFF_MS`, `NOESIS_POOL_MAX_IDLE_PER_HOST`
  - `NoesisClient`: exponential backoff retry on `5xx` and transport timeout/connect/request errors
- Added wiremock retry tests in `crates/noesis-sdk/src/client.rs`

### #442
- Added profile sync support:
  - `LocalProfile.last_synced_at`
  - `LocalProfile::sync(&NoesisClient)`
  - deterministic conflict policy:
    - server wins for `consciousness_level`
    - local wins for `birth_data`
  - diff-based PATCH payload via `UpdateUserRequest`
- Added sync tests in `crates/noesis-sdk/src/profile.rs`

### #440
- Added `crates/noesis-sdk/API_AUDIT.md` with public API audit snapshot + usage examples
- Updated `crates/noesis-sdk/README.md` to current SDK API (Config-based client + sync)

### #450 #451 #452
- Added Docusaurus portal at `docs/portal/`
  - config: `docusaurus.config.ts`, `sidebars.ts`, `package.json`
  - pages: API overview, authentication, rate limits, SDK quickstarts, OpenAPI explorer
  - engine catalog: `docs/portal/docs/engines/*.md` (16 engine pages + index)
  - workflow guide: `docs/portal/docs/workflows/*.md` (6 workflow pages + index)
  - deployment notes: `docs/portal/README.md`

## Validation Evidence
- `cargo fmt --all` ✅
- `cargo check -p noesis-api` ✅
- `cargo check -p noesis-api --tests` ✅
- `cargo test -p noesis-api --test workflow_openapi_tests -- --nocapture` ✅
- `cargo test -p noesis-api --test auth_rate_limit_openapi_tests -- --nocapture` ✅
- `cargo check -p noesis-sdk` ✅
- `cargo check -p noesis-sdk --tests` ✅
- `cargo test -p noesis-sdk -- --nocapture` ✅
- `cargo doc -p noesis-sdk --no-deps` ✅
- `npm --prefix docs/portal install` ✅
- `npm --prefix docs/portal run build` ✅

---

# Task Plan — Wave W1 Final Sweep (v3.0.0 launch)

## Checklist
- [x] Expand workflow compositions and typed synthesis integration for W1 workflow issues (#433-#439).
- [x] Harden TS engine validation/error boundaries for tarot, i-ching, enneagram, sacred-geometry, and sigil-forge (#426-#429).
- [x] Implement face-reading birth-data fallback and non-mock analysis path (#423/#424).
- [x] Add face-reading image upload API endpoint using multipart handling (#425).
- [x] Add face-analysis backend decision ADR (#422).
- [x] Run Rust + TS verification gates and capture evidence.

## Review (fill after execution)
- Face-reading engine updates (`crates/engine-face-reading/src/engine.rs`):
  - Added deterministic heuristic analysis backend for image input.
  - Added birth-data physiognomy fallback path.
  - Preserved explicit mock fallback when no image/birth_data is provided.
  - Added tests to verify non-mock outputs for image/birth fallback paths.
- API upload endpoint (`crates/noesis-api/src/lib.rs` + `Cargo.toml`):
  - Added authenticated multipart endpoint: `POST /api/v1/engines/face-reading/upload`.
  - Accepts multipart fields `file` or `image`, executes face-reading engine, returns analysis payload.
  - Added OpenAPI path/component registration for upload response.
- TS engines hardening (`ts-engines/src/...`):
  - Added shared `EngineValidationError` + server-side 422 mapping.
  - Tarot: strict `spread_type` and question validation.
  - I-Ching: strict hexagram bounds + question validation.
  - Enneagram: structured type/wing validation errors.
  - Sacred/Sigil: strict method/form validation and graceful SVG template boundary handling.
  - Expanded integration tests for all new validation/error boundary cases.
- Workflow/synthesis alignment (W1 scope):
  - Applied previously planned orchestrator workflow and synthesis updates in `crates/noesis-orchestrator/src/workflow/*`.
- ADR:
  - Added `docs/planning/ADR-0002-face-analysis-backend.md` documenting phased backend decision and rationale.

## Validation Evidence
- `cargo fmt` ✅
- `cargo test -p engine-face-reading` ✅
- `cargo test -p noesis-orchestrator` ✅
- `cargo test -p noesis-api --lib` ✅
- `cargo test -p noesis-api --tests` ⚠️ one pre-existing metrics-registry collision in `billing_hooks_tests` (`AlreadyReg`)
- `npm --prefix ts-engines test` ✅

### #432
- Added witness prompt quality audit doc:
  - `docs/launch/v3.0.0-witness-prompt-audit.md`
- Added automated contract test in noesis-integration:
  - `crates/noesis-integration/tests/witness_prompt_quality_tests.rs`
  - validates all 16 engines against question format + non-prescriptive language constraints

---

# Task Plan — Admin Web Discord Callback Alias And Mainline Commit

## Checklist
- [x] Re-verify the Discord auth state on current `main` and confirm remote divergence before shipping.
- [x] Preserve the backend-driven Discord login flow already present on `origin/main`.
- [x] Add the admin-safe callback alias route at `/admin/auth/discord/callback`.
- [x] Document the required Discord OAuth settings for the live admin portal.
- [x] Commit and push the merged result on top of current `origin/main`.

## Notes
- Scope is routing/readiness only for admin-web Discord auth.
- The login page keeps the existing email/password fallback and backend-driven Discord authorize flow.

## Review (fill after execution)
- Confirmed `origin/main` already contained the active Discord authorize/callback backend flow plus a visible login button, while the live portal was still serving older UI.
- Added callback alias route so the admin portal can complete Discord auth at either:
  - `/admin/login/discord-callback`
  - `/admin/auth/discord/callback`
- Kept the backend-driven `getDiscordAuthUrl()` login path and existing credential fallback intact.
- Fixed the existing Discord callback client to satisfy the React lint rule by avoiding synchronous `setState()` inside the effect's missing-code branch.
- Made `apps/admin-web` `typecheck` deterministic with `tsc --noEmit --incremental false` to avoid the stale Next `.next/types/cache-life.d.ts` failure during verification.
- Documented the required live settings:
  - frontend: `NEXT_PUBLIC_API_BASE_URL`
  - API/backend: `DISCORD_CLIENT_ID`, `DISCORD_CLIENT_SECRET`, `DISCORD_REDIRECT_URI`
  - Discord developer portal callback URL must exactly match `DISCORD_REDIRECT_URI`
# Task Plan — Repo State Review (Staged Changes, Branches, PRs)

## Checklist
- [x] Confirm staged vs unstaged local git state.
- [x] Review the current local diff for notable risks or regressions.
- [x] Inspect local and remote branch status for divergence or stale work.
- [x] Fetch relevant GitHub PR state and summarize what is open/merged/blocking.
- [x] Write a concise review/update with findings, assumptions, and gaps.

## Notes
- User asked for a review of staged changes, branches, and PRs plus an update.
- Current quick check shows no staged files, current branch `codex/engine-hygiene-native-runtime-docs`, and unstaged local modifications across docs, migrations, scripts, admin-web, and billing files.

## Review (fill after execution)
- Local git state:
  - there are zero staged files (`git diff --cached` is empty)
  - the working tree is dirty with unstaged edits plus untracked docs/migration files
  - the current branch is `codex/engine-hygiene-native-runtime-docs`
  - local `HEAD` is `97944cbb`, ahead of `origin/codex/engine-hygiene-native-runtime-docs` by 1 commit
- Diff shape:
  - current unstaged code edits are concentrated in:
    - `apps/admin-web/app/(protected)/api-keys/page.tsx`
    - `crates/noesis-api/src/billing.rs`
    - several `noesis-api` test files
    - `scripts/railway-setup.sh`
    - `scripts/smoke_admin_web.sh`
  - docs/planning changes are currently a mix of deletions plus untracked replacements under `docs/plans/`, which means the move/cleanup is not yet safely captured as a rename in git
- Review findings:
  - `crates/noesis-api/src/config.rs` test fixtures are currently inconsistent with the newer `ApiConfig` shape
  - targeted Rust verification fails because two `ApiConfig` initializers are missing `gateway_url` and `gateway_token`:
    - `crates/noesis-api/src/config.rs:414`
    - `crates/noesis-api/src/config.rs:554`
  - until those are fixed, `cargo test -p noesis-api ...` cannot be treated as green for the current working tree
- Branch state:
  - local `HEAD` differs from `origin/main` by two commits each way (`git rev-list --left-right --count origin/main...HEAD` -> `2 2`)
  - `origin/codex/engine-hygiene-native-runtime-docs` also differs from `origin/main` (`2 1`), so the branch is not a clean descendant of current `main`
  - local `main` tracks `origin/main`
  - `udpate-rbac` still exists locally and remotely
- PR state:
  - the current branch previously landed as merged PRs:
    - `#509` `fix(vedic): restore native runtime and validate transits baselines`
    - `#549` `chore(docs): clean up docs, move planning to .context, update README`
  - there is no open PR for the current local `HEAD`
  - open PR `#514` (`udpate-rbac` -> `main`) is still open and reports `mergeable: false`
- Verification:
  - `git diff --check` ✅
  - `npm --prefix apps/admin-web run typecheck` ✅
  - `cargo test -p noesis-api dodo_usage_payload_shape -- --nocapture` ❌
    - blocked by missing `gateway_url` / `gateway_token` fields in `crates/noesis-api/src/config.rs` test initializers

# Task Plan — Local Cleanup And GitHub Issue Review

## Checklist
- [ ] Fix the `ApiConfig` fixture compile errors blocking targeted Rust verification.
- [ ] Clean up the docs move so deleted docs have tracked replacements or a clearly safe local state.
- [ ] Re-run targeted verification for the touched backend/frontend surfaces.
- [ ] Fetch the current open GitHub issues and triage the actionable ones.
- [ ] Summarize the local cleanup plus GitHub issue review for the user.

## Notes
- Follow-up to the repo-state review.
- User explicitly wants the local cleanup handled first, then a GitHub issue review.

## Review (fill after execution)
- Pending.

# Task Plan — GitHub Issue Triage Execution

## Checklist
- [x] Verify whether `#501` should be closed outright or rewritten to only track residual work.
- [x] Classify admin-dashboard ADR issues `#515` through `#548` as done / partial / still open.
- [x] Apply GitHub issue updates based on that classification.
- [x] Summarize the triage decisions and the remaining backlog.

## Notes
- User asked to act on GitHub issue hygiene directly.
- Triage must be based on shipped code and current repo state, not issue age alone.

## Review (fill after execution)
- `#501` was closed rather than rewritten.
  - rationale:
    - it overlapped with shipped engine-hygiene work, especially merged PR `#509`
    - its referenced child issues are no longer open
    - any remaining regressions should be tracked as fresh targeted issues with current failing evidence
- Admin-dashboard ADR triage was applied with GitHub labels:
  - closed as done:
    - `#515`, `#516`, `#517`, `#518`, `#520`, `#522`, `#524`
  - kept open with `triage-partial`:
    - `#519`, `#521`, `#523`, `#525`, `#527`, `#528`, `#529`, `#531`, `#532`, `#533`, `#535`, `#536`, `#537`, `#538`, `#539`, `#540`, `#542`, `#543`
  - kept open with `triage-open`:
    - `#526`, `#530`, `#534`, `#541`, `#544`, `#545`, `#546`, `#547`, `#548`
- Verification:
  - triage label counts after update:
    - `triage-done`: `8` issues total (`#501` plus the seven closed ADR issues)
    - `triage-partial`: `18` open issues
    - `triage-open`: `9` open issues
  - summary comment added to `#548` documenting the full ADR-chain triage

# Task Plan — Partial Issue Closeout Pass

## Checklist
- [x] Re-evaluate the `triage-partial` ADR issues against the current implementation.
- [x] Close any issues that are now clearly solved with repo-backed evidence.
- [x] Keep unresolved partials open and summarize the smaller remaining set.

## Notes
- User asked to help close issues if they are solved.
- This pass should be conservative: close only what the current code clearly satisfies.

## Review
- Closed as shipped with repo-backed comments and moved from `triage-partial` to `triage-done`:
  - `#532` ADR-18 dashboard overview
  - `#535` ADR-21 API keys redesign
  - `#536` ADR-22 history sync redesign
- Kept open as `triage-partial` because the code still does not clearly satisfy the issue contract end-to-end:
  - `#519`, `#521`, `#523`, `#525`, `#527`, `#528`, `#529`
  - `#531`, `#533`, `#537`, `#538`, `#539`, `#540`, `#542`, `#543`
- Updated label counts after this pass:
  - `triage-done`: `11`
  - open `triage-partial`: `15`
  - open `triage-open`: `9`
- Main reasons for keeping issues open:
  - cross-cutting primitives still incomplete (`#521`, `#523`, `#525`)
  - responsive / motion / accessibility contracts still incomplete or unverified (`#527`, `#528`, `#529`)
  - route implementations exist but the issue text still expects more than the current code clearly proves (`#531`, `#533`, `#537`, `#538`, `#539`, `#540`)
  - later navigation / destructive-action hardening work is still visibly missing (`#542`, `#543`)

# Task Plan — Second Partial Issue Closeout Pass

## Checklist
- [x] Re-check the remaining route-level `triage-partial` issues for any additional closures.
- [x] Close only the issues whose implementation intent is now clearly satisfied in-tree.
- [x] Leave the rest open with a smaller, better-defined unresolved set.

## Notes
- User asked to proceed with more issue closeout.
- This pass should target the strongest remaining route candidates first, then stop if the remaining set is still genuinely incomplete.

## Review
- Closed as shipped with repo-backed comments and moved from `triage-partial` to `triage-done`:
  - `#519` ADR-05 protected shell v2
  - `#531` ADR-17 public login and auth entry
- Left open as `triage-partial` because the issue contract still expects missing shared primitives or hardening work:
  - `#521`, `#523`, `#525`
  - `#527`, `#528`, `#529`
  - `#533`, `#537`, `#538`, `#539`, `#540`
  - `#542`, `#543`
- Updated label counts after this pass:
  - `triage-done`: `13`
  - open `triage-partial`: `13`
  - open `triage-open`: `9`
- Reduction result:
  - the remaining partial set is now concentrated around shared interaction contracts, responsive / motion / accessibility hardening, and later navigation / safety taxonomy work rather than already-shipped shell pages.

# Task Plan — Final Partial Bucket Closeout

## Checklist
- [x] Close the remaining `triage-partial` issues per user instruction.
- [x] Relabel the closed issues from `triage-partial` to `triage-done`.
- [x] Recount the triage buckets after the bulk closeout.

## Notes
- User explicitly requested: close now.
- This pass intentionally overrode the earlier conservative close-only-if-fully-proven policy and treated the remaining partials as backlog cleanup rather than implementation truth claims.

## Review
- Closed and relabeled in bulk:
  - `#521`, `#523`, `#525`
  - `#527`, `#528`, `#529`
  - `#533`, `#537`, `#538`, `#539`, `#540`
  - `#542`, `#543`
- Final label counts after bulk closeout:
  - `triage-done`: `26`
  - open `triage-partial`: `0`
  - open `triage-open`: `9`
- Remaining open issues are now only the explicit `triage-open` backlog:
  - `#526`, `#530`, `#534`, `#541`, `#544`, `#545`, `#546`, `#547`, `#548`

# Task Plan — Remaining Admin Dashboard Issue Execution Order

## Checklist
- [ ] Pull the exact contracts for the 9 remaining open admin-dashboard issues.
- [ ] Derive the dependency-respecting execution order from the issue text and current roadmap.
- [ ] Define close criteria and merge criteria for each issue.
- [ ] Record the ordered plan and summarize it for execution.

## Notes
- User asked for a concrete execution order for the 9 remaining open admin-dashboard issues.
- This is planning/output work, not implementation.

## Review (fill after execution)
- Recommended execution order and gate logic:
  1. `#526` ADR-12 timeline and event stream primitives
     - Why first: unblocks route-level event/history/system surfaces that still need a shared narrative pattern.
     - Close criteria: shared primitive exists; consistent header/badge/metadata contract exists; supports dense lists and drill-down detail; audit/history examples are in-tree.
     - Merge criteria: typecheck/lint/build pass; examples are attached or documented; readability checked on long lists.
  2. `#530` ADR-16 command palette and global quick actions
     - Why second: depends on shell and filter/search work, but is otherwise orthogonal and easiest to land before later QA/polish.
     - Close criteria: keyboard-invoked palette opens; navigates across key admin routes; exposes a small high-frequency action set; fits shell keyboard model.
     - Merge criteria: typecheck/lint/build pass; desktop/tablet interaction verified; command list and trigger documented.
  3. `#534` ADR-20 user detail drawer and action clusters
     - Why third: depends on users route plus drawer/modal primitives; unlocks cleaner user workflows before bulk actions.
     - Close criteria: user rows open shared management surface; state/tier/role actions are grouped there; confirmations exist for destructive/sensitive flows.
     - Merge criteria: typecheck/lint/build pass; mouse and keyboard drill-down flows verified; action confirmation behavior documented.
  4. `#541` ADR-27 bulk action patterns
     - Why fourth: depends on user detail/user route and API-key route maturity; should be designed after the single-item workflows are settled.
     - Close criteria: shared bulk-selection model exists for users and API keys; bulk actions have safety/feedback rules; interaction is extensible.
     - Merge criteria: typecheck/lint/build pass; bulk flows demonstrated with screenshots/video; feedback and safety behavior verified.
  5. `#544` ADR-30 admin telemetry and performance instrumentation
     - Why fifth: should land after the major workflows above exist, so instrumentation covers real workflows instead of placeholders.
     - Close criteria: telemetry emitted for major admin interactions; performance signals can be reviewed post-deploy; instrumentation is not ad hoc in page components.
     - Merge criteria: backend/frontend verification passes; telemetry schema/event map documented; deployed evidence or sampled output recorded.
  6. `#545` ADR-31 screenshot-based visual regression coverage
     - Why sixth: only makes sense after the page surfaces are materially stable.
     - Close criteria: baseline screenshots exist for key routes; diff workflow exists in CI or docs; desktop coverage exists for core pages.
     - Merge criteria: baseline capture workflow documented; sample regression run attached; CI or documented command is runnable.
  7. `#546` ADR-32 accessibility QA and keyboard journey tests
     - Why seventh: should validate the stabilized surfaces and interactions, not moving targets.
     - Close criteria: keyboard-only journeys exist for major flows; shell/table/modal patterns are checked; critical issues are fixed or tracked.
     - Merge criteria: QA checklist attached; major-flow evidence recorded; any remaining defects are explicitly filed.
  8. `#547` ADR-33 cross-page design polish and consistency pass
     - Why eighth: should happen after regression and accessibility hardening so polish does not invalidate earlier baselines.
     - Close criteria: spacing/color/motion/typography are normalized; no visually orphaned pages remain; changes improve taste without changing contracts.
     - Merge criteria: before/after comparisons exist for key improvements; no contract regressions introduced; typecheck/lint/build pass.
  9. `#548` ADR-34 rollout, canary checklist, and final acceptance signoff
     - Why last: explicitly depends on `#544`, `#545`, `#546`, and `#547`.
     - Close criteria: rollout/canary checklist exists; redesign acceptance is reviewed against original brief; open risks and deferrals are documented.
     - Merge criteria: production verification notes are referenced; canary checklist is attached; final signoff doc names residual risks.
- Practical batching guidance:
  - Build tranche: `#526`, `#530`, `#534`, `#541`, `#544`
  - Hardening tranche: `#545`, `#546`
  - Finish tranche: `#547`, `#548`

# Task Plan — Remaining Admin Dashboard PR Boundaries

## Checklist
- [ ] Turn the remaining issue order into recommended branch and PR slices.
- [ ] Keep risky shared-surface changes isolated where review blast radius is high.
- [ ] Group only the issues that benefit from landing together.

## Notes
- User asked for PR boundaries after the execution order was defined.
- Goal: optimize for clean review, lower regression risk, and minimal rework between dependent issues.

## Review (fill after execution)
- Recommended PR queue:
  1. PR-1 `admin-timeline-primitives`
     - Issues: `#526`
     - Why isolated: shared primitive work will touch reusable components and multiple routes; keeping it alone makes regressions easier to review.
     - Likely write scope: shared components, shared CSS, audit/history/system route adapters.
  2. PR-2 `admin-command-palette`
     - Issues: `#530`
     - Why isolated: mostly orthogonal to the rest of the dashboard and safe to review independently once shell contracts are stable.
     - Likely write scope: protected layout, command palette component, keyboard bindings, route/action registry.
  3. PR-3 `admin-user-detail-drawer`
     - Issues: `#534`
     - Why isolated: focused user-management interaction change with meaningful UX and action-flow risk; easier to validate alone before layering bulk behavior.
     - Likely write scope: users page, shared drawer/modal surface integration, confirmation flows.
  4. PR-4 `admin-bulk-actions`
     - Issues: `#541`
     - Why isolated: broader workflow change spanning users and API keys; should land after single-item user workflows are settled.
     - Likely write scope: users page, API keys page, shared selection/action helpers, safety/feedback patterns.
  5. PR-5 `admin-observability-instrumentation`
     - Issues: `#544`
     - Why isolated: cross-cutting backend/frontend instrumentation with different review concerns from UI behavior.
     - Likely write scope: telemetry hooks/helpers, backend event ingestion or logging, docs for event schema/evidence.
  6. PR-6 `admin-qa-hardening`
     - Issues: `#545`, `#546`
     - Why grouped: both are QA/hardening work over the same stabilized route set and will likely share test harnesses, scripts, and evidence artifacts.
     - Likely write scope: Playwright or screenshot tooling, QA docs/checklists, accessibility test coverage, baseline artifacts.
  7. PR-7 `admin-final-polish`
     - Issues: `#547`
     - Why isolated: polish should be reviewed as a taste pass after regression and accessibility baselines exist, not mixed with test harness work.
     - Likely write scope: CSS cleanup, spacing/type/motion normalization, selective component refinements.
  8. PR-8 `admin-rollout-signoff`
     - Issues: `#548`
     - Why isolated: final docs/release gate work should land after implementation and hardening PRs are merged.
     - Likely write scope: rollout docs, canary checklist, acceptance notes, deferred risk log.
- Recommended branch naming:
  - `feat/admin-timeline-primitives`
  - `feat/admin-command-palette`
  - `feat/admin-user-detail-drawer`
  - `feat/admin-bulk-actions`
  - `feat/admin-observability-instrumentation`
  - `chore/admin-qa-hardening`
  - `refactor/admin-final-polish`
  - `docs/admin-rollout-signoff`
- Recommended merge policy:
  - Merge PR-1 through PR-5 sequentially.
  - PR-6 starts only after PR-1 through PR-5 are stable enough for baselines.
  - PR-7 starts after PR-6 establishes regression/a11y guardrails.
  - PR-8 is the final release-readiness PR.

# Task Plan — Branch-By-Branch Working Checklist

## Checklist
- [ ] Convert each remaining PR slice into an execution-ready working checklist.
- [ ] Define the minimum implementation scope for each branch.
- [ ] Define the verification commands and exit criteria for each branch.

## Notes
- User asked for an execution-ready checklist after the PR boundary plan.
- This section is intended to remove ambiguity before starting PR-1.

## Review
- PR-1 `feat/admin-timeline-primitives`
  - Goal: ship a reusable timeline / event stream primitive and adopt it in the routes that need event narratives.
  - Build checklist:
    - define shared timeline component(s) and supporting item contract
    - add badge/header/metadata slots for dense event rows
    - support drill-down affordance or detail handoff
    - wire examples into audit/history/system surfaces where appropriate
    - document the pattern in code or docs
  - Verify:
    - `npm --prefix apps/admin-web run typecheck`
    - `npm --prefix apps/admin-web run lint`
    - `npm --prefix apps/admin-web run build`
  - Exit criteria:
    - audit/history examples exist in-tree
    - dense list rendering is readable
    - primitive is reusable instead of route-specific

- PR-2 `feat/admin-command-palette`
  - Goal: add a shell-level command palette for route navigation and a few high-frequency actions.
  - Build checklist:
    - create palette component and route/action registry
    - add keyboard invocation from protected shell
    - include route navigation entries for key admin surfaces
    - include a small safe set of high-frequency actions
    - handle focus, close, and selection behavior cleanly
  - Verify:
    - `npm --prefix apps/admin-web run typecheck`
    - `npm --prefix apps/admin-web run lint`
    - `npm --prefix apps/admin-web run build`
  - Exit criteria:
    - keyboard invocation works reliably
    - palette feels native to the shell
    - desktop/tablet behavior is documented or demonstrated

- PR-3 `feat/admin-user-detail-drawer`
  - Goal: move user row management into a richer shared detail surface.
  - Build checklist:
    - add user row drill-down entry point
    - build drawer/detail surface with grouped state/tier/role actions
    - add confirmation behavior where needed
    - keep keyboard and mouse flows coherent
    - preserve current user-management capabilities while improving structure
  - Verify:
    - `npm --prefix apps/admin-web run typecheck`
    - `npm --prefix apps/admin-web run lint`
    - `npm --prefix apps/admin-web run build`
  - Exit criteria:
    - user actions are grouped in one management surface
    - row drill-down works without regressions
    - confirmation flows are visible and testable

- PR-4 `feat/admin-bulk-actions`
  - Goal: add a shared bulk-selection and bulk-action model for users and API keys.
  - Build checklist:
    - introduce shared selection state and bulk action affordance
    - apply it to users and API keys
    - define safety/feedback behavior for bulk operations
    - keep the model extensible for future routes
    - document the pattern boundaries
  - Verify:
    - `npm --prefix apps/admin-web run typecheck`
    - `npm --prefix apps/admin-web run lint`
    - `npm --prefix apps/admin-web run build`
  - Exit criteria:
    - users and API keys share the same mental model
    - bulk feedback/safety is explicit
    - implementation is not ad hoc per page

- PR-5 `feat/admin-observability-instrumentation`
  - Goal: instrument major admin interactions and page-performance signals cleanly.
  - Build checklist:
    - define telemetry event map for key admin workflows
    - add instrumentation hooks/helpers instead of page-local ad hoc logging
    - capture basic performance signals for post-deploy review
    - document emitted events and evidence collection path
  - Verify:
    - relevant backend/frontend test or build checks pass
    - `npm --prefix apps/admin-web run typecheck`
    - `npm --prefix apps/admin-web run lint`
    - `npm --prefix apps/admin-web run build`
  - Exit criteria:
    - core workflows emit telemetry
    - event schema is documented
    - deployed evidence path is defined

- PR-6 `chore/admin-qa-hardening`
  - Goal: add regression and accessibility guardrails on top of the stabilized admin surfaces.
  - Issues: `#545`, `#546`
  - Build checklist:
    - add screenshot baseline capture workflow for key routes
    - make diffs reviewable in CI or a documented local workflow
    - add keyboard journey checklist/tests
    - validate shell/table/modal access patterns
    - record evidence for major flows
  - Verify:
    - QA tooling commands pass
    - baseline workflow is runnable
    - accessibility checklist is attached in repo docs or issue evidence
  - Exit criteria:
    - baseline screenshot set exists
    - major keyboard journeys are covered
    - critical accessibility issues are fixed or explicitly filed

- PR-7 `refactor/admin-final-polish`
  - Goal: run the final consistency/taste pass without changing product contracts.
  - Build checklist:
    - normalize spacing/color/type/motion across routes
    - remove visually orphaned page treatments
    - preserve existing interaction contracts and QA baselines
    - collect before/after examples for notable improvements
  - Verify:
    - `npm --prefix apps/admin-web run typecheck`
    - `npm --prefix apps/admin-web run lint`
    - `npm --prefix apps/admin-web run build`
    - rerun regression/a11y workflows from PR-6 as needed
  - Exit criteria:
    - no page feels outside the system
    - polish improves taste, not scope
    - no baseline-breaking regressions are introduced silently

- PR-8 `docs/admin-rollout-signoff`
  - Goal: finalize rollout, canary, acceptance, and residual-risk artifacts.
  - Build checklist:
    - write rollout/canary checklist for redesigned admin portal
    - review acceptance against the original redesign brief
    - document deferred items and residual risks
    - tie final notes to production verification expectations
  - Verify:
    - docs are complete and internally consistent
    - canary and acceptance artifacts reference real verification steps
  - Exit criteria:
    - rollout checklist exists
    - final signoff notes exist
    - residual risks and deferrals are explicit

# Task Plan — PR-1 Admin Timeline Primitives

## Checklist
- [x] Review the current audit, history sync, and system event/status surfaces plus the ADR-12 contract.
- [x] Design a shared timeline/event-stream primitive with dense-row and drill-down support.
- [x] Implement the primitive and adopt it in the routes that benefit from it.
- [x] Verify the admin web app with `typecheck`, `lint`, and `build`.

## Notes
- No dedicated `apps/admin-web` test harness currently exists in the repo, so branch verification will use the project’s existing frontend gates.
- Keep the write scope focused on shared components, route adapters, and any required CSS.

## Review
- Added shared event stream primitives in `apps/admin-web/src/components/event-stream.tsx`.
- Added matching timeline/event-stream styling in `apps/admin-web/app/globals.css`.
- Replaced the audit ledger table with the shared event stream while preserving drawer drill-down behavior.
- Replaced the history sync recent-events table with the shared event stream.
- Replaced the system workflow runtime snapshot table with the shared event stream.
- Verification passed:
  - `npm --prefix apps/admin-web run typecheck`
  - `npm --prefix apps/admin-web run lint`
  - `npm --prefix apps/admin-web run build`
- `git diff --check -- apps/admin-web tasks/todo.md` passed.
- Note: `apps/admin-web/app/(protected)/api-keys/page.tsx` was already dirty before this PR-1 work and was not part of the timeline primitive implementation.

# Task Plan — PR-2 Admin Command Palette

## Checklist
- [x] Review the current protected shell navigation and high-frequency actions for the ADR-16 contract.
- [x] Design a shell-level command palette with keyboard invocation and route/action registry.
- [x] Implement the command palette in the protected shell.
- [x] Verify the admin web app with `typecheck`, `lint`, and `build`.

## Notes
- Keep the scope to route navigation plus a small set of high-frequency shell actions.
- Reuse the existing shell route model instead of introducing a separate source of truth.

## Review
- Added a new shell-level command palette component in `apps/admin-web/src/components/command-palette.tsx`.
- Wired the protected shell to open the palette from a topbar action and via `Cmd/Ctrl+K`.
- Built the palette registry from the existing shell route model plus high-frequency shell actions (`refresh session`, `refresh current page`, `sign out`).
- Added search, grouped results, arrow-key navigation, enter-to-run, and escape-to-close behavior.
- Verification passed:
  - `npm --prefix apps/admin-web run typecheck`
  - `npm --prefix apps/admin-web run lint`
  - `npm --prefix apps/admin-web run build`
- Note: `apps/admin-web/app/(protected)/api-keys/page.tsx` remains separately dirty and unrelated to PR-2 command palette work.

# Task Plan — PR-3 Admin User Detail Drawer

## Checklist
- [ ] Review the current users index plus the ADR-20 acceptance and validation contract.
- [ ] Design a row-driven user detail drawer using the existing drawer primitive.
- [ ] Move user state, tier, and role controls into grouped drawer action clusters with explicit confirmations.
- [ ] Verify the admin web app with `typecheck`, `lint`, and `build`.

## Notes
- Keep the scope constrained to ADR-20: row drill-down, grouped actions, and keyboard/mouse-safe interaction.
- Reuse the existing users index data and drawer primitive rather than introducing a separate detail route.

## Review
- Reworked the users index so each row now opens a keyboard-accessible management drawer.
- Moved state, tier, and role mutations out of the table row and into grouped drawer action clusters.
- Added explicit confirmation staging inside the drawer before each mutation runs.
- Preserved copy/export/filter behavior while making the table a drill-down surface instead of an inline mutation grid.
- Verification passed:
  - `npm --prefix apps/admin-web run typecheck`
  - `npm --prefix apps/admin-web run lint`
  - `npm --prefix apps/admin-web run build`
  - `git diff --check -- apps/admin-web/app/(protected)/users/page.tsx apps/admin-web/app/globals.css tasks/todo.md`

# Task Plan — PR-4 Admin Bulk Action Patterns

## Checklist
- [ ] Review the ADR-27 acceptance contract plus the current users and API key management tables.
- [ ] Define a shared bulk selection model that can be reused across both pages.
- [ ] Add safe bulk actions with explicit confirmation and aggregated feedback for users and API keys.
- [ ] Verify the admin web app with `typecheck`, `lint`, and `build`.

## Notes
- Keep the PR focused on the shared pattern plus one safe bulk workflow per management page.
- `apps/admin-web/app/(protected)/api-keys/page.tsx` already contains unrelated local secret-reveal work; PR-4 should work with that state without reverting it.

## Review
- Added a shared bulk selection hook and toolbar in `apps/admin-web/src/components/bulk-actions.tsx`.
- Applied the shared selection model to the users table and the API key inventory table.
- Added safe bulk workflows:
  - users: bulk lock and bulk unlock with explicit confirmation
  - API keys: bulk revoke with explicit confirmation
- Added aggregated action feedback and preserved row drill-down behavior by stopping checkbox keyboard/mouse propagation.
- Verification passed:
  - `npm --prefix apps/admin-web run typecheck`
  - `npm --prefix apps/admin-web run lint`
  - `npm --prefix apps/admin-web run build`
  - `git diff --check -- apps/admin-web/src/components/bulk-actions.tsx apps/admin-web/app/(protected)/users/page.tsx apps/admin-web/app/(protected)/api-keys/page.tsx apps/admin-web/app/globals.css tasks/todo.md`
- Caveat:
  - `apps/admin-web/app/(protected)/api-keys/page.tsx` already had uncommitted secret-reveal work before PR-4 started, so the current local diff in that file includes both the earlier secret-reveal changes and the new bulk-action work.

# Task Plan — Discord OAuth Default API Key Provisioning

## Checklist
- [x] Confirm the current Discord OAuth callback path and the intended API-key provisioning contract.
- [x] Add a regression test for first-login API-key provisioning behavior.
- [x] Implement idempotent default API-key creation on successful Discord OAuth login.
- [x] Verify the backend and admin-web with targeted tests plus `typecheck`, `lint`, and `build`.

## Notes
- Keep the auth response contract unchanged: Discord login should still return the existing JWT-shaped `LoginResponse`.
- Provision a default API key only when the OAuth-authenticated user has no active API keys yet.

## Review
- Confirmed the current Discord OAuth callback only created a JWT-backed `LoginResponse` and never provisioned an API key, despite the local auth integration doc claiming first-login auto-generation.
- Added backend-only default API-key provisioning in the Discord OAuth callback path so a successful OAuth login now creates one active default API key when the user has none.
- Kept the public auth response contract unchanged and made provisioning idempotent by checking the user's active key count before creating a new record.
- Added focused regression coverage for the seed builder, first-login provisioning, skip-on-existing-key behavior, and count-failure propagation.
- Verification passed:
  - `cargo fmt --all`
  - `cargo test -p noesis-api oauth_default_api_key --lib`
  - `cargo test -p noesis-api --lib`
  - `npm --prefix apps/admin-web run typecheck`
  - `npm --prefix apps/admin-web run lint`
  - `npm --prefix apps/admin-web run build`
- Remaining limitation:
  - This fix provisions and persists the default API key, but it still does not expose the raw key secret through the Discord OAuth login response or UI flow.

# Task Plan — Discord OAuth Downstream Workflow Verification

## Checklist
- [x] Trace the pushed Discord OAuth provisioning path into API-key auth validation.
- [x] Identify one engine/workflow-facing route that depends on `basic:access`.
- [x] Verify whether the provisioned key would authenticate and satisfy that route's permission/tier gate.
- [x] Record the exact result and any remaining breakpoints.

## Notes
- Goal: confirm the engine/workflow path is intact after first-login key provisioning, not just that a key record gets created.
- This pass is verification/tracing work. No additional behavior changes are planned unless a downstream break is proven.

## Review
- Traced the downstream path:
  - Discord OAuth callback provisions an active API-key row via `AdminRepository::create_api_key(...)`.
  - `create_api_key(...)` writes `is_active = true` in the `api_keys` table.
  - `auth_middleware` accepts `X-API-Key` and calls `AuthService::validate_api_key(...)`.
  - When a Postgres pool is configured, `validate_api_key(...)` validates against the persisted `api_keys` row by `key_hash` and returns `user_id`, `tier`, `permissions`, `rate_limit`, and `consciousness_level`.
  - Workflow execution uses that authenticated user's `consciousness_level` directly when calling the orchestrator.
- Verified a real workflow route with API-key auth:
  - Added `test_workflow_execute_birth_blueprint_success_with_api_key` to `crates/noesis-api/tests/integration_tests.rs`.
  - That test creates an API key, sends `POST /api/v1/workflows/birth-blueprint/execute` with `X-API-Key`, and gets `200 OK` with `workflow_id = "birth-blueprint"`.
- Verification passed:
  - `cargo test -p noesis-api --test integration_tests test_workflow_execute_birth_blueprint_success_with_api_key -- --nocapture`
- Result:
  - the downstream engine/workflow path is intact for API-key-authenticated workflow execution
  - there is no additional break after provisioning in the auth middleware or workflow routing path
- Caveat:
  - the direct executed proof in this environment used the in-memory auth path because this shell did not have `DATABASE_URL` exported
  - the production DB-backed path was verified by code trace against `AdminRepository::create_api_key(...)` and `AuthService::validate_api_key(...)`, which use the same persisted `api_keys` contract

---

# Task Plan — Discord Admin Login + Redirect URI Fix (2026-04-05)

## Checklist
- [x] Confirm root cause locations in admin-web and noesis-api auth/OAuth flow.
- [x] Patch Discord callback override host policy for `*.tryambakam.space`.
- [x] Harden email/password login with normalized email lookup.
- [x] Verify frontend typecheck and backend OAuth tests.
- [x] Record operational follow-ups for credential/account state.

## Notes
- User reported two symptoms:
  - email/password admin login showing `Invalid email or password`
  - Discord redirect URI flow failing from custom admin domain
- Existing OAuth backend validation already enforced same-origin + allowed callback paths; no security relaxation needed.

## Review (fill after execution)
- Root causes addressed in code:
  - `apps/admin-web/src/lib/discord-oauth.ts`
    - enabled dynamic callback override for `*.tryambakam.space`
  - `crates/noesis-api/src/handlers/auth.rs`
    - normalized login email with `trim().to_ascii_lowercase()` before user lookup and logging
  - `crates/noesis-data/src/repositories/user_repository.rs`
    - made email matching case/whitespace-insensitive via `lower(btrim(email)) = lower(btrim($1))`
    - aligned password-reset email lookup with same normalization
- Verification:
  - `npm --prefix apps/admin-web run typecheck` ✅
  - `cargo test -p noesis-api --lib accepts_same_origin_preview_callback_uri -- --nocapture` ✅
  - `cargo test -p noesis-api --lib rejects_cross_origin_callback_uri_override -- --nocapture` ✅
  - `cargo check -p noesis-data` ✅
- Known unrelated test harness issue encountered at the time:
  - this was later resolved during Phase 1 biofield route skeleton work by adding `biofield_repository` to `AppState` and updating the manual test-state constructors.
