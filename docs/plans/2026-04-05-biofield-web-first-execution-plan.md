# Biofield Web-First Execution Plan

Date: 2026-04-05
Repo: `Sheshiyer/Selemene-engine`
Supersedes: `docs/plans/2026-04-05-biofield-viewer-native-integration-plan.md`
Primary goal: ship the first native, authenticated, production-grade web implementation of the biofield viewer inside Selemene without losing the reusable BV-PIP browser pipeline or the Python calculation path.

## Discovery Summary

- Planning depth: deeply detailed
- Delivery mode: production
- CI/CD expectation: production-grade baseline
- Release model: phased rollout
- Quality bar:
  - keep the browser hot path local
  - preserve Python analysis fidelity during migration
  - keep Noesis as the only public backend contract
  - keep Railway as the service topology
  - keep Supabase auth and data ownership intact
- Team topology assumption:
  - small squad plus agent-assisted execution
  - frontend, backend, data, python, and QA concerns can run as separate swarms once contracts are frozen
- External constraints already confirmed:
  - dedicated user-facing biofield surface
  - web first, mobile second
  - do not transplant the standalone BV-PIP app wholesale
  - do not overload `engine-biofield` in Wave 1

## Assumptions And Constraints

- `apps/admin-web` remains separate from the biofield product surface.
- `apps/biofield-web` should follow the existing standalone Next.js pattern from `apps/admin-web`, including `output: "standalone"`.
- Do not introduce a repo-wide JS workspace migration in Phase 1.
- Reuse the generic `readings` table for persisted capture outputs with `engine_id = "biofield-capture"`.
- Add dedicated biofield tables for session and artifact concepts instead of distorting the existing reading model.
- Upgrade the existing `python-services/biofield_cv_service` rather than creating a second Python biofield service tree.
- Keep the public API boundary in Rust:
  - browser -> Noesis API
  - Noesis API -> private Python sidecar
- Use Supabase Storage for capture artifacts if available in the target environment. If storage provisioning lags, keep the storage path contract stable and allow a provider-equivalent object store behind the same repository boundary.
- Mobile support affects shared package design now, but mobile shell implementation is explicitly not part of this execution plan.

## Explicit Non-Goals For This Plan

- No mobile app implementation.
- No per-frame server-side inference loop.
- No full Rust rewrite of the Python analysis logic.
- No immediate rewrite of `engine-biofield` into a live session engine.
- No attempt to merge biofield-web into `apps/admin-web`.

## Recommended File And Service Boundaries

### New web app

- `apps/biofield-web/package.json`
- `apps/biofield-web/next.config.mjs`
- `apps/biofield-web/tsconfig.json`
- `apps/biofield-web/app/(public)/login/page.tsx`
- `apps/biofield-web/app/(protected)/viewer/page.tsx`
- `apps/biofield-web/app/(protected)/history/page.tsx`
- `apps/biofield-web/app/(protected)/readings/[readingId]/page.tsx`
- `apps/biofield-web/src/features/live-pip/*`
- `apps/biofield-web/src/features/capture/*`
- `apps/biofield-web/src/features/history/*`
- `apps/biofield-web/src/lib/api/*`

### New shared TS packages

- `packages/biofield-domain/package.json`
- `packages/biofield-domain/src/session.ts`
- `packages/biofield-domain/src/analysis.ts`
- `packages/biofield-domain/src/history.ts`
- `packages/biofield-domain/src/baseline.ts`
- `packages/biofield-api-client/package.json`
- `packages/biofield-api-client/src/index.ts`
- `packages/biofield-api-client/src/biofield-client.ts`

### Rust API and data layer

- `crates/noesis-api/src/handlers/biofield.rs`
- `crates/noesis-api/src/handlers/mod.rs`
- `crates/noesis-api/src/biofield_client.rs`
- `crates/noesis-api/src/config.rs`
- `crates/noesis-api/src/lib.rs`
- `crates/noesis-data/src/models/biofield.rs`
- `crates/noesis-data/src/models/mod.rs`
- `crates/noesis-data/src/repositories/biofield_repository.rs`
- `crates/noesis-data/src/repositories/mod.rs`

### Persistence and migrations

- `migrations/017_biofield_sessions.sql`
- `supabase/migrations/20260405000017_017_biofield_sessions.sql`

### Python sidecar

- `python-services/biofield_cv_service/main.py`
- `python-services/biofield_cv_service/analyze.py`
- `python-services/biofield_cv_service/algorithms/`
- `python-services/shared/models.py`
- `python-services/tests/test_biofield_analyze.py`
- `python-services/Dockerfile.biofield`

### Docs and operational tooling

