---
project: Selemene-engine
task: "Continue Selemene waves through verified planning, gates and infrastructure"
effort: E4
effort_source: classifier
phase: verify
progress: 320/325
mode: algorithm
iteration: 5
started: 2026-07-18T15:00:00+05:30
updated: 2026-09-05T13:47:55.589355+00:00
---

> **Active continuation — 2026-09-05:** Restore the existing seven-wave Selemene program, use GSD research/discuss/plan/execute/verify with recommended defaults, review infrastructure/dependencies and refresh CodeGraph. Criteria ISC-279 onward describe this continuation. Older entries are preserved dated evidence; their checked status does not imply current production acceptance. This file is the acceptance ledger; `.planning/` is the execution adapter.

> The earlier active-scope statement below is retained as a historical record.


> **Active scope — 2026-08-26:** This iteration establishes one language-neutral v1 contract authority inside the isolated `codex/selemene-contract-convergence` worktree. It adds golden fixtures, fail-closed validation, and additive Rust, OpenAPI, and TypeScript parity without changing engine calculations, routing, provider behavior, external repositories, or live state. Contract criteria begin at ISC-235.

## Problem

The verified gate foundation exists on an isolated branch while local `main` contains unrelated user-owned planning changes. Contract work cannot safely begin until the branch is merged without altering those changes, the merged revision passes the same repository gate under the primary checkout's real environment, and a fresh isolated contracts workspace starts from that exact revision.

### Continuation — 2026-09-05

The current checkout lacked the canonical plan files while GitHub retained the full program. Its ISA described a July consumer UI task, capability PR #1486 has failing formatting CI, dependency audits expose current Node findings, and provider metadata cannot yet connect every running service to the current source. See `docs/plans/selemene-engine/RECOVERY-2026-09-05.md` for the observed state.

## Vision

Local `main` contains the gate foundation with byte-for-byte preservation evidence for all prior edits. The retired gate worktree leaves no orphaned state, and contract convergence begins in a clean isolated workspace whose baseline passes the merged repository gate. The next bounded work establishes canonical versioned request, result, error, consent, provenance, and capability-discovery contracts before engine semantics change.

### Continuation — 2026-09-05

An operator can select any remaining wave, identify its existing GitHub work, navigate the real source through CodeGraph, and see exactly which local, CI, deployment and operational evidence is still missing. Routine choices flow automatically; concrete high-impact decisions receive a reviewable gate.

## Out of Scope

- Planning or implementing Sankalpa, Urania 137, FalseEarth, Raycast, or any other consumer repository.
- Changing engine semantics, public API contracts, providers, media behavior, or runtime routing in this gate-foundation slice.
- Renumbering, deleting, or rewriting existing migrations without production-schema evidence.
- Mutating credentials, databases, cloud resources, GitHub settings, branch protection, or live deployment state.
- Claiming all Wave 1 gates, contracts, engines, deployment, or operations complete from this bounded slice.
- Publishing packages, pushing commits, opening issues, triggering releases, or performing any merge beyond the explicitly authorized local gate integration.

### Continuation — 2026-09-05

Historical scope restrictions above applied to their recorded iterations. This continuation remains within Selemene and its declared infrastructure; external consumer implementation, destructive cleanup, unrequested provider generation, secret disclosure and invented completion claims are excluded. The current request authorizes routine reversible repairs and plan reconciliation; critical production/security boundaries remain explicit.

## Principles

- Current source and reproducible evidence outrank stale status narration.
- Completion is a vector: declaration, implementation, execution, integration, deployment, and operations are independent claims.
- Preserve local-first media handling and explicit consent before pixels or audio leave the client boundary.
- Health checks observe configuration and reachability; they never perform paid generation.
- Engine output is the authoritative deterministic anchor; narrative layers may reflect it but never silently replace it.
- External products test compatibility but do not define this repository's implementation plan.

### Continuation — 2026-09-05

- Preserve stable authority and historical IDs while allowing fresh evidence to refute stale narration.
- Keep six completion axes separate.
- Resolve routine implementation choices from documented defaults; do not fabricate human approvals.

## Constraints

- Preserve every existing and concurrent user change in the primary worktree.
- Make subsequent contract changes only in the isolated `codex/selemene-contract-convergence` worktree branch.
- Preserve ISC-1 through ISC-195 as immutable historical evidence and continue numbering from ISC-196.
- Distinguish 17 public mirrors from 19 supported runtime IDs everywhere a count is used.
- Use current registry, manifests, tests, and fresh probes as primary evidence.
- Mark unknown external deployment state as unknown rather than inferring it from configuration files.
- Keep fallbacks, mocks, placeholders, optional services, and database-conditional registration explicit.
- Do not assign or perform external-repository work.
- Treat a required check that does not execute as failure; intentional opt-outs require explicit configuration and visible status.
- Bind deploy eligibility to the exact revision that passed repository-owned validation.

### Continuation — 2026-09-05

- Root acceptance ledger is ISA.md; GSD files are executable views of the existing wave scope.
- Preserve the untouched root checkout, stash and existing worktrees. Source changes live in the new Superset worktree.
- Use Railway project 11eedde4-41e6-4f51-b86b-cf77111cf592 and production environment 702b945e-2c66-4d5a-bae1-4c67ea14c3bb explicitly; Cloudflare profile 9d9d must address account 9d9d23b27f32e70ae3afb6a1aa2c0f10.
- No broad engine expansion before remaining registry/build gates.
- Merges that trigger production, deployments, schema/data changes, new DNS/security access and paid generation require their concrete critical gate.

## Goal

Establish a canonical v1 contract authority that every maintained language surface can verify without a flag-day runtime rewrite. Done means six versioned schemas and representative fixtures are valid, repository gates reject drift, Rust/OpenAPI/TypeScript surfaces prove additive parity, existing envelope behavior remains compatible, and the complete repository gate passes from the isolated contract worktree.

### Continuation — 2026-09-05

Complete the existing dependency-ordered Selemene program through the GSD phase flow, preserving its GitHub work and acceptance history. Start by recovering authority, current dependency and infrastructure truth, and CodeGraph navigation; progress through remaining gates and engine/runtime/distribution work until all original wave exits are verified or a concrete critical human decision is reached. No recovery, healthy endpoint or passing local test substitutes for the remaining program.

## Criteria

### Recovery and preservation

- [x] ISC-1: The primary Selemene worktree is on branch `main` (probe: `git branch --show-current`).
- [x] ISC-2: The primary worktree has no unresolved merge state (probe: `git status --short --branch`).
- [x] ISC-3: Main ancestry contains the raaga clip merge commit (probe: `git log --oneline`).
- [x] ISC-4: Main ancestry contains the face CV merge commit (probe: `git log --oneline`).
- [x] ISC-5: Main ancestry contains the P4 SDK merge commit (probe: `git log --oneline`).
- [x] ISC-6: Sankalpa ancestry contains the combined T-115/T-120 commit (probe: `git -C ../sankalpa log -1`).
- [x] ISC-7: Sankalpa primary worktree is clean (probe: `git -C ../sankalpa status --short`).
- [x] ISC-8: The P4 API, bridge, and health worktrees all exist (probe: `git worktree list`).
- [x] ISC-9: The existing T-094 status edit remains present until health verification completes (probe: `git diff -- EXECUTION-STATUS.md`).
- [x] ISC-10: No pre-existing uncommitted P4 worktree change is discarded (probe: before/after `git diff --stat` comparison).

### P4 API endpoints

- [x] ISC-11: Biofield API input accepts the top-level FROZEN `image_data` field (probe: targeted integration test).
- [x] ISC-11.1: Legacy biofield payloads using `options` remain accepted (probe: regression test).
- [x] ISC-11.2: A golden request fixture serializes identically to the merged SDK contract (probe: JSON equality test).
- [x] ISC-12: Biofield API input accepts explicit FROZEN consent (probe: targeted integration test).
- [x] ISC-13: Face-reading API input accepts the top-level FROZEN `image_data` field (probe: targeted integration test).
- [x] ISC-14: Face-reading API input accepts explicit FROZEN consent (probe: targeted integration test).
- [x] ISC-14.1: Multipart PNG/JPEG input reaches face CV as lossless base64 bytes (probe: byte-equality test).
- [x] ISC-15: Raaga API input accepts the top-level FROZEN `audio_ref` field (probe: targeted integration test).
- [x] ISC-16: Raaga API output preserves top-level `generated_audio` (probe: targeted integration test).
- [x] ISC-17: Sigil-forge API input accepts the generation request fields (probe: targeted integration test).
- [x] ISC-18: Sigil-forge API output preserves top-level `generated_image` (probe: targeted integration test).
- [x] ISC-18.1: Golden generated-media responses match the merged SDK contract exactly (probe: JSON snapshot equality).
- [x] ISC-19: Unknown engine identifiers remain rejected rather than defaulted (probe: existing/targeted regression test).

### P4 bridge

- [x] ISC-20: The split Biofield boundary remains explicit: `biofield.calculate` uses the native Rust API engine, live `biofield.analyze` uses the Python `/analyze` sidecar, and the persisted-reading `biofield-capture` lookup is not repurposed (probe: SDK plus runtime-boundary tests).
- [x] ISC-21: The orchestrator/API registers face-reading at its native/Python boundary (probe: registration test).
- [x] ISC-22: The TS bridge registers raaga at its actual JSON engine route (probe: bridge unit test).
- [x] ISC-23: The TS bridge registers sigil-forge at its actual JSON engine route (probe: bridge unit test).
- [x] ISC-23.1: The TS bridge focus set rejects native/Python engine identifiers (probe: negative registration test).
- [x] ISC-24: Bridge serialization preserves image media fields (probe: bridge unit test).
- [x] ISC-25: Bridge serialization preserves audio media fields (probe: bridge unit test).
- [x] ISC-26: Bridge serialization preserves consent scopes (probe: bridge unit test).
- [x] ISC-27: Bridge cache keys vary when consent/media inputs vary (probe: bridge unit test).

### P4 health

- [x] ISC-28: `GET /health/dependencies` returns HTTP 200 with the detailed JSON report (probe: API handler test).
- [x] ISC-29: Health reports exactly the four focus engines (probe: JSON assertion).
- [x] ISC-30: Health reports biofield and face Python sidecars by stable ID with sanitized reasons and no URLs (probe: exact JSON assertion).
- [x] ISC-31: Health reports only configured NVIDIA, nano-banana, and Kimi providers by stable ID; empty/unset tokens are omitted (probe: JSON assertion).
- [x] ISC-32: Missing sidecars degrade or mark unavailable without panicking (probe: unreachable-sidecar test).
- [x] ISC-33: Only configured providers with non-empty tokens affect readiness, without network generation (probe: code inspection plus test double).
- [x] ISC-34: Health output excludes provider tokens, credential-bearing URLs, and raw internal errors (probe: response-body assertion).
- [x] ISC-34.1: Dependency-health JSON contains only the documented allowlisted fields (probe: exact-key assertion).
- [x] ISC-35: Existing `/ready` behavior remains compatible (probe: existing API tests).
- [x] ISC-35.1: Existing `/health` and `/health/live` preserve exact HTTP status and legacy body fields (probe: regression test).
- [x] ISC-36: Existing `/health/live` behavior remains compatible (probe: existing API tests).

### Integration and evidence

- [x] ISC-37: `cargo test -p noesis-bridge` exits zero on integrated main (probe: command).
- [x] ISC-38: Targeted P4 API tests exit zero on integrated main (probe: command).
- [x] ISC-39: `cargo test -p noesis-api --lib` exits zero on integrated main (probe: command).
- [x] ISC-40: Relevant TS engine tests exit zero or only retain documented pre-existing flakes (probe: command).
- [x] ISC-41: `EXECUTION-STATUS.md` describes only test-verified P4 results (probe: diff review).
- [x] ISC-42: Anti: no paid provider call, secret output, force operation, stale bulk merge, or unrelated remote sync occurs (probe: diff/command audit).

### P5 SDK distribution

- [x] ISC-43: The engine SDK package exports executable JavaScript from `dist` (probe: package-manifest assertion).
- [x] ISC-44: The SDK build emits JavaScript entrypoints (probe: file existence assertion).
- [x] ISC-45: The SDK build emits declaration entrypoints (probe: file existence assertion).
- [x] ISC-46: The packed SDK artifact excludes raw `src` implementation files (probe: `npm pack --dry-run --json`).
- [x] ISC-47: A temporary standalone TypeScript project imports the packed SDK (probe: fixture typecheck).
- [x] ISC-48: SDK default headers reach authenticated requests unchanged (probe: mocked-fetch assertion).
- [x] ISC-49: The SDK creates a typed Biofield session at the canonical route (probe: mocked-fetch assertion).
- [x] ISC-50: The SDK uploads Biofield media under the accepted multipart `image` field (probe: FormData assertion).

### P5 trusted desktop gateway

