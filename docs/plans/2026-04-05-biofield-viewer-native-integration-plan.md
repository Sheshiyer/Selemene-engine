# Biofield Viewer Native Integration Plan

Date: 2026-04-05
Repo: `Sheshiyer/Selemene-engine`
Primary surfaces:
- `crates/noesis-api`
- `crates/noesis-orchestrator`
- `crates/noesis-bridge`
- `crates/engine-biofield`
- `crates/noesis-data`
- `apps/` future authenticated user-facing biofield surface
- `docs/deployment/RAILWAY.md`
- `/Volumes/madara/2026/twc-vault/01-Projects/BV-PIP/bv-pip-analysis`

## Discovery Summary

- Planning depth: deeply detailed
- Delivery mode: production
- Release model: phased rollout
- CI/CD expectation: production-grade
- Quality bar:
  - no loss of BV-PIP calculation fidelity during the first integration waves
  - Railway remains the authoritative compute stack
  - Supabase-backed auth and persistence remain the system of record
  - browser hot-path processing stays low-latency and does not depend on per-frame server round trips
  - internal Python services are private infrastructure, not a second public product
  - the standalone BV-PIP project is mined for proven code, not transplanted whole
- Team / agent topology:
  - planner / orchestrator: architecture, contract freeze, phase gating
  - frontend: native viewer surface, camera pipeline, client metrics, UX
  - python / cv: service extraction, parity preservation, CPU-only Railway packaging
  - backend / data: Noesis endpoints, repositories, Supabase schema, auth ownership
  - validation / ops: parity corpus, performance, Railway health, rollout gates

## Confirmed Product Direction

- A dedicated user-facing biofield product surface is required.
- It must support both:
  - web
  - mobile
- Delivery order is:
  - web first
  - mobile second
- `apps/admin-web` is explicitly out of scope as the long-term home for this feature.
- The architecture should therefore optimize for:
  - shared domain contracts across web and mobile
  - shared backend APIs and persistence
  - runtime-specific UI shells where camera, permissions, and rendering differ

## Inputs Reviewed

- Selemene architecture and runtime:
  - `.context/architecture/overview.md`
  - `.context/workflows.md`
  - `.context/engines/biofield.md`
  - `docs/contributing/engine-onboarding.md`
  - `docs/deployment/RAILWAY.md`
  - `docs/PYTHON_SIDECAR_GUIDE.md`
  - `docs/PROJECT_OVERVIEW.md`
  - `crates/engine-biofield/src/lib.rs`
  - `crates/engine-biofield/src/engine.rs`
  - `crates/noesis-bridge/src/lib.rs`
  - `crates/noesis-bridge/src/python_client.rs`
  - `crates/noesis-orchestrator/src/lib.rs`
  - `crates/noesis-orchestrator/src/workflow/registry.rs`
  - `crates/noesis-api/src/lib.rs`
  - `crates/noesis-api/src/config.rs`
  - `crates/noesis-core/src/types.rs`
- BV-PIP standalone implementation:
  - `PIP_Analysis_System_Specification.md`
  - `PIP_Implementation_Plan.md`
  - `backend/main.py`
  - `backend/config.py`
  - `backend/requirements.txt`
  - `backend/api/routes/analysis.py`
  - `backend/api/routes/websocket.py`
  - `frontend/package.json`
  - `frontend/src/App.tsx`
  - `frontend/src/services/PIPRenderer.ts`

## What The Investigation Shows

### Selemene side

- Selemene already has the right runtime seam for this work:
  - Rust API and orchestrator are authoritative.
  - TypeScript engines already run as a separate Railway sidecar.
  - The repo already documents a Python sidecar pattern for `biofield-cv` and MediaPipe services.
- `engine-biofield` is currently a stub / mock-oriented engine with a birth-data-driven conceptual contract.
- Workflows already include `biofield`, but that current engine contract is not the same thing as a live camera-driven PIP viewer.
- Supabase Postgres is the current persistence layer for auth and runtime data.
- Railway deployment is already multi-service, so adding a Python CV service is architecturally normal, not exceptional.

### BV-PIP side

- The strongest reusable assets are:
  - the WebGL PIP renderer
  - the browser camera / segmentation / metric pipeline
  - the Python metric and score calculation modules