- `docs/contracts/biofield-domain.md`
- `docs/contracts/biofield-api.md`
- `docs/contracts/biofield-python-sidecar.md`
- `docs/runbooks/biofield-capture-analysis.md`
- `docs/deployment/RAILWAY.md`
- `scripts/smoke_biofield_web.sh`

## Web-First Route Contract

The web slice should freeze around these backend routes:

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

The initial shipping milestone only requires:

- session create and close
- capture upload and analysis
- persisted reading creation
- history list
- reading detail

Baselines and exports can land after the beta milestone, but the route namespace should be frozen now.

## Data Model Direction

### Reuse

- `readings`
  - use `engine_id = "biofield-capture"`
  - keep `result_data` as the canonical persisted analysis payload
  - keep `input_data` as capture request metadata and processing settings

### Add

- `biofield_sessions`
  - `id`
  - `user_id`
  - `status`
  - `client_device_id`
  - `started_at`
  - `closed_at`
  - `viewer_version`
  - `notes`

- `biofield_capture_artifacts`
  - `id`
  - `session_id`
  - `reading_id`
  - `artifact_kind`
  - `storage_path`
  - `mime_type`
  - `byte_size`
  - `capture_metadata`
  - `created_at`

- later phase tables:
  - `biofield_baselines`
  - `biofield_baseline_readings`
  - optional `biofield_exports`

## Service Boundary Direction

### Browser-local only

- camera access
- MediaPipe segmentation
- WebGL PIP rendering
- lightweight live metrics
- viewer state and timing

### Rust API owned

- auth
- session lifecycle
- multipart capture ingestion
- artifact persistence
- reading persistence
- history, baseline, and export endpoints
- sidecar invocation and retries
- monitoring and audit surface

### Python sidecar owned

- deep capture analysis
- extracted BV-PIP metric logic
- quality assessment
- algorithm versioning
- deterministic capture result assembly

## Phase Map

### Phase 1 - Foundation And Thin Vertical Slice

Objective:
- stand up the app shell, shared packages, API namespace, data seam, and a persisted capture flow that proves the architecture.

Exit criteria:
- authenticated user can open `biofield-web`, start a session, upload a capture, get a Python-backed analysis result, and see the saved reading in history.

### Phase 2 - Local Realtime Viewer UX

Objective:
- make the browser viewer feel native and production-grade without depending on server round trips.

Exit criteria:
- live camera, PIP render, segmentation, HUD, capture controls, and failure handling are stable on desktop and usable on mobile web layouts.

### Phase 3 - Deep Capture Analysis And Persisted History

Objective:
- replace Python stubs with extracted BV-PIP logic, persist artifacts, and make history/readings reliable.

Exit criteria:
- capture analysis is driven by real algorithms, history and detail views are complete, and regression suites prove end-to-end persistence.

### Phase 4 - Baselines, Exports, And Synthesis Hooks

Objective:
- add baseline and export capabilities while preserving future compatibility with workflow-level somatic synthesis.

Exit criteria:
- users can create baselines, compare readings, request exports, and the engine/workflow boundary is documented.

### Phase 5 - Hardening, Ops, And Release

Objective:
- prove the system is shippable on Railway with observability, parity checks, performance budgets, and rollback coverage.

Exit criteria:
- release checklist is complete, parity and performance gates pass, and go/no-go review is documented.

## Detailed Phase 1 Wave And Swarm Layout

### Wave 1.1 - Contract Freeze And Scaffolding

Goal:
- freeze the shared language and create the empty runtime shells before any feature logic lands.

Swarm A - Contract freeze
- own docs/contracts and namespace decisions
- produce the nouns, route shapes, and sidecar request/response contract

Swarm B - Web and package scaffolds
- create `apps/biofield-web`, `packages/biofield-domain`, and `packages/biofield-api-client`
- keep them independently buildable without a repo-wide workspace migration

Swarm C - API seam and smoke plumbing
- add biofield env config
- add sidecar client shell
- add smoke scripts and local setup docs

Wave 1.1 exit:
- every later task can depend on a frozen contract and concrete package/service roots.

### Wave 1.2 - Data Model And Service Seam

Goal:
- add the minimal persistence and route skeleton needed for a real vertical slice.

Swarm D - Data model and migrations
- create biofield session/artifact schema
- wire repository layer

Swarm E - API route skeleton
- create handler module, route registration, and OpenAPI surface

Swarm F - Python contract alignment
- version the request/response contract
- add contract tests before algorithm extraction

Wave 1.2 exit:
- Noesis can accept biofield traffic structurally even before the viewer is fully ported.

### Wave 1.3 - Thin Vertical Slice

Goal:
- prove browser -> Noesis -> Python -> persistence -> history works end to end.

Swarm G - Viewer shell
- authenticated viewer page, camera bootstrap, device readiness, renderer port

Swarm H - Capture and persistence flow
- session create, capture upload, sidecar proxy, reading save