- [x] ISC-51: The Electron gateway accepts only the four focus engine IDs (probe: allowlist unit test).
- [x] ISC-52: The Electron main process owns backend-origin resolution (probe: source assertion).
- [x] ISC-53: Renderer bundles contain no API token environment variable access (probe: production-bundle text scan).
- [x] ISC-54: Preload exposes only typed engine, Biofield, and reading operations (probe: API-shape unit test).
- [x] ISC-55: A disallowed engine ID causes zero network calls (probe: mocked-fetch assertion).
- [x] ISC-56: Revoked or missing consent causes zero IPC calls (probe: renderer adapter unit test).
- [x] ISC-57: HTTP 401 and 403 are classified as fail-closed auth errors (probe: gateway error test).
- [x] ISC-58: HTTP 422 is classified as a fail-closed validation or consent error (probe: gateway error test).
- [x] ISC-59: Network, timeout, 502, 503, and 504 errors alone are fallback-eligible (probe: table-driven classifier test).
- [x] ISC-60: Focus-engine renderer modules contain no direct `fetch()` calls (probe: source scan).
- [x] ISC-61: Face media consent serializes to scope `face-image` (probe: serializer unit test).
- [x] ISC-62: Sigil generation consent serializes to scope `sigil-gen` (probe: serializer unit test).
- [x] ISC-63: Raaga clip requests set `request_clip: true` (probe: request-mapper unit test).

### P5 real Biofield capture

- [x] ISC-64: Local-only Biofield capture causes zero IPC calls (probe: transport mock assertion).
- [x] ISC-65: Remote Biofield submission creates a session before uploading media (probe: ordered-call assertion).
- [x] ISC-66: Camera capture no longer installs fixed fabricated metrics (probe: source and mapper test).
- [x] ISC-67: A remote Biofield result contains all eleven canonical metrics (probe: response fixture assertion).
- [x] ISC-68: A remote Biofield result preserves quality assessment (probe: response fixture assertion).
- [x] ISC-69: Biofield results visibly distinguish local from remote provenance (probe: view-model unit test).
- [x] ISC-70: Successful Biofield persistence exposes stable session and reading identifiers (probe: response adapter test).

### P5 Noesis result handoff

- [x] ISC-71: Fixture adapters normalize Biofield, Face, Raaga, and Sigil outputs (probe: parameterized unit test).
- [x] ISC-72: Engine result artifacts strip inline base64 media before persistence (probe: redaction unit test).
- [x] ISC-73: Engine result artifacts enforce a bounded serialized size (probe: oversized-fixture test).
- [x] ISC-74: `buildSectionsFromPayload` consumes normalized `engine_results` (probe: Noesis section test).
- [x] ISC-75: Engine witness prompts appear in the resulting witness section (probe: section-text assertion).
- [x] ISC-76: The handoff CTA opens the exact cached `#/noesis/depth/:id` route (probe: path assertion).
- [x] ISC-77: The non-predictive self-inquiry disclaimer survives engine-result integration (probe: prose assertion).
- [x] ISC-78: T-125 remains satisfied by the existing Command Center instead of duplicate UI (probe: route/action source assertion).

### Wave integration

- [x] ISC-79: Selemene SDK tests and typecheck exit zero on the wave branch (probe: package commands).
- [x] ISC-80: Sankalpa tests, typecheck, and production build exit zero on the wave branch (probe: package commands).
- [x] ISC-81: Cross-repository contract fixtures remain green after SDK packaging (probe: fixture validator command).
- [x] ISC-82: Anti: no credential enters renderer code, no raw media enters reading storage, and no remote sync or destructive git action occurs (probe: bundle, cache, diff, and command audit).

### P6 current practice state

- [x] ISC-83: Current practice day keys derive from an injected timestamp and timezone (probe: deterministic date test).
- [x] ISC-84: Current practice reading IDs are stable within one explicit local day (probe: ID equality test).
- [x] ISC-85: The first engine artifact creates a new current-practice reading (probe: empty-cache unit test).
- [x] ISC-86: Updating inquiry context preserves existing engine artifacts (probe: merge assertion).
- [x] ISC-87: Adding an engine artifact sanitizes it at session ingress (probe: adversarial fixture assertion).
- [x] ISC-88: A newer artifact for one engine replaces that engine's prior artifact (probe: unit test).
- [x] ISC-89: The store accepts exactly the four focus-engine artifact IDs (probe: parameterized unit test).
- [x] ISC-90: An unknown engine artifact is rejected before persistence (probe: negative unit test).
- [x] ISC-91: Adding one engine preserves every unrelated engine slot (probe: merge assertion).
- [x] ISC-92: Crossing midnight in the injected timezone creates a new reading ID (probe: rollover test).
- [x] ISC-93: Corrupt stored threshold JSON degrades to the default draft (probe: malformed-storage test).
- [x] ISC-94: A threshold localStorage write failure does not throw into the renderer (probe: throwing-storage test).
- [x] ISC-95: Oversized aggregate payloads fail before replacing the prior cached reading (probe: transactional size test).
- [x] ISC-96: Stored practice state contains no raw media bodies (probe: serialized-record scan).

### P6 reading assembly

- [x] ISC-97: Reading assembly includes every sanitized current-practice artifact (probe: four-fixture equality test).
- [x] ISC-98: Reading assembly preserves the practice subject (probe: payload assertion).
- [x] ISC-99: Reading assembly preserves the practice inquiry in witness context (probe: payload assertion).
- [x] ISC-100: Assembled readings use the deterministic current-practice identifier (probe: ID assertion).
- [x] ISC-101: Assembled readings use a Phase 6 workflow identifier (probe: workflow assertion).
- [x] ISC-102: Assembled payload creation enforces the existing route-size ceiling (probe: oversized-payload test).
- [x] ISC-103: Caching an assembled reading stores the exact sanitized payload (probe: cache equality test).
- [x] ISC-104: The assembled hash decodes to the exact cached payload (probe: route round-trip test).
- [x] ISC-105: Existing single-artifact witness construction remains compatible (probe: regression test).

### P6 daily-use interface

- [x] ISC-106: Handoff navigation occurs only after transactional aggregate caching succeeds (probe: navigation-target test).
- [x] ISC-107: A successful add reports the engine label (probe: rendered-text assertion).
- [x] ISC-108: A successful add reports local or backend provenance (probe: rendered-text assertion).
- [x] ISC-109: The handoff reports the current artifact count (probe: rendered-text assertion).
- [x] ISC-110: Opening the combined witness navigates to its exact assembled route (probe: navigation-target test).
- [x] ISC-111: Failed combined assembly exposes an accessible alert (probe: component test).
- [x] ISC-112: Threshold submission uses the deterministic current-practice reading ID (probe: integration test).
- [x] ISC-113: Threshold submission seeds the current practice inquiry (probe: integration test).
- [x] ISC-114: Threshold preview includes already accumulated engine artifacts (probe: integration test).
- [x] ISC-115: Command Center shows the current practice artifact count (probe: browser text assertion).
- [x] ISC-116: Command Center resume opens the exact assembled witness route (probe: browser route assertion).
- [x] ISC-117: Command Center clear removes the resumable practice (probe: browser interaction assertion).
- [x] ISC-118: Command Center shows an explicit empty-practice state after clear (probe: browser text assertion).

### P6 wave gate

- [x] ISC-119: Sankalpa documentation describes local multi-engine practice assembly (probe: documentation scan).
- [x] ISC-120: The full Sankalpa test suite exits zero (probe: `npm test`).
- [x] ISC-121: Sankalpa typecheck and production build exit zero (probe: package commands).
- [x] ISC-122: Anti: session, cache, history, hash, and production bundle scans reveal no raw media or credential aliases (probe: adversarial source and runtime scan).
- [x] ISC-123: Antecedent: Resume is offered only when the current practice contains inquiry context or an engine artifact (probe: empty/non-empty fixture test).
- [x] ISC-124: Current-practice read and write paths share one day-key function across DST boundaries (probe: timezone parity test).
- [x] ISC-125: A failed current-practice cache transaction produces no resumable ghost entry (probe: throwing-storage rollback test).
- [x] ISC-126: A cached current practice with the wrong assembly version fails closed (probe: stale-version fixture test).
- [x] ISC-127: Clearing today's practice removes matching last-reading and history references (probe: referential-consistency test).
- [x] ISC-128: Canonical timezone identity is part of the reading ID so equal day keys in different zones cannot collide (probe: cross-zone ID test).
- [x] ISC-129: Unicode subject and inquiry context round-trip through the exact hash before any cache commit (probe: UTF-8 route and precommit-failure tests).
- [x] ISC-130: Cached subject and witness context are rebuilt through a strict sanitizing allowlist before resume (probe: poisoned-cache fixture).
- [x] ISC-131: Last-reading and history commit through one atomic versioned cache envelope (probe: structural-corruption and throwing-storage tests).

### P6 Wave 2 performance boundary

- [x] ISC-132: The Noesis route dynamically imports `DepthGallery` instead of including it in the shell entry graph (probe: source assertion and bundle manifest).
- [x] ISC-133: Three.js is absent from the initial renderer entry chunk (probe: production bundle import graph).
- [x] ISC-134: The initial renderer entry JavaScript remains below 250 KiB raw and 100 KiB gzip (probe: build artifact sizes).
- [x] ISC-135: Three.js and depth rendering ship in a separately named asynchronous chunk (probe: production asset inventory).
- [x] ISC-136: A readable loading fallback is visible while the asynchronous depth surface resolves (probe: component/browser assertion).
- [x] ISC-137: The build reports the large optional visualization separately rather than treating it as initial application cost (probe: build log and chunk inventory).

### P6 Wave 2 reading recovery

- [x] ISC-138: Reading resolution reports whether the payload came from hash, cache, or remote (probe: loader fixture tests).
- [x] ISC-139: An invalid inline hash safely falls through to cache and remote resolution (probe: malformed-hash fixture).
- [x] ISC-140: The Noesis reading route exposes an explicit accessible loading state (probe: rendered-state assertion).
- [x] ISC-141: An absent reading produces a truthful empty/recovery surface and never a fabricated gallery payload (probe: not-found fixture and source audit).
- [x] ISC-142: Remote load failure produces a bounded safe error category without backend details (probe: gateway-error fixtures).
- [x] ISC-143: Retry repeats reading resolution and can transition error or missing state to loaded (probe: state-machine test).
- [x] ISC-144: A stale asynchronous response cannot overwrite a newer route request (probe: deferred-promise race test).
- [x] ISC-145: Reading recovery offers a direct Threshold action (probe: rendered action assertion).
- [x] ISC-146: Reading recovery offers the cached current practice when one meaningfully exists (probe: cache fixture assertion).
- [x] ISC-147: Reading failures are announced through an accessible alert without exposing payload contents (probe: accessibility and sensitive-text assertion).

### P6 Wave 2 runtime status

- [x] ISC-148: Account reads the real typed `getGatewayStatus` preload method (probe: component/source test).
- [x] ISC-149: Account exposes an explicit gateway-status loading state (probe: deferred-status fixture).
- [x] ISC-150: An unconfigured gateway is described as local-only operation (probe: status fixture).
- [x] ISC-151: A configured but unauthenticated gateway is described as authentication required (probe: status fixture).
- [x] ISC-152: An authenticated configured gateway is described as remote-ready (probe: status fixture).
- [x] ISC-153: Gateway status failure degrades to safe local guidance (probe: rejected-status fixture).
- [x] ISC-154: Account can retry gateway-status resolution after failure (probe: interaction test).
- [x] ISC-155: Anti: Account never renders gateway URLs, tokens, authorization headers, or secret-shaped configuration (probe: source and rendered-text scan).

### P6 Wave 2 accessible depth and gate

- [x] ISC-156: Reduced-motion preference prevents continuous depth animation while preserving the reading navigator (probe: media-query fixture and RAF assertion).
- [x] ISC-157: WebGL initialization failure falls back to readable section navigation and full text (probe: throwing-renderer fixture).
- [x] ISC-158: Arrow keys move backward and forward through reading sections (probe: keyboard interaction test).
- [x] ISC-159: Opening full text moves focus into the dialog (probe: focus assertion).
- [x] ISC-160: Escape closes full text and restores focus to its opener (probe: keyboard/focus assertion).
- [x] ISC-161: Texture callbacks and animation work are ignored after depth unmount (probe: delayed-loader cleanup test).
- [x] ISC-162: Full tests, typecheck, production build, and browser journeys pass at desktop, narrow, recovery, and reduced-motion states (probe: repository and browser gates).
- [x] ISC-163: Anti: Wave 2 introduces no raw media, credentials, unrestricted renderer fetch, or Electron-boundary regression (probe: source, bundle, and trust-boundary scans).

### Active Selemene-only planning authority