- The standalone backend should not be adopted as-is:
  - realtime websocket routes are still placeholder-oriented
  - history and database read paths are TODOs
  - config assumes a standalone Postgres + Redis + local upload directory
  - CORS / transport / persistence are built for an isolated app, not a shared platform
- The standalone frontend is closer to reusable production code than the standalone backend shell.

## Architectural Decision Set

### Decision 1: What becomes native to Selemene?

Recommendation:
- make the viewer, capture flow, history, baseline management, and synthesis surfaces native to Selemene
- do not keep BV-PIP as a separate product shell

Implication:
- Noesis owns the product contract
- BV-PIP contributes reusable frontend modules and Python calculation logic

### Decision 2: Where does the realtime hot path live?

Recommendation:
- keep PIP rendering, MediaPipe segmentation, and lightweight realtime metrics in the browser
- do not stream every frame to Railway

Why:
- Railway is a bad fit for high-frequency frame analysis over public network links
- BV-PIP already assumes client-side rendering and low-latency metrics
- the current standalone websocket backend is not a production-strength reason to move the hot path server-side

### Decision 3: What happens to the Python calculation code?

Recommendation:
- keep Python as the source of truth for deeper capture analysis in the first migration waves
- package it as an internal Railway sidecar service behind Noesis
- do not rewrite the calculation stack into Rust before parity harnesses exist

Why:
- it preserves metric fidelity
- it aligns with the documented Python sidecar pattern already present in Selemene
- it avoids a high-risk parity rewrite before the feature is productized

### Decision 4: Does the viewer become a generic engine route?

Recommendation:
- no, not in Wave 1
- create a dedicated `biofield` session / capture domain in the API
- keep the current `engine-biofield` contract separate until capture-native semantics are stabilized

Why:
- a live viewer is session-oriented and image-driven
- `ConsciousnessEngine::calculate()` is a poor fit for camera sessions, uploads, baselines, and exports
- forcing the viewer into the current engine route would muddle engine semantics and break current workflow assumptions

## Options Considered

| Option | Description | Advantages | Risks | Verdict |
|---|---|---|---|---|
| A | Full Rust rewrite before integration | Single runtime, long-term simplification | Highest parity risk, longest timeline, easy to lose Python-calculation fidelity | Reject for first implementation |
| B | Keep BV-PIP as a separate standalone app and link to it | Fastest apparent delivery | Split auth, split persistence, split observability, not truly native to Selemene | Reject |
| C | Native Selemene product shell plus internal Python biofield sidecar | Preserves calculation logic, fits Railway topology, clean auth boundary, incremental migration | Requires deliberate contract design and parity discipline | Recommended |
| D | Force all viewer behavior through `engine-biofield` immediately | Reuses existing engine ID | Session, upload, and history semantics become awkward and brittle | Reject |

## Recommended Target Architecture

### Product boundary

Treat the biofield viewer as a first-class authenticated product capability with four nouns:

- `session`
- `capture`
- `reading`
- `baseline`

That means the user-facing surface is not "call an engine". It is:

1. start a biofield session
2. render live camera + PIP locally
3. compute lightweight client metrics locally
4. capture frames or upload images for deep analysis
5. persist readings, baselines, and exports through Noesis
6. optionally synthesize those stored readings into higher-order somatic insights later

### Railway topology

```
Authenticated Viewer App
    |
    | JWT / HTTPS
    v
Noesis API (Rust, Railway)
    |
    | private internal HTTP
    v
Biofield Python Service (Railway internal only)
    |
    +--> Supabase Postgres via Noesis repositories
    +--> Supabase Storage or equivalent object storage via Noesis-owned writes
    +--> Redis only when caching or queueing is justified
```

Key rule:
- the browser never talks directly to the Python sidecar

### Runtime split

Browser responsibilities:
- camera access
- WebGL2 PIP shader rendering
- MediaPipe segmentation
- low-latency metrics and charts
- session UX

Noesis API responsibilities:
- auth and ownership
- session lifecycle
- capture ingestion
- persistence
- history / baseline / export APIs
- workflow integration
- private invocation of Python analysis

Python sidecar responsibilities:
- deeper image analysis
- nonlinear metrics
- score calculation where Python libraries remain the source of truth
- deterministic per-capture analysis output

## Why This Fits Railway

- Selemene already runs as multi-service on Railway:
  - Rust API
  - TypeScript engines
  - Redis