Swarm I - Validation and docs
- integration tests, smoke script, local bootstrap docs

Wave 1.3 exit:
- the web-first architecture is proven with a real saved capture flow.

## Full Task List

### Phase 1 - Foundation And Thin Vertical Slice

#### Wave 1.1 - Contract Freeze And Scaffolding

| ID | Title | Area | Owner Role | Hrs | Dependencies | Deliverable | Acceptance | Validation |
|---|---|---|---|---:|---|---|---|---|
| BWF-001 | Freeze biofield domain nouns and state model | product | Tech Lead | 6 | - | `docs/contracts/biofield-domain.md` defines session, capture, reading, baseline, and state transitions. | Domain doc is approved and referenced by web, API, and Python sections. | Doc diff plus checklist review. |
| BWF-002 | Freeze Noesis biofield route contract | backend | Backend Eng | 6 | BWF-001 | `docs/contracts/biofield-api.md` defines route shapes, auth rules, and payloads. | Route namespace and minimum beta endpoints are stable. | Contract doc review and route inventory check. |
| BWF-003 | Freeze Python sidecar request and response contract | backend | Python Eng | 5 | BWF-001 | `docs/contracts/biofield-python-sidecar.md` defines multipart fields, options, and response versioning. | Sidecar contract can be consumed by Rust without ad hoc translation. | Contract doc review and sample payload snapshot. |
| BWF-004 | Scaffold `packages/biofield-domain` | frontend | Frontend Eng | 5 | BWF-001 | Package skeleton with initial types and build scripts exists. | Package installs and typechecks independently. | `npm --prefix packages/biofield-domain run typecheck`. |
| BWF-005 | Scaffold `packages/biofield-api-client` | frontend | Frontend Eng | 6 | BWF-001, BWF-002 | Package skeleton with typed client entrypoints exists. | Package builds against the frozen API contract. | `npm --prefix packages/biofield-api-client run typecheck`. |
| BWF-006 | Scaffold `apps/biofield-web` as standalone Next app | frontend | Frontend Eng | 8 | BWF-004, BWF-005 | New Next app mirrors the standalone build pattern used by `apps/admin-web`. | App boots locally with placeholder protected routes and no root workspace migration. | `npm --prefix apps/biofield-web run build`. |
| BWF-007 | Add biofield env and config surfaces to `noesis-api` | backend | Backend Eng | 5 | BWF-002, BWF-003 | `ApiConfig` exposes biofield service URL, timeouts, and feature flags. | API config validates biofield-specific envs without breaking existing services. | `cargo test -p noesis-api config`. |
| BWF-008 | Create Noesis biofield sidecar client shell | backend | Backend Eng | 7 | BWF-003, BWF-007 | `crates/noesis-api/src/biofield_client.rs` can call the Python service with typed request handling. | API code can construct sidecar requests without handler duplication. | Targeted `cargo test -p noesis-api biofield_client`. |
| BWF-009 | Add Phase 1 smoke script and local bootstrap notes | qa | QA Eng | 4 | BWF-006, BWF-007, BWF-008 | `scripts/smoke_biofield_web.sh` and local setup notes cover the vertical slice path. | Another engineer can reproduce the bootstrap steps without tribal knowledge. | Bash syntax check and doc walkthrough. |

#### Wave 1.2 - Data Model And Service Seam