- [x] ISC-164: The active planning directory states that only `Selemene-engine` is in implementation scope (probe: exact scope text).
- [x] ISC-165: External products are named only as delivery-context consumers (probe: active roadmap assignment scan).
- [x] ISC-166: The planning authority defines declared, implemented, executable, integrated, deployed, and operational as separate states (probe: status table).
- [x] ISC-167: Document precedence ranks source and reproducible evidence above derived plans and historical ledgers (probe: authority-order inspection).
- [x] ISC-168: The repository boundary covers engines, contracts, orchestration, API, persistence, packages, infrastructure, operations, and assets (probe: topology and inventory tables).
- [x] ISC-169: The active README links the Selemene-only authority, ledger, and roadmap (probe: link check).
- [x] ISC-170: The historical project overview points unsupported completion claims to the active ledger (probe: banner inspection).
- [x] ISC-171: The historical integration ledger points completion labels to the active Selemene authority (probe: banner inspection).
- [x] ISC-172: The prior ecosystem map, ledger, and roadmap are explicitly marked context-only or superseded (probe: banner scan).

### Engine and platform evidence

- [x] ISC-173: The ledger contains all 19 IDs from `SUPPORTED_ENGINE_IDS` exactly once (probe: registry-to-table comparison).
- [x] ISC-174: The ledger distinguishes 12 unconditional native engines, one database-conditional native engine, and six TypeScript engines (probe: registration comparison).
- [x] ISC-175: Public count guidance distinguishes 17 mirrors from 19 runtime IDs (probe: planning-doc scan).
- [x] ISC-176: Each engine row identifies runtime path and all six completion axes (probe: table validation).
- [x] ISC-177: Biofield calculation, Biofield capture lookup, and Biofield CV analysis are not conflated (probe: ledger text inspection).
- [x] ISC-178: Face Reading distinguishes MediaPipe analysis from heuristic fallback provenance (probe: ledger text inspection).
- [x] ISC-179: Sigil Forge distinguishes configured providers, mocks, and placeholder output (probe: ledger text inspection).
- [x] ISC-180: Raaga distinguishes theory, local audio, and hosted clip capability (probe: ledger and roadmap inspection).
- [x] ISC-181: Platform rows cover API, orchestration, bridge, SDKs, CLI/tool server, TUI, admin, witness, and verification packages (probe: ledger row scan).
- [x] ISC-182: Infrastructure rows cover Postgres, migrations, Redis, images, Railway, Kubernetes, workers, monitoring, CI, release, and assets (probe: ledger row scan).
- [x] ISC-183: Fresh health evidence records endpoint, date, response status, engine count, workflow count, and authentication result (probe: probe receipt).
- [x] ISC-184: Anti: Fresh health evidence does not claim authenticated journeys, provider execution, sidecar deployment, or revision identity (probe: wording audit).

### Dependency roadmap and contradictions

- [x] ISC-185: The contradiction register corrects engine counts, bridge counts, migration counts, deployment topology, CI meaning, and health meaning (probe: row scan).
- [x] ISC-186: Wave 0 establishes machine-readable registry, evidence axes, deployment identity, and asset classification (probe: roadmap inspection).
- [x] ISC-187: Wave 1 makes builds, migrations, typecheck, required smokes, and release gates fail closed and reproducible (probe: roadmap inspection).
- [x] ISC-188: Wave 2 converges cross-language contracts, routing, discovery, and registry parity (probe: roadmap inspection).
- [x] ISC-189: Wave 3 closes semantic and provenance gaps in partial media and TypeScript engines (probe: roadmap inspection).
- [x] ISC-190: Wave 4 orders schema, state, auth, cache, health, retention, and recovery work (probe: roadmap inspection).
- [x] ISC-191: Wave 5 defines repository-owned package and tool distribution without assigning external implementation (probe: roadmap inspection).
- [x] ISC-192: Wave 6 aligns service deployment, observability, asset governance, and generated documentation (probe: roadmap inspection).
- [x] ISC-193: Every wave has falsifiable exit evidence (probe: roadmap heading and exit scan).
- [x] ISC-194: Anti: No active roadmap work item assigns implementation to Sankalpa, Urania, FalseEarth, Raycast, or another repository (probe: imperative-context scan).
- [x] ISC-195: Final link, table, diff, test, scope, and independent-review gates find no critical contradiction in the planning artifacts (probe: verification receipt).

### Active gate-foundation implementation

- [x] ISC-196: Production changes occur only in a clean isolated Selemene worktree (probe: branch and worktree status).
- [x] ISC-197: Primary-worktree user changes remain byte-for-byte preserved during implementation (probe: before/after status and diff inventory).
- [x] ISC-198: This slice changes no external repository, live service, cloud resource, database, or GitHub setting (probe: command and diff audit).
- [x] ISC-199: Gate semantics define success as execution against the exact candidate revision (probe: gate contract inspection).
- [x] ISC-200: The pre-change TypeScript typecheck failure is captured as RED evidence (probe: `bun run typecheck`).
- [x] ISC-201: TypeScript path aliases use syntax accepted by the installed compiler (probe: `bun run typecheck`).
- [x] ISC-202: TypeScript typechecking executes without suppression in local and CI gates (probe: command and workflow inspection).
- [x] ISC-203: All existing TypeScript engine tests remain green after gate repair (probe: `bun test`).
- [x] ISC-204: Migration integrity behavior is covered by executable temporary-fixture tests (probe: focused test command).
- [x] ISC-205: Historical migration numbering exceptions are explicit, bounded, and checksummed (probe: baseline manifest inspection).
- [x] ISC-206: New duplicates, gaps, untracked files, and checksum mutation fail migration validation (probe: negative fixtures).
- [x] ISC-207: The current migration tree passes its documented integrity policy unchanged (probe: validator command).
- [x] ISC-208: Docker manifest caching covers every current Cargo workspace member and declared target (probe: manifest-parity validator).
- [x] ISC-209: Docker dependency prebuild contains no broad success-masking fallback (probe: Dockerfile validator).
- [x] ISC-210: The production Docker build path reaches a real fail-closed Cargo build step (probe: Docker validation or build receipt).
- [x] ISC-211: CI executes migration integrity without `|| true`, ignored exit codes, or equivalent masking (probe: workflow validator).
- [x] ISC-212: CI executes TypeScript typecheck as a required failing step (probe: workflow validator).
- [x] ISC-213: One repository-owned verification command composes the focused gate checks (probe: root command).
- [x] ISC-214: Deployment jobs cannot start when repository validation fails or is absent (probe: workflow dependency validation).
- [x] ISC-215: Deployment artifact identity derives from the exact validated triggering commit (probe: workflow expression validation).
- [x] ISC-216: Required API and TypeScript build artifacts share the same validated source revision (probe: workflow dependency inspection).
- [ ] ISC-217: Toolchain and action versions used by the gate are deterministic rather than floating (probe: workflow and manifest scan).
- [x] ISC-218: The unified repository gate exits zero from a clean isolated worktree (probe: fresh full command).
- [x] ISC-219: Changed-file formatting and whitespace checks pass (probe: format checks and `git diff --check`).
- [x] ISC-220: Independent correctness review finds no false-green or migration-history blocker (probe: adversarial review).
- [x] ISC-221: Anti: Existing migrations are not renumbered, deleted, or rewritten (probe: migration diff audit).
- [x] ISC-222: Anti: No deployment, provider generation, package publication, push, or merge occurs (probe: command audit).
- [x] ISC-223: Gate documentation states both covered guarantees and remaining Wave 1 gaps (probe: evidence wording audit).
- [x] ISC-224: ReReadCheck confirms the user-selected order remains gates, contracts, then engines (probe: final request comparison).

### Gate merge and contracts transition

- [x] ISC-225: The gate-foundation branch contains committed, whitespace-clean changes before integration (probe: commit and cached-diff inspection).
- [x] ISC-226: The gate-foundation commits are ancestors of local `main` (probe: `git merge-base --is-ancestor`).
- [x] ISC-227: Every pre-existing dirty primary-worktree file retains its exact SHA-256 digest after both merges (probe: before/after checksum comparison).
- [x] ISC-228: The merged primary checkout passes the complete repository gate under its real local environment (probe: `pnpm run gate`).
- [x] ISC-229: Ambient image-provider credentials cannot make gate tests perform provider calls (probe: sentinel-credential focused test).
- [x] ISC-230: The completed gate worktree is removed only after the merged gate passes (probe: command sequence and `git worktree list`).
- [x] ISC-231: The fully merged local gate branch is deleted without force (probe: `git branch -d` output).
- [x] ISC-232: The contracts worktree starts from the final verified merge revision (probe: branch HEAD equality).
- [x] ISC-233: The contracts worktree is clean and its full baseline gate exits zero (probe: status and `pnpm run gate`).
- [x] ISC-234: Anti: This transition changes no external repository, live service, database, cloud resource, remote branch, or engine semantics (probe: command and diff audit).

### Contract authority v1

- [x] ISC-235: Contract work occurs only on `codex/selemene-contract-convergence` in its isolated worktree (probe: branch, worktree, and diff paths).
- [x] ISC-236: The primary worktree's pre-existing user and planning changes remain untouched (probe: primary status and scoped diff audit).
- [x] ISC-237: Anti: No external repository, live service, database, cloud resource, remote branch, package publication, or engine calculation is changed (probe: command and diff audit).
- [x] ISC-238: A repository-root `contracts/v1` directory is the documented language-neutral authority (probe: path and README inspection).
- [x] ISC-239: The contract manifest declares one immutable `v1` contract identifier (probe: manifest assertion).
- [x] ISC-240: The manifest enumerates request, result, error, consent, provenance, and capability-discovery schemas (probe: exact-name assertion).
- [x] ISC-241: The request schema requires an explicit contract version (probe: schema validation test).
- [x] ISC-242: The request schema represents `consciousness_level` and free-form `parameters` without engine-specific semantics (probe: property assertion).
- [x] ISC-243: The request schema preserves legacy birth, time, location, precision, and options compatibility fields additively (probe: golden legacy fixture validation).
- [x] ISC-244: The request schema supports bounded image and audio references through explicit media fields (probe: schema and fixture validation).
- [x] ISC-245: The consent schema requires grant state, scope, and capture timestamp (probe: valid and invalid fixture tests).
- [x] ISC-246: Media-bearing requests reference the canonical consent definition rather than duplicating consent shapes (probe: `$ref` assertion).
- [x] ISC-247: The quality schema defines bounded score and optional diagnostics without provider-specific leakage (probe: valid and invalid fixture tests).
- [x] ISC-248: The result schema requires contract version, engine identifier, result payload, and consciousness level (probe: schema validation test).
- [x] ISC-249: The result schema preserves singular and plural witness-prompt compatibility (probe: golden result variants).
- [x] ISC-250: The result schema includes calculation timestamp and processing duration (probe: required-field assertion).
- [x] ISC-251: Generated image and audio fields remain optional top-level compatibility fields (probe: fixture variants).
- [x] ISC-252: The universal error schema preserves status, code, message, legacy error, trace, details, and contract version (probe: golden error validation).
- [x] ISC-253: Error fixtures contain no stack, secret, credential, token, or internal URL fields (probe: sensitive-key scan).
- [x] ISC-254: The provenance schema records runtime kind, implementation version, cache state, and fallback state (probe: schema validation test).
- [x] ISC-255: Provenance permits sanitized provider/backend identifiers but forbids credentials and raw endpoints (probe: allowlist assertion).
- [x] ISC-256: Capability discovery exposes declared, available, degraded, and unavailable states (probe: enum assertion).
- [x] ISC-257: Capability discovery distinguishes native, TypeScript, Python, database-conditional, and composed runtimes (probe: enum assertion).
- [x] ISC-258: Capability discovery declares explicit dependencies and contract version (probe: golden capability validation).
- [x] ISC-259: A canonical valid request fixture validates against the request schema (probe: repository contract validator).
- [x] ISC-260: A canonical valid result fixture validates against the result schema (probe: repository contract validator).
- [x] ISC-261: A canonical error fixture validates against the error schema (probe: repository contract validator).
- [x] ISC-262: A canonical capability fixture validates against the capability schema (probe: repository contract validator).
- [x] ISC-263: The contract validator rejects malformed schemas, missing manifest entries, and invalid fixtures (probe: focused negative tests).
- [x] ISC-264: Contract validation is composed into the root repository gate (probe: package script and gate execution).
- [x] ISC-265: Rust core contract types deserialize canonical request, result, error, provenance, consent, and capability fixtures (probe: focused Cargo test).
- [x] ISC-266: Rust contract types serialize canonical fixtures without unversioned field drift (probe: round-trip equality test).
- [x] ISC-267: OpenAPI schemas retain existing route fields while matching canonical v1 required compatibility fields (probe: OpenAPI parity test).
- [x] ISC-268: The engine SDK exports the v1 contract identifier and additive canonical contract types (probe: package typecheck and fixture test).
- [x] ISC-269: TypeScript engine types accept the same canonical request and result fixtures (probe: compile-time and runtime fixture test).
- [x] ISC-270: The general TypeScript SDK cannot silently diverge from canonical engine IDs and envelope fields (probe: parity test).
- [x] ISC-271: Anti: Existing endpoint paths, authentication headers, HTTP status behavior, and calculation routing remain unchanged (probe: regression and diff audit).
- [x] ISC-272: Anti: Existing engine algorithms, providers, fallbacks, and generated-media execution paths remain unchanged (probe: changed-file and source audit).
- [x] ISC-273: Focused language-neutral contract tests pass from a clean dependency state (probe: validator test command).
- [x] ISC-274: Focused Rust contract and OpenAPI tests pass (probe: Cargo commands).
- [x] ISC-275: Focused TypeScript SDK and engine parity tests pass (probe: Bun/npm commands).
- [x] ISC-276: The full root `pnpm run gate` exits zero after contract convergence (probe: fresh full command).
- [x] ISC-277: Independent review finds no P0/P1 contract drift, compatibility break, false-green path, or engine-scope leak (probe: read-only review).
- [x] ISC-278: ReReadCheck confirms this iteration proceeds only through contracts after the merged gate foundation (probe: final request comparison).