- Adding `biofield-python` is a normal extension of that topology.
- Railway should not be used as a per-frame realtime inference plane for the public viewer.
- Railway is appropriate for:
  - authenticated capture analysis
  - batched deep metrics
  - history materialization
  - exports
  - private internal service-to-service compute

## How To Preserve The Existing Calculations

1. Do not rewrite the Python metric stack before parity harnesses exist.
2. Extract calculation modules from BV-PIP into a dedicated in-repo Python package.
3. Freeze a versioned analysis contract:
   - input image / capture metadata
   - analysis mode
   - requested algorithm set
   - versioned metric and score output
4. Build a golden corpus of representative images and expected outputs.
5. Treat Python outputs as the reference implementation until parity gates allow partial Rust or WASM replacement.

## Contract Freeze

### API boundary contract

Recommended backend-owned endpoints:

- `POST /api/v1/biofield/sessions`
- `POST /api/v1/biofield/sessions/{session_id}/captures`
- `GET /api/v1/biofield/readings/{reading_id}`
- `GET /api/v1/biofield/history`
- `POST /api/v1/biofield/baselines`
- `GET /api/v1/biofield/baselines`
- `POST /api/v1/biofield/exports`

Important:
- do not expose the Python service publicly
- do not make the viewer depend on a websocket to the Python service

### Auth contract

- browser authenticates with existing Noesis JWT flow
- Noesis API authorizes every session / capture / reading
- Python sidecar is internal-only and should trust only Noesis, not end users
- if extra hardening is desired, add an internal shared-secret header between Rust and Python services

### Persistence contract

Recommended data model additions:

- `biofield_sessions`
- `biofield_readings`
- `biofield_baselines`
- `biofield_exports`
- optional `biofield_capture_artifacts`

Recommended artifact storage:
- use Supabase Storage or another object store for source captures, processed overlays, and export bundles
- do not rely on Railway ephemeral disk as durable storage

### Engine contract

Freeze this rule before implementation diverges:

- keep current `engine-biofield` stable in Phase 1
- do not overload it with live-camera session semantics
- add a capture-derived engine or synthesis adapter only after the reading domain is stable

Recommended future direction:
- introduce `biofield-capture` or `biofield-somatic` as a separate engine surface if capture-derived data needs workflow participation

## Product And Data Semantics

### Session

A live interaction window where the user runs the viewer, but not every session must persist every frame.

### Capture

A discrete frame or uploaded image selected for deep analysis.

### Reading

The persisted analysis result derived from a capture. This is the stable object used for history, export, and later synthesis.

### Baseline

A user-approved comparison reference built from one or more readings, not a transient browser metric buffer.

## What We Should Reuse Directly

### Reuse from BV-PIP frontend

- `frontend/src/services/PIPRenderer.ts`
- realtime metric calculators and score presentation logic
- segmentation hooks and MediaPipe integration patterns
- detailed analysis page structure

### Reuse from BV-PIP backend

- `core/metrics/*`
- `core/scores/*`
- segmentation / zone logic that is actually used in capture analysis

### Do not lift unchanged

- standalone `backend/main.py`
- standalone DB assumptions in `backend/config.py`
- placeholder websocket routes
- standalone CORS / local upload directory assumptions
- direct frontend-to-backend transport design

## Frontend Integration Strategy

### Recommended

Port the viewer into a new authenticated user-facing product surface, not `apps/admin-web`.

Reason:
- `apps/admin-web` is an operational dashboard, not the user experience for somatic work
- the viewer needs camera permissions, session UX, capture history, and likely mobile-first behavior

Practical path:
- create a dedicated user-facing app layer that is designed for both web and mobile
- extract the shared biofield domain model, API client, and reading/session state into reusable packages
- keep platform-specific camera and rendering code separate

### Web + Mobile app shape

Recommended structure:

```text
apps/
  biofield-web/        # first delivery wave
  biofield-mobile/     # second delivery wave

packages/
  biofield-domain/     # types, session model, reading model, score model
  biofield-api-client/ # authenticated Noesis API client
  biofield-shared-ui/  # only truly cross-platform presentational components
```

Rules:
- share domain code aggressively
- share API client code aggressively
- do not force shared camera / rendering code if the runtime realities diverge
- keep WebGL/browser MediaPipe assumptions in web-specific code
- keep mobile camera / native performance decisions in mobile-specific code
- do not block `biofield-web` delivery on `biofield-mobile` implementation