| ID | Title | Area | Owner Role | Hrs | Dependencies | Deliverable | Acceptance | Validation |
|---|---|---|---|---:|---|---|---|---|
| BWF-010 | Design biofield session and artifact schema | data | Data Eng | 6 | BWF-001, BWF-002 | Schema design covers `biofield_sessions` and `biofield_capture_artifacts` plus reading linkage. | Schema supports the beta flow without changing the existing reading contract. | Schema review checklist. |
| BWF-011 | Author root migration for biofield tables | data | Data Eng | 6 | BWF-010 | `migrations/017_biofield_sessions.sql` exists. | Migration applies cleanly after `016_dodo_billing_foundation.sql`. | Migration dry run and SQL review. |
| BWF-012 | Mirror biofield migration into Supabase migration tree | data | Data Eng | 4 | BWF-010 | Matching Supabase migration file exists. | Local Supabase path stays in sync with root migration history. | Diff review between root and Supabase migrations. |
| BWF-013 | Add `noesis-data` biofield models | data | Backend Eng | 5 | BWF-011 | New session and artifact models exist under `crates/noesis-data/src/models`. | Rust types map cleanly to the migration schema. | `cargo test -p noesis-data models`. |
| BWF-014 | Implement `biofield_repository` create and query primitives | data | Backend Eng | 8 | BWF-013 | Repository supports session create/get, artifact create/list, and reading linkage. | Repository covers all Phase 1 write and read paths. | `cargo test -p noesis-data biofield_repository`. |
| BWF-015 | Wire biofield repository into exports and app state creation | backend | Backend Eng | 5 | BWF-014 | Repository is exported and constructed anywhere `AppState` is built. | API startup can access the biofield repository in prod and test harnesses. | `cargo test -p noesis-api test_harness`. |
| BWF-016 | Add biofield handler module skeleton | backend | Backend Eng | 6 | BWF-002, BWF-015, BWF-008 | `crates/noesis-api/src/handlers/biofield.rs` exists with placeholder route handlers and DTOs. | Handler module compiles and owns the biofield route namespace. | `cargo test -p noesis-api biofield_handler_smoke`. |
| BWF-017 | Register biofield routes and OpenAPI surface | backend | Backend Eng | 6 | BWF-016 | `lib.rs` wires biofield routes and OpenAPI docs. | The route namespace is visible in the API router and docs output. | Router smoke test plus OpenAPI snapshot. |
| BWF-018 | Add sidecar contract versioning models in Python shared models | backend | Python Eng | 5 | BWF-003 | `python-services/shared/models.py` exposes versioned biofield response models. | Rust and Python agree on required fields and version tags. | `pytest python-services/tests/test_biofield_analyze.py -k contract`. |
| BWF-019 | Add Python-side invalid payload and contract parity tests | qa | Python Eng | 5 | BWF-018 | Python tests cover malformed uploads, options parsing, and required response shape. | Sidecar rejects invalid inputs deterministically and keeps the frozen contract. | `pytest python-services/tests/test_biofield_analyze.py`. |

#### Wave 1.3 - Thin Vertical Slice

| ID | Title | Area | Owner Role | Hrs | Dependencies | Deliverable | Acceptance | Validation |
|---|---|---|---|---:|---|---|---|---|
| BWF-020 | Scaffold authenticated biofield-web layout and env wiring | frontend | Frontend Eng | 7 | BWF-006, BWF-005 | App layout, protected routes, and env bootstrap are in place. | Authenticated pages render with placeholder session data. | `npm --prefix apps/biofield-web run build`. |
| BWF-021 | Port camera bootstrap and device readiness modules | frontend | Frontend Eng | 8 | BWF-020 | Camera permissions, device checks, and fallback messaging are wired into the viewer shell. | Supported devices can reach a ready state without server involvement. | Browser smoke check plus typecheck. |
| BWF-022 | Port WebGL PIP renderer into feature boundary | frontend | Frontend Eng | 10 | BWF-020 | BV-PIP renderer lives under `src/features/live-pip` with local inputs and no standalone app assumptions. | Viewer can render a local PIP scene inside biofield-web. | Local browser verification plus build. |
| BWF-023 | Implement biofield session create and close flow | backend | Backend Eng | 8 | BWF-017, BWF-014, BWF-020 | Session routes create and close `biofield_sessions` and return typed payloads. | Viewer can obtain and close a real server-backed session ID. | `cargo test -p noesis-api biofield_sessions`. |
| BWF-024 | Implement capture upload client and server multipart path | backend | Full Stack Eng | 8 | BWF-020, BWF-017, BWF-003 | Browser client can submit multipart captures to the new biofield API route. | Uploads reach the handler with validated image payloads. | API integration test plus browser upload smoke. |
| BWF-025 | Proxy capture analysis from Noesis to Python sidecar | backend | Backend Eng | 8 | BWF-024, BWF-008, BWF-019 | Handler forwards capture payloads to the private sidecar and returns typed analysis. | A capture request succeeds through the full HTTP hop chain. | Targeted API test with mocked or local sidecar. |
| BWF-026 | Persist reading row and artifact link on successful capture | backend | Backend Eng | 8 | BWF-025, BWF-014 | Successful analysis saves a `readings` row and a linked biofield artifact record. | Each completed capture has durable history metadata. | Repository assertion test plus API integration test. |
| BWF-027 | Render saved reading summary and history stub in biofield-web | frontend | Frontend Eng | 6 | BWF-026, BWF-005 | Web app shows the latest saved capture result and a basic history list. | User can confirm persistence from the app after one capture. | Browser flow test. |
| BWF-028 | Add vertical slice integration tests and local run docs | qa | QA Eng | 6 | BWF-021, BWF-026, BWF-027 | Integration tests and docs prove browser to API to sidecar to DB flow. | Another engineer can reproduce the Phase 1 slice reliably. | Full vertical slice smoke run. |

### Phase 2 - Local Realtime Viewer UX