### Continuation — 2026-09-05

- [x] ISC-279: Recovered ISA retains all 170 criterion IDs from the starting checkout.
- [x] ISC-280: Canonical README, ROADMAP and CAPABILITY-LEDGER are restored with original baseline content.
- [x] ISC-281: Fresh engine issue index contains exactly 570 unique issues in 19 groups of 30 slots.
- [x] ISC-282: Master and seven wave control issues retain their original GitHub IDs and wave scope.
- [x] ISC-283: GSD parser recognizes all seven continuation phases and their original-wave mapping.
- [x] ISC-284: Phase 1 context records the recommended discussion choices and critical gate boundaries.
- [x] ISC-285: Phase 1 research cites current source, captured service evidence and primary provider documentation.
- [x] ISC-286: Independent plan-review concerns are explicitly resolved or retained as named verification limits.
- [x] ISC-287: Railway service inventory records seven service IDs and active deployment digests.
- [x] ISC-288: Cloudflare account 9d9d ownership of tryambakam.space is verified by scoped API readback.
- [x] ISC-289: Three live Selemene Workers have source-to-binding metadata recorded.
- [x] ISC-290: DNS permission denial is recorded as unknown DNS state rather than absent records.
- [x] ISC-291: Undeployed pattern-memory declarations are explicitly distinguished from live infrastructure.
- [x] ISC-292: Public API curl liveness records HTTP 200 with 19 loaded engines.
- [x] ISC-293: TS production capability-route HTTP 404 is recorded as an undeployed slice.
- [x] ISC-294: CodeGraph index reports a completed current-source scan.
- [x] ISC-295: CodeGraph sync completes successfully after indexing.
- [x] ISC-296: CodeGraph context resolves the actual capability handler and bridge registry symbols.
- [x] ISC-297: Rust dependency audit records zero known vulnerabilities and the yanked dependency disposition.
- [x] ISC-298: Production Node dependency audit has no unresolved high or critical vulnerabilities.
- [x] ISC-299: Python dependency evidence distinguishes local environment findings from unproven image resolution.
- [x] ISC-300: TS capability formatter repair passes lint, typecheck and the complete TS test suite.
- [x] ISC-301: GSD Phase 2 has dependency-ordered plans for the remaining gate and source-config work.
- [x] ISC-302: Capability documentation no longer claims native or conditional engines are covered by six TS rows.
- [x] ISC-303: GitHub control planning is reconciled against the new evidence without closing unproven work.
- [x] ISC-304: Current source changes are packaged as scoped reviewable commits with exact verification evidence.
- [ ] ISC-305: Required remote CI is green on the reviewed source revision before merge approval.
- [x] ISC-306: Immutable action pins and complete Python/admin release-gate coverage are verified.
- [ ] ISC-307: Production release can identify source and schema revisions for the approved image digests.
- [ ] ISC-308: Original Wave 0 through Wave 6 exit criteria are all verified before overall goal completion.
- [x] ISC-309: Anti: starting checkout, preserved planning stash and existing user worktrees are not overwritten or discarded.
- [x] ISC-310: Anti: recovery artifacts contain no credential values, personal runtime payloads or secret variable values.
- [x] ISC-311: Accepted Hands worker output has substantive artifacts and actual provider/model attribution.
- [ ] ISC-312: Every originally partial engine row has semantic and integration evidence or an explicitly approved reduced scope.
- [x] ISC-313: Critical DNS/security/production decisions remain explicit until the exact requested operation is authorized.
- [x] ISC-314: Current phase evidence distinguishes local source, remote CI, deployed artifact and operational verification.


- [x] ISC-315: Complete production and development Node audit reports zero known vulnerabilities for the candidate lockfile.
- [x] ISC-316: Agent merge lane rejects missing, pending, stale and untrusted CI results in executable mocked workflow tests.
- [x] ISC-317: Authenticated browser verifies the Selemene API, admin and witness DNS records in the 9d9d zone.
- [x] ISC-318: Authenticated admin observations distinguish nineteen registry rows from six TypeScript health results.

## Test Strategy

| isc | type | check | threshold | tool |
|---|---|---|---|---|
| ISC-1..10 | state | branch, ancestry, dirtiness, preservation | exact expected state | `git status`, `git log`, `git diff --stat`, `git worktree list` |
| ISC-11..19 | API contract | four focus engines cross HTTP/auth/orchestrator with FROZEN media and legacy compatibility | all targeted assertions pass | `cargo test -p noesis-api --test p4_media_engines_tests` |
| ISC-20..27 | bridge contract | registration, serialization, cache isolation | all bridge assertions pass | `cargo test -p noesis-bridge` |
| ISC-28..36 | health contract | separate detailed response, degradation, sanitization, liveness compatibility | all handler assertions pass | `cargo test -p noesis-api --lib` plus focused handler test |
| ISC-37..40 | regression | package-level Rust and TS suites | zero new failures | Cargo and Bun test commands |
| ISC-41 | documentation | status statements map to command evidence | no unsupported completion claims | `git diff` plus evidence table |
| ISC-42 | anti | review commands and diffs for prohibited actions | zero prohibited actions | shell history in this run plus `git diff` |
| ISC-43..50 | SDK artifact | build, pack, standalone install, auth header, Biofield multipart | all artifact and mocked-fetch assertions pass | Bun, npm pack, temporary TypeScript fixture |
| ISC-51..63 | desktop trust boundary | IPC allowlist, auth ownership, error classes, canonical consent/request mapping | all unit and bundle scans pass | Vitest, TypeScript, Vite bundle scan |
| ISC-64..70 | Biofield journey | local zero-call, ordered remote session/upload, real response mapping, provenance | all fixtures and transport assertions pass | Vitest plus optional live stack probe |
| ISC-71..78 | Noesis handoff | normalize, redact, bound, sectionize, preserve witness/disclaimer, open cache route | all pure-function and source assertions pass | Vitest |
| ISC-79..82 | wave gate | both repository gates, contract fixtures, anti audit | zero new failures or prohibited state | Bun, npm, git, source scans |
| ISC-83..96, ISC-124..131 | practice identity | timezone/day identity, exact IDs, versioning, replacement, rollover, corruption, atomic write failure, Unicode, context sanitization, clear consistency, size | all date, merge, route, and storage fixtures pass | Vitest with injected clocks and MemoryStorage variants |
| ISC-97..105 | reading assembly | subject, inquiry, four artifacts, stable IDs, cache/hash equality, size, compatibility | exact payload assertions pass | Vitest |
| ISC-106..118, ISC-123 | daily-use UI | transactional navigation, provenance/count status, combined open, threshold seed, command resume/clear, meaningful resume antecedent | helper, component, and browser probes pass | Vitest plus browser automation |
| ISC-119..122 | wave gate | docs, full suite, type/build, persistence and bundle anti-scans | zero failures or sensitive fields | npm, rg, browser automation |
| ISC-132..137 | performance | lazy route boundary, initial-entry size, async depth inventory, loading fallback | initial entry under 250 KiB raw/100 KiB gzip and Three.js excluded | Vite build, asset graph, Vitest, browser automation |
| ISC-138..147 | recovery | source-aware resolution, malformed hash fallback, loading/empty/error/retry/race and recovery actions | exact state transitions and zero fabricated evidence | Vitest plus browser automation |
| ISC-148..155 | runtime status | live typed status, local/auth-required/remote-ready/error/retry and anti-secret rendering | exact fixtures and zero sensitive configuration | Vitest, rg, browser automation |
| ISC-156..163 | accessible depth gate | reduced motion, WebGL fallback, keyboard/dialog focus, cleanup, full gates and trust anti-scan | all focused and repository gates pass | Vitest, npm, rg, browser automation |
| ISC-164..195 | active Selemene planning | scope, registry parity, completion axes, internal inventory, probe truth, contradictions, roadmap sequencing, anti-assignment | exact links/tables/counts; independent audit has no critical planning contradiction | `rg`, registry comparison, link checker, tests, live probes, Cato review |
| ISC-196..203 | isolation and TypeScript gates | worktree preservation, compiler configuration, unmasked typecheck, regression tests | exact branch isolation; typecheck and 88-test suite exit zero | Git, Bun, TypeScript, workflow inspection |
| ISC-204..207, ISC-221 | migration integrity | fixture behavior, historical baseline, duplicates, gaps, tracking, checksums, preservation | every negative fixture fails; current unchanged tree passes | repository validator and temporary fixtures |
| ISC-208..210 | production build gate | Cargo member/target parity, cache prebuild failure semantics, production build path | no missing manifest or broad success mask; real build step is reachable | Dockerfile validator and available container/build tooling |
| ISC-211..218 | CI and deploy eligibility | composed gate, failure propagation, exact revision, artifact parity, deterministic tools | validator passes and unified clean-worktree gate exits zero | repository scripts, workflow parser, GitHub workflow inspection |
| ISC-219..224 | final gate evidence | formatting, independent audit, anti-mutation, residual-gap documentation, request reread | zero diff errors or critical false-green findings; precise bounded claim | formatters, `git diff --check`, adversarial review, ReReadCheck |
| ISC-225..234 | merge and transition | ancestry, checksum preservation, ambient-credential isolation, cleanup ordering, contract baseline | exact ancestry; identical hashes; both merged and contract gates exit zero | Git, SHA-256, pnpm, Bun |
| ISC-235..278 | contract authority v1 | schema and fixture validity, negative drift, Rust/OpenAPI/TypeScript parity, compatibility, anti-scope, full gate | all focused and full gates pass; independent review has no P0/P1 finding | JSON Schema, Python, Cargo, Bun, TypeScript, Git |

### Continuation — 2026-09-05

| ISC | Type | Check | Threshold | Tool |
|---|---|---|---|---|
| 279–286 | artifact/structure | recovered history, corpus, GSD parsers, discussion/research/review | exact IDs, valid phases, explicit limits | git, Python, gsd-sdk |
| 287–293 | live read | explicit Railway/CF metadata and HTTP probes | scoped success or recorded denied/absent result | provider CLI/API, curl |
| 294–296 | graph | status, sync and real symbol context | indexed, up to date, source matches | codegraph MCP/CLI |
| 297–301 | dependency/code | audit feeds and TS lint/typecheck/test | exact result; no unresolved high/critical Node advisory | cargo, pnpm, pip-audit, bun |
| 302–306 | plan/release | GSD execution plans, corrected scope, issue readback, reviewed commits, remote CI | concrete source revision | gsd-sdk, git, gh |
| 307–308 | release | action pin audit, Python/admin gates, deployed source/schema receipt | full required checks and identified artifact | CI/provider readback |
| 309,312 | program | original wave exits and engine evidence rows | all named exits satisfied | per-phase named probes |
| 310–311,313–314 | anti/evidence | preserved state, artifact scan, permission decisions, evidence separation | no violation | git, bounded scans, review |

| ISC-315–318 | audit/browser/executable mock | Complete audit, merge rejection fixtures, targeted DNS/admin readback | Each explicit probe passes | pnpm/pytest/IAB |

## Features