### Next.js migration note

The BV-PIP viewer is Vite + React today. The hardest migration points are not UI; they are:

- browser-only MediaPipe / WebGL lifecycle
- Web Worker asset handling
- SSR avoidance for camera APIs
- stable model / wasm asset hosting

Recommendation:
- pin and self-host the MediaPipe model / wasm assets instead of relying on `latest` CDN URLs

## Python Sidecar Strategy

Create a dedicated in-repo package, for example:

```text
python-services/
  biofield_cv/
    app/
    analyzers/
    metrics/
    scores/
    tests/
```

Keep the service stateless in Phase 1:

- no direct database writes
- no direct auth
- no user ownership logic
- no public history endpoints

The Python service should answer one question well:
- given an authenticated Noesis-owned capture, return a versioned analysis payload

## Workflow And Engine Strategy

### Phase 1 rule

Do not replace current workflow behavior with capture-native semantics.

Why:
- current workflows already assume `biofield` is a calculable engine in birth-blueprint and self-inquiry
- capture-native biofield data is user-session-dependent, not birth-data-deterministic

### Recommended future path

Phase 1:
- viewer and reading domain land first

Phase 2:
- introduce a new capture-aware engine or adapter that consumes `reading_id`

Phase 3:
- build synthesis rules that combine:
  - capture-derived somatic state
  - existing birth-derived biofield or other somatic engines

## Migration Phases

### Phase P1 - Contract Freeze And Extraction Map

Goal:
- freeze product, runtime, persistence, and engine boundaries before any code migration

Exit criteria:
- target architecture approved
- API nouns approved
- internal service topology approved
- parity strategy approved
- import / rewrite boundaries approved

### Phase P2 - Native Viewer Surface

Goal:
- port the viewer experience into Selemene as an authenticated web product surface while laying shared foundations for mobile

Exit criteria:
- `apps/biofield-web` exists and is the first-class delivery surface
- camera, PIP rendering, local segmentation, and local metrics work inside the web app
- capture flow reaches Noesis API
- no standalone frontend dependency remains for primary user flow
- shared domain / API layers are created in a way that mobile can consume later without reworking backend contracts

### Phase P3 - Python Analysis Service

Goal:
- extract BV-PIP Python calculations into a deployable Railway sidecar

Exit criteria:
- private service deploys on Railway
- Noesis API can call it for deep capture analysis
- golden corpus smoke results are recorded

### Phase P4 - Noesis API, Supabase, And Storage

Goal:
- make Noesis the owner of sessions, readings, baselines, history, and exports

Exit criteria:
- authenticated users can persist and retrieve readings
- artifacts are durably stored
- root migrations and mirrored Supabase migrations are aligned

### Phase P5 - Capture-Aware Somatic Integration

Goal:
- connect persisted biofield readings into the engine / workflow world without breaking current contracts

Exit criteria:
- capture-derived output can participate in a controlled somatic synthesis path
- current `biofield` engine semantics remain explicit

### Phase P6 - Hardening, Rollout, And De-standalone Cleanup

Goal:
- prove parity, performance, degraded-mode behavior, and launch readiness

Exit criteria:
- rollout gates pass
- runbooks exist
- standalone deployment assumptions are removed from the active path

## Detailed Task Inventory

