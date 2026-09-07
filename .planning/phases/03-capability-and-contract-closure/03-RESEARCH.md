# Phase 3: Capability and contract closure - Research

**Researched:** 2026-09-06
**Domain:** Cross-language capability discovery, API contracts, routing and workflow outcome truth
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

## Phase Boundary

Complete Selemene's Wave 2 contract and routing convergence inside this repository. One canonical identity and schema authority must survive Rust, TypeScript, Python, HTTP, bridge, SDK, CLI/tool server, TUI, admin and workflow boundaries while current runtime availability remains truthful. This phase may repair contract adapters, discovery, routing, validation, error envelopes and synthesis status. Engine calculation semantics belong to Phase 4; stateful production behavior, package release and live deployment belong to Phases 5–7.

## Implementation Decisions

### Catalogue identity and runtime state
- **D-01:** Keep the versioned 19-row engine registry as identity authority: 12 native, one database-conditional and six TypeScript runtime IDs, with 17 public mirror groups. Runtime discovery overlays observed availability; it does not create or remove identities.
- **D-02:** Preserve the canonical `declared`, `available`, `degraded` and `unavailable` capability states. Missing required dependencies fail closed as unavailable; optional or partially reachable dependencies become degraded. Responses carry bounded machine-readable dependency/reason data and never omit an engine silently.

### Cross-surface contract ownership
- **D-03:** Generate or validate every maintained catalogue and contract view from `contracts/v1`; surface-specific labels and presentation are allowed, but independent engine-ID lists, envelope shapes and auth conventions are not.
- **D-04:** Roll out additively through canonical fixtures and compatibility adapters. Supported v1 payloads remain valid; incompatible contract-version or schema skew fails closed with the canonical JSON error envelope and an actionable stable error code rather than permissive coercion.

### Routing, validation and failure behavior
- **D-05:** Freeze API-key transport, native/TypeScript/Python routing, unknown-ID behavior, timeouts and partial dependency failures in executable cross-language tests. Authentication and validation failures must cause zero downstream engine/provider calls and must not expose secrets or internal URLs.
- **D-06:** Keep `/engines/:id/validate` only if the server and engine boundary can provide real validation under the canonical contract. Otherwise remove the unsupported bridge/client assumption and return an explicit capability state; no stub-success validation is acceptable.

### Workflow synthesis truth
- **D-07:** Connect the public workflow path to the existing richer synthesis executor where the current architecture supports it. Any unsupported or failed synthesis remains explicit and carries per-engine partial-failure information; an empty or partial result cannot be labelled successful or LLM-powered.

### the agent's Discretion
The planner and executors may choose the exact generator, build step, module boundaries, fixture layout and compatibility-adapter internals. Prefer small dependency-ordered slices, current repository patterns and fail-closed validators. Do not expand engine semantics, call paid providers, mutate live services, alter production data or edit consumer repositories.

### Deferred Ideas (OUT OF SCOPE)

- Per-engine calculation corrections, fallback provenance and real provider/media semantics belong to Phase 4.
- Migration, durable auth/token state, billing, cache and persisted-media behavior belong to Phase 5.
- Independent package publication and consumer compatibility belong to Phase 6.
- Railway, Cloudflare, DNS, production images, monitoring and rollback mutation belong to Phase 7 and retain critical HITL.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CON-01 | Report native/TS/Python/conditional capabilities from actual runtime state. | Use the canonical 19-row registry as immutable identity, add dependency requirements there, and reduce bounded native registration, database presence, TypeScript self-check and Python health observations into the four existing availability states. [VERIFIED: `.planning/REQUIREMENTS.md`; `contracts/v1/registries/engines.json`; `crates/noesis-core/src/contract.rs`] |
| CON-02 | Prove schemas, errors, auth, routing and catalogue parity across repository boundaries. | Extend the existing fail-closed authority validator and fixtures, then add recording transport tests for the API, bridges including Hermes, SDKs, CLI/tool server, TUI, admin, witness, verification source adapter and workflow boundaries. [VERIFIED: `.planning/REQUIREMENTS.md`; `scripts/validate_contracts.py`; `docs/plans/selemene-engine/CAPABILITY-LEDGER.md`; maintained source inventory] |
</phase_requirements>

## Summary

Phase 3 starts from a strong identity authority rather than a missing catalogue: `contracts/v1/registries/engines.json` already fixes 19 runtime IDs, their 12/1/6 runtime-class split and the separate 17-public-mirror count, while the core contract already defines the four required availability values. The remaining defect is projection: each runtime and consumer constructs a different, incomplete view of that authority. [VERIFIED: `contracts/v1/registries/engines.json`; `crates/noesis-core/src/contract.rs`; `docs/plans/selemene-engine/CAPABILITY-LEDGER.md`]

The central implementation should therefore be a deterministic capability resolver owned by the Rust orchestration/API boundary. It should start with all 19 declared rows, overlay bounded observations from native registration, the optional database engine, the six TypeScript engines and the two Python service paths, and emit one canonical envelope without deleting unavailable rows. Python services should appear as dependency observations for the canonical `biofield` and `face-reading` capabilities, not as new engine identities, because D-01 locks the registry at 19 rows. [VERIFIED: `.planning/phases/03-capability-and-contract-closure/03-CONTEXT.md` D-01/D-02; `crates/noesis-api/src/lib.rs`; `crates/noesis-bridge/src/python_client.rs`; `python-services/*_service/health.py`]

Contract closure must also make failure a first-class result. The TypeScript validation route is absent, general SDK list shapes disagree with the API, CLI/tool code sends API keys as bearer tokens, several client catalogues are stale, and both workflow implementations discard failed engines. The plan should close these in dependency order: authority and fixtures, resolver and runtime adapters, API/OpenAPI/error/auth, bridge validation, consumers, then workflow outcome and synthesis truth. [VERIFIED: `crates/noesis-bridge/src/lib.rs`; `ts-engines/src/server/app.ts`; `crates/noesis-sdk/src/client.rs`; `packages/noesis-sdk-ts/src/index.ts`; `bridges/cli/src/core/http.ts`; `bridges/universal-tool-server/main.py`; `crates/noesis-orchestrator/src/lib.rs`; `crates/noesis-orchestrator/src/workflow/executor.rs`]

**Primary recommendation:** Build one registry-derived capability and workflow-outcome contract, emit it through the protected Rust API, and make every other surface either consume that response or prove a generated projection against the same fixtures.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Engine identity and runtime requirements | API / Backend contract authority | Database / Storage only as an observed dependency | Identity belongs to `contracts/v1` and must not change with process configuration; database state only changes `biofield-capture` availability. [VERIFIED: `contracts/v1/registries/engines.json`; `.planning/phases/03-capability-and-contract-closure/03-CONTEXT.md` D-01] |
| Native and composed registration observation | API / Backend orchestration | — | `EngineRegistry` is the executable membership boundary for the 12 native rows, including the composed `financial-biosensor` identity. [VERIFIED: `crates/noesis-orchestrator/src/lib.rs`] |
| TypeScript capability observation | Internal TypeScript service | API / Backend resolver | The TypeScript registry owns its six loaded modules and self-checks; the Rust API owns public projection and authentication. [VERIFIED: `ts-engines/src/server/registry.ts`; `ts-engines/src/server/app.ts`; `crates/noesis-api/src/middleware.rs`] |
| Python dependency observation | Internal Python sidecars | API / Backend resolver | The sidecars can report local dependency facts; they do not own new engine IDs in the locked registry. [VERIFIED: `python-services/biofield_cv_service/health.py`; `python-services/mediapipe_service/health.py`; `contracts/v1/registries/engines.json`] |
| Canonical capability endpoint and error envelope | API / Backend | OpenAPI projection | The trusted Rust boundary already authenticates protected routes and owns `ErrorMapper` and the OpenAPI document. [VERIFIED: `crates/noesis-api/src/lib.rs`; `crates/noesis-api/src/middleware.rs`; `crates/noesis-api/src/error_mapper.rs`] |
| SDK, CLI, TUI and admin catalogue presentation | Browser / Client or native client | API / Backend | Clients may label and filter capabilities but must not redefine identity, envelope or auth transport. [VERIFIED: `.planning/phases/03-capability-and-contract-closure/03-CONTEXT.md` D-03] |
| Workflow routing and partial-failure accounting | API / Backend orchestration | Client presentation | Only the orchestrator sees every requested engine result and can distinguish complete, partial and failed execution before the API serializes it. [VERIFIED: `crates/noesis-orchestrator/src/lib.rs`; `crates/noesis-orchestrator/src/workflow/executor.rs`] |
| Synthesis support and status | API / Backend workflow layer | Witness client presentation | Existing workflow synthesizers run over engine outputs; consumers should only display the resulting explicit support/status receipt. [VERIFIED: `crates/noesis-orchestrator/src/workflow/synthesis/mod.rs`; `crates/noesis-api/src/handlers/witness.rs`] |
| Deployment identity and production operations | CDN / Static and deployment control plane | API health | These claims remain Phase 7 and must not be inferred from capability availability. [VERIFIED: `.planning/ROADMAP.md` Phase 7; `docs/plans/selemene-engine/CAPABILITY-LEDGER.md`] |

## Project Constraints

No repository-local `AGENTS.md` exists in this worktree; the effective constraints come from the task, `ISA.md` and the locked phase context. [VERIFIED: filesystem search in this worktree; `ISA.md`; `.planning/phases/03-capability-and-contract-closure/03-CONTEXT.md`]

- Work only on `codex/selemene-contract-convergence` in the specified isolated Superset worktree and preserve all other worktrees and user-owned state. [VERIFIED: task scope; `ISA.md` ISC-235]
- Keep 19 supported runtime identities distinct from 17 public mirrors in every count and label. [VERIFIED: `ISA.md` Constraints; `contracts/v1/registries/engines.json`]
- Do not mutate providers, network services, schema/data, production, secrets, remotes, merges or consumer repositories during this phase. [VERIFIED: task scope; `.planning/phases/03-capability-and-contract-closure/03-CONTEXT.md`]
- A required test that does not execute is a failure; health, local tests, configured routes and deployment are separate evidence claims. [VERIFIED: `ISA.md` Principles/Constraints; `docs/plans/selemene-engine/CAPABILITY-LEDGER.md`]
- Keep engine calculations and provider/media semantics out of Phase 3, durability out of Phase 5, packaging out of Phase 6 and deployment out of Phase 7. [VERIFIED: `.planning/ROADMAP.md`; `.planning/phases/03-capability-and-contract-closure/03-CONTEXT.md`]
- Do not commit this research artifact; the parent orchestration task owns commits. [VERIFIED: task scope]