| name | description | satisfies | depends_on | parallelizable |
|---|---|---|---|---|
| StateRecovery | Reconcile transcript claims against branches, commits, worktrees, and Sankalpa | ISC-1..10 | none | true |
| ApiMediaExposure | Complete four-engine P4 media request/response coverage | ISC-11..19 | StateRecovery | true |
| BridgeVerification | Integrate minimal bridge registration and lossless forwarding | ISC-20..27 | StateRecovery | true |
| AggregatedHealth | Integrate engine, sidecar, and provider readiness reporting | ISC-28..36 | StateRecovery | true |
| RegressionGate | Run focused and package-level suites on the integrated result | ISC-37..40 | ApiMediaExposure, BridgeVerification, AggregatedHealth | false |
| StatusReconciliation | Rewrite status claims to match verified evidence | ISC-41..42 | RegressionGate | false |
| PackagedEngineSdk | Build a standalone SDK artifact with authenticated Biofield session support | ISC-43..50, ISC-79, ISC-81 | ApiMediaExposure | true |
| TrustedDesktopGateway | Keep origin, credentials, consent mapping, and error taxonomy outside the renderer | ISC-51..63, ISC-82 | PackagedEngineSdk | false |
| RealBiofieldCapture | Replace fabricated camera analysis with ordered consented session and capture transport | ISC-64..70 | TrustedDesktopGateway | true |
| NoesisResultHandoff | Normalize, redact, cache, and narrate four engine result artifacts | ISC-71..78 | TrustedDesktopGateway | true |
| P5WaveGate | Verify both repositories and reconcile status evidence | ISC-79..82 | RealBiofieldCapture, NoesisResultHandoff | false |
| CurrentPracticeIdentity | Reuse one versioned, transactionally cached reading per local day instead of duplicating session state | ISC-83..96, ISC-124..131 | NoesisResultHandoff | true |
| PracticeReadingAssembly | Build, cache, and route the exact combined Noesis payload | ISC-97..105 | CurrentPracticeIdentity | true |
| DailyPracticeControls | Add, resume, clear, and seed the current practice across engine, threshold, and command surfaces | ISC-106..118, ISC-123 | CurrentPracticeIdentity, PracticeReadingAssembly | false |
| P6WaveGate | Verify storage, routing, user journey, documentation, sensitive-data boundaries, Unicode, and atomic persistence | ISC-119..122, ISC-128..131 | DailyPracticeControls | false |
| LazyDepthBoundary | Remove optional Three.js depth rendering from the initial shell graph and expose a readable loading boundary | ISC-132..137 | NoesisResultHandoff | true |
| ReadingRecoveryState | Preserve load source and bounded failures across explicit loading, empty, retry, race, and recovery actions | ISC-138..147 | CurrentPracticeIdentity, LazyDepthBoundary | true |
| GatewayStatusSurface | Render the existing narrow gateway readiness contract without exposing privileged configuration | ISC-148..155 | TrustedDesktopGateway | true |
| AccessibleDepthFallback | Preserve witness navigation under reduced motion, WebGL failure, keyboard use, dialog focus, and unmount races | ISC-156..161 | LazyDepthBoundary | true |
| P6Wave2Gate | Verify performance, recovery, runtime status, accessibility, trust boundaries, and responsive browser journeys | ISC-162..163 | ReadingRecoveryState, GatewayStatusSurface, AccessibleDepthFallback | false |
| SelemenePlanningBoundary | Define repository ownership, evidence states, precedence, and external delivery context | ISC-164..172 | none | true |
| SelemeneCapabilityLedger | Audit all engines, contracts, packages, services, infrastructure, operations, and assets by evidence axis | ISC-173..185 | SelemenePlanningBoundary | true |
| SelemeneInternalRoadmap | Order remediation from authority and safe delivery through contracts, runtime truth, state, distribution, and operations | ISC-186..194 | SelemeneCapabilityLedger | false |
| SelemenePlanningGate | Verify links, tables, counts, tests, scope, and independent review | ISC-195 | SelemeneInternalRoadmap | false |
| GateWorktreeBoundary | Isolate gate implementation while preserving every existing user-owned change | ISC-196..198, ISC-221..222 | SelemenePlanningGate | false |
| TypeScriptValidationGate | Repair compiler configuration and make TypeScript validation mandatory | ISC-199..203 | GateWorktreeBoundary | true |
| MigrationIntegrityGate | Preserve deployed history while rejecting new structural or checksum drift | ISC-204..207, ISC-221 | GateWorktreeBoundary | true |
| ProductionBuildGate | Make Docker manifest caching complete and Cargo prebuild fail closed | ISC-208..210 | GateWorktreeBoundary | true |
| UnifiedReleaseGate | Compose validation and bind deployment eligibility to one exact revision | ISC-211..218 | TypeScriptValidationGate, MigrationIntegrityGate, ProductionBuildGate | false |
| GateFoundationEvidence | Verify the bounded slice and document remaining Wave 1 work honestly | ISC-219..224 | UnifiedReleaseGate | false |
| GateMergeAndContractTransition | Integrate verified gates, preserve primary edits, and establish the isolated contracts boundary | ISC-225..234 | GateFoundationEvidence | false |
| ContractAuthorityV1 | Establish canonical schemas, fixtures, validation, and additive cross-language parity before engine work | ISC-235..278 | GateMergeAndContractTransition | false |

### Recovered architecture (historical scope)

<!-- arch-assets:start -->

_Auto-maintained by `ArchitectureAssetsSync.hook.ts` on release events._
_Last refreshed: 2026-08-25T17:34:40.307Z_

| Asset | Status | How it's generated |
|---|---|---|
| [`docs/architecture/SERVICES.md`](docs/architecture/SERVICES.md) | ✓ current | auto (file scan) |
| [`docs/architecture/DEPENDENCY-GRAPH.md`](docs/architecture/DEPENDENCY-GRAPH.md) | ✓ current | auto (file scan) |
| [`docs/architecture/architecture.html`](docs/architecture/architecture.html) | ✗ not yet generated | manual (LLM skill) |
| [`docs/architecture/notebooklm-prompt.md`](docs/architecture/notebooklm-prompt.md) | ✗ not yet generated | manual (LLM skill) |

**To refresh LLM-generated assets:** invoke `/refresh-architecture` in any Claude Code session.

<!-- arch-assets:end -->

### Continuation — 2026-09-05

| Feature | Satisfies | Depends on | Parallelizable |
|---|---|---|---|
| Authority/GSD recovery | ISC-279–286, ISC-301–304 | preserved history | true |
| Infrastructure and CodeGraph | ISC-287–296 | provider read access | true |
| Dependency/CI/config repairs | ISC-297–300, ISC-305–306 | recovered gate scope | true, disjoint files/worktrees |
| Contract/native capability closure | existing Wave 2 criteria | required gates | false |
| Engine/state/distribution/ops continuation | ISC-307–308, ISC-312 | original wave prerequisites | per phase plan |
| Preservation, privacy and evidence | ISC-309–311, ISC-313–314 | every phase | continuous |

## Decisions

- 2026-08-26 13:31: Contract authority v1 is complete on the isolated `codex/selemene-contract-convergence` branch. Six schemas, five fixtures, fail-closed validation, and additive Rust/OpenAPI/TypeScript parity are the completed contract layer; engine semantic repair remains separately authorized next work.
- 2026-08-26 13:31: Provenance remains optional at compatibility adapters because current runtime paths do not yet expose one truthful unified producer. The contract defines the sanitized shape but does not fabricate provider, backend, cache, or fallback facts.
- 2026-08-26 13:31: Versioned `v1` API requests fail closed on version, canonical parameters, bounds, unknown fields, media, consent, and quality; unversioned legacy requests remain additive compatibility inputs rather than being silently reclassified as v1.
- 2026-08-26 13:31: Independent review issued five P1 findings across two BLOCK cycles. Each became an executable schema, validator, parity, root-gate, or HTTP-boundary protection; the exact final candidate then received GO with no P0/P1 findings.
- 2026-08-26 13:31: The required Forge rail produced the initial RED contract test before its bounded execution expired; native TDD completed and verified the implementation. Both required Advisor attempts failed on expired OAuth, so no Advisor verdict is claimed.
- 2026-08-26 12:36: The approved contract seam is language-neutral JSON Schema plus golden fixtures under `contracts/v1`; Rust, OpenAPI, and maintained TypeScript packages consume or test that authority instead of defining competing canonical shapes.
- 2026-08-26 12:36: Existing Rust and TypeScript runtime DTOs remain compatibility adapters during this slice. Contract convergence is additive and does not authorize a flag-day endpoint, routing, provider, calculation, or persistence migration.
- 2026-08-26 12:36: The Observe combo did not resolve within the bounded wait, and the required Advisor checkpoint could not refresh its expired OAuth session. No external verdict is claimed; repository source, tests, the approved roadmap, Forge implementation, and independent read-only review remain the evidence path.
- 2026-08-26 12:36: Forge owns the bounded test-first implementation in the isolated contract worktree; the native orchestrator retains plan authority, diff review, independent verification, and final scope reconciliation.

- 2026-08-26 01:17: The user's explicit request selects local merge. Because local `main` already matched `origin/main`, integration used two non-fast-forward local merges without pulling or pushing; every pre-existing dirty path was disjoint and checksum-verified before and after.
- 2026-08-26 01:17: The merged-checkout gate exposed an ambient `NVIDIA_API_KEY` test leak that the clean worktree could not reveal. Provider tests now remove and restore credential variables and inject a mock for generation, making repository gates network-free under credential-bearing developer environments.
- 2026-08-26 01:17: Contract convergence begins on `codex/selemene-contract-convergence` from merge `01160e5`. The first slice is contract authority and cross-language parity; engine semantic repair remains Wave 3 and may not leak into this boundary.
- 2026-08-26 01:17: Delegation was intentionally not selected because merge, destination verification, cleanup, and successor-worktree creation mutate one shared Git topology and require strict serialization. Separate agents would only have duplicated overlap auditing and contract-inventory reading, both completed locally with exact hashes and source scans.
- 2026-08-26 00:20: The user-selected dependency order is binding: repository gates first, contract convergence second, engine completion third. This iteration implements only a bounded gate foundation and cannot close later layers by implication.
- 2026-08-26 00:20: First-principles deconstruction defines a trustworthy green gate as four irreducible facts: the intended check executed, failure propagated, the checked revision is identifiable, and the deployable artifact derives from that same revision.
- 2026-08-26 00:20: Systems Iceberg analysis identifies the structural generator of false-green status as fragmented commands and workflows whose outputs are not consumed by deployment. The highest feasible leverage is a repository-owned composed gate plus an explicit deploy dependency, not another status label.
- 2026-08-26 00:20: Root-cause fishbone prioritizes success masking in methods, floating or invalid tooling in machines, incomplete migration evidence in materials, and workflow-green status in measurement. Each vital cause receives an executable repository-local falsification probe.
- 2026-08-26 00:20: Iterative-depth passes add four safeguards: literal sequencing forbids premature contract/engine edits; failure analysis forbids skipped checks becoming green; temporal analysis preserves deployed migration history; operator analysis requires one reproducible command and bounded residual-gap wording.
- 2026-08-26 00:20: Existing duplicate or gapped migration numbering is treated as historical evidence requiring a checksummed baseline, not permission to rewrite history or silently accept future drift.
- 2026-08-26 01:02: The migration runner records exact filename and checksum rather than numeric version, preserving both historical `007` files while making `038+` incremental. Transactional files apply and journal atomically; `CREATE INDEX CONCURRENTLY` uses advisory locking and a fail-loud `applying` state because PostgreSQL forbids that command inside a transaction.
- 2026-08-26 01:02: A nonempty database without a populated repository journal is never guessed or replayed. Baseline adoption is an explicit validator-backed operator action; ordinary execution validates the immutable ledger before the first database call.
- 2026-08-26 01:02: Cato was requested for the final review but the role was unavailable. The mandated fail-open adversarial reviewer repeatedly BLOCKed synthetic-only assumptions, then returned PASS only after a real PostgreSQL 16 first-run and second-run replay closed the final defects.
- 2026-08-26 01:02: The Docker gate is complete as a truth mechanism but the production image is not green. Native arm64 exposes a pre-existing `libswisseph-sys 0.1.2` pointer-signedness compile blocker; amd64 emulation is inconclusive after GCC crashed under QEMU. This slice records and blocks on those facts rather than expanding into dependency repair.
- 2026-08-26 01:02: The post-deliverable Advisor retry failed because its OAuth session was expired and unrefreshable. No advisor verdict is claimed; executable gates, real PostgreSQL, actionlint, shellcheck, Docker receipts, and independent adversarial PASS form the completion evidence.