| ID | Title | Area | Owner Role | Hrs | Dependencies | Deliverable | Acceptance | Validation |
|---|---|---|---|---:|---|---|---|---|
| BWF-029 | Audit remaining reusable BV-PIP frontend modules | frontend | Frontend Eng | 5 | BWF-028 | Inventory identifies what is still portable and what must be rewritten. | Porting work has an explicit source map instead of ad hoc copying. | Audit doc committed and reviewed. |
| BWF-030 | Port segmentation adapter into browser-only module | frontend | Frontend Eng | 10 | BWF-029, BWF-022 | Segmentation logic is isolated under `src/features/live-pip` with no backend dependency. | Viewer supports live person segmentation locally. | Browser verification and perf trace. |
| BWF-031 | Port lightweight live metric subset to browser-only module | frontend | Frontend Eng | 9 | BWF-029, BWF-022 | Local viewer computes the approved low-cost metric subset client-side. | HUD updates without capture uploads or sidecar calls. | Unit tests and browser demo. |
| BWF-032 | Normalize live metric and event types in `biofield-domain` | frontend | Frontend Eng | 5 | BWF-031, BWF-004 | Shared types exist for live metrics, capture metadata, and viewer events. | Web and future mobile work can reuse the same TS contract. | `npm --prefix packages/biofield-domain run typecheck`. |
| BWF-033 | Build viewer session store and reducer | frontend | Frontend Eng | 7 | BWF-032, BWF-020 | Viewer state management owns device, live metrics, and capture UI state. | Viewer logic is testable without tightly coupling to page components. | Component tests and typecheck. |
| BWF-034 | Build HUD cards and status badges | frontend | Frontend Eng | 8 | BWF-031, BWF-033 | Viewer shows readable live status, segmentation, and metric cards. | Operators can understand live state without opening devtools. | Visual QA screenshots plus build. |
| BWF-035 | Build capture controls and cooldown logic | frontend | Frontend Eng | 7 | BWF-033, BWF-024 | Capture controls manage cooldowns, disabled states, and in-flight uploads. | Users cannot spam uploads or lose track of capture state. | Browser interaction test. |
| BWF-036 | Build permission-denied and unsupported-device UX | frontend | Frontend Eng | 6 | BWF-021, BWF-033 | Dedicated UX exists for no camera, denied permissions, and unsupported browsers. | Failures are recoverable and clearly explained. | Manual matrix QA plus Playwright check. |
| BWF-037 | Add responsive mobile-web viewer layout | frontend | Frontend Eng | 8 | BWF-034, BWF-035 | Viewer layout remains usable on narrow viewports. | The web-first app works on mobile browsers without a separate mobile codebase. | Responsive screenshot sweep. |
| BWF-038 | Add live performance instrumentation hooks | frontend | Frontend Eng | 5 | BWF-030, BWF-031 | Viewer logs FPS, segmentation cadence, and capture latency metrics locally. | Performance regressions become measurable during beta. | Metric log snapshot and browser trace. |
| BWF-039 | Add camera recovery and reconnect UX | frontend | Frontend Eng | 6 | BWF-021, BWF-033 | Viewer can recover from camera disconnects or stream restarts. | Camera interruptions do not force full page reloads. | Browser fault-injection test. |
| BWF-040 | Add feature flag and protected route gating | backend | Full Stack Eng | 5 | BWF-020, BWF-034 | Biofield-web can be enabled per environment without leaking unfinished routes. | Rollout can be staged safely. | Config test plus route guard check. |
| BWF-041 | Add end-to-end happy path test from login to live viewer | qa | QA Eng | 8 | BWF-035, BWF-040 | E2E test covers auth, viewer boot, and session start. | The core happy path is reproducible in CI. | Playwright or equivalent green run. |
| BWF-042 | Benchmark local render and segmentation loop against budgets | qa | QA Eng | 6 | BWF-038, BWF-041 | Benchmark report documents CPU and latency budgets for the browser hot path. | Viewer meets or documents exceptions to agreed performance budgets. | Benchmark report committed. |

### Phase 3 - Deep Capture Analysis And Persisted History