| ID | Phase | Wave | Swarm | Track | Task | Depends on | Exit criteria |
|---|---|---|---|---|---|---|---|
| BIO-001 | P1 | W1 | S1 | product | Freeze the product objective as "native Selemene biofield viewer, not standalone BV-PIP" | none | objective statement approved |
| BIO-002 | P1 | W1 | S1 | architecture | Freeze session / capture / reading / baseline as the primary domain nouns | BIO-001 | noun contract written |
| BIO-003 | P1 | W1 | S1 | architecture | Freeze the rule that viewer flows do not enter via `ConsciousnessEngine::calculate()` in Wave 1 | BIO-002 | engine boundary documented |
| BIO-004 | P1 | W1 | S1 | platform | Freeze Railway multi-service topology with a private Python sidecar | BIO-001 | service topology approved |
| BIO-005 | P1 | W1 | S2 | security | Freeze auth boundary: browser talks only to Noesis API, never directly to Python | BIO-004 | trust boundary documented |
| BIO-006 | P1 | W1 | S2 | data | Freeze persistence ownership under Noesis + Supabase | BIO-002 | storage owner documented |
| BIO-007 | P1 | W2 | S1 | frontend | Inventory BV-PIP frontend files to port directly vs rewrite | BIO-001 | import map recorded |
| BIO-008 | P1 | W2 | S1 | python | Inventory BV-PIP Python modules to preserve as canonical analyzers | BIO-001 | python reuse map recorded |
| BIO-009 | P1 | W2 | S1 | frontend | Freeze MediaPipe-only segmentation strategy for both browser and Python service | BIO-007,BIO-008 | segmentation decision published |
| BIO-010 | P1 | W2 | S2 | frontend | Freeze model / wasm asset strategy: pin versions and self-host assets | BIO-009 | asset policy approved |
| BIO-011 | P1 | W2 | S2 | validation | Define golden corpus parity harness for images and expected metrics | BIO-008 | parity plan approved |
| BIO-012 | P1 | W2 | S2 | docs | Publish the architecture / migration contract document | BIO-001,BIO-011 | plan merged |
| BIO-013 | P2 | W1 | S1 | frontend | Create the new authenticated biofield surface in the monorepo | BIO-012 | route shell loads |
| BIO-014 | P2 | W1 | S1 | frontend | Port `PIPRenderer` into the new surface with browser-only lifecycle guards | BIO-013 | live PIP render works |
| BIO-015 | P2 | W1 | S1 | frontend | Port camera acquisition and permission handling | BIO-013 | camera feed works |
| BIO-016 | P2 | W1 | S1 | frontend | Port realtime metric calculation hooks and local timeline buffering | BIO-014,BIO-015 | local metrics update reliably |
| BIO-017 | P2 | W1 | S2 | frontend | Port MediaPipe segmentation into the new surface | BIO-015,BIO-010 | segmentation works locally |
| BIO-018 | P2 | W1 | S2 | frontend | Port score cards, charts, and session dashboard panels | BIO-016 | dashboard parity reached |
| BIO-019 | P2 | W2 | S1 | frontend | Port detailed analysis and capture review UI | BIO-018 | detailed analysis page works |
| BIO-020 | P2 | W2 | S1 | frontend | Implement capture upload contract from viewer to Noesis API | BIO-019 | upload request succeeds |
| BIO-021 | P2 | W2 | S1 | frontend | Add session state model for local-only metrics vs persisted readings | BIO-016,BIO-020 | state split is explicit |
| BIO-022 | P2 | W2 | S2 | frontend | Add authenticated session bootstrap against Noesis JWT flow | BIO-013 | viewer is auth-gated |
| BIO-023 | P2 | W2 | S2 | frontend | Add feature flags for internal rollout and degraded server analysis mode | BIO-022 | feature flags control visibility |
| BIO-024 | P2 | W2 | S2 | frontend | Add mobile and camera-permission UX hardening for Railway-hosted production use | BIO-015,BIO-022 | mobile capture flow reviewed |
| BIO-025 | P3 | W1 | S1 | python | Create `python-services/biofield-cv` package structure in-repo | BIO-012 | package scaffold exists |
| BIO-026 | P3 | W1 | S1 | python | Migrate reusable metric modules from BV-PIP `core/metrics/*` | BIO-025,BIO-008 | metrics package imports cleanly |
| BIO-027 | P3 | W1 | S1 | python | Migrate reusable score calculators from BV-PIP `core/scores/*` | BIO-025,BIO-008 | scores package imports cleanly |
| BIO-028 | P3 | W1 | S1 | python | Migrate only the segmentation / zone code needed for deep capture analysis | BIO-025,BIO-008 | deep-analysis segmentation works |
| BIO-029 | P3 | W1 | S2 | python | Replace standalone FastAPI shell with a stateless internal analysis service | BIO-026,BIO-028 | new app contract defined |
| BIO-030 | P3 | W1 | S2 | python | Remove standalone DB, websocket, and public CORS assumptions from service scope | BIO-029 | service is stateless |
| BIO-031 | P3 | W2 | S1 | python | Define versioned request / response schemas for capture analysis | BIO-029 | schema contract checked in |
| BIO-032 | P3 | W2 | S1 | python | Add health and readiness endpoints with dependency flags | BIO-029 | `/health` and `/ready` work |
| BIO-033 | P3 | W2 | S1 | python | Package CPU-only Railway dependencies and runtime start command | BIO-029 | service boots in container |
| BIO-034 | P3 | W2 | S2 | security | Add optional internal service authentication header verification | BIO-031 | internal auth path works |
| BIO-035 | P3 | W2 | S2 | validation | Add local sample-image smoke runner for the Python service | BIO-031,BIO-011 | smoke runner passes |
| BIO-036 | P3 | W2 | S2 | ops | Add Railway service config and internal networking env contract | BIO-033 | service deploy plan defined |
| BIO-037 | P4 | W1 | S1 | backend | Extend `ApiConfig` with Python biofield service URL and feature toggles | BIO-004,BIO-036 | config loads correctly |
| BIO-038 | P4 | W1 | S1 | backend | Add Noesis-side biofield service client wrapper using internal HTTP | BIO-037 | client call works |
| BIO-039 | P4 | W1 | S1 | data | Define Noesis data models for sessions, readings, baselines, and exports | BIO-002,BIO-006 | model contract approved |
| BIO-040 | P4 | W1 | S1 | data | Add root SQL migrations for new biofield tables | BIO-039 | root migrations compile |
| BIO-041 | P4 | W1 | S2 | data | Mirror the biofield migrations into `supabase/migrations` | BIO-040 | migration authority preserved |
| BIO-042 | P4 | W1 | S2 | backend | Add repositories for session and reading persistence in `noesis-data` | BIO-040 | repository tests pass |
| BIO-043 | P4 | W2 | S1 | backend | Add `POST /api/v1/biofield/sessions` and session lifecycle handlers | BIO-042 | session endpoint works |
| BIO-044 | P4 | W2 | S1 | backend | Add capture analysis endpoint that uploads image, invokes Python, and persists reading | BIO-038,BIO-042,BIO-043 | capture endpoint works |
| BIO-045 | P4 | W2 | S1 | backend | Add history and reading detail endpoints | BIO-042,BIO-044 | history endpoints work |
| BIO-046 | P4 | W2 | S2 | backend | Add baseline CRUD endpoints and baseline comparison model | BIO-042,BIO-045 | baseline endpoints work |
| BIO-047 | P4 | W2 | S2 | storage | Add durable artifact storage for original capture, processed image, and exports | BIO-006,BIO-044 | artifact writes are durable |
| BIO-048 | P4 | W2 | S2 | security | Add ownership checks and auth guards for all biofield resources | BIO-043,BIO-047 | unauthorized access denied |
| BIO-049 | P5 | W1 | S1 | architecture | Preserve current `engine-biofield` semantics during viewer rollout | BIO-003,BIO-012 | engine contract unchanged |
| BIO-050 | P5 | W1 | S1 | architecture | Decide and document whether the capture-derived engine is `biofield-capture` or `biofield-somatic` | BIO-049 | engine naming frozen |
| BIO-051 | P5 | W1 | S1 | core | Define a capture-derived schema in `noesis-core` for persisted biofield readings | BIO-050 | schema merged |
| BIO-052 | P5 | W1 | S2 | backend | Implement a capture-derived engine / adapter that resolves `reading_id` to persisted analysis | BIO-051,BIO-045 | adapter works |
| BIO-053 | P5 | W1 | S2 | orchestrator | Register the new capture-derived engine without disturbing current workflows | BIO-052 | engine appears in registry |
| BIO-054 | P5 | W1 | S2 | docs | Document explicit differences between birth-derived biofield and capture-derived biofield | BIO-049,BIO-053 | docs updated |
| BIO-055 | P5 | W2 | S1 | orchestrator | Define which workflows, if any, may consume capture-derived somatic readings | BIO-053 | workflow policy approved |
| BIO-056 | P5 | W2 | S1 | synthesis | Add optional synthesis logic that combines recent reading data with broader somatic context | BIO-052,BIO-055 | synthesis path works |
| BIO-057 | P5 | W2 | S1 | backend | Add fallbacks when a requested `reading_id` is missing or stale | BIO-052 | graceful error path works |
| BIO-058 | P5 | W2 | S2 | validation | Add workflow and engine tests for capture-aware somatic integration | BIO-053,BIO-056 | tests pass |
| BIO-059 | P5 | W2 | S2 | product | Surface latest reading summary into the user-facing biofield home / history UX | BIO-045,BIO-056 | summary renders |
| BIO-060 | P5 | W2 | S2 | roadmap | Define the later parity path for selective Rust or WASM ports of Python algorithms | BIO-011,BIO-058 | future-port roadmap documented |
| BIO-061 | P6 | W1 | S1 | validation | Build golden image corpus fixtures from standalone BV-PIP examples and curated captures | BIO-011,BIO-025 | corpus checked in |
| BIO-062 | P6 | W1 | S1 | validation | Run parity tests across browser metrics, Python service, and persisted readings | BIO-061,BIO-044 | parity report published |
| BIO-063 | P6 | W1 | S1 | performance | Set and test performance budgets for viewer FPS, capture latency, and reading retrieval | BIO-024,BIO-044 | performance targets met |
| BIO-064 | P6 | W1 | S2 | observability | Add tracing, metrics, and degraded-mode reporting for Python sidecar calls | BIO-038,BIO-033 | telemetry visible |
| BIO-065 | P6 | W1 | S2 | ops | Add Railway readiness checks and service dependency monitoring for `biofield-python` | BIO-036,BIO-064 | health gates defined |
| BIO-066 | P6 | W1 | S2 | docs | Add env, secrets, and operational runbooks for the new service chain | BIO-065 | runbooks merged |
| BIO-067 | P6 | W2 | S1 | migration | Define historical data migration path from any BV-PIP standalone records worth preserving | BIO-045 | migration decision recorded |
| BIO-068 | P6 | W2 | S1 | rollout | Run internal alpha rollout with feature flags and capture success monitoring | BIO-023,BIO-065 | alpha results reviewed |
| BIO-069 | P6 | W2 | S1 | rollout | Run beta rollout with baseline and history enabled | BIO-068,BIO-046 | beta results reviewed |
| BIO-070 | P6 | W2 | S2 | qa | Execute launch checklist covering auth, camera, capture, history, exports, and degraded mode | BIO-069 | launch checklist passes |
| BIO-071 | P6 | W2 | S2 | cleanup | Remove or archive standalone-only assumptions from imported BV-PIP code paths | BIO-070 | no active standalone dependency remains |
| BIO-072 | P6 | W2 | S2 | docs | Publish final integration docs and the follow-on roadmap for deeper somatic synthesis | BIO-071 | handoff complete |