- 2026-08-25 23:45: Active planning is repository-bound to Selemene Engine. Sankalpa, Urania, FalseEarth, Raycast, and other products are acceptance and delivery context only; their repositories receive no work assignments in this plan.
- 2026-08-25 23:45: First-principles decomposition defines Selemene ownership as computation, contracts, routing, API, state, packages/tools, deployable services, operations, and assets. Product UX and downstream release ownership remain outside the boundary.
- 2026-08-25 23:45: Systems Iceberg analysis traced recurring completion drift from visible count/status conflicts to duplicated catalogues, fail-open gates, health/registration conflation, unversioned external deploy state, and the mental model that merged or tested implies operational.
- 2026-08-25 23:45: Root-cause fishbone grouped vital causes under authority, build/release method, multi-language material, incomplete measurement, externally held deployment state, and fragmented ownership. The high-leverage intervention is one generated capability manifest plus immutable deployment receipts and fail-closed gates.
- 2026-08-25 23:45: Iterative-depth experiential criterion: a maintainer must be able to open one directory and know what is real, where it runs, what evidence proves it, and which internal dependency is next. Meta conclusion: Selemene is planned as a platform product, not as a collection of consumer UI projects.
- 2026-08-25 23:45: Pre-build and post-deliverable Advisor calls were attempted as required, but both failed because the OAuth session was expired and unrefreshable. No advisor result was fabricated; independent runtime, package/contract, infrastructure, and fail-open Cato audits supplied the evidence.
- 2026-08-25 23:45: Deep audits refuted broad completion in several connected paths: TypeScript validation has no server route; public workflows return no synthesis; both general SDKs disagree with API envelopes/auth/methods; CLI API-key auth is wrong; migrations are not deployably durable; cache is not connected to calculation; and current delivery can proceed despite red checks.
- 2026-08-25 23:45: Safety ordering changed accordingly. Protecting main, gating deployment on a verified digest, clearing critical advisories, and making builds/migrations/typecheck/smokes fail closed precede new engine features.
- 2026-07-19 02:15: refined: The live integration plan names Phase 6 Wave 2 as performance, empty states, and error handling; Wave 1 is complete and remains immutable.
- 2026-07-19 02:15: Root-cause-at-ingestion: payload provenance and failure detail disappear inside `payloadLoader`, while Three.js enters the shell through the synchronous `App.tsx` import; both seams must be fixed before display polish.
- 2026-07-19 02:15: The accessible fallback is an equivalent text-and-section navigator, not an error page; reduced motion and unavailable WebGL must never hide witness content.
- 2026-07-19 02:15: Runtime status reuses the existing typed preload method and may expose only configured/authenticated/transport classes, never origins, headers, or credentials.
- 2026-07-19 02:25: Advisor's conditional GO added module-loading status, depth-scoped styles, and explicit proof that reduced motion and WebGL failure share one fallback mode; its proposed payload-before-module boot chain was rejected because payload resolution precedes lazy rendering by construction.
- 2026-07-19 02:30: The external Temperance worktree rail produced invalid wholesale-deletion checkouts and no index or summary, so no external diff was integrated; fail-open Codex rails completed the same two isolated contracts.
- 2026-07-19 03:00: ReReadCheck and independent final audit returned GO after confirming provenance, stale-request rejection, packaged-relative chunk loading, cleanup, focus restoration, status secrecy, and exact entry-budget enforcement.
- 2026-07-18 23:00: refined: Live state shows T-130/T-135 are complete; the next unresolved roadmap boundary is Phase 6 Wave 1, multi-engine witness assembly, not more engine transport.
- 2026-07-18 23:00: The current practice is a local derived session containing only sanitized artifacts and inquiry context; it is not a new backend entity or cloud-sync surface.
- 2026-07-18 23:00: Phase 6 Wave 2 resilience and performance work stays separate so this wave has one falsifiable journey: accumulate, inspect, resume, and open one witness.
- 2026-07-18 23:05: Council live-code audits converged on deterministic current-reading accumulation; the design reuses the reading cache and existing multi-artifact renderer instead of creating a second artifact store.
- 2026-07-18 23:05: The independently discovered depth recovery, package-size, accessibility, and runtime-status gaps are queued for Phase 6 Wave 2 rather than mixed into witness assembly.
- 2026-07-18 23:12: Advisor initially assumed database transactions, RNG, and replay that do not exist in this local artifact-assembly slice; the required conflict re-call removed those phantom requirements.
- 2026-07-18 23:12: Advisor's remaining valid blockers became ISC-124..127: canonical DST-safe identity, assembly version rejection, cache-write ghost prevention, and referentially consistent clear.
- 2026-07-18 23:15: Root-cause-at-ingestion: `cacheEngineResultForDepth` creates a fresh one-artifact payload before cache ingress. Fixing aggregation there upgrades all four handoffs simultaneously; display-only merging would leave the data loss intact.
- 2026-07-18 23:15: Deliverable D1 maps to CurrentPracticeIdentity/P6WaveGate through Codex plus external worktree rails; D2 maps to all four P6 features and ISC-83..127.
- 2026-07-18 23:49: The fail-open adversarial rail reproduced Unicode route failure, timezone ID collision, poisoned context reuse, and two-key rollback risk; ISC-128..131 require upstream fixes before verification.
- 2026-07-18: refined: P4 remains complete and immutable; this iteration appends a P5 trusted-desktop and Biofield-to-Noesis feature surface at ISC-43 onward without renumbering prior criteria.
- 2026-07-18: Live ancestry and source audit supersede stale status prose. T-125 is already satisfied by the Command Center actions and is not reopened.
- 2026-07-18: Quick Council convergence selected a single desktop gateway and provenance-bearing result artifact before any further UI expansion.
- 2026-07-18: Advisor verdict `GO` for an ordered four-task wave: packable SDK, trusted gateway, real Biofield capture, then T-130 Noesis handoff.
- 2026-07-18: The external Temperance rail created a run directory but produced no index, summary, or task artifacts; the protocol's fail-open path re-dispatched the same audits to Codex agents.
- 2026-07-18: Authentication, authorization, validation, and consent failures fail closed. Only availability-class failures may enter an explicit local fallback state.
- 2026-07-18: Renderer-held credentials, generic raw-fetch IPC, sibling source links, fabricated remote analysis, and raw/base64 reading persistence are prohibited.
- 2026-07-18: The full swarm plan owns the historical T-105 Biofield/T-110 Raaga meaning; condensed-list drift is recorded rather than resolved by renumbering.
- 2026-07-18: refined: The stale handoff proposed redispatching three P4 tasks, but live state proves those worktrees already exist; continuation starts by auditing and integrating them instead of creating duplicate branches.
- 2026-07-18: The P4 API and health worktrees contain uncommitted changes and are preserved as user work. Integration will use reviewed patches or commits, never destructive cleanup.
- 2026-07-18: The bridge worktree has a large committed diff because it reintroduces FROZEN contracts from an older base. Only changes absent from current main should be integrated.
- 2026-07-18: `task-master-planner` was inspected but removed from the active skill set because no new 80–100 task plan was requested; the existing detailed task list is input evidence only.
- 2026-07-18: The E4 soft floor of 128 ISCs is intentionally not manufactured. Forty-two atomic probes cover the bounded continuation surface; additional criteria would duplicate package-level tests rather than improve falsifiability.
- 2026-07-18: Systems leverage analysis selected information flow (Meadows level 6): branch ancestry and test evidence become authoritative, with the tactical bundle of preserving worktree diffs before any integration.
- 2026-07-18: First-principles classification: preserving user changes and FROZEN contracts are hard operational constraints; re-dispatching because the transcript said so is an invalidated assumption; forcing Kimi is a removable soft constraint.
- 2026-07-18: Kepner-Tregoe result: broad P4 attempts returned empty while split worktrees contain output because the broad task combined overlapping API/bridge/health contexts and the external Kimi rail exhausted quota. Smaller, isolated tasks plus fail-open routing explain both IS and IS-NOT cases.
- 2026-07-18: RedTeam was removed after selection because its 32-agent workflow would consume the execution rail needed for the requested P4 work. Mandatory E4 Cato verification supplies the adversarial audit.
- 2026-07-18: refined: Rejected the uncommitted health worktree as an integration source. Its full package does not compile, it has no feature-specific tests, changes liveness semantics, exposes internal URLs/errors, and its 71 library passes equal current main's baseline.
- 2026-07-18: refined: Rejected bridge commit `f57a2e7f9` wholesale. Its focused 41 tests pass while unrelated workspace files contain parse errors; native/Python engines are incorrectly routed through the TS bridge; generated media is still discarded.
- 2026-07-18: refined: Rejected the stale monolithic API diff. Its tests exercise legacy `options` rather than the merged SDK's top-level shape and would certify response nesting that contradicts FROZEN output.
- 2026-07-18: selected a clean integration worktree from current main. Salvage concepts and test cases only; reimplement the minimal API adapter, TS-only bridge transport, and separate dependency-health endpoint.
- 2026-07-18 15:35: refined: Advisor approved the clean-worktree direction but required executable golden fixtures for FROZEN request/response conformance, an exact health-response allowlist/leak test, exact legacy liveness regression coverage, and a negative proof that native/Python engines cannot enter the TS bridge. Added ISC-11.2, ISC-18.1, ISC-23.1, ISC-34.1, and ISC-35.1.
- 2026-07-18 15:42: root-cause-at-ingestion: the contract loss begins when `noesis-api` deserializes directly into legacy `EngineInput` and silently ignores merged-SDK top-level media. Normalize once at the API boundary; downstream native/Python/TS routes then receive a single canonical internal shape.
- 2026-07-18: refined after adversarial audit: `biofield-capture` is a persisted-reading lookup, not the live image-analysis transport. The frozen SDK intentionally keeps `biofield.calculate` on the native Rust API engine and `biofield.analyze` on Python `/analyze`; the ISA must test this split rather than invent a TS or generic-calculate route for capture analysis.
- 2026-07-18: refined after adversarial audit: dependency health is privacy-first. Sidecars are identified by stable IDs without URLs, and only providers with non-empty configured tokens appear. Earlier criteria demanding URLs/all providers contradicted the allowlist and secret-exposure constraints.
- 2026-07-18: refined after adversarial audit: added `p4_media_engines_tests` to prove all four SDK fixtures cross HTTP authentication, consent validation, canonical normalization, orchestrator dispatch, generated-media dual paths, legacy options, unknown-ID rejection, and native-versus-TS runtime selection.
- 2026-07-18: merged the clean recovery commit `0430cff9` into primary main as `a45952482`; the preserved status edit and stale P4 worktrees did not conflict and remain intact.
- 2026-07-18: final advisor verdict `GO`; fail-open adversarial re-audit verdict `GO` with no remaining blockers. The Cato role itself was unavailable, so the required audit was retried through a read-only explorer and explicitly re-audited after fixes.
- 2026-07-18: billing is recorded as unchanged environmental failure, not validated behavior. The same four `billing_e2e_tests` fail at line 55 with `PoolTimedOut` on both pre-change and post-merge main because local Postgres is absent.
- 2026-07-18 17:50: Documentation sync skipped because this run changed no PAI system file; only project code, project status, ISA, and required learning logs changed.
- 2026-07-18: Merged the packable SDK as `f2cc7a83a` and the Raaga clip-consent correction as `d4f071434`; the final package has 22 files and no raw source entries.
- 2026-07-18: Merged Sankalpa trusted gateway and result continuity as `7e08431`; Electron main now owns authenticated transport, exact operation consent, runtime input validation, CSP/navigation restrictions, and trusted-top-frame IPC.
- 2026-07-18: Final independent boundary review returned `GO`, and browser Gate 4 returned `PASS` after measured Sigil/Face layout and Biofield consent-control corrections.

### Continuation — 2026-09-05

- 2026-09-05T10:35:44.959123+00:00: refined: richer stashed ISA restored after a stable-ID subset check; no starting criterion ID was lost. Historical statuses remain dated evidence.
- 2026-09-05T10:35:44.959123+00:00: User explicitly requests recommended GSD discussion/research flow and critical HITL only; routine technical defaults are auto-selected once per phase.
- 2026-09-05T10:35:44.959123+00:00: GSD phase 01 is a recovery entry point mapped to original Wave 0; original Wave 0 registry/asset/release-receipt exits remain tracked in phase 02. This is not evidence that Wave 0 fully closed.
- 2026-09-05T10:35:44.959123+00:00: The independent noesis-plan advisor identified acceptance-ownership and deployment-falsifier concerns. Explicit ledger authority and exact source/CI/deploy separation are now required. Its erroneous 570x30 wording was rejected against live 19x30 evidence.
- 2026-09-05T10:35:44.959123+00:00: Earlier noesis-observe and initial Hands calls returned gateway errors/empty outputs and are not accepted as completed work. A smaller isolated Superset Claude worker continues on noesis-execute.
- 2026-09-05T10:35:44.959123+00:00: Selected analytical capabilities: ContextSearch (prior planning evidence), ISA (history reconciliation), Advisor (bounded plan review), FirstPrinciples, SystemsThinking, Science and ReReadCheck (explicit lenses in the external review). None of those reviews substitutes for source verification.

- 2026-09-05T13:47:55.589355+00:00: GitHub control bodies are reconciled with readback hashes; the missing public admin smoke URL is configured to its verified origin. The next critical decision is the additive main CI requirement. Production promotion remains held for release identity and rollback evidence.

## Changelog

- 2026-08-26 | conjectured: schema-valid fixtures and additive DTOs alone proved cross-language contract convergence
  refuted_by: independent review found optional-result drift, unvalidated API payloads, validator escape paths, an ungated maintained SDK, and extractor errors outside the universal envelope
  learned: contract authority must bind schema variants, real deserialization behavior, every maintained adapter, negative validator cases, and the composed root gate
  criterion_now: ISC-235..278 require exact authority, payload-level rejection, bidirectional parity, scope audit, full-gate execution, and final independent GO
- 2026-08-26 | conjectured: a gate passing in its isolated worktree was sufficient evidence for immediate cleanup
  refuted_by: the merged primary checkout loaded an ambient provider credential and timed out in a supposedly network-free test
  learned: integration verification must run under the destination checkout's real environment before worktree retirement
  criterion_now: ISC-228..231 require merged-environment execution, credential isolation, and cleanup only after green evidence
- 2026-08-25 | conjectured: the broad ecosystem ledger should remain the current planning authority
  refuted_by: the user narrowed planning to Selemene Engine and external repository rows assigned work outside the authorized boundary
  learned: consumer repositories define compatibility evidence but require their own accepted planning authority
  criterion_now: ISC-164..172 require a repository-only boundary, explicit precedence, and superseded context banners