| ID | Title | Area | Owner Role | Hrs | Dependencies | Deliverable | Acceptance | Validation |
|---|---|---|---|---:|---|---|---|---|
| BWF-043 | Extract BV-PIP Python algorithm modules into repo package | backend | Python Eng | 12 | BWF-019 | Reusable algorithm modules live under `python-services/biofield_cv_service/algorithms`. | Real logic is imported from extracted modules instead of the stub generator. | `pytest python-services/tests/test_biofield_analyze.py`. |
| BWF-044 | Replace stub metric generation with real algorithm pipeline | backend | Python Eng | 14 | BWF-043, BWF-025 | `/analyze` runs real metric calculation code on uploaded captures. | Sidecar response values come from extracted algorithms, not seeded mocks. | Golden-fixture test run. |
| BWF-045 | Add algorithm selection and version tagging in response payload | backend | Python Eng | 6 | BWF-044, BWF-018 | Sidecar responses declare algorithm set and version metadata. | Persisted readings can always be traced to a calculation version. | Contract snapshot update and tests. |
| BWF-046 | Add image quality rejection semantics and operator-facing reasons | backend | Python Eng | 7 | BWF-044 | Sidecar returns deterministic rejection reasons for unusable captures. | Low-quality captures fail clearly instead of producing misleading readings. | Negative-case pytest coverage. |
| BWF-047 | Add fixture corpus and golden-output tests | qa | QA Eng | 8 | BWF-044, BWF-045 | Curated test corpus covers representative capture shapes and expected outputs. | Algorithm changes cannot land without parity evidence. | Corpus test suite in CI. |
| BWF-048 | Add server-side upload validation and sanitization | backend | Backend Eng | 6 | BWF-024, BWF-046 | API validates mime type, size, and basic payload safety before sidecar calls. | Invalid uploads are rejected at the API edge. | `cargo test -p noesis-api biofield_upload_validation`. |
| BWF-049 | Integrate object storage for source captures and overlays | infra | Backend Eng | 10 | BWF-026, BWF-048 | Capture artifacts are stored in a durable bucket with stable path conventions. | Saved readings can retrieve their source artifact metadata. | Storage integration test plus smoke upload. |
| BWF-050 | Persist artifact metadata and retrieval URLs | data | Backend Eng | 6 | BWF-049, BWF-014 | Artifact rows include storage paths, sizes, and retrieval metadata. | History and detail views can resolve the saved artifact metadata. | Repository test plus API response check. |
| BWF-051 | Add paginated biofield history API | backend | Backend Eng | 7 | BWF-050, BWF-026 | API lists persisted biofield readings with session and artifact metadata. | History queries support pagination and user scoping. | `cargo test -p noesis-api biofield_history`. |
| BWF-052 | Add biofield reading detail API | backend | Backend Eng | 6 | BWF-050, BWF-045 | API returns a single reading with metric groups, quality, and artifact metadata. | Web detail page can render without extra client-side joins. | `cargo test -p noesis-api biofield_reading_detail`. |
| BWF-053 | Build biofield history list UI | frontend | Frontend Eng | 8 | BWF-051, BWF-027 | History page shows saved captures with filters, pagination, and status. | Users can browse prior captures without direct API tooling. | Browser flow test and screenshots. |
| BWF-054 | Build reading detail UI with metrics and quality panels | frontend | Frontend Eng | 9 | BWF-052, BWF-053 | Reading detail page visualizes metric groups, quality gates, and artifact metadata. | Detailed reading review is possible entirely from the web app. | Browser flow test and visual QA. |
| BWF-055 | Add retry and reprocess route and UI action | backend | Full Stack Eng | 7 | BWF-052, BWF-046 | Failed or outdated captures can be reprocessed on demand. | Users and operators can rerun analysis without new uploads when allowed. | API test plus browser action check. |
| BWF-056 | Add end-to-end capture to history regression suite | qa | QA Eng | 8 | BWF-053, BWF-054, BWF-055 | Regression suite proves capture upload, analysis, persistence, history, and detail rendering. | The beta slice is covered by repeatable end-to-end checks. | Full CI green run. |

### Phase 4 - Baselines, Exports, And Synthesis Hooks

