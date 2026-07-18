---
project: Selemene-engine
task: "P5 trusted desktop gateway and Biofield-to-Noesis handoff"
effort: E3
effort_source: classifier
phase: complete
progress: 82/82
mode: algorithm
started: 2026-07-18T15:00:00+05:30
updated: 2026-07-18T22:50:00+05:30
---

## Problem

P4 is now verified on Selemene main, and Sankalpa already presents camera, Raaga, Sigil, Face, and Command Center surfaces. The remaining user journey is fragmented: the SDK is private source-only TypeScript, renderer components call authenticated API routes directly without authorization, Biofield camera capture fabricates an analyzed record before a real backend response, and `ReadingPayload.engine_results` is ignored by the Noesis section builder. The next wave must create one trusted desktop boundary and carry a consented Biofield result into a provenance-safe Noesis witness flow.

## Vision

Sankalpa feels like one coherent instrument rather than four disconnected demos. A user can keep a capture local, explicitly consent to authenticated analysis, understand whether the result came from local approximation or the backend, and continue that exact result into Noesis depth and witness reflection. Credentials remain in the Electron main process, unavailable services degrade honestly, and auth or consent failures never masquerade as successful local analysis.

## Out of Scope

- Reopening the completed P4 API, bridge, health, raaga clip, face CV, or media-contract work.
- Rebuilding T-125: the live Sankalpa Command Center already launches Noesis and all four focus engines.
- Production Discord OAuth, token refresh, operating-system keychain storage, billing, or account synchronization.
- Publishing the SDK to a public registry; this wave proves a packable standalone artifact.
- Persisting raw camera pixels or generated base64 media inside localStorage or Noesis reading payloads.
- Pulling, rebasing, force-updating, or otherwise reconciling the three remote commits that local main is behind.
- Pushing branches, opening pull requests, deploying, or mutating external issue trackers.
- Changing FROZEN media contracts unless a failing test proves an existing contract cannot round-trip.
- Making paid NVIDIA, RunComfy, Kimi, or other generation calls from health probes.
- Treating narrative witness output as a substitute for deterministic engine output.
- Repairing or redesigning billing E2E infrastructure; its four Postgres `PoolTimedOut` failures are reproduced unchanged on pre-P4 main.

## Principles

- Live repository state outranks stale session narration.
- A merge is complete only when ancestry and tests prove it.
- Preserve local-first media handling and explicit consent before pixels or audio leave the client boundary.
- Health checks observe configuration and reachability; they never perform paid generation.
- Integration should minimize overlap: API, bridge, and health changes remain separable until individually verified.
- Engine output is the authoritative deterministic anchor; narrative layers may reflect it but never silently replace it.

## Constraints

- Preserve the user's existing modification to `docs/plans/engine-integration/EXECUTION-STATUS.md` until its T-094 claim is independently verified.
- Preserve all uncommitted changes in `.worktrees/p4-api-codex` and `.worktrees/p4-health-codex`; no reset, checkout-overwrite, or cleanup is permitted.
- Keep the P4 bridge work isolated until its large FROZEN-contract diff is compared against current main.
- Reuse the existing FROZEN media shapes and consent semantics.
- Keep `/health`, `/ready`, and `/health/live` backward compatible.
- Provider readiness may inspect token presence only; secrets must never be printed or returned.
- API and bridge code must compile in the current Rust workspace and TS engine tests must remain compatible.
- Sankalpa remains a separate repository and must consume a built SDK artifact, never a sibling source-tree link.
- Renderer code must never receive API credentials or an unrestricted raw-fetch IPC primitive.
- Availability failures may activate explicit local fallback; authentication, authorization, validation, and consent failures must fail closed.
- Local preview, local approximation, and authenticated remote analysis are distinct provenance classes.
- Keep Sankalpa's Electron isolation guarantees: context isolation on, Node integration off, sandbox on, allowlisted IPC only.

## Goal

Ship the next integration wave across Selemene and Sankalpa: a distributable typed engine SDK, a main/preload-owned authenticated EngineGateway, a real consented Biofield session-and-capture path, and a bounded engine-result artifact that opens in Noesis depth with provenance and witness context. Done means the packaged client installs standalone, no focus-engine request needs renderer-held credentials, local-only consent produces zero network activity, real Biofield responses replace fabricated analysis, all four result adapters are cache-safe, and repository gates pass on the integrated branches. Completion evidence must include both integrated main branches, package and runtime gates, security review, browser validation, and the fully checked ISC record.

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

## Decisions

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

## Changelog

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

## Verification

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