- 2026-08-25 | conjectured: registered, green, or live capabilities could share one completion label
  refuted_by: missing TS validation, disconnected workflow synthesis, SDK/API mismatches, simulated media paths, fail-open CI, and unidentified deployment revision
  learned: completion must remain a six-axis evidence vector through every engine, contract, package, service, and operational surface
  criterion_now: ISC-173..185 require exact registry parity, per-axis rows, probe limitations, and contradiction correction
- 2026-08-25 | conjectured: feature and integration work should lead the next roadmap
  refuted_by: unprotected main, deploy-after-red-CI behavior, security advisories, invalid deployment configuration, non-durable migrations, and broken release gates
  learned: authority and safe fail-closed delivery precede runtime and experience expansion
  criterion_now: ISC-186..193 enforce dependency-ordered waves with falsifiable exits
- 2026-08-25 | conjectured: the first Selemene-only draft was complete after three domain audits
  refuted_by: adversarial review found overstated deployment axes, missing billing/publication/onboarding subsystems, and a Wave 0/1 dependency cycle
  learned: independent verification must challenge both evidence coverage and the plan's own causal ordering
  criterion_now: ISC-181..195 include the missing subsystems, partial deployment evidence, corrected wave sequence, and final GO re-audit
- 2026-07-18 | conjectured: the OpenCode handoff accurately described the current stopping point
  refuted_by: live git ancestry shows main and Sankalpa already contain several commits the handoff still described as pending
  learned: cold-start continuation must compare narrative state with repository state before dispatching or merging
  criterion_now: ISC-1 through ISC-10 require branch, ancestry, worktree, and preservation probes before P4 integration
- 2026-07-18 | conjectured: the stale health and bridge worktrees were safe merge sources
  refuted_by: their route semantics, contract shape, and test evidence contradicted current main and the merged SDK
  learned: salvage intent and tests, then rebuild on the current integration base when divergent worktrees cross architectural boundaries
  criterion_now: ISC-20 through ISC-36 require real runtime-boundary, privacy, and liveness proofs
- 2026-07-18 | conjectured: DTO-level fixtures were sufficient evidence for the P4 endpoints
  refuted_by: adversarial review showed they did not exercise HTTP authentication, consent rejection, orchestrator dispatch, or unknown-engine routing
  learned: contract fixtures and route-level probes are complementary; neither substitutes for the other
  criterion_now: ISC-11 through ISC-19 are backed by both exact fixture equality and `p4_media_engines_tests`
- 2026-07-18 | conjectured: compile-time IPC types and renderer consent controls were sufficient for a trusted desktop boundary
  refuted_by: correctness review reproduced clip generation, malformed media, and unauthenticated requests crossing the boundary before runtime guards existed
  learned: main must validate sender, authentication, operation-specific consent, serialization, size, MIME, and base64 before the first remote call
  criterion_now: ISC-51 through ISC-70 require zero-call proofs for invalid, unauthenticated, revoked, tokenless, and wrong-scope requests
- 2026-07-18 | conjectured: a generic sanitized object walk would remove every sensitive engine field
  refuted_by: nested `credential`, `x_api_key`, and `auth_header` aliases survived the first redaction list
  learned: persistence boundaries need adversarial key-alias fixtures and a second sanitization pass at cache ingress
  criterion_now: ISC-72 and ISC-82 require sensitive aliases to be absent from artifact, cache, history, and hash payload
- 2026-07-18 | conjectured: functional focus-engine forms were ready once unit and build gates passed
  refuted_by: browser QA measured Sigil overflow, a collapsed Face camera, and duplicate Biofield consent controls at the desktop viewport
  learned: continuity features need measured browser geometry and interaction validation in addition to pure-function tests
  criterion_now: ISC-80 includes Gate 4 browser validation of the integrated focus-to-Noesis journey

### Continuation — 2026-09-05

- 2026-09-05 | conjectured: current checkout and completed root ISA identify the active project completion state
  refuted_by: three authority files were only in a preserved stash, its ISA contains 285 historical criteria, two capability PRs are unmerged, 570 engine issues are open and production capabilities return 404
  learned: recover source authority by stable identity and keep local, CI, deployment and operational evidence distinct
  criterion_now: ISC-279 through ISC-314 govern the current continuation without renumbering historical criteria

- 2026-09-05T13:47:55.589355+00:00 | conjectured: local gate success implied the candidate would pass every clean CI surface
  refuted_by: clean-runner imports/build ordering, Vercel packaging and a database-backed fixture lifetime failure
  learned: validate provider packaging and DB-configured test lifetimes on disposable services before promotion
  criterion_now: ISC-305 remains open until current-source CI Gate passes; ISC-306 records verified Python/admin coverage without claiming complete release receipts

## Verification