| ID | Title | Area | Owner Role | Hrs | Dependencies | Deliverable | Acceptance | Validation |
|---|---|---|---|---:|---|---|---|---|
| BWF-057 | Design baseline contract and comparison semantics | product | Tech Lead | 6 | BWF-051, BWF-052 | Baseline inputs, outputs, and lifecycle are documented. | Baseline work does not distort the capture-reading contract. | Contract doc review. |
| BWF-058 | Add baseline tables and migrations | data | Data Eng | 8 | BWF-057 | Schema exists for baselines and baseline-reading linkage. | Migration applies cleanly after Phase 3 schema. | Migration dry run and review. |
| BWF-059 | Add baseline repository and API surfaces | backend | Backend Eng | 9 | BWF-058, BWF-014 | Repository and routes support baseline create, list, and get flows. | Baseline CRUD is available to the web app. | `cargo test -p noesis-api biofield_baselines`. |
| BWF-060 | Build create-baseline UI from selected readings | frontend | Frontend Eng | 8 | BWF-059, BWF-053 | History UI can select readings and create a baseline. | Users can create baselines without leaving the web app. | Browser interaction test. |
| BWF-061 | Implement baseline comparison calculation contract | backend | Backend Eng | 8 | BWF-057, BWF-059, BWF-045 | API returns normalized deltas between a reading and a baseline. | Comparison results are versioned and reproducible. | Contract test and repository assertions. |
| BWF-062 | Build comparison UI and delta visualization | frontend | Frontend Eng | 9 | BWF-061, BWF-060, BWF-054 | Reading detail page can compare current reading versus selected baseline. | Users can interpret baseline deltas visually. | Browser flow and screenshot review. |
| BWF-063 | Design export job contract | product | Tech Lead | 5 | BWF-052, BWF-057 | Export request types and artifact bundle rules are documented. | Export scope is frozen before backend implementation starts. | Contract doc review. |
| BWF-064 | Implement export backend and artifact persistence | backend | Backend Eng | 10 | BWF-063, BWF-049, BWF-050 | Backend can generate and persist export bundles for biofield readings. | Requested exports complete and store retrievable artifacts. | API integration test plus artifact existence check. |
| BWF-065 | Add export request and download UI | frontend | Frontend Eng | 7 | BWF-064, BWF-054 | Users can request and download export artifacts from the web app. | Export flow is self-service in the authenticated product. | Browser flow test. |
| BWF-066 | Define synthesis hook contract to engine and workflow layer | backend | Tech Lead | 6 | BWF-061, BWF-067 | Contract defines how capture-derived readings can later feed somatic synthesis without mutating current engines. | Future workflow work has a stable seam and no Phase 1 semantic regressions. | ADR review. |
| BWF-067 | Write ADR for `biofield-capture` versus `engine-biofield` boundary | product | Tech Lead | 5 | BWF-001, BWF-066 | ADR records why session/capture logic stays separate in the first web release. | The boundary decision is explicit and reviewable. | ADR committed and linked from plan docs. |
| BWF-068 | Add baseline and export verification docs and tests | qa | QA Eng | 6 | BWF-062, BWF-065, BWF-066 | Test plan and runbook cover baseline and export flows. | Later feature work has the same proof standard as the beta slice. | QA doc review plus targeted tests. |

### Phase 5 - Hardening, Ops, And Release

| ID | Title | Area | Owner Role | Hrs | Dependencies | Deliverable | Acceptance | Validation |
|---|---|---|---|---:|---|---|---|---|
| BWF-069 | Review auth, ownership, and RLS posture for biofield entities | security | Backend Eng | 7 | BWF-059, BWF-050 | Security review covers tables, routes, storage ownership, and user scoping. | Biofield data cannot cross user boundaries. | Review notes plus auth tests. |
| BWF-070 | Add route-specific rate limits and payload limits | backend | Backend Eng | 6 | BWF-048, BWF-069 | Biofield routes enforce body limits and tighter request policies. | Abuse scenarios are constrained without breaking normal capture flows. | API rate-limit test and config diff. |
| BWF-071 | Add Railway service config and runbook for biofield sidecar | infra | DevOps | 6 | BWF-044, BWF-049 | Railway deployment guidance covers the Python sidecar as a first-class service. | Ops can deploy and restart the sidecar without ad hoc steps. | Doc review and container boot smoke. |
| BWF-072 | Add CI jobs for biofield-web, shared packages, Python tests, and Docker build | infra | DevOps | 8 | BWF-056, BWF-071 | CI validates the new app, packages, Rust routes, Python tests, and sidecar container. | Biofield changes are gated by automated verification. | CI workflow green run. |
| BWF-073 | Add observability dashboards and alerts for biofield routes and sidecar | infra | DevOps | 8 | BWF-055, BWF-071 | Monitoring covers latency, error rate, sidecar health, and capture failure spikes. | Production incidents become detectable within minutes. | Alert rule review and synthetic probes. |
| BWF-074 | Add incident runbook for capture-analysis failures | qa | DevOps | 5 | BWF-073 | Runbook explains how to diagnose viewer, API, sidecar, and storage failures. | On-call responders can follow a deterministic debug path. | Runbook tabletop review. |
| BWF-075 | Add phased rollout plan and environment-specific flags | product | Tech Lead | 5 | BWF-040, BWF-073 | Rollout plan defines dev, staging, beta, and public availability gates. | Biofield-web can be exposed incrementally instead of all at once. | Rollout checklist review. |
| BWF-076 | Run corpus parity verification against standalone BV-PIP | qa | QA Eng | 10 | BWF-047, BWF-044 | Parity report compares captured outputs against the standalone implementation on a curated corpus. | Migration does not silently drift away from known-good outputs. | Committed parity report. |
| BWF-077 | Run API and sidecar load and timeout verification | qa | QA Eng | 8 | BWF-072, BWF-073 | Load test report covers upload latency, sidecar throughput, and timeout behavior. | Production limits and timeouts are evidence-based. | Load report committed. |
| BWF-078 | Run accessibility, responsive, and manual QA sweep | qa | QA Eng | 8 | BWF-037, BWF-065, BWF-075 | Manual QA covers keyboarding, contrast, resizing, and common failure paths. | Web product meets the agreed accessibility and UX bar. | QA checklist and screenshots. |
| BWF-079 | Prepare release checklist and rollback steps | product | Tech Lead | 6 | BWF-074, BWF-075, BWF-077 | Release doc includes deploy order, smoke checks, rollback, and comms. | Launch can happen without improvising operational steps. | Release checklist review. |
| BWF-080 | Hold ship-readiness review and go/no-go signoff | product | Tech Lead | 4 | BWF-076, BWF-077, BWF-078, BWF-079 | Final review records launch status, blockers, and approved scope. | The team has an explicit documented decision to ship or hold. | Signed review notes linked from `tasks/todo.md`. |