## Verification Gates

### Contract gate

Must pass before implementation branches diverge:
- product nouns frozen
- service topology frozen
- auth boundary frozen
- engine boundary frozen

### Viewer gate

Must pass before Python service integration:
- PIP rendering works inside the new native surface
- segmentation works locally
- capture handoff to Noesis API works
- auth gating works

### Python service gate

Must pass before workflow integration:
- internal service boots on Railway-compatible container
- Noesis can invoke it privately
- golden corpus smoke tests run locally

### Data gate

Must pass before beta rollout:
- sessions, readings, baselines, and artifacts persist correctly
- ownership checks pass
- root and Supabase migrations stay aligned

### Launch gate

Must pass before public rollout:
- parity report published
- degraded mode documented and tested
- Railway health monitors in place
- capture latency within agreed budget

## Open Decisions To Resolve With You

These do not block the architecture, but they do affect Phase P2 sequencing:

1. Should Wave 1 include baseline + history, or should the first shipped slice stop at live viewer + capture + persisted reading detail?
   - Recommendation: ship live viewer + capture + persisted reading first, then baseline / history immediately after.
2. Do you want capture-derived somatic analysis to enter workflows under a new engine ID later, or do you want it to remain a standalone product capability for a while?
   - Recommendation: introduce a new capture-derived engine later; do not overload the current `biofield` engine.

## Recommended First Execution Slice

If we want the safest high-value first implementation wave, it is this:

1. `apps/biofield-web` shell plus shared `packages/biofield-domain` and `packages/biofield-api-client`
2. local camera + PIP + segmentation + realtime metrics
3. authenticated capture upload to Noesis
4. private Python deep-analysis service
5. persisted reading detail page

That gives you a real integrated biofield viewer inside Selemene without betting the project on a premature Rust parity rewrite or a standalone-service compromise, and without entangling the first ship with mobile-specific runtime work.