## Standard Stack

Use the repository's locked stack and add no runtime package for this phase. The needed validation, HTTP, serialization, test-double and schema capabilities already exist in the workspace. [VERIFIED: `Cargo.lock`; `pnpm-lock.yaml`; `python-services/uv.lock`; package manifests]

### Core

| Library / Tool | Locked or declared version | Purpose | Why Standard Here |
|----------------|----------------------------|---------|-------------------|
| Rust workspace | 3.3.1 packages; rustc/cargo 1.97.0 available | Core contracts, resolver, orchestration, API and SDK | This is the existing public trusted boundary and all relevant crates already share workspace types. [VERIFIED: `Cargo.toml`; environment probe] |
| Serde / serde_json | 1.0.228 / 1.0.149 | Strict Rust wire types and fixture round trips | `EngineCapability` already uses Serde with `deny_unknown_fields`. [VERIFIED: `Cargo.lock`; `crates/noesis-core/src/contract.rs`] |
| Axum | 0.7.9 | Protected capability/workflow routes and canonical responses | The API router, middleware and current contract tests are Axum based. [VERIFIED: `Cargo.lock`; `crates/noesis-api/src/lib.rs`] |
| Utoipa | 5.5.0 | OpenAPI projection | Existing API schemas and security schemes are generated through Utoipa and tested in `openapi_schema_tests`. [VERIFIED: `Cargo.lock`; `crates/noesis-api/Cargo.toml`; `crates/noesis-api/tests/openapi_schema_tests.rs`] |
| Reqwest | 0.12.28 | Bounded Rust-to-TypeScript/Python transport | Both bridge clients already use it and have timeout/error normalization seams. [VERIFIED: `Cargo.lock`; `crates/noesis-bridge/src/lib.rs`; `crates/noesis-bridge/src/python_client.rs`] |
| TypeScript | exact 5.9.3 at the root | Generated/static projection checking | The contract validator deliberately loads the compiler API from this exact root dependency. [VERIFIED: `package.json`; `pnpm-lock.yaml`; `scripts/resolve_typescript_anchors.cjs`] |
| Bun / Elysia | Bun 1.3.13 available; Elysia declared ^1.0.0 | Six-engine internal service and route tests | The current service, self-check and registry tests use this stack. [VERIFIED: environment probe; `ts-engines/package.json`] |
| Python / FastAPI / Pydantic | Python >=3.11; CI 3.11 and 3.12 | Sidecar observations and tool-server adapters | The locked Python workspace already models health with Pydantic and tests ASGI routes with FastAPI TestClient. [VERIFIED: `python-services/pyproject.toml`; `.github/workflows/test.yml`] |
| JSON Schema Draft 2020-12 | Contract manifest authority | Payload and fixture validation | All six current schemas declare Draft 2020-12 and the repository validator resolves them offline. [VERIFIED: `contracts/v1/manifest.json`; `scripts/validate_contracts.py`] |

### Supporting

| Library / Tool | Version | Purpose | When to Use |
|----------------|---------|---------|-------------|
| Tokio | 1.49.0 | Parallel bounded observations and workflow execution | Reuse current async runtime; keep probes bounded and non-generative. [VERIFIED: `Cargo.lock`; `crates/noesis-orchestrator/src/workflow/executor.rs`] |
| httpx | declared >=0.25 in Python dev extras | Mockable tool-server transport | Use an injected/MockTransport client for header, timeout, path and sanitized-error tests. [VERIFIED: `python-services/pyproject.toml`; `bridges/universal-tool-server/main.py`] |
| Vitest | ^3.2.6 in the maintained general SDK, witness and verification packages | Client adapter contract tests | Reuse after frozen pnpm hydration. [VERIFIED: `packages/noesis-sdk-ts/package.json`; `packages/witness-pipeline/package.json`; `packages/verification/package.json`] |
| Rust in-process test routers | existing test harness | Auth, content-type and zero-downstream assertions | Prefer the current Axum harness and recording registries over a live service. [VERIFIED: `crates/noesis-api/tests/common/test_harness.rs`; `crates/noesis-api/tests/routing_enforcement_tests.rs`] |
| Existing contract validator | repository script | Cross-surface drift and negative fixture rejection | Extend it for generated catalogues, envelopes, dependency metadata and method/header assertions. [VERIFIED: `scripts/validate_contracts.py`; `tests/scripts/test_validate_contracts.py`] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Registry-derived checked-in projections plus validation | Runtime introspection from every client | Runtime introspection cannot provide compile-time ID unions and makes offline clients dependent on service reachability. [VERIFIED: current static client types and dynamic API routes] |
| One Rust-owned public capability resolver | Expose TypeScript and Python capability routes directly | Direct exposure would fragment authentication, error and envelope authority; the current sidecars do not implement the Rust public auth boundary. [VERIFIED: `crates/noesis-api/src/middleware.rs`; `ts-engines/src/server/app.ts`; `python-services/*_service/main.py`] |
| Additive new canonical envelope plus legacy adapter | Change every existing list response in one step | Existing SDKs already disagree about list shapes, and strict decoders make a flag-day response change risky. [VERIFIED: `crates/noesis-sdk/src/client.rs`; `packages/noesis-sdk-ts/src/index.ts`; `crates/noesis-core/src/contract.rs`] |
| Recording in-process transport tests | Start real local/provider services | The phase must prove routing and zero-call behavior without provider, production or network-service effects. [VERIFIED: task scope; `.planning/phases/03-capability-and-contract-closure/03-CONTEXT.md` D-05] |

**Installation:** no new dependency is recommended. Plan 01 hydrates only existing locks with `pnpm install --offline --ignore-scripts --frozen-lockfile`, `cd ts-engines && bun install --frozen-lockfile`, `cd bridges/cli && bun install --frozen-lockfile`, and `cd python-services && uv sync --locked --extra dev --python 3.12`. After the shared reader source exists, Plan 03b may update `pnpm-lock.yaml` only for existing `workspace:*` links to `@selemene/engine-sdk`, then must rerun the frozen install. No external package, Bun lock, Python lock, or ambient runtime is allowed. [VERIFIED: `pnpm-lock.yaml`; `ts-engines/bun.lock`; `bridges/cli/bun.lock`; `python-services/uv.lock`; `.planning/phases/02-reproducible-gates-dependency-repair/02-REVIEW-RECHECK-5.md`]

**Shared TypeScript package boundary:** `packages/noesis-engine-sdk/src/contract-v1.ts` is the sole runtime decoder/guard authority, exported through `packages/noesis-engine-sdk/src/index.ts`. Every actual importing pnpm workspace package declares `@selemene/engine-sdk: workspace:*`; Plan 03b owns those declarations and the required lock metadata, runs `pnpm --filter @selemene/engine-sdk build`, and proves generated `dist/index.js` and `dist/index.d.ts` before SDK, admin, Witness or verification consumers execute. The independent `ts-engines` Bun project remains outside the pnpm workspace and reuses the Plan 03 source authority by explicit relative import. Consumer tests reuse the public export and never add independent decoders or validators. [VERIFIED: `packages/noesis-engine-sdk/package.json`; current workspace package manifests; Plan 03b dependency/build receipt]

## Package Legitimacy Audit

No new external package is selected or installed by the implementation plan, so the Package Legitimacy Gate does not apply. Existing dependencies remain governed by the three checked-in lockfiles and the Phase 2 dependency gates. [VERIFIED: recommended stack above; `.planning/phases/02-reproducible-gates-dependency-repair/02-VERIFICATION.md`]

## Current Contract Gap Map