## Dependency Rationale

1. Freeze contracts before scaffolding so the app, API, and sidecar all target the same nouns and payloads.
2. Scaffold packages before feature code so the web-first work stays mobile-ready without forcing a workspace migration.
3. Land migrations and repository plumbing before handler logic so persistence semantics are explicit.
4. Land the vertical slice before deep UX work so the chosen architecture is proven end to end early.
5. Replace Python stubs only after contract tests and corpus fixtures exist.
6. Defer baselines and exports until the core capture and history path is stable.
7. Put parity, performance, and rollout work in the plan as first-class delivery tasks rather than postscript cleanup.

## Verification Strategy

### Per-layer commands

- Web app:
  - `npm --prefix apps/biofield-web run typecheck`
  - `npm --prefix apps/biofield-web run lint`
  - `npm --prefix apps/biofield-web run build`
- Shared packages:
  - `npm --prefix packages/biofield-domain run typecheck`
  - `npm --prefix packages/biofield-api-client run typecheck`
  - `npm --prefix packages/biofield-api-client run build`
- Rust:
  - `cargo test -p noesis-data biofield`
  - `cargo test -p noesis-api biofield`
- Python:
  - `pytest python-services/tests/test_biofield_analyze.py`
  - `docker build -f python-services/Dockerfile.biofield python-services`

### Required proofs by milestone

- Phase 1:
  - vertical slice smoke run
  - viewer boot plus session create
  - capture upload plus persisted reading visible in history
- Phase 2:
  - browser hot path benchmark report
  - responsive viewer screenshots
- Phase 3:
  - golden corpus parity suite
  - capture-to-history regression suite
- Phase 4:
  - baseline and export flow verification
  - ADR and contract docs linked
- Phase 5:
  - load report
  - alert coverage
  - release and rollback checklist

## GitHub Sync And Dispatch Strategy

Recommended issue model:

- one umbrella epic:
  - `Epic: Biofield Web First`
- one issue per Phase 1 wave:
  - `Phase 1 / Wave 1.1`
  - `Phase 1 / Wave 1.2`
  - `Phase 1 / Wave 1.3`
- one issue per later phase:
  - `Phase 2`
  - `Phase 3`
  - `Phase 4`
  - `Phase 5`
- task IDs in issue bodies as checklists so progress can be mirrored without losing the task inventory

Recommended labels:

- `biofield`
- `phase:1` through `phase:5`
- `wave:1.1` through `wave:1.3`
- `area:frontend`
- `area:backend`
- `area:data`
- `area:infra`
- `area:qa`

Dispatch guidance:

- dispatch swarms only after the contract tasks for that wave are marked complete
- do not run frontend, backend, and python extraction tasks in parallel until contract IDs for that wave are closed
- use task IDs in PR titles or bodies to preserve traceability

## Risks And Fallback Plan

### Risk 1 - Browser hot path is heavier than expected

Fallback:
- keep live HUD smaller
- reduce segmentation cadence
- keep only capture-time deep analysis on the server

### Risk 2 - Python algorithm extraction takes longer than expected

Fallback:
- hold public beta behind a feature flag
- keep the Phase 1 vertical slice for internal beta while finishing corpus and parity work

### Risk 3 - Supabase Storage integration is delayed

Fallback:
- keep the artifact repository boundary stable
- ship persisted readings first
- gate downloadable artifacts until storage is live

### Risk 4 - Route shape drifts across layers

Fallback:
- treat contract docs and sidecar snapshots as blocking artifacts
- reject implementation work that changes shapes without updating the frozen contracts

### Risk 5 - `engine-biofield` scope creep returns

Fallback:
- enforce the ADR from BWF-067
- keep session and capture semantics in the dedicated biofield namespace until a later synthesis phase explicitly changes that boundary

## Recommended Execution Order

If implementation starts immediately after this plan, the highest-leverage order is:

1. Phase 1 / Wave 1.1
2. Phase 1 / Wave 1.2
3. Phase 1 / Wave 1.3
4. Phase 2
5. Phase 3
6. Phase 5 hardening tasks needed for beta
7. Phase 4 baselines and exports if beta feedback does not force reprioritization

That ordering ships the core product sooner:

- real app
- real capture flow
- real persistence
- then refined analysis, history, baselines, and exports
