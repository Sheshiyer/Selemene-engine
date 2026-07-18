---
project: Selemene-engine
task: "Resume and complete P4 API bridge health integration"
effort: E4
effort_source: classifier
phase: complete
progress: 49/49
mode: interactive
started: 2026-07-18T15:00:00+05:30
updated: 2026-07-18T17:50:00+05:30
---

## Problem

An OpenCode execution session stopped at a Kimi billing-cycle limit while coordinating the Selemene + Sankalpa media integration. Its narrative handoff is already stale: main now contains the raaga clip, face CV, and P4 SDK merges; Sankalpa contains committed T-115/T-120 surfaces; and three split P4 worktrees contain different degrees of API, bridge, and health progress. The remaining risk is not merely unfinished code but integrating uncommitted and divergent worktree changes without losing user work, duplicating already-landed changes, or accepting reported success without test evidence.

## Vision

The continuation feels seamless: the operator does not need to reconstruct the previous session, completed changes remain intact, and the unfinished P4 surface converges into a small set of reviewed commits. One authoritative Rust API exposes all four focus engines with FROZEN media fields, verifies each engine at its real native/TS/Python boundary, and offers a detailed dependency-health report without turning liveness into a network fan-out or making paid calls. Every claim is backed by branch ancestry, focused tests, and repository-level regression checks.

## Out of Scope

- Reimplementing the already-merged raaga clip, face CV, or P4 SDK work.
- Rewriting the committed Sankalpa T-115 sigil and T-120 face-reading surfaces.
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
- Sankalpa is a separate repository; its clean committed state is evidence, not a target for new edits in this continuation.

## Goal

Safely complete and integrate the remaining P4 API endpoint, engine-boundary verification, and aggregated dependency-health work on Selemene main while preserving all prior OpenCode progress and user changes. Done means the four focus engines accept the merged SDK's real top-level FROZEN media shape, generated media remains top-level, native/Python engines avoid the TS bridge, `/health` and `/health/live` stay cheap and backward compatible, a separate detailed endpoint reports engines/sidecars/configured providers without paid calls or secret exposure, focused and regression tests pass, and status documentation matches the proven commit state.

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

## Features

| name | description | satisfies | depends_on | parallelizable |
|---|---|---|---|---|
| StateRecovery | Reconcile transcript claims against branches, commits, worktrees, and Sankalpa | ISC-1..10 | none | true |
| ApiMediaExposure | Complete four-engine P4 media request/response coverage | ISC-11..19 | StateRecovery | true |
| BridgeVerification | Integrate minimal bridge registration and lossless forwarding | ISC-20..27 | StateRecovery | true |
| AggregatedHealth | Integrate engine, sidecar, and provider readiness reporting | ISC-28..36 | StateRecovery | true |
| RegressionGate | Run focused and package-level suites on the integrated result | ISC-37..40 | ApiMediaExposure, BridgeVerification, AggregatedHealth | false |
| StatusReconciliation | Rewrite status claims to match verified evidence | ISC-41..42 | RegressionGate | false |

## Decisions

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