| Surface | Current truth | Required Phase 3 closure |
|---------|---------------|--------------------------|
| Canonical authority | The manifest has six schemas and five fixtures; capability is an item schema with string dependency IDs, while no canonical list envelope or structured dependency-reason object exists. [VERIFIED: `contracts/v1/manifest.json`; `contracts/v1/schemas/engine-capability.schema.json`] | Add a canonical capability-list envelope fixture/schema and additive dependency observations plus operation support; retain old valid item payloads. |
| Registry | The registry fixes 19 rows and class counts but rows do not declare runtime dependency requirements. [VERIFIED: `contracts/v1/registries/engines.json`] | Add bounded requirement metadata for TypeScript transport, database conditionality and the Python paths; do not add identities. |
| Rust orchestration | `SUPPORTED_ENGINE_IDS` has all 19; native registration has 12, bridge registration six, and database registration is conditional. [VERIFIED: `crates/noesis-orchestrator/src/lib.rs`; `crates/noesis-api/src/lib.rs`] | Expose these as observations to one resolver rather than returning only registered IDs. |
| Public/API capability | The only Rust capability handler is the admin route, returns a raw array, traverses only the six bridge engines and reduces health to available/unavailable. [VERIFIED: `crates/noesis-api/src/handlers/admin.rs::engine_capabilities`; `crates/noesis-api/src/lib.rs`] | Add a protected non-admin canonical envelope over all 19; make admin reuse it and add operational decoration separately. |
| TypeScript service | It emits only its six rows; engines without `selfCheck` receive `healthy: true` and “default health check passed,” which then maps to available. [VERIFIED: `ts-engines/src/server/app.ts`; `ts-engines/src/server/registry.ts`] | Require explicit, non-generative checks; no check yields declared/degraded with a reason, never available. |
| Python services | Health returns a scalar capability status and raw boolean module checks; Biofield degrades without optional MediaPipe and fails unavailable without required OpenCV/NumPy, while MediaPipe is available only when its module imports. [VERIFIED: `python-services/biofield_cv_service/health.py`; `python-services/mediapipe_service/health.py`; `python-services/tests/test_capability_health.py`] | Adapt these facts into dependencies on `biofield` and `face-reading` with bounded IDs/reason codes; do not leak service URLs or create engine rows. |
| Runtime kind projection | Admin classifies `biofield-capture` as `python-sidecar` even though the authority class is `database-conditional`. [VERIFIED: `crates/noesis-api/src/handlers/admin.rs`; `contracts/v1/registries/engines.json`] | Remove the local classifier and project the canonical runtime class. |
| Bridge validation | Rust posts to `/engines/:id/validate`, but the TypeScript server declares neither that route nor a validation interface method. [VERIFIED: `crates/noesis-bridge/src/lib.rs`; `ts-engines/src/server/app.ts`; `ts-engines/src/types/engine.ts`] | Mark TypeScript validation unsupported and stop the network call unless a real per-engine validator is implemented and contract-tested. |
| Rust SDK | The constant contains 17 IDs, omitting `raaga` and `biofield-capture`; `list_engines` expects `Vec<EngineInfo>`, does not authenticate the GET, and the API returns an object containing string IDs. [VERIFIED: `crates/noesis-sdk/src/client.rs`; `crates/noesis-api/src/lib.rs`] | Generate/validate the correct projections, parse the canonical envelope, and route through the authenticated request builder. |
| General TypeScript SDK | Its 17 IDs omit `financial-biosensor` and `biofield-capture`; `listEngines` expects an array although the API returns an object; `updateMe` uses PUT where the API uses PATCH. [VERIFIED: `packages/noesis-sdk-ts/src/index.ts`; `crates/noesis-api/src/lib.rs`; `crates/noesis-api/src/handlers/users.rs`] | Validate IDs, envelope and methods from canonical fixtures/OpenAPI and add mock-server parity tests. |
| Focus engine SDK | It intentionally serves four media focus engines, defaults calculations to the TypeScript service when `apiUrl` is absent, and calls the Python analysis service directly. [VERIFIED: `packages/noesis-engine-sdk/src/index.ts`; `packages/noesis-engine-sdk/src/client.ts`] | Keep the four-engine product scope explicit, require the protected Rust API for public calculation, and label any direct sidecar transport internal/test-only. |
| CLI bridge | The HTTP helper sends an API key as `Authorization: Bearer`, the generated LangChain client embeds the same mistake, and health only tests reachability/status. [VERIFIED: `bridges/cli/src/core/http.ts`; `bridges/cli/src/generators/langchain.ts`] | Use `X-API-Key` for API keys in both runtime and generated output; snapshot generated clients and model reachable, authenticated and capability-available as separate states. |
| Universal tool server | It repeats the bearer-key mistake, interpolates arbitrary path arguments, returns its own error model with raw upstream bodies, advertises direct TypeScript tools and assumes engine validation exists. [VERIFIED: `bridges/universal-tool-server/main.py`] | Generate/validate its tool table from canonical public operations, allowlist/encode IDs, send the correct header, sanitize canonical errors and remove unsupported/direct-internal operations. |
| Hermes bridge | Its tool definitions, dispatch allowlists, system prompt and README hard-code 16 engines and describe Full Spectrum as all engines; its Noesis transport already uses `X-API-Key`. [VERIFIED: `bridges/hermes/tools.py`; `bridges/hermes/agent.py`; `bridges/hermes/README.md`] | Validate generated tool enums and human-facing counts against the canonical public projection, advertise only the supported workflow/capability subset, and retain the correct auth header. |
| TUI | The picker uses a hard-coded 16-entry list and its connected flag can be set by client construction or config presence. [VERIFIED: `crates/noesis-tui/src/screens/engine_picker.rs`; `crates/noesis-tui/src/app.rs`] | Render canonical capabilities dynamically and keep configured, reachable, authenticated and available indicators distinct. |
| Admin | The system list uses registered runtime rows and analytics-derived healthy/degraded/unknown status, separate from the six-row capability route. [VERIFIED: `crates/noesis-api/src/handlers/admin.rs`; `apps/admin-web`] | Join canonical capability state with operational analytics without overwriting either evidence axis. |
| Witness package | It maintains a 16-ID set and routing maps, omitting `raaga`, `financial-biosensor` and `biofield-capture`. [VERIFIED: `packages/witness-pipeline/src/selemene/types.ts`] | Derive an explicit witness-eligible subset from canonical IDs and use capability state to skip with a recorded reason rather than silently omitting. |
| Verification package | Its source adapter delegates to the witness package and has useful isolated error tests, while the package entrypoint is empty and its golden/scoring suite does not prove API, SDK, admin or bridge contract parity. [VERIFIED: `packages/verification/src/sources/selemene.ts`; `packages/verification/src/sources/selemene.test.ts`; `packages/verification/src/index.ts`; `packages/verification/src/runner.test.ts`; `docs/plans/selemene-engine/CAPABILITY-LEDGER.md`] | Reuse the source-adapter tests for Phase 3 error/auth projection, but keep golden semantic assertions in Phase 4 and consumable package/export work in Phase 6; the Phase 3 gate must compose the real boundary targets directly. |
| Public workflow | `WorkflowOrchestrator` logs and drops failed engines and always returns `synthesis: None`. [VERIFIED: `crates/noesis-orchestrator/src/lib.rs`; `crates/noesis-core/src/types.rs`] | Return requested IDs, successes, bounded failures, completion status and synthesis status through a compatibility adapter. |
| Rich workflow executor | It also drops failed engines; it specializes Birth Blueprint and Daily Practice but routes all other synthesis types to a “pending” generic result, despite Decision Support, Self Inquiry and Creative Expression already implementing the common `Synthesizer` trait. [VERIFIED: `crates/noesis-orchestrator/src/workflow/executor.rs`; `crates/noesis-orchestrator/src/workflow/synthesis/mod.rs`] | Preserve failures, dispatch the five compatible synthesizers, and mark Full Spectrum/None unsupported unless a real adapter is proven. |
| Witness Dyad | The handler runs a hard-coded seven-engine subset, drops each engine error, unconditionally puts `biofield` in `engines_available`, and can label empty strings from a failed LLM call as `llm_powered: true`. [VERIFIED: `crates/noesis-api/src/handlers/witness.rs`; `crates/noesis-witness/src/lib.rs`] | Resolve the eligible subset through capability state, distinguish supplied live scores from an executable engine, preserve per-engine failures, and derive the powered label from a non-empty successful generation receipt. |
| Error handling | `ErrorMapper` sanitizes bridge/internal response text, but auth details echo the reason, service-unavailable text is public, Sentry receives raw `err.to_string()`, and the Biofield bridge mapper can return internal URLs/raw upstream bodies. [VERIFIED: `crates/noesis-api/src/error_mapper.rs`; `crates/noesis-api/src/handlers/biofield.rs`; `crates/noesis-bridge/src/error.rs`] | Use stable public reason codes, redact before both response and telemetry, and keep raw upstream data in no user-visible field. |

## Architecture Patterns

### System Architecture Diagram

~~~mermaid
flowchart TD
    A[contracts/v1 registry + schemas + fixtures] --> B[Fail-closed validator / projection generator]
    B --> C[Rust contract types]
    B --> D[TypeScript and client projections]
    B --> E[OpenAPI schemas]

    N[Native/composed registered set] --> R[Capability resolver]
    DB{Database configured?} --> R
    TS[Six TS explicit self-check observations] --> R
    PY[Python dependency health observations] --> R
    A --> R

    Q{Authenticated public request?} -->|no| X[Canonical JSON auth error; zero engine calls]
    Q -->|yes| R
    R --> L[Canonical 19-row capability envelope]
    L --> SDK[SDK / CLI / tool server / TUI]
    L --> ADM[Admin capability + separate operational analytics]
    L --> WSEL[Witness eligibility filter]

    WR[Authenticated workflow request] --> V{Contract and input valid?}
    V -->|no| X
    V -->|yes| EX[WorkflowExecutor routed engine calls]
    EX --> OUT{Successful outputs?}
    OUT -->|none| F[failed + per-engine failures]
    OUT -->|some| P{All requested succeeded?}
    P -->|no| PR[partial + per-engine failures]
    P -->|yes| OK[complete]
    F --> SYN[synthesis_status unavailable/failed]
    PR --> SYN2{Supported synthesizer?}
    OK --> SYN2
    SYN2 -->|yes and succeeds| SR[synthesis available]
    SYN2 -->|no| SU[synthesis unsupported]
    SYN2 -->|error| SF[synthesis failed]
~~~

The diagram intentionally ends before semantic result verification and deployment: those are Phase 4 and Phase 7 evidence respectively. [VERIFIED: `.planning/ROADMAP.md`]

### Recommended Project Structure

~~~text
contracts/v1/
├── registries/engines.json                 # identity + dependency requirements
├── schemas/engine-capability.schema.json   # additive item contract
├── schemas/engine-capability-list.schema.json
└── fixtures/                               # full list, failure and workflow outcome fixtures
scripts/
├── validate_contracts.py                   # all source/projection drift gates
└── generate_contract_projections.*         # optional deterministic flat projections
crates/noesis-core/src/
└── contract.rs                             # shared wire types
crates/noesis-orchestrator/src/
├── capability.rs                           # pure reducer, no HTTP
└── workflow/                               # executor + explicit outcome/synthesis status
crates/noesis-api/src/
├── capability.rs                           # observation adapters and public route
└── error_mapper.rs                         # canonical sanitized boundary
ts-engines/src/server/
└── capability.ts                           # explicit local self-check observations
python-services/shared/
└── models.py                               # dependency observation response
~~~

The exact new filenames are planner discretion; the responsibility split is the important constraint. [VERIFIED: `.planning/phases/03-capability-and-contract-closure/03-CONTEXT.md` agent discretion; existing module boundaries]

### Pattern 1: Identity Plus Observation Reduction

**What:** Load every canonical row first, collect bounded observations independently, then apply one pure precedence reducer. Required-unavailable wins; otherwise optional-unavailable or any degraded observation yields degraded; all required observations available plus an executable registration yields available; absence of sufficient observation remains declared. [VERIFIED: `.planning/phases/03-capability-and-contract-closure/03-CONTEXT.md` D-01/D-02]

**When to use:** Every capability response, admin projection and client catalogue.

**Planning rule:** Keep `declared` meaningful. Under the locked phase model it means authority exists but current executability was not proved; it must never be promoted to `available` merely because a module lacks a check. [VERIFIED: `.planning/phases/03-capability-and-contract-closure/03-CONTEXT.md` D-01/D-02]

### Pattern 2: Python Paths as Subcapabilities