- ISC-235..278: Contract work remained isolated on `codex/selemene-contract-convergence` from baseline `01160e5`; the final evidence commit is `59a38ed`. Changed paths are confined to the v1 authority, validators/gates/tests, additive contract types, API/OpenAPI compatibility boundaries, maintained SDK parity, and contract documentation. Primary-worktree planning changes remained present and no external repository, live service, database, cloud resource, remote, publication, deployment, provider, routing, or engine algorithm was changed.
- ISC-238..264: `contracts/v1` contains exactly six Draft 2020-12 schemas and five uniquely mapped fixtures. The validator rejects malformed or missing authority entries, duplicate/zero fixtures, invalid payloads, traversal, unresolved local fragments, network references, and sensitive error/provenance keys; root gate composition executed 47 Python behaviors.
- ISC-265..275: Rust canonical contracts passed 6/6; OpenAPI parity passed 4/4; API calculate-boundary integration passed 16/16; engine SDK passed 35/35 plus typecheck; general TypeScript SDK passed 11/11 plus typecheck; verification passed 36/36 plus typecheck; TypeScript engines passed 90/90 plus typecheck.
- ISC-276: Fresh `pnpm run gate` exited zero at committed implementation candidate `0df208d`. The subsequent evidence-only commit `59a38ed` passed `git diff --check` and does not alter executable behavior.
- ISC-277: Independent read-only review returned GO at `0df208d` after two earlier BLOCK cycles and found no remaining P0/P1 contract drift, compatibility break, false-green path, or engine-scope leak. It independently reran `gate:scripts`, `gate:contracts`, and `git diff --check 01160e5..HEAD`.
- ISC-278 ReReadCheck: “Gate foundation merged; contract convergence is isolated, green, and next.- proceed” — addressed exactly through the contract layer. The branch remains isolated and unmerged; engines remain deliberately untouched until separately authorized work begins.
- Remaining criterion: ISC-217 stays open because GitHub Actions still use floating version tags rather than immutable commit SHAs. Contract completion does not overwrite or imply closure of that historical gate gap.
- ISC-225..231: Gate changes were committed as `c11bc39` and `5cf48b9`, integrated into local `main` by merge commits `130397a` and `01160e5`, and verified as ancestors. All fourteen pre-existing dirty files retained identical SHA-256 digests. The completed worktree was removed and the merged branch deleted with `git branch -d`, never force.
- ISC-228..229: The first merged gate run correctly failed when the primary checkout exposed an ambient provider credential. A sentinel-credential focused run then passed 21/21 after credential isolation, and the final merged `pnpm run gate` passed 31 Python gate tests, 36 verification tests, strict TypeScript checks, and 88/88 Bun tests with 268 expectations.
- ISC-232..234: `.worktrees/selemene-contract-convergence` was created on `codex/selemene-contract-convergence` from `01160e5`. After pnpm and Bun dependency setup, its baseline `pnpm run gate` passed the same 31, 36, and 88 test groups. No external repository, live service, database, cloud resource, remote branch, or engine semantic path was changed.
- ISC-196..224: Gate work remained isolated on `codex/selemene-gate-foundation`; primary-worktree status and all unrelated user changes were preserved. Contracts, engines, external repositories, live services, databases, GitHub settings, deployment, publication, push, merge, and release remained untouched.
- ISC-199..218: Fresh `pnpm run gate` exited zero with 31 script behaviors, 36 verification tests, both TypeScript typechecks, and 88/88 Bun tests with 268 expectations. The only open criterion is ISC-217 because GitHub Actions still use version tags rather than immutable commit SHAs.
- ISC-204..207: Disposable PostgreSQL 16 applied and journaled all 36 historical filenames on a fresh database, including both `007` files and nontransactional `030`; the immediate second run reported `applied=0 skipped=36`, and the exact disposable container was removed afterward.
- ISC-208..210: Docker workspace/target validation passes and the cache-stage Cargo command is no longer masked. Real `linux/arm64` construction now fails closed on 12 pre-existing `libswisseph-sys 0.1.2` `E0308` errors; the `linux/amd64` QEMU attempt reached the same real step but was inconclusive after GCC `cc1` segfaulted.
- ISC-211..220: `actionlint` 1.7.7 returned zero findings for both workflows; warning-level ShellCheck 0.10.0 returned zero findings for the canonical runner and Suno operator; YAML, TOML, Python compilation, Bash syntax, and `git diff --check` passed. The Cato-role fallback's final verdict was PASS with no remaining P0/P1 false-green path.
- ISC-223..224: The gate evidence states both guarantees and residual Wave 1 gaps. ReReadCheck compared the latest instruction, “Next work starts with gates, contracts, then engines,” and confirmed this iteration completed only the bounded gate foundation; contracts remain next and engines remain after contracts.
- ReReadCheck: “we dont need to plan for the rest of the repos” — addressed: no active roadmap item assigns external-repository implementation. “as context and understanding the delivery to various platforms and connected infra” — addressed: the delivery-context map is retained, and Selemene's compatibility, package, service, and infrastructure boundaries are represented. “focus a deep pass only on the selemene engine” — addressed: the active authority, ledger, roadmap, ISCs, tests, and adversarial review are repository-local.
- ISC-164..172: The active `docs/plans/selemene-engine/` authority states the Selemene-only boundary and six evidence states; README, historical overview, and integration status point to it. All three ecosystem documents are visibly superseded/context-only.
- ISC-173..180: Automated registry comparison found exactly 19 unique ledger rows matching `SUPPORTED_ENGINE_IDS`; source audit confirmed 12 unconditional native registrations, one database-conditional capture engine, and six TypeScript bridges. Engine rows record the specific fallback, mock, provider, or semantic gaps discovered in source.
- ISC-181..185: Platform and infrastructure ledgers include SDKs, CLI/TUI/admin, witness/assets, verification, billing/entitlements/credits, Dodo webhooks, Living Reading publication/invitations, OpenClaw onboarding, persistence/cache, images/topologies, CI/release/security, operations, and assets. The contradiction register records the material false-completion patterns.
- ISC-183..184: Fresh 2026-08-25 production probes returned HTTP 200 for liveness/readiness with version 3.3.1, 19 loaded engines, six workflows, healthy Postgres/Redis/orchestrator/bridge/six TS engines, and structured 401 for protected unauthenticated routes. The ledger explicitly withholds authenticated-journey, provider, Python-sidecar, and revision-bound claims.
- ISC-186..194: Seven internal waves each have exit evidence. External product names appear once in the active roadmap only to bound Selemene-side compatibility fixtures and explicitly prohibit downstream implementation assignment.
- ISC-195: Local Markdown link validation passed across nine active/historical planning surfaces; registry parity passed 19/19; every roadmap wave has an exit; `git diff --check` passed. Independent adversarial review initially BLOCKed on three P1 findings, then returned GO after deployment axes, missing subsystems, and Wave 0/1 ordering were corrected.
- Fresh executable evidence: `cargo test -p noesis-api --lib --locked` passed 94/94; `cargo test -p noesis-orchestrator --lib --locked` passed 96/96; `bun test` in `ts-engines` passed 88/88; root `pnpm verify && pnpm verify:typecheck` passed 36/36 and typecheck.
- Fresh negative evidence: `bun run typecheck` in `ts-engines` fails on removed `baseUrl` and invalid non-relative paths; main CI currently masks this exact failure. Package/infra audits additionally recorded witness/Python/tooling gaps and current red dependency/security gates rather than converting them into completion claims.
- ISC-132..137: Production Vite build emitted `index-CYgISPpb.js` at 249,275 raw/79,819 gzip bytes and `DepthGallery-5EhUnFqv.js` at 515,503 raw/129,200 gzip bytes; `npm run check:renderer-budget` proved Three.js is absent from entry and present in the single async depth chunk.
- ISC-138..147: Focused loader tests prove hash/cache/remote provenance, malformed-hash fallback, 404 not-found, typed failure, safe rejected-IPC conversion, legacy nullable compatibility, and the deferred-promise gate rejects stale completions.
- ISC-148..155: Gateway renderer tests cover all configured/authenticated combinations; browser Account QA displayed only `Local-only`, fixed guidance, and no URL, token, header, exception, console error, or failed request.
- ISC-156..161: Pure navigation tests cover arrows/Home/End; browser reduced-motion QA rendered no canvas, changed 1/7→2/7→7/7→1/7, opened a native modal with focus inside, and restored the exact opener after Escape.
- ISC-162: Full gate passed 12 test files and 100 tests, renderer/Electron typechecks, production build, unsigned arm64 package, desktop 1440×1000 and narrow 390×844 browser journeys, zero overflow, zero console warnings/errors, and six successful requests including the async chunk.
- ISC-163: `git diff --check`, direct-renderer-fetch scan, production-bundle credential scan, main/preload contract review, and independent read-only final audit returned clean/GO.
- Package smoke: `npm run package:dir` produced unsigned `release/mac-arm64/Sankalpa.app`; the packaged `app.asar` contains `dist/index.html`, the renderer entry, CSS, and the asynchronous `DepthGallery` asset.
- Temperance dispatch: external worktree checkouts were invalid and emitted neither `index.json` nor `SUMMARY.md`; fail-open Codex workers changed only their assigned loader, gateway, and depth files and all focused gates passed.
- ISC-1: `git branch --show-current` returned `main`.
- ISC-2: `git status --short --branch` showed no merge state; only the pre-existing status edit and untracked `.agents/`/`.worktrees/` entries.
- ISC-3: `git log --oneline` contains `51c3e6f5 Merge raaga-clip-codex`.
- ISC-4: `git log --oneline` contains `8e47ee88 Merge face-cv-codex`.
- ISC-5: `git log --oneline` contains `a3a5b410 Merge p4-sdk-codex`.
- ISC-6: `git -C ../sankalpa log -1` returned `528a247 feat(prong2): T-115 sigil-forge UI + T-120 face-reading surface`.
- ISC-7: `git -C ../sankalpa status --short --branch` returned only `## main...origin/main [ahead 2]` with no file changes.
- ISC-8: `git worktree list --porcelain` lists `p4-api-codex`, `p4-api-endpoints-codex`, `p4-bridge-verify-codex`, and `p4-health-codex`.
- ISC-9: The pre-existing T-094 edit was retained through health verification, then corrected in place to reference merged commits and actual test counts.
- ISC-10: Final `git status`/`git diff --stat` in `p4-api-codex` and `p4-health-codex` matches the preserved uncommitted surfaces; no cleanup/reset/checkout-overwrite command was used.
- ISC-11..19: `cargo test -p noesis-api --test p4_media_engines_tests` returned 5/5 on merged main. The suite POSTs all four frozen SDK fixtures through HTTP auth, proves canonical and legacy inputs, exact consent failure before dispatch, generated-media dual paths, and unknown-ID 404 behavior. API unit fixtures additionally prove byte-lossless multipart and exact JSON equality.
- ISC-20..27: `cargo test -p noesis-bridge` returned 40/40 plus doc-test 1 pass/1 ignored on merged main. SDK tests prove Biofield's native-calculate/direct-Python-analyze split; route tests prove native Biofield/face versus TS Raaga/sigil sets; bridge tests prove media/consent forwarding, timestamp preservation, generated-media recovery, negative focus registration, and media/consent-sensitive cache keys.
- ISC-28..36: `cargo test -p noesis-api --test p4_health_contract_tests --test routing_enforcement_tests` returned 2/2 and 20/20. Unreachable loopback dependencies force the unavailable path; the response remains HTTP 200, reports exactly four engines and two stable sidecar IDs, exposes exact allowlisted keys, and rejects tokens, URLs, versions, stacks, traces, and raw errors. Legacy liveness and readiness tests pass unchanged.
- ISC-31/33: `dependency_providers_include_only_nonempty_configuration` proves empty/unset values are omitted and secret values are never serialized; the handler performs no provider network or generation call.
- ISC-37: `cargo test -p noesis-bridge` returned 40 passed on integrated main.
- ISC-38: P4 media 5/5 and P4 health 2/2 passed on integrated main.
- ISC-39: `cargo test -p noesis-api --lib` returned 83 passed on integrated main.
- ISC-40: `bun test` in `packages/noesis-engine-sdk` returned 27/27; `bun install --frozen-lockfile` followed by focused Raaga/Sigil tests returned 33/33; the TS schema validator returned 15/15.
- ISC-41: `EXECUTION-STATUS.md` now cites `0430cff9`/`a45952482`, exact suite counts, the separate health route, privacy rules, and the billing environment limitation without claiming billing correctness.
- ISC-42: Command/diff audit found no provider generation, secret output, force/reset operation, stale bulk merge, pull/rebase, push, deployment, issue-tracker mutation, or unrelated remote synchronization.
- Regression: `cargo check -p noesis-api -p noesis-bridge --all-targets --locked` passed; target-only Clippy with `--no-deps -D warnings` passed. Workspace-wide formatting remains blocked only by pre-existing formatting in `engine-biofield` and `handlers/assets.rs`, neither changed by this integration.
- Environmental parity: pre-change and post-merge `cargo test -p noesis-api --test billing_e2e_tests --locked` each produced exactly four `PoolTimedOut` failures at line 55 and no additional failures; billing behavior itself remains unvalidated without Postgres.
- Reviews: the final Advisor returned `GO`; the fail-open adversarial re-audit of `0430cff9` returned `GO — all three prior blockers are resolved`.
- ISC-43..50: `bun test` in `packages/noesis-engine-sdk` returned 33/33; `npm run typecheck` and `npm run build` exited zero. `npm pack --dry-run --json` listed 22 files, executable ESM/declarations, 20.4 kB packed/76.3 kB unpacked, and no `src` files. The packed artifact imported in standalone Node and in Sankalpa; mocked requests prove default auth headers, canonical Biofield session route, consent-before-fetch, and multipart field `image`.
- ISC-51..63: Sankalpa main-gateway, renderer-adapter, media-contract, and desktop-foundation tests prove the four-ID allowlist; main-owned HTTPS/loopback origin, headers, and timeout; narrow preload methods; zero calls for disallowed/malformed/unauthenticated requests; exact auth/consent/validation/rate classifications; exact four scopes; and `request_clip: true`. Source and bundle audits found no focus renderer `fetch`, backend Vite variable, or token access; production CSP contains no backend origin.
- ISC-64..70: Biofield tests prove local-only zero IPC, pixel-derived local analysis, independent capture/consent transitions, revocation, strict MIME/canonical-base64/size checks, main consent and auth preflight, session-before-capture ordering, canonical `image` multipart, backend metric/quality replacement, and stable session/reading identifiers.
- ISC-71..78: Four engine fixtures normalize into versioned artifacts; recursive redaction removes base64, data URLs, consent, raw errors, tokens, credentials, `x_api_key`, and `auth_header`; artifact and route ceilings are 24 KiB/48 KiB; cache and hash share the exact payload; Noesis renders provenance, safe summaries, witness prompts, and non-prescriptive self-inquiry language. Browser QA exercised Raaga to the exact depth route with a 1,508-byte safe payload. Existing Command Center actions remain the T-125 surface.
- ISC-79: SDK 33/33, package typecheck, build, pack, and packed runtime import passed on merged Selemene code.
- ISC-80: A clean Sankalpa `npm ci` followed by 66/66 tests, renderer and Electron typecheck, production build, and unsigned arm64 package completed. Merged-main verification again returned 66/66 after `vitest.config.ts` excluded nested `.worktrees`; production-only `npm audit --omit=dev` found zero vulnerabilities, and the known Vite advisory remains only a Three.js chunk-size optimization note.
- ISC-81: Sankalpa installs the rebuilt SDK tarball by verified SHA-512 lock integrity; its production Electron build and main-gateway tests execute the packaged declarations/runtime rather than a sibling source path.
- ISC-82: Source, bundle, persistence, git-command, and final read-only boundary audits found no renderer credential, raw-media reading persistence, backend CSP bypass, unauthorized file navigation, remote sync, or destructive git action. Final review verdict: `GO — no correctness blockers remain`.
- Merge evidence: Selemene main contains `f2cc7a83a` and `d4f071434`; Sankalpa main is `8692370`, containing integration merge `7e08431` from source `7002d5f` plus the nested-worktree test-discovery guard.
- Browser evidence: Gate 4 at 1440×1000 measured Sigil and Face main widths at 1140/1140 with zero horizontal overflow, Face camera at a readable 420 px single column, one Biofield consent control, no page errors or failed requests, and an exact sanitized Raaga-to-Noesis handoff.
- ISC-83..105 and ISC-123..131: Focused current-practice, threshold-storage, and Noesis regression suites passed after adversarial remediation, covering DST/day identity, collision-free canonical timezone IDs, four-engine replacement, Unicode routes, poison-cache allowlisting, aggregate limits, stale versions, and atomic envelope failures.
- ISC-106..119: Browser Gate 4 passed Unicode Threshold → Raaga → Sigil → combined Noesis → clear. It visibly proved two observations, retained inquiry, exact engine labels, a concrete current-practice route, and the explicit empty state without functional console, page, request, or HTTP errors.
- ISC-120..122: The final Sankalpa gate passed with 85/85 tests, both TypeScript projects, production build, whitespace checks, and sensitive fixture-value bundle scans. The only build warning is the known Three.js chunk-size optimization queued for Wave 2.
- Final review: Independent code audit returned GO with no P0/P1 findings after Unicode, strict context allowlists, single-envelope cache atomicity, malformed-state resilience, and timezone-collision remediation.

### Continuation — 2026-09-05

- 2026-09-05T11:07:35.247112+00:00: ISC-279–286: readback proved all 170 original stable IDs survive within 321 total criteria; each of three original stash authority files remains an exact substring of its recovered file. The issue snapshot contains 570 unique issues, 19 groups, 30 slots each; all eight controls are OPEN. GSD recognizes seven phases and both Phase 1 plans validate. Advisor concerns and its incorrect corpus arithmetic are explicitly dispositioned in research.
- ISC-287–293,299: source provider snapshots identify seven Railway deployments and the 9d9d-owned active zone, three live Worker bindings, denied DNS reads and absent pattern-memory. Public curl returns 200/19 engines; production TS capability discovery returns 404. Python audit describes the local pip finding and withholds production resolution proof. See INFRASTRUCTURE-MAP.json and DEPENDENCY-AUDIT-2026-09-05.md.
- ISC-294–296: CodeGraph reports 859 indexed files, 14,850 nodes and 36,207 edges. Repeated sync exits zero with Already up to date. A fresh context query resolves engine_capabilities at admin.rs:4643 and returns the actual six-engine bridge traversal.
- ISC-302: runtime capability evidence now identifies the six TS rows and explicitly withholds native, conditional and complete Python capability parity.
- ISC-309,311,313–314: original checkout remains clean and stash ref a6c7a0e9f04ba318fe0d98671cc80b720a529861 is retained. Superset Hands emitted real formatting/config/dependency edits; actual noesis-execute provider receipt is claude/claude-sonnet-5. Critical production/security operations are still pending. These continuous safeguards must be rechecked before promotion.
- Historical verification above retains its original date and scope. No full-wave, remote-CI or deployment completion is implied.

- ISC-297: command audit — cargo audit reports "found":false and "count":0; the unchanged chacha20 0.10.1 optional HTTP/3 lockfile yank is traced and explicitly retained.
- ISC-298: command audit — final pnpm audit reports info 0, low 0, moderate 0, high 0, critical 0 across production and development.
- ISC-300: command tests — repaired TS lint/typecheck and all 93 recovery tests pass; PR #1486 at exact head 520e439 separately passes all CI jobs.
- ISC-304: git history — source changes are scoped commits 8e5a8d3,ee2b39f,d20d1b6,b456c18,ade679f,86cbae9,f6d777e, with local evidence and remaining remote checks explicit.
- ISC-315: command audit — pnpm-audit-final-all.json returns zero vulnerabilities at every severity.
- ISC-316: executable workflow probe — 14 mocked scenarios execute the actual agent-merge-lane.yml script and pass; full script suite: 67 passed.
- ISC-317: browser read — account 9d9d zone DNS table shows selemene->Railway proxied,144->Vercel proxied,48->Railway DNS only; no DNS mutation.
- ISC-318: browser read — deployed admin lists 19 engine routes; sidecar health shows six TS engines. Stale count and Python description corrected in source; candidate is not yet deployed.

- ISC-303: GitHub readback — all eight control bodies updated; original exit checkboxes, OPEN state and labels preserved. GITHUB-RECONCILIATION-2026-09-05.json records hashes. All 570 engine issues remain untouched.
- ISC-306: GitHub CI — at source 9b618de, both Python image import jobs, Python 3.11/3.12 contracts/smoke and required admin checks pass. Both release paths depend on complete CI; all 77 Action uses pass the immutable pin validator. Full CI remains separately governed by ISC-305.
- ISC-310: privacy verification — 32 recovery artifacts pass personal-email/private-path checks; GitHub TruffleHog job 101312749961 passes. No authenticated personal records, credential values or secret variable values were saved in these receipts.
- Capability fixture: disposable PostgreSQL 18.4 reproduces the original 500/timeout; a shared test-binary runtime passes all three existing cases against the same migrated database. Remote PostgreSQL 16 rerun is required.