**What:** Add registry-owned dependency requirements such as `python:biofield-cv` and `python:mediapipe-face-mesh` to the owning engine rows, then adapt sidecar health booleans into bounded observations. [VERIFIED: `python-services/biofield_cv_service/health.py`; `python-services/mediapipe_service/health.py`]

**When to use:** Reporting Python-backed enhancement/fallback availability without violating the fixed 19-ID authority.

**Planning rule:** A Python observation changes the availability/reason data of `biofield` or `face-reading`. It never creates a twentieth engine and never changes calculation provenance; provenance semantics remain Phase 4. [VERIFIED: `.planning/phases/03-capability-and-contract-closure/03-CONTEXT.md` D-01; `.planning/ROADMAP.md` Phase 4]

### Pattern 3: Additive Envelope With Compatibility Adapter

**What:** Define a canonical envelope containing `contract_version`, `capabilities` and `count`; extend item fields optionally; update all maintained strict readers before the canonical endpoint emits new fields. Preserve current legacy shapes only behind named adapters and fixture tests. The existing Rust type rejects unknown fields, and JSON Schema `additionalProperties: false` rejects undeclared fields, so producer-first rollout is unsafe. [VERIFIED: `crates/noesis-core/src/contract.rs`; `contracts/v1/schemas/engine-capability.schema.json`] [CITED: https://json-schema.org/draft/2020-12/json-schema-core]

**When to use:** Capability, engine list, workflow outcome and error evolution under D-04.

**Recommended additive capability fields:**

~~~json
{
  "reason_code": "OPTIONAL_DEPENDENCY_UNAVAILABLE",
  "dependency_observations": [
    {
      "dependency_id": "python:mediapipe-face-mesh",
      "dependency_kind": "python",
      "requirement": "optional",
      "availability": "unavailable",
      "reason_code": "MODULE_UNAVAILABLE"
    }
  ],
  "operations": {
    "calculate": "supported",
    "validate": "unsupported"
  }
}
~~~

Keep reason codes allowlisted and bounded; do not include raw exception text, URLs, hostnames or secret-bearing configuration.

### Pattern 4: Anti-Corruption Adapters at Every Transport

**What:** Rust bridge, Python health adapter, SDK decoder, CLI and tool server each translate their local representation into the canonical types once. No downstream client should reinterpret raw sidecar error text or guess auth from a token-shaped string. [VERIFIED: current divergent adapters in `crates/noesis-bridge`, `bridges/cli` and `bridges/universal-tool-server`]

**When to use:** Any boundary that changes language, process or HTTP server.

**Planning rule:** Public engine calls go through the protected Rust API. Direct TypeScript/Python routes remain internal observations/transports and must not appear as equivalent public operations.

### Pattern 5: Explicit Partial Workflow Outcome

**What:** Return `requested_engine_ids`, `engine_outputs`, `engine_failures`, `execution_status` and `synthesis_status`. A zero-success result is failed, a mixed result is partial, and only all required successes are complete. Synthesis may be available, unsupported or failed independently. [VERIFIED: `.planning/phases/03-capability-and-contract-closure/03-CONTEXT.md` D-07; current omission in `crates/noesis-orchestrator/src/lib.rs`]

**When to use:** Public workflow execution and Witness Dyad aggregation.

**Planning rule:** Import and dispatch the existing `DecisionSupportSynthesis`, `SelfInquirySynthesis` and `CreativeExpressionSynthesis` implementations because they already satisfy the common trait. Keep `FullSpectrum` unsupported until its different aggregate input is adapted with real tests; delete the generic “pending” success path. [VERIFIED: `crates/noesis-orchestrator/src/workflow/synthesis/mod.rs`; `crates/noesis-orchestrator/src/workflow/executor.rs`]

### Anti-Patterns to Avoid

- **Filtering before projection:** listing only registered or healthy engines destroys canonical identity and hides unavailable rows. [VERIFIED: D-01/D-02]
- **Boolean health as capability truth:** reachability or module registration alone does not prove required dependencies, auth, routing or semantic correctness. [VERIFIED: `ts-engines/src/server/app.ts`; `ISA.md` Principles]
- **Producer-first schema changes:** strict Rust and JSON Schema consumers can reject new undeclared fields. [VERIFIED: `crates/noesis-core/src/contract.rs`; `contracts/v1/schemas/engine-capability.schema.json`]
- **SDK mock servers that copy client assumptions:** current SDK tests can pass while reproducing the wrong server envelope. [VERIFIED: `docs/plans/selemene-engine/CAPABILITY-LEDGER.md`]
- **Direct public sidecar routing:** it bypasses the Rust auth/error authority and multiplies contracts. [VERIFIED: `packages/noesis-engine-sdk/src/client.ts`; `python-services/*_service/main.py`]
- **String interpolation for route parameters:** arbitrary IDs can alter path interpretation; allowlist canonical IDs and URL-encode after validation. [CITED: https://raw.githubusercontent.com/OWASP/ASVS/master/5.0/docs_en/OWASP_Application_Security_Verification_Standard_5.0.0_en.flat.json V1.2.2]
- **Successful empty synthesis:** an empty or placeholder synthesis is unsupported/failed, never successful or LLM-powered. [VERIFIED: D-07]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Schema parsing and reference resolution | Regex JSON contract checking | Existing `scripts/validate_contracts.py` and Draft 2020-12 validation | The current validator already fails on malformed, missing, duplicate, traversing and unresolved authority. [VERIFIED: `tests/scripts/test_validate_contracts.py`] |
| TypeScript source/symbol parsing | Regex over TS catalogues | Existing compiler-API helper | Phase 2 added exact TypeScript 5.9.3 because parser-backed anchors avoid comment/string false positives. [VERIFIED: `scripts/resolve_typescript_anchors.cjs`; `.planning/phases/02-reproducible-gates-dependency-repair/02-REVIEW-RECHECK-5.md`] |
| Auth scheme inference | “Bearer if a secret exists” | OpenAPI security-scheme metadata and explicit config variants | OpenAPI models API-key header and HTTP bearer as different scheme types. [CITED: https://spec.openapis.org/oas/v3.1.2.html#security-scheme-object] |
| Retry/timeout/circuit state | New ad hoc loops in every client | Existing Reqwest bridge timeout/circuit code and client request wrappers | Multiple custom loops would produce inconsistent timeout and error behavior. [VERIFIED: `crates/noesis-bridge/src/lib.rs`; `packages/noesis-sdk-ts/src/index.ts`] |
| Runtime engine identity | Directory scans or service-returned IDs | `contracts/v1/registries/engines.json` | Service state is observation, not identity authority. [VERIFIED: D-01] |
| Error redaction | Per-handler string replacements | Canonical `ErrorMapper` plus structured safe error variants | Raw upstream text currently leaks through handlers that bypass the mapper. [VERIFIED: `crates/noesis-api/src/error_mapper.rs`; `crates/noesis-api/src/handlers/biofield.rs`] |
| Workflow success inference | `outputs.len() > 0` or optional synthesis | An explicit outcome reducer with requested/succeeded/failed sets | Current silent omission makes complete and partial execution indistinguishable. [VERIFIED: `crates/noesis-orchestrator/src/lib.rs`] |

**Key insight:** The hard part is preserving meaning across boundaries, not serializing another list. Reusing one authority and one reducer prevents each client from inventing a plausible but incompatible truth.

## Common Pitfalls

### Pitfall 1: Confusing Identity With Availability

**What goes wrong:** An unavailable engine disappears, counts oscillate between six, 16, 17 and 19, and clients cannot explain why a canonical engine is absent. [VERIFIED: `docs/plans/selemene-engine/CAPABILITY-LEDGER.md` contradiction register; current client constants]

**Why it happens:** Existing list APIs enumerate registered process objects rather than joining them onto the authority. [VERIFIED: `crates/noesis-api/src/lib.rs`; `crates/noesis-api/src/handlers/admin.rs`; `ts-engines/src/server/app.ts`]

**How to avoid:** Assert exactly 19 unique IDs in every canonical capability response under every dependency matrix; vary only availability and reasons.

**Warning signs:** `count` changes when the database or a sidecar is disabled.

### Pitfall 2: Treating “No Self-Check” as Healthy

**What goes wrong:** TypeScript engines appear available without observing any dependency or executable invariant. [VERIFIED: `ts-engines/src/server/app.ts`]

**Why it happens:** The current fallback returns `healthy: true`. [VERIFIED: `ts-engines/src/server/app.ts`]

**How to avoid:** Require explicit non-generative checks and map absent evidence to declared, not available.

**Warning signs:** A test removes `selfCheck` and capability remains available.

### Pitfall 3: Breaking Strict v1 Consumers With an “Additive” Field

**What goes wrong:** The producer emits dependency observations while a strict reader rejects them as unknown. [VERIFIED: `deny_unknown_fields` in `crates/noesis-core/src/contract.rs`; `additionalProperties: false` in the schema]

**Why it happens:** Additive JSON is only compatible when readers permit or know the new field. [VERIFIED: strict current Rust/schema readers] [CITED: https://json-schema.org/draft/2020-12/json-schema-core]

**How to avoid:** Land schema/types/fixtures/readers first, then emit through the new canonical endpoint; keep legacy adapters tested.

**Warning signs:** Fixture passes Python schema validation but Rust deserialization fails.

### Pitfall 4: Reusing Bearer for API Keys

**What goes wrong:** The server interprets the API key as a JWT, authentication fails, and credential material enters the wrong validation/logging path; the CLI generator can reproduce the defect in every emitted LangChain client. [VERIFIED: `bridges/cli/src/core/http.ts`; `bridges/cli/src/generators/langchain.ts`; `bridges/universal-tool-server/main.py`; `crates/noesis-api/src/middleware.rs`]

**Why it happens:** Current CLI/tool configuration stores one API-key string and then formats it as bearer, rather than preserving distinct auth kinds. [VERIFIED: `bridges/cli/src/core/http.ts`; `bridges/cli/src/generators/langchain.ts`; `bridges/universal-tool-server/main.py`]

**How to avoid:** Model `apiKey` and `bearerToken` as exclusive typed options and assert exact headers at the recording server and in generated-client snapshots.

**Warning signs:** Any API-key test contains `Authorization: Bearer`.

### Pitfall 5: Assuming Validation Exists Because the Trait Has a Method

**What goes wrong:** Rust calls a TypeScript route that is not registered, or a stub returns success without checking an invariant. [VERIFIED: `crates/noesis-bridge/src/lib.rs`; `ts-engines/src/server/app.ts`]

**Why it happens:** The Rust trait method exists while the TypeScript transport route and interface do not. [VERIFIED: `crates/noesis-bridge/src/lib.rs`; `ts-engines/src/server/app.ts`; `ts-engines/src/types/engine.ts`]

**How to avoid:** Publish per-engine operation support. For TypeScript engines, return canonical `OPERATION_UNSUPPORTED` without transport until a real validator and negative fixture exist.

**Warning signs:** Validation tests assert only HTTP 200 or `valid: true`.

### Pitfall 6: Letting Sidecar Errors Cross the Trust Boundary

**What goes wrong:** Internal URLs, raw bodies, parser messages or secret-bearing configuration appear in JSON or telemetry. [VERIFIED: `crates/noesis-api/src/handlers/biofield.rs`; `crates/noesis-api/src/error_mapper.rs`]

**Why it happens:** Bridge errors carry URLs/bodies as strings and some handlers serialize those values directly. [VERIFIED: `crates/noesis-bridge/src/error.rs`; `crates/noesis-api/src/handlers/biofield.rs`]

**How to avoid:** Convert at the boundary to allowlisted error code, dependency ID and trace ID; redact before logging/Sentry as well as before response serialization.

**Warning signs:** Response snapshots contain `http://`, `https://`, `token`, `api_key`, upstream body text or stack/version details.

### Pitfall 7: Partial Workflow Success Masquerading as Complete

**What goes wrong:** Failed engines vanish, a generic placeholder is emitted, or empty LLM output is labelled powered. [VERIFIED: `crates/noesis-orchestrator/src/lib.rs`; `crates/noesis-orchestrator/src/workflow/executor.rs`; `crates/noesis-api/src/handlers/witness.rs`]

**Why it happens:** Both current executors collect only successful values. [VERIFIED: `crates/noesis-orchestrator/src/lib.rs`; `crates/noesis-orchestrator/src/workflow/executor.rs`]

**How to avoid:** Preserve every requested ID and classify execution before synthesis; require a non-empty successful synthesis receipt for `available`/`llm_powered`.

**Warning signs:** The response has fewer outputs than the workflow definition and no failure array.

### Pitfall 8: Letting Contract Work Expand Into Later Phases

**What goes wrong:** The phase begins correcting calculations, durable auth, package publication or production topology and loses a testable contract boundary. [VERIFIED: `.planning/ROADMAP.md` Phases 4-7]

**How to avoid:** Use deterministic fixtures and injected transports only. Record semantic/deployment fields as unknown or out of scope.

**Warning signs:** A Phase 3 task calls a provider, changes a migration, packs a release or requires a deployed URL.

## Code Examples

These are planning patterns, not drop-in final code; exact module placement remains planner discretion. [VERIFIED: `.planning/phases/03-capability-and-contract-closure/03-CONTEXT.md` agent discretion]

### Pure Capability Reducer

~~~rust
fn resolve_capability(
    declared: RegistryEngine,
    registered: bool,
    observations: &[DependencyObservation],
) -> EngineCapability {
    let availability = if observations.iter().any(|o| o.required && o.state == Unavailable) {
        Unavailable
    } else if observations.iter().any(|o| !o.required && o.state == Unavailable)
        || observations.iter().any(|o| o.state == Degraded)
    {
        Degraded
    } else if registered && observations.iter().filter(|o| o.required).all(|o| o.state == Available) {
        Available
    } else {
        Declared
    };

    EngineCapability::from_registry(declared, availability, observations)
}
~~~

Source basis: four states and required/optional precedence are locked by D-02; executable membership already comes from `EngineRegistry`. [VERIFIED: `.planning/phases/03-capability-and-contract-closure/03-CONTEXT.md`; `crates/noesis-orchestrator/src/lib.rs`]

### Typed Authentication Choice

~~~typescript
type Auth =
  | { kind: 'api-key'; value: string }
  | { kind: 'bearer'; value: string }

function authHeaders(auth: Auth): Record<string, string> {
  return auth.kind === 'api-key'
    ? { 'X-API-Key': auth.value }
    : { Authorization: `Bearer ` + auth.value }
}
~~~

Source basis: the Rust middleware explicitly accepts `X-API-Key` for API keys and bearer for JWTs, matching distinct OpenAPI security schemes. [VERIFIED: `crates/noesis-api/src/middleware.rs`] [CITED: https://spec.openapis.org/oas/v3.1.2.html#security-scheme-object]

### Recording Zero-Downstream Test

~~~rust
#[tokio::test]
async fn invalid_contract_version_never_calls_engine() {
    let recorder = RecordingEngine::default();
    let app = test_app_with_engine(recorder.clone());

    let response = request(app)
        .api_key("test-key")
        .json(json!({ "contract_version": "v2", "parameters": {} }))
        .post("/api/v1/engines/tarot/calculate")
        .await;

    assert_eq!(response.status(), 422);
    assert_canonical_error(response, "CONTRACT_VERSION_UNSUPPORTED");
    assert_eq!(recorder.calls(), 0);
}
~~~

Source basis: existing routing enforcement uses recording seams, and D-04/D-05 require fail-closed version skew with zero downstream calls. [VERIFIED: `crates/noesis-api/tests/routing_enforcement_tests.rs`; `.planning/phases/03-capability-and-contract-closure/03-CONTEXT.md`]

### Explicit Workflow Outcome

~~~json
{
  "contract_version": "v1",
  "workflow_id": "birth-blueprint",
  "requested_engine_ids": ["numerology", "human-design", "vimshottari"],
  "engine_outputs": {
    "numerology": {}
  },
  "engine_failures": [
    { "engine_id": "human-design", "error_code": "DEPENDENCY_UNAVAILABLE" },
    { "engine_id": "vimshottari", "error_code": "ENGINE_TIMEOUT" }
  ],
  "execution_status": "partial",
  "synthesis_status": "available",
  "synthesis": {}
}
~~~

The failure entries must be bounded and sanitized; they should carry stable code, engine ID and optional safe reason code, not raw bridge text. [VERIFIED: D-05/D-07; `contracts/v1/schemas/error.schema.json`]

## State of the Art

| Old / Current Approach | Phase 3 Approach | Impact |
|------------------------|------------------|--------|
| Runtime registration list is treated as catalogue | Canonical identity joined with runtime observations | Disabled services remain visible and explainable. [VERIFIED: D-01/D-02] |
| Health is boolean and missing checks pass | Four-state availability with required/optional observations | No evidence remains `declared`; partial dependencies become `degraded`. [VERIFIED: `ts-engines/src/server/app.ts`; D-02] |
| Each SDK owns IDs and list shapes | Generated or validator-enforced projection plus canonical API envelope | Catalogue, method and envelope drift fails CI. [VERIFIED: D-03; current SDK divergence] |
| API keys and JWTs share a generic secret path | Explicit auth-kind transport | `X-API-Key` and bearer JWT cannot be conflated. [VERIFIED: `crates/noesis-api/src/middleware.rs`] [CITED: https://spec.openapis.org/oas/v3.1.2.html] |
| Engine failures are logged and omitted | Complete/partial/failed outcome with per-engine failures | Consumers can distinguish usable partial output from success. [VERIFIED: current executors; D-07] |
| Placeholder synthesis is returned as synthesis | Supported synthesizers execute; unsupported/failed states are explicit | Empty or generic text cannot imply semantic or LLM work. [VERIFIED: `crates/noesis-orchestrator/src/workflow/executor.rs`; `crates/noesis-api/src/handlers/witness.rs`] |

**Deprecated/outdated:**

- Hard-coded “all engines” arrays in SDK, CLI/tool, Hermes, TUI, witness and admin presentation are obsolete as catalogue authorities. [VERIFIED: D-03; current source arrays]
- `default health check passed` for engines without checks is a false-positive availability path and should be removed. [VERIFIED: `ts-engines/src/server/app.ts`]
- API key via `Authorization: Bearer` in CLI runtime, generated LangChain clients and the universal tool server is incompatible with the server contract. [VERIFIED: `bridges/cli/src/core/http.ts`; `bridges/cli/src/generators/langchain.ts`; `bridges/universal-tool-server/main.py`; `crates/noesis-api/src/middleware.rs`]
- The generic “Full synthesis implementation pending” success path should be removed. [VERIFIED: `crates/noesis-orchestrator/src/workflow/executor.rs`; D-07]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | No factual implementation claim remains training-only; repository facts were inspected and standards claims use official sources. | All | — |

Proposed field names and module paths are recommendations under the agent's discretion, not claims about current source. [VERIFIED: `.planning/phases/03-capability-and-contract-closure/03-CONTEXT.md` agent discretion]

## Open Questions (RESOLVED)

1. **How long should the legacy admin capability array remain?**
   - What we know: D-04 requires supported v1 payloads to remain valid, while the existing admin route returns a raw six-row array. [VERIFIED: `.planning/phases/03-capability-and-contract-closure/03-CONTEXT.md` D-04; `crates/noesis-api/src/handlers/admin.rs`]
   - What's unclear: No reviewed repository policy assigns a removal version/date. [VERIFIED: repository policy search]
   - RESOLVED: Retain the legacy raw admin capability-array adapter through Phase 3, document it as compatibility-only, and require maintained consumers to use the canonical envelope. Removal/versioning policy remains a Phase 6 concern.

2. **Can Full Spectrum use the common synthesis result without semantic redesign?**
   - What we know: five workflow synthesizers accept the common engine-output map, while `FullSpectrumSynthesizer` consumes a different `FullSpectrumResult` aggregate. [VERIFIED: `crates/noesis-orchestrator/src/workflow/synthesis/mod.rs`; `crates/noesis-orchestrator/src/workflow/synthesis/full_spectrum.rs`]
   - What's unclear: The reviewed tests do not prove that a lossless adapter can be added without changing synthesis semantics. [VERIFIED: `crates/noesis-orchestrator/src/workflow/synthesis/full_spectrum.rs` and tests]
   - RESOLVED: Full Spectrum remains explicitly `unsupported` throughout Phase 3. It may be advertised as executable only after a lossless adapter test proves the existing aggregate input and output semantics unchanged; no such adapter is planned in this phase.

Planning is unblocked by these fail-closed resolutions: the legacy admin adapter has a bounded Phase 3 lifetime, and Full Spectrum stays visible but unsupported pending a lossless adapter test. [VERIFIED: resolutions above]

## Resolved Planning Constraints

- **Legacy admin adapter:** The existing raw admin capability array remains available through Phase 3 as a named compatibility adapter. The canonical protected capability envelope is the only maintained consumer target; no plan may delete the adapter or treat it as canonical authority.
- **Full Spectrum:** The workflow registry may expose the identity and its unsupported status, but no producer or bridge may claim executable support without a lossless adapter test. The Phase 3 plan set therefore tests explicit unsupported behavior and zero synthesis/provider calls.
- **Locked Python test runtime:** The checked-in `python-services/pyproject.toml` and `python-services/uv.lock` are the sole test environment for sidecar, universal-tool and Hermes suites. Plan 01 must prove the required imports through `uv run --locked --extra dev` before any such suite runs; if that preflight fails, the phase fails closed. No fallback project, dependency addition, lockfile mutation or ambient `python3` execution is valid.

## Environment Availability

| Dependency | Required By | Available | Version / State | Notes |
|------------|-------------|-----------|-----------------|----------|
| Cargo / rustc | Rust tests | ✓ | 1.97.0 / 1.97.0 | Use locked workspace toolchain policy. [VERIFIED: environment probe] |
| Node / npm / pnpm | TS SDK/admin/witness checks | ✓ | Node 26.8.1, npm 11.19.0, pnpm 10.33.0 | — [VERIFIED: environment probe] |
| Bun | TypeScript engine and focus SDK tests | ✓ | 1.3.13 | — [VERIFIED: environment probe] |
| Python 3.12 | Locked sidecar test environment | ✓ | `/opt/homebrew/opt/python@3.12/bin/python3.12` | Python 3.11.15 is also installed through uv. [VERIFIED: `uv python find`] |
| uv | Python lock hydration | ✓ | 0.7.13 | — [VERIFIED: environment probe] |
| Root pnpm dependencies | Contract compiler helper and Vitest | ✗ | `node_modules` absent; general SDK and verification-package Vitest binaries absent | Run the frozen/offline Phase 2 hydration first. [VERIFIED: filesystem and command probes; `.planning/phases/02-reproducible-gates-dependency-repair/02-REVIEW-RECHECK-5.md`] |
| Python sidecar environment | Python health tests | ✗ in global 3.14 | pytest exists, OpenCV and MediaPipe absent | Use locked Python 3.12 uv environment; do not use global Python for the gate. [VERIFIED: environment import probe; `python-services/uv.lock`] |
| Context7 | Documentation lookup | ✗ | CLI absent and no MCP tool available | Official specifications were used directly. [VERIFIED: tool and command inventory] |
| Local database/provider services | Not required | — | Deliberately not probed or mutated | Use lazy pools, fake engines and recording transports. [VERIFIED: task scope; existing test harnesses] |

**Missing dependencies after frozen hydration:** none. If the required import preflight fails, the phase is blocked until the existing locked project can hydrate; no alternate project is permitted. [VERIFIED: lockfiles and available runtimes]

Environment hydration notes:

- Root Node dependencies are absent: hydrate from `pnpm-lock.yaml` before contract validation. [VERIFIED: filesystem probe]
- Python CV modules are absent from global Python: use `uv sync --locked --extra dev --python 3.12` in `python-services`, then run all suites through its locked `uv run`. [VERIFIED: import probe; `python-services/uv.lock`]

Research-time focused receipts:

- `cargo test -p noesis-core --test contract_v1_authority --locked` passed; `cargo test -p noesis-api --test capability_route_tests --locked` passed; `cargo test -p noesis-bridge -p noesis-sdk --locked` passed; API OpenAPI/routing tests passed; and workflow-focused orchestrator tests passed. [VERIFIED: commands executed 2026-09-06]
- `bun test src/server/__tests__/registry-authority.test.ts` in `ts-engines` and `pnpm --filter @selemene/engine-sdk test` passed. [VERIFIED: commands executed 2026-09-06]
- The locked validator command, its Python tests, the sidecar health tests, `@noesis/sdk` Vitest and the focused `@noesis/verification` source-adapter test could not form valid gates in the unhydrated checkout because the TypeScript compiler dependency, CV modules and Vitest binaries were absent. These are environment gaps, not product failures; Plan 01's frozen hydration and import preflight must pass before the suites run. [VERIFIED: command failures and filesystem probes 2026-09-06]

## Validation Architecture

Nyquist validation is enabled, so every implementation slice needs a sub-30-second focused command where feasible; each wave runs its focused targets, and Plan 21 alone runs the expanded contract gate once. [VERIFIED: `.planning/config.json` `workflow.nyquist_validation=true`]

### Dependency Ordering

| Order | Slice | Why it must precede the next slice |
|------:|-------|------------------------------------|
| 0 | Hydrate frozen pnpm/Bun/uv environments | Otherwise contract and SDK failures are false environment negatives. [VERIFIED: research-time failures] |
| 1 | Extend registry metadata, schemas, fixtures and validator | Every language adapter needs a frozen target before code changes. [VERIFIED: D-03/D-04] |
| 2 | Update Rust/TypeScript/Python wire readers and checked-in projections | Strict readers must accept additive fields before any producer emits them. [VERIFIED: `deny_unknown_fields` and `additionalProperties:false`] |
| 3 | Implement pure capability reducer and native/conditional observations | Establish identity and availability semantics without HTTP. [VERIFIED: CON-01] |
| 4 | Add explicit TypeScript self-check and Python dependency adapters | Complete all runtime observation inputs before public projection. [VERIFIED: D-02/D-05] |
| 5 | Add protected API envelope, OpenAPI and sanitized errors | Public consumers need one stable boundary. [VERIFIED: CON-02] |
| 6 | Fix bridge operation support, auth transport and route allowlists | These adapters depend on the public contract and capability operation metadata. [VERIFIED: D-05/D-06] |
| 7 | Converge SDK, CLI/tool, Hermes, TUI, admin, witness and verification-adapter views | Consumer tests can now execute against the authoritative server fixture. [VERIFIED: D-03; maintained surfaces in `docs/plans/selemene-engine/CAPABILITY-LEDGER.md`] |
| 8 | Wire workflow executor and explicit outcome/synthesis status | Workflow responses reuse canonical errors, capability filtering and consumer types. [VERIFIED: D-07] |
| 9 | Run cross-language/full gate and independent negative review | The phase exits only when drift and false-green paths fail. [VERIFIED: Wave 2 exit in `docs/plans/selemene-engine/ROADMAP.md`] |

### Test Layers and Exact Commands

| Layer | Target behavior | Focused command |
|-------|-----------------|-----------------|
| Authority and negative fixtures | Exact 19 IDs/17 mirrors/classes; schema/envelope/dependency/operation metadata; duplicate/missing/skew failures | `uv run --project python-services --locked --extra dev python scripts/validate_contracts.py && uv run --project python-services --locked --extra dev python -m pytest tests/scripts/test_validate_contracts.py -q` |
| Rust fixture parity | New capability-list and workflow-outcome fixtures round-trip through strict Rust types | `cargo test -p noesis-core --test contract_v1_authority --locked` |
| Native/conditional resolver | All 19 rows survive no-pool/lazy-pool and synthetic dependency-health matrices; registration and availability remain separate | `cargo test -p noesis-orchestrator capability --locked && env -u DATABASE_URL -u TEST_DATABASE_URL cargo test -p noesis-api database_conditional_registration --locked` |
| TypeScript observations | Six exact IDs; missing self-check is not available; thrown/timeout/optional degradation mapping | `cd ts-engines && bun test src/server/__tests__/registry-authority.test.ts src/server/__tests__/capability-route.test.ts` |
| Python observations | Required/optional dependency matrices and canonical adapter shape | `cd python-services && uv run --locked --extra dev python -m pytest tests/test_capability_health.py -q --tb=line` |
| API capability/auth/error | Protected 19-row envelope, admin permission, content type, stable codes, no leaks | `cargo test -p noesis-api --test capability_route_tests --test error_handling_tests --test error_response_snapshot_tests --locked` |
| OpenAPI | Capability/workflow schemas, response envelopes and distinct bearer/API-key schemes | `cargo test -p noesis-api --test openapi_schema_tests --test auth_rate_limit_openapi_tests --locked` |
| Routing/zero calls | Native/TS/Python dispatch, unknown ID, auth, validation, version skew, timeouts | `cargo test -p noesis-api --test routing_enforcement_tests --test integration_tests --locked` |
| Rust bridge | Calculate/validate support, timeout and sanitized upstream failures against a recording HTTP server | `cargo test -p noesis-bridge --locked` |
| Rust SDK/TUI | Authenticated capability decoding and dynamic picker projection | `cargo test -p noesis-sdk -p noesis-tui --locked` |
| General TypeScript SDK | IDs, envelopes, methods, header choice, canonical error parsing | `pnpm --filter @noesis/sdk test && pnpm --filter @noesis/sdk typecheck` |
| Focus SDK | Four-engine subset remains explicit; public calculations require Rust API; consent rejects before fetch | `pnpm --filter @selemene/engine-sdk test && pnpm --filter @selemene/engine-sdk typecheck` |
| CLI bridge | Correct header in runtime and generated LangChain output, timeout, JSON/non-JSON error adaptation and health evidence axes | `cd bridges/cli && bun test && bun run typecheck` |
| Universal tool server | Generated tool list, ID allowlist/encoding, correct auth, timeout and redaction | `(cd python-services && PYTHONPATH=.. uv run --locked --extra dev python -m pytest ../bridges/universal-tool-server/tests -q)` |
| Hermes bridge | Canonical tool enums/counts, supported workflow subset, exact `X-API-Key`, invalid tool rejection and zero-call behavior | `(cd python-services && PYTHONPATH=.. uv run --locked --extra dev python -m pytest ../bridges/hermes/tests -q)` |
| Admin | Capability/operational axes join without identity/status redefinition | `pnpm --filter admin-web test && pnpm --filter admin-web typecheck` |
| Witness package | Canonical eligible subset, unavailable skip receipts, correct auth and partial failures | `pnpm --filter @noesis/witness-pipeline test && pnpm --filter @noesis/witness-pipeline typecheck` |
| Verification source adapter | Canonical witness result/error projection and safe boundary failures; semantic golden scoring remains a Phase 4 concern | `pnpm --filter @noesis/verification test -- src/sources/selemene.test.ts && pnpm --filter @noesis/verification typecheck` |
| Workflow core/API | Failure retention, complete/partial/failed, five supported synthesizers, unsupported Full Spectrum, no empty powered label | `cargo test -p noesis-orchestrator workflow --locked && cargo test -p noesis-api --test workflow_execution_tests --test workflow_tests --locked && cargo test -p noesis-witness --locked` |
| Phase gate | Every authority, adapter and root regression check | `env -u DATABASE_URL -u TEST_DATABASE_URL pnpm run gate` after expanding `gate:contracts` |

Commands naming new files are Wave 0 gaps until those tests are created; the planner should create the test target in the same slice before implementation. [VERIFIED: filesystem scan]

### Required Negative Cases

| Boundary | Negative case | Required assertion |
|----------|---------------|--------------------|
| Registry | Missing, duplicate or extra engine; wrong 12/1/6 class split; wrong 17-mirror policy | Validator fails before any generated projection changes. |
| Compatibility | Old valid capability item; unknown contract version; undeclared field under a strict reader | Old fixture passes; version/schema skew returns canonical stable error; strict-reader rollout order is tested. |
| Native/conditional | No pool; lazy pool without a health observation; synthetic healthy/unhealthy database observations | `biofield-capture` always remains in the 19 rows; registration may become present with a lazy pool, but availability is only available after a bounded successful observation. No real DB connection occurs. |
| TypeScript | No `selfCheck`, false result, thrown error, timeout, malformed response | Never map missing/failed evidence to available; reason codes stay bounded. |
| Python | Missing required OpenCV/NumPy; missing optional MediaPipe; malformed/timeout health | Unavailable, degraded and unavailable respectively; no URL/raw exception is emitted. |
| Auth | Missing/invalid API key, malformed/expired bearer, non-admin on admin route | Canonical JSON 401/403 with matching content type and zero engine/provider/sidecar calls. |
| Routing | Unknown/overlong/slashed engine ID; unsupported operation | Stable 404/422/501-class contract response, zero downstream call and no path escape. |
| Input | Bad contract version, malformed JSON, invalid bounds, missing consent | Canonical error and zero downstream calls. |
| Bridge | Connection refusal, timeout, non-JSON body, oversized body, upstream 4xx/5xx | Stable sanitized code; no internal URL/body/secret in response or captured telemetry. |
| SDK/CLI/generator | API key supplied, bearer supplied, generated LangChain client, plain-text/non-JSON server error | Exact correct header in runtime and generated source; parser returns canonical client error rather than throwing a JSON parse accident. |
| Hermes | Missing/extra/stale engine ID, unsupported Full Spectrum claim, malformed tool name, API key supplied | Tool declarations match the canonical public projection, unsupported synthesis is explicit, invalid names cause zero HTTP calls, and the API key remains only in `X-API-Key`. |
| Workflow | All succeed, one fails, all fail, engine absent/unavailable | Complete, partial, failed, failed; every requested ID accounted for. |
| Synthesis | Supported success, supported error, unsupported type, empty provider response | Available, failed, unsupported, failed; empty result is never LLM-powered. |
| Witness Dyad availability | Live scores supplied while `biofield` execution is unavailable; one of the seven context engines fails | Supplied evidence is distinguished from engine availability and each failure is retained rather than silently dropped. |

### Evidence Semantics

| Claim | Minimum evidence | What it does not prove |
|-------|------------------|------------------------|
| Identity | Exact row in the versioned canonical registry and projection parity | Loaded process, reachable service, calculation result or deployment. [VERIFIED: D-01] |
| Availability | Current bounded observation of executable registration plus required dependencies | Engine semantic correctness, provider output quality or deployed revision. [VERIFIED: D-02; `ISA.md` Principles] |
| Reachability | A bounded transport receives a response | Authentication, correct route, capability availability or semantics. [VERIFIED: current CLI health implementation] |
| Integration/routing | Recording test proves the intended adapter and exact downstream call, including zero-call rejection cases | Real production topology or calculation correctness. [VERIFIED: D-05] |
| Authenticated contract | Protected request succeeds/fails with correct scheme, envelope and authorization | Token durability/revocation, billing state or production identity. [VERIFIED: Phase 5 boundary] |
| Semantic correctness | Phase 4 per-engine golden fixture and provenance assertions | Deployment or operational health. [VERIFIED: `.planning/ROADMAP.md` Phase 4] |
| Deployed state | Phase 7 immutable source/image/config receipt from the target | Authenticated end-to-end semantic correctness. [VERIFIED: `.planning/ROADMAP.md` Phase 7; Phase 2 production HOLD] |
| Operational state | Revision-bound authenticated journey and ongoing monitoring receipt | Future availability or every semantic case. [VERIFIED: `docs/plans/selemene-engine/CAPABILITY-LEDGER.md` evidence axes] |

### Sampling Rate

- **Per task commit:** run the single focused command for that layer and its negative cases.
- **Per wave merge:** run only that wave's focused targets; defer `pnpm run gate:contracts` until Plan 21 and invoke it exactly once there.
- **Phase gate:** frozen dependencies hydrated, `env -u DATABASE_URL -u TEST_DATABASE_URL pnpm run gate` green, Python 3.11/3.12 sidecar matrix green in CI, and independent review finds no high-severity false-green or leak. [VERIFIED: Phase 2 gate pattern; `.planning/config.json` security block level]

### Wave 0 Gaps

- [ ] Add canonical full capability-list and workflow-outcome fixtures/schemas and negative mutations under `contracts/v1`.
- [ ] Extend `tests/scripts/test_validate_contracts.py` for every maintained catalogue, method, auth scheme and envelope.
- [ ] Add `ts-engines/src/server/__tests__/capability-route.test.ts` for explicit self-check states.
- [ ] Expand `crates/noesis-api/tests/capability_route_tests.rs` from the current six-bridge admin behavior to the 19-row protected envelope.
- [ ] Add bridge recording-server tests for validation support and redaction.
- [ ] Add a `test` script and transport tests to `bridges/cli/package.json`.
- [ ] Add CLI generator snapshots that assert `X-API-Key` and canonical public tool operations in emitted LangChain source.
- [ ] Add `bridges/universal-tool-server/tests`; none exists today. Run it only with the locked `python-services` uv project. [VERIFIED: filesystem scan]
- [ ] Add `bridges/hermes/tests`; none exists today, and freeze tool IDs, workflow support and recording-transport behavior there through the locked `python-services` uv project. [VERIFIED: filesystem scan]
- [ ] Add TUI dynamic capability tests and admin dual-axis projection tests.
- [ ] Extend `packages/verification/src/sources/selemene.test.ts` only for Phase 3 adapter/auth/error parity; leave golden-result semantics and package export completion to Phases 4 and 6. [VERIFIED: current package scope and roadmap boundaries]
- [ ] Add workflow outcome fixtures and failure-retention tests before switching the public executor.
- [ ] Expand `gate:contracts` to run all new parity targets; the current script covers only core/OpenAPI/calculation/focus SDK/general SDK/witness build. [VERIFIED: `package.json`]

## Security Domain

Security enforcement is enabled at ASVS Level 1 and the project blocks on high-severity findings. ASVS 5.0.0 is the current stable OWASP release, and control identifiers below use the versioned `v5.0.0-...` form recommended by OWASP. [VERIFIED: `.planning/config.json`] [CITED: https://owasp.org/www-project-application-security-verification-standard/]

### Applicable ASVS 5.0.0 Controls

| ASVS Control | Applies | Phase 3 control |
|--------------|---------|-----------------|
| v5.0.0-1.2.2 Dynamic URL encoding (L1) | Yes | Allowlist canonical engine/workflow IDs and encode each path parameter after validation; never substitute arbitrary strings into tool routes. [CITED: https://raw.githubusercontent.com/OWASP/ASVS/master/5.0/docs_en/OWASP_Application_Security_Verification_Standard_5.0.0_en.flat.json] |
| v5.0.0-2.1.1 Document input rules (L1) | Yes | Schemas and registry metadata define ID, version, dependency, operation and workflow-state rules. [CITED: https://raw.githubusercontent.com/OWASP/ASVS/master/5.0/docs_en/OWASP_Application_Security_Verification_Standard_5.0.0_en.flat.json] |
| v5.0.0-2.2.1 Positive validation and logical limits (L1) | Yes | Enforce ID allowlists, contract version, bounded arrays/strings/reasons, request size and workflow fanout at the API. [CITED: https://raw.githubusercontent.com/OWASP/ASVS/master/5.0/docs_en/OWASP_Application_Security_Verification_Standard_5.0.0_en.flat.json] |
| v5.0.0-2.2.2 Trusted-layer validation (L1) | Yes | Client checks improve errors, but Rust must reject before any engine, bridge or provider call. [CITED: https://raw.githubusercontent.com/OWASP/ASVS/master/5.0/docs_en/OWASP_Application_Security_Verification_Standard_5.0.0_en.flat.json] |
| v5.0.0-4.1.1 Accurate Content-Type (L1) | Yes | Every error with a body is canonical JSON with an application/json content type; SDKs still tolerate non-JSON legacy failures safely. [CITED: https://raw.githubusercontent.com/OWASP/ASVS/master/5.0/docs_en/OWASP_Application_Security_Verification_Standard_5.0.0_en.flat.json] |
| v5.0.0-8.1.1 Authorization rules documented (L1) | Yes | Document public protected capability access versus admin permission and internal sidecar boundaries. [CITED: https://raw.githubusercontent.com/OWASP/ASVS/master/5.0/docs_en/OWASP_Application_Security_Verification_Standard_5.0.0_en.flat.json] |
| v5.0.0-8.2.1 Function permission (L1) | Yes | Preserve admin authorization for operational detail while exposing only safe capability data to ordinary authenticated clients. [CITED: https://raw.githubusercontent.com/OWASP/ASVS/master/5.0/docs_en/OWASP_Application_Security_Verification_Standard_5.0.0_en.flat.json] |
| v5.0.0-8.2.2 Data-specific permission (L1) | Yes where admin data is joined | Do not leak admin analytics/configuration fields into the general capability envelope. [CITED: https://raw.githubusercontent.com/OWASP/ASVS/master/5.0/docs_en/OWASP_Application_Security_Verification_Standard_5.0.0_en.flat.json] |
| v5.0.0-8.3.1 Trusted-layer authorization (L1) | Yes | Enforce auth/permission in Rust, not TUI/admin visibility or client filtering. [CITED: https://raw.githubusercontent.com/OWASP/ASVS/master/5.0/docs_en/OWASP_Application_Security_Verification_Standard_5.0.0_en.flat.json] |
| v5.0.0-9.1.1/9.1.2/9.1.3 Token integrity, algorithm allowlist and trusted keys (L1) | Regression scope | Do not redesign token auth in Phase 3; preserve and regression-test the current validator before capability/workflow routing. [CITED: https://raw.githubusercontent.com/OWASP/ASVS/master/5.0/docs_en/OWASP_Application_Security_Verification_Standard_5.0.0_en.flat.json] |
| v5.0.0-9.2.1 Token time span (L1) | Regression scope | Expired/not-yet-valid bearer tokens must fail before downstream work. [CITED: https://raw.githubusercontent.com/OWASP/ASVS/master/5.0/docs_en/OWASP_Application_Security_Verification_Standard_5.0.0_en.flat.json] |
| v5.0.0-16.5.1/16.5.2/16.5.3 Error secrecy, secure external failure and fail-closed exceptions (L2 defense in depth) | Yes by project decision | Although these error controls are ASVS L2, D-05 explicitly requires no secret/internal URL leaks and fail-closed dependency behavior. [CITED: https://raw.githubusercontent.com/OWASP/ASVS/master/5.0/docs_en/OWASP_Application_Security_Verification_Standard_5.0.0_en.flat.json] [VERIFIED: `03-CONTEXT.md` D-05] |

### Threat Model and Blocking Risks

| Severity | Threat / STRIDE | Current evidence | Plan must block completion until |
|----------|-----------------|------------------|----------------------------------|
| HIGH | Credential-type confusion / Spoofing | CLI runtime, generated LangChain clients and the universal tool server send API keys as bearer JWTs; the server distinguishes these paths. [VERIFIED: `bridges/cli/src/core/http.ts`; `bridges/cli/src/generators/langchain.ts`; `bridges/universal-tool-server/main.py`; `crates/noesis-api/src/middleware.rs`] | Recording and generated-output tests prove API keys use only `X-API-Key`, bearer remains a separate typed option, and missing/invalid auth causes zero downstream calls. |
| HIGH | Internal data disclosure / Information Disclosure | Biofield error mapping can return internal URLs and raw upstream bodies; telemetry receives raw engine error strings. [VERIFIED: `crates/noesis-api/src/handlers/biofield.rs`; `crates/noesis-api/src/error_mapper.rs`; `crates/noesis-bridge/src/error.rs`] | Response and telemetry snapshots reject URLs, tokens, keys, raw body, stack/version details and sensitive auth reasons across every Phase 3 route. |
| HIGH | Fail-open capability / Tampering and Availability | Missing TypeScript self-check maps to available and missing runtime rows can disappear from list responses. [VERIFIED: `ts-engines/src/server/app.ts`; `crates/noesis-api/src/handlers/admin.rs`] | No-observation maps to declared, required failures map unavailable, optional failures degraded, and every dependency matrix returns exactly 19 identities. |
| HIGH | Operation/path injection / Tampering or Elevation | Universal tool routing interpolates unvalidated argument strings into privileged, credential-forwarding paths. [VERIFIED: `bridges/universal-tool-server/main.py`] | Tool names and IDs are canonical allowlists, path values are encoded, direct internal TS tools are not public, and adversarial slash/dot/overlong cases cause zero HTTP calls. |
| HIGH | Stale executable inventory / Tampering and Improper Inventory | Universal tools advertise unsupported/direct operations while Hermes freezes a 16-engine catalogue and presents Full Spectrum as all engines. [VERIFIED: `bridges/universal-tool-server/main.py`; `bridges/hermes/tools.py`; `bridges/hermes/agent.py`] | Every maintained tool declaration is derived from or parity-checked against canonical public identities and operation support; stale, internal-only or unsupported operations fail the contract gate. |
| HIGH if ingress-exposed | Unauthenticated sidecar access / Elevation | TypeScript and Python sidecar apps have no Rust auth middleware; Python apps configure wildcard CORS with credentials. [VERIFIED: `ts-engines/src/server/app.ts`; `python-services/*_service/main.py`] | Phase 3 defines them as internal-only and public SDK/tool/OpenAPI surfaces route through Rust; any actual ingress/auth change remains a Phase 7 deployment gate. |
| HIGH | False successful workflow / Tampering | Both workflow executors omit failures; Witness Dyad drops seven-engine subset failures, advertises `biofield` from supplied live scores, and can label empty failed synthesis as LLM-powered. [VERIFIED: `crates/noesis-orchestrator/src/lib.rs`; `crates/noesis-orchestrator/src/workflow/executor.rs`; `crates/noesis-api/src/handlers/witness.rs`] | Every requested engine and supplied-evidence source is accounted for separately, and all-failed/partial/empty synthesis cases produce failed/partial statuses with no powered claim. |
| HIGH | Validation bypass / Tampering | Rust bridge calls a nonexistent TypeScript validation route, encouraging either 404 drift or stub-success repair. [VERIFIED: `crates/noesis-bridge/src/lib.rs`; `ts-engines/src/server/app.ts`] | Unsupported validation returns a stable capability/error without network; supported validation has meaningful positive and negative engine invariants. |
| MEDIUM | Error parser denial / Availability | General TypeScript SDK unconditionally JSON-parses response text. [VERIFIED: `packages/noesis-sdk-ts/src/index.ts`] | Empty, plain-text and malformed error bodies normalize to a safe canonical client error. |
| MEDIUM | Probe amplification / Denial of Service | Tool calls allow 30-second requests and workflow execution fans out over engine sets. [VERIFIED: `bridges/universal-tool-server/main.py`; `crates/noesis-orchestrator/src/workflow/executor.rs`] | Timeouts, body limits and concurrency remain bounded; auth/input rejection precedes fanout; tests use canonical finite sets. |

The OWASP API Security Top 10 categories most relevant here are API2 Broken Authentication, API4 Unrestricted Resource Consumption, API5 Broken Function Level Authorization, API8 Security Misconfiguration, API9 Improper Inventory Management and API10 Unsafe Consumption of APIs. [CITED: https://owasp.org/API-Security/editions/2023/en/0x11-t10/]

### Security Gate

Because `security_block_on` is `high`, none of the HIGH rows above may remain an untested assumption at phase completion. A deferred production ingress change is acceptable only when the repository contract makes the sidecar internal-only and no maintained public consumer bypasses the Rust boundary. [VERIFIED: `.planning/config.json`; task phase boundary]

## Sources

### Primary (HIGH confidence)

- `.planning/phases/03-capability-and-contract-closure/03-CONTEXT.md` — locked scope and decisions. [VERIFIED: file read]
- `.planning/REQUIREMENTS.md` and `.planning/ROADMAP.md` — CON-01/CON-02 and phase boundaries. [VERIFIED: files read]
- `ISA.md` — acceptance, isolation and evidence semantics. [VERIFIED: file read]
- `contracts/v1/manifest.json`, `contracts/v1/schemas/*` and `contracts/v1/registries/engines.json` — contract and identity authority. [VERIFIED: files read and validator tests]
- `docs/plans/selemene-engine/ROADMAP.md` and `docs/plans/selemene-engine/CAPABILITY-LEDGER.md` — canonical Wave 2 tasks and contradiction register. [VERIFIED: files read]
- Rust, TypeScript and Python source paths cited inline — current implementation behavior. [VERIFIED: direct source inspection]
- Cargo, Bun, pnpm and Python focused commands executed on 2026-09-06 — environment and baseline receipts. [VERIFIED: command outputs]
- Local CodeGraph index — current at 867 files, 15,164 nodes and 37,101 edges; capability, bridge and workflow queries were used to locate cross-tier seams. [VERIFIED: `codegraph status` and local query results]

### Official standards (HIGH confidence)

- https://owasp.org/www-project-application-security-verification-standard/ — ASVS 5.0.0 status and versioned citation guidance. [CITED: https://owasp.org/www-project-application-security-verification-standard/]
- https://raw.githubusercontent.com/OWASP/ASVS/master/5.0/docs_en/OWASP_Application_Security_Verification_Standard_5.0.0_en.flat.json — exact ASVS controls and levels. [CITED: https://raw.githubusercontent.com/OWASP/ASVS/master/5.0/docs_en/OWASP_Application_Security_Verification_Standard_5.0.0_en.flat.json]
- https://owasp.org/API-Security/editions/2023/en/0x11-t10/ — API threat categories. [CITED: https://owasp.org/API-Security/editions/2023/en/0x11-t10/]
- https://spec.openapis.org/oas/v3.1.2.html — distinct API-key and HTTP bearer security scheme semantics. [CITED: https://spec.openapis.org/oas/v3.1.2.html]
- https://json-schema.org/draft/2020-12/json-schema-core — `additionalProperties` and validation behavior. [CITED: https://json-schema.org/draft/2020-12/json-schema-core]

### Secondary (MEDIUM confidence)

- None required.

### Tertiary (LOW confidence)

- None.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — versions came from checked-in lockfiles/manifests and local probes. [VERIFIED: lockfiles, manifests and environment probe]
- Architecture: HIGH — responsibilities follow locked decisions and inspected current call/route boundaries. [VERIFIED: `.planning/phases/03-capability-and-contract-closure/03-CONTEXT.md` and direct source inspection]
- Pitfalls: HIGH — each has a current source contradiction or official control. [VERIFIED: inline provenance]
- Validation: HIGH — existing runnable targets were inspected and focused baseline commands were executed; new filenames are explicitly marked Wave 0 gaps. [VERIFIED: test inventory and command outputs]
- Security: HIGH — source findings were mapped to official ASVS 5.0.0 and OWASP API controls. [CITED: https://owasp.org/www-project-application-security-verification-standard/] [CITED: https://owasp.org/API-Security/editions/2023/en/0x11-t10/]

**Research date:** 2026-09-06
**Valid until:** 2026-10-06, or until capability/auth/workflow source changes, whichever comes first.
