# Selemene Runtime Capability Evidence

Date: 2026-08-27
Branch: `codex/selemene-whitepaper-clean`
Scope: TypeScript sidecar runtime capability adoption only; no engine algorithms, providers, credentials, databases, deployments, remotes, GitHub issues, or external services changed.

## Boundary

This slice begins the post-contract runtime layer by exposing registered TypeScript engines as live `contracts/v1` capability records. It derives each record from the actual sidecar registry and existing per-engine self-checks, so registered metadata is no longer the only runtime discovery surface.

## Behavior

- `GET /engines/capabilities` returns `{ capabilities, count }`.
- Each capability uses contract version `v1`, runtime kind `typescript`, display name, required phase, implementation version, and an explicit dependencies array.
- Registered engines with passing self-checks report `availability: "available"`.
- Registered engines with failing self-checks report `availability: "unavailable"` instead of being advertised as live.
- The endpoint does not call providers, generation APIs, databases, remotes, or deployment surfaces.

## TDD receipts

### RED

- `bun test tests/baseline_registry.test.ts tests/health.test.ts`: failed because `/engines/capabilities` returned `404`.

### GREEN

- `bun test tests/baseline_registry.test.ts tests/health.test.ts`: 9 passed, 0 failed.
- `bun run typecheck && bun test`: typecheck passed; 92 tests passed, 0 failed.

## Remote infra readback

All commands in this section were read-only. No deploy, config edit, variable read, secret read, database write, queue write, KV read, R2 object read, or remote mutation was performed.

### Railway

- CLI identity: `railway whoami` reported the Mage Narayan account.
- Local checkout state: `railway status` reported no linked project; explicit `--project` and `--environment` selectors were used instead of mutating local `.railway` linkage.
- Project: `robust-adventure` (`11eedde4-41e6-4f51-b86b-cf77111cf592`), workspace `871e554f-2c1b-4a0e-850a-d09019b4036d`.
- Environment: `production` (`702b945e-2c66-4d5a-bae1-4c67ea14c3bb`).
- Services read back as `SUCCESS` / `RUNNING`: `Selemene-engine`, `ts-engines`, `biofield-cv-service`, `suno-bridge`, `witness-agents`, `Postgres`, and `Redis`.
- Root service: `Selemene-engine`, repository `Sheshiyer/Selemene-engine`, config `/railway.toml`, Dockerfile `Dockerfile.prod`, healthcheck `/health/live`, active custom domain `selemene.tryambakam.space`, target port `8163`.
- TypeScript sidecar: `ts-engines`, repository `Sheshiyer/Selemene-engine`, root directory `/ts-engines`, config `/ts-engines/railway.toml`, Dockerfile `Dockerfile`, healthcheck `/health`, active domains `ts-engines-production.up.railway.app` and `ts-engines-production-56f0.up.railway.app`, target port `3001`.
- Python sidecar: `biofield-cv-service`, repository `Sheshiyer/Selemene-engine`, root directory `python-services`, config `/python-services/railway.toml`, Dockerfile `Dockerfile.biofield`, healthcheck `/health`.
- Runtime storage: Railway Postgres uses `postgres-volume` mounted at `/var/lib/postgresql/data`; Railway Redis uses `redis-volume-h8mJ` mounted at `/data`.
- Live deployment commit for root and TypeScript services: `b0827e1a6e870277e6b86cfc1ee8cfd2fe930709`, which predates this local capability slice.

### Cloudflare

- CLI profile: `wrangler --profile 9d9d` was used for resource reads.
- Identity readback: `wrangler whoami --json` reported OAuth login for `thoughtseedlabs@gmail.com` with account id `9d7cec1b5a32b2df8c6cdc1321ccd00b`; profile-targeted Worker calls addressed account `9d9d23b27f32e70ae3afb6a1aa2c0f10`.
- Worker `selemene-gw`: live deployment `8942c715-0cc0-4b35-9208-2ba552193504`, 100% version `fc97ef70-02e8-4ede-95fc-23a0f88b1752`, created `2026-07-27T20:33:16.18809Z`.
- Worker `selemene-llm-proxy`: live deployments exist; latest readback included 100% versions created between `2026-07-23T19:31:03.778563Z` and `2026-07-23T20:04:57.238249Z`.
- Worker `selemene-admin-api-proxy`: live deployments exist; latest readback included deployment `68f553c9-6086-41ed-8a1e-ea470a40ca31`, 100% version `546af0d2-b617-4a80-89d0-35502f39783d`, created `2026-07-06T13:47:34.861545Z`.
- Worker `selemene-pattern-memory`: declared in source, but `wrangler deployments list --name selemene-pattern-memory --profile 9d9d --json` returned Cloudflare error `10007` (`This Worker does not exist on your account`).
- KV namespaces present: `SELEMENE_SECRETS` (`7d11ab631a5145bfa8076546da6a9e27`), `LLM_SECRETS` (`310ee556e91d465eb54d55586a541e49`), and `LLM_SECRETS_preview` (`ab6f688eed234a0c9648920921d21ed7`).
- R2 buckets present with Selemene/Noesis relevance: `selemene-raga-clips` and `noesis-packs`.
- Vectorize index present with Selemene relevance: `witness-wisdom-corpus`; declared `SELEMENE_REPORT_PATTERNS` was not present in live `vectorize list` output.
- D1 databases present with Noesis/Witness relevance: `noesis-auth`, `witnessos-db`, and `witnessos-consciousness`; declared `selemene-patterns-d1` was not present in live `d1 list` output.
- Cloudflare Pages projects in the profile were Thoughtseed/Urania/WitnessOS projects; no Selemene Pages project was observed in the `pages project list` readback.
- Cloudflare Queues in the profile were `teamforge-sync`, `teamforge-sync-dlq`, `wtfmedia-ingest`, and `wtfmedia-ingest-dlq`; no Selemene queue was observed.

### Live health

- `https://selemene.tryambakam.space/health/live`: `ok`, version `3.3.1`, 19 engines loaded, 6 workflows loaded.
- `https://selemene.tryambakam.space/health/ready`: Redis and Postgres `ok`, orchestrator `ready`, bridge `available`, bridge engines healthy for tarot, i-ching, enneagram, sacred-geometry, sigil-forge, and raaga.
- `https://ts-engines-production.up.railway.app/health`: `healthy`, six engines listed, version `1.0.0`.
- `https://ts-engines-production.up.railway.app/health/ready`: six bridge engines healthy, no failed engines.
- `https://ts-engines-production.up.railway.app/engines/capabilities`: `404` on live deployment, confirming this local endpoint is not yet deployed.

## Slice 2: Rust/API and Python sidecar capability parity (2026-08-31, Task 3)

Date: 2026-08-31. Branch: `codex/selemene-task3-task4-capability-parity`, stacked on `codex/selemene-runtime-capability-endpoint` (PR #1486). Scope: extend the `contracts/v1` capability-discovery surface from TypeScript-only to Rust/API and Python sidecars, closing two of the boundaries this document originally left open. No provider, generation-API, database, or remote calls were added.

### Behavior

- `GET /api/v1/admin/engines/capabilities` (new, `noesis-api`) returns a JSON array of `contracts/v1` `EngineCapability` records for the bridge-proxied TypeScript engines, gated behind the same `admin:system:read` permission as the existing `/admin/bridge/health` route. Availability is derived from `state.bridge().readiness_status()` — the identical self-check data source `/admin/bridge/health` already uses — with no new provider/database/remote calls.
- `crates/noesis-core::contract::EngineCapability`, `CapabilityAvailability`, and `RuntimeKind` (already defined pre-existing this session) are reused, not redefined.
- `python-services/shared/models.py` adds `HealthResponse.capability_status: Literal["available","degraded","unavailable"]`, computed only from each sidecar's existing local self-check booleans:
  - `biofield-cv`: `available` iff both `opencv` and `numpy` are up; `degraded` if `opencv`+`numpy` are up but `mediapipe` is missing; `unavailable` if `opencv` or `numpy` is missing.
  - `mediapipe-face-mesh`: `available` if `mediapipe` is up, else `unavailable`.

### TDD receipts

**RED**
- `cargo test -p noesis-api --test capability_route_tests --locked`: 3 tests failed against the not-yet-existing `/api/v1/admin/engines/capabilities` route.
- `python3 -m pytest python-services/tests/test_capability_health.py -q`: failed, `capability_status` field absent from `/health`.

**GREEN**
- `cargo test -p noesis-api --test capability_route_tests --locked`: 3 passed, 0 failed (independently re-run this session).
- `PATH="$PWD/python-services/.venv/bin:$PATH" python3 -m pytest python-services/tests -q`: 61 passed, 0 failed, 1 pre-existing unrelated deprecation warning (independently re-run this session, matching the 53-prior + 8-new count reported by the implementing pass).
- `pnpm run gate:contracts`: passed.
- `cargo build --workspace --locked`: no regressions across the Rust workspace.

Every claim above was independently reproduced in a fresh check this session (not just accepted from the implementing pass) before this doc was written.

## Slice 3: Tarot provenance/confidence truth surface (2026-08-31, Task 4, partial slice against #1461)

Date: 2026-08-31. Same branch as Slice 2. Scope: **partial** slice toward GitHub issue [#1461](https://github.com/Sheshiyer/Selemene-engine/issues/1461) (`[W3E:tarot:07] Expose provenance, confidence, and degradation`) — Task 4's own execution steps (one missing-state test, then the minimal truth surface) intentionally do not cover #1461's full six-axis, multi-fixture acceptance criteria. Tarot was selected as the pilot engine because it is the only Task-4 candidate with its full W3E slot set (`06,07,09,10,17,18,25,27,28`) open as distinct issues and it already reports `available` on the capability endpoint.

### Behavior

- `ts-engines/src/types/engine.ts`: `ContractProvenance` gains an optional `confidence?: number` field (0–1).
- `ts-engines/src/engines/tarot/engine.ts`: `TarotEngine.calculate()` now populates `EngineOutput.provenance` with `runtime_kind: 'typescript'`, `implementation_version` (engine metadata version), `cached: false`, `fallback_used: false`, `confidence: 1` — accurate as-is because tarot's interpretation text comes entirely from the local deterministic `wisdom.ts` data with no fallback/generated-text path today.
- No other engine's output changed.

### TDD receipts

**RED**: `bun test tests/tarot_provenance.test.ts` failed — `output.provenance` was `undefined`.

**GREEN**
- `bun test tests/tarot_provenance.test.ts`: passed.
- `bun test tests/integration.test.ts tests/baseline_registry.test.ts && bun run typecheck`: passed, no other engine's behavior changed.
- Full suite, independently re-run this session: `bun run typecheck && bun test` → typecheck clean, **93 passed, 0 failed** (up from the 92/0 baseline recorded in Slice 1 — exactly the one new provenance test, nothing else moved).

### What #1461 still needs (explicitly not covered by this slice)

- Confidence/degradation for the other five engines' evidence axes described in #1461's acceptance criteria (positive/boundary/negative/degraded fixtures, bridge/API/SDK/CLI compatibility probes, six-axis evidence table).
- Any actual fallback path for tarot — none exists today, so `fallback_used` is trivially always `false`; if a fallback path is later added, this field must be revisited.
- W3E slots `06`, `09`, `10`, `17`, `18`, `25`, `27`, `28` for tarot, and slot `07` (this pattern) for the other six Task-4 candidate engines (biofield, face-reading, raaga, sigil-forge, i-ching, sacred-geometry) — all remain open follow-up work.

## Slice 4: Consumer surface (2026-09-27)

Date: 2026-09-27. Branch: `codex/selemene-capability-consumer-surface`, HEAD `23f48d28`. Scope: Phase A of the capability-parity consumer rollout — align the contract's `confidence` field, thread real provenance through the bridge, expose a public capability-discovery route, and update the two SDKs plus the vendored sankalpa consumer tarball. Tasks 1–4 covered the four items below; this task (5) packs the tarball and records this evidence.

### Behavior

- **A1 — `confidence` alignment (commit `5d7d682b`):** `confidence` added to `contracts/v1/schemas/provenance.schema.json` (min 0, max 1), the `engine-result.json` fixture, Rust `Provenance.confidence: Option<f64>`, and both SDKs' `ContractProvenance`.
- **A2 — bridge provenance pass-through (commits `8a4127cd`, `54d088de`):** bridge `TsEngineResponse.provenance: Option<Value>` is stashed under the reserved key `__contract_provenance` (`noesis_bridge::CONTRACT_PROVENANCE_KEY`). `ApiEngineOutputResponse::from` lifts only that key when present; otherwise it derives `Provenance` from `CalculationMetadata` (native/typescript/python from backend, engine_version, cached, fallback_used=false). A `tracing::warn!` fires on contract parse failure. Engine-owned `result["provenance"]` (engine-financial-biosensor) is left intact.
- **A3 — public capability route (commit `c24e5306`):** new `crates/noesis-api/src/capabilities.rs::collect_capabilities`; public `GET /api/v1/engines/capabilities` added to the auth-middleware group, returning `{capabilities,count}` with native rows (`available`, runtime_kind native) plus bridge rows (readiness-derived). The admin route now delegates to the same collector.
- **A4 — SDK methods and versions (commit `23f48d28`):** `packages/noesis-engine-sdk` bumped to 0.2.0 with `EngineClient.capabilities()`; `packages/noesis-sdk-ts` bumped to 3.4.0 with `listCapabilities()` and new `EngineCapability`/`EngineCapabilitiesResponse` types; `PythonSidecarHealthResponse.capability_status?` added.
- **A5 (this task):** vendored tarball rebuilt for sankalpa at `/Volumes/madara/2026/Projects/tryambakam-noesis/sankalpa/vendor/selemene-engine-sdk-0.2.0.tgz` from `packages/noesis-engine-sdk` 0.2.0. The prior `selemene-engine-sdk-0.1.0.tgz` was left in place (not deleted). Nothing was staged or committed in the sankalpa repo — that is a later task.

### TDD receipts (exact numbers copied from the completed tasks)

**RED**
- A1: `cargo test -p noesis-core --test contract_v1_authority canonical_result_provenance_round_trips_confidence --locked` — compile error, `no field \`confidence\`` on `Provenance`.

**GREEN**
- A1: `contract_v1_authority` 7/7 passed; `noesis-engine-sdk` bun 36/36 passed + typecheck clean; `noesis-sdk-ts` vitest 11/11 passed + typecheck clean; `pnpm run gate:contracts` green.
- A2: `noesis-bridge` 41+1 passed; `test_calculate_*` 17 passed; `openapi_schema_tests` 4 passed; 3 new `noesis-api` unit tests passed; `cargo build --workspace --locked` clean; `pnpm run gate:contracts` green.
- A3: `capability_route_tests` 5/5 passed; `route_inventory` 1/1 passed (snapshot `docs/baseline/api-route-inventory.json` regenerated, +2 routes); `noesis-api` lib 97 passed; workspace build clean; `pnpm run gate:contracts` green. RED for this task was the route returning `404` before the handler existed.
- A4: `engine-sdk` 39/39 passed + typecheck clean; `noesis-sdk-ts` 12/12 passed + typecheck clean; `pnpm run gate:contracts` green.

### A5 tarball verification (this task, run this session)

- `cd packages/noesis-engine-sdk && pnpm build` — `tsc -p tsconfig.build.json`, clean.
- `pnpm pack --pack-destination /Volumes/madara/2026/Projects/tryambakam-noesis/sankalpa/vendor/` produced `selemene-engine-sdk-0.2.0.tgz` directly (no rename needed) — the pack name follows the package name `@selemene/engine-sdk`.
- `tar -tzf selemene-engine-sdk-0.2.0.tgz` confirmed `package/dist/client.js` is present.
- `tar -xzOf selemene-engine-sdk-0.2.0.tgz package/dist/client.js | grep -c capabilities` → `3` (> 0).
- `ls -la /Volumes/madara/2026/Projects/tryambakam-noesis/sankalpa/vendor/` showed both `selemene-engine-sdk-0.1.0.tgz` (20378 bytes, untouched) and the new `selemene-engine-sdk-0.2.0.tgz` (22757 bytes).
- **Note:** sankalpa is tombstoned (R4, 2026-07-28), so this tarball is a port baseline for a dead consumer, not a release artifact for an active one — it exists so a future reactivation or port has a known-good 0.2.0 vendor snapshot to start from.

### Remaining boundaries (Slice 4)

- Consumer wiring — sankalpa (or any other consumer) actually importing `capabilities()`/`listCapabilities()` and reading `capability_status` — is Phase B of the plan, not this slice. Nothing was deployed, and no consumer repo's code or git state was touched by this task.

## Remaining boundaries

- ~~Native Rust/API runtime capability adoption remains a separate slice.~~ **Closed by Slice 2** above (2026-08-31) for the bridge-proxied capability-discovery surface specifically; native (non-bridge-proxied) Rust engines, if any are added later, are still a separate concern.
- ~~Python/database-conditional capability reporting remains a separate slice.~~ **Partially closed by Slice 2**: Python sidecar local self-check capability status is done. `database-conditional` `RuntimeKind` reporting (for any future DB-backed engine) remains open — nothing in this repo currently needs it.
- Per-engine semantic completion repair remains a separate slice, now begun (partially) for tarot slot `07` only — see Slice 3.
- GitHub Actions immutable SHA pinning (`ISC-217`) remains open.
- Consumer wiring for the capability surface (Phase B: sankalpa and other repos actually consuming the new SDK methods) remains open — see Slice 4.
- No push, publication, deployment, or remote mutation to GitHub issues beyond what was explicitly authorized occurred in Slices 2–4; no deploy occurred.

## Consumer rollout receipts (2026-09-27)

Cross-repo verification of the capability-parity consumer rollout. Each repo was already checked out on its own branch; nothing was checked out or modified by this receipt except this doc. All gate commands were run once, in place, on the branch/SHA listed, and results below are copied verbatim from the actual runs.

### Selemene-engine — `codex/selemene-capability-consumer-surface` @ `53c6e331`

- HEAD confirmed: `53c6e331` (matches).
- `pnpm run gate:contracts` — exit 0. Rust `integration_tests`: 17 passed, 0 failed. `noesis-engine-sdk` bun test: 39 pass, 0 fail (119 expect() calls); typecheck clean. `witness-pipeline` build clean. `noesis-sdk-ts` vitest: 2 files, 12 passed; typecheck clean.
- `cargo build --workspace --locked` — exit 0. `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 30.36s`.
- `(cd ts-engines && bun run typecheck && bun test)` — typecheck exit 0 (`bunx tsc --noEmit`, clean). `bun test`: 93 pass, 0 fail (292 expect() calls) across 7 files.
- Result: **all gates green**.

### urania-137 — `codex/urania-capability-parity` @ `437b55e`

- HEAD confirmed: `437b55e` (matches).
- No `npm run typecheck` script exists; ran the brief's substitute.
- `npm test` (vitest) — exit 0. `Test Files 118 passed (118)`, `Tests 984 passed (984)`.
- `npx tsc --noEmit -p tsconfig.json` — exit 0, no output.
- `npm run typecheck:functions` (`tsc -p tsconfig.functions.json --noEmit`) — exit 0, no output.
- Result: **all gates green**.

### noesis-raycast — `codex/raycast-capability-parity` @ `604c212`

- HEAD confirmed: `604c212` (matches).
- `npm test` (`bun test --tsconfig-override tsconfig.test.json`) — exit 0. `83 pass, 0 fail` (9 expect() calls) across 13 files. (Bun printed one internal directory-mismatch warning for `tsconfig.test.json`, explicitly marked by bun as harmless — "You don't need to do anything, but this indicates a bug" — did not affect the pass count.)
- `npm run lint` (`ray lint`) — exit 2. ESLint: `✖ 13 problems (13 errors, 0 warnings)`, all `@typescript-eslint/no-unused-vars` across `src/components/daily-ritual-command.tsx` (3), `src/components/execution-forms.tsx` (4), `src/dashboard.tsx` (2), `src/developer-tools.tsx` (2), `src/lib/design-tokens.ts` (1), `src/menubar.tsx` (1). Prettier then reported code-style issues in 6 files (`access-gate.tsx`, `daily-ritual-command.tsx`, `execution-forms.tsx`, `onboarding-form.tsx`, `design-tokens.ts`, `menubar.tsx`). Per the task brief, full `npm run lint` has pre-existing failures on base `d7a671d`; this 13-error/0-warning ESLint count plus the 6-file Prettier list is recorded here as **pre-existing**, not introduced by the capability-parity branch — not fixed as part of this receipt task per the brief's scope (record verbatim, do not fix).
- Result: tests green; lint pre-existingly red (13 errors, recorded above).

### antahkarana — `codex/antahkarana-capability-parity` @ `032f12d`

- HEAD confirmed: `032f12d` (matches).
- `bun run test:containment` — exit 0. Rust unit tests: `test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`. `main.rs` unittests: 0 passed. Doc-tests: 0 passed.
- `bun run check:truth` — exit 0. `TRUTH TABLE: PASS — 95 checks, every revived surface's declared source resolves, no mocks, honest states present.`
- `bun run check:secrets` (`./scripts/check-secrets.sh`) — exit 0. `SECRETS GATE: PASS — no nk_-prefixed literals in repo`.
- `bunx tsc --noEmit` — exit 0, no output.
- Result: **all gates green**.

### sankalpa — `codex/sankalpa-capability-parity` @ `c05e24e`

- HEAD confirmed: `c05e24e` (matches).
- `npm test` (vitest) — exit 1. `Test Files 1 failed | 11 passed (12)`, `Tests 1 failed | 101 passed (102)`. The one failure is `tests/desktop-foundation.test.ts > desktop foundation assets > has explicit logo and icon assets for runtime and packaging`: `expect(existsSync(path.join(root, "build/icon.svg"))).toBe(true)` — `AssertionError: expected false to be true`. This matches the brief's flagged pre-existing failure (`build/icon.svg` deleted in `ca3a0f2`) and is recorded here as **pre-existing**, not introduced by the capability-parity branch.
- `npm run typecheck` (`tsc --noEmit -p tsconfig.json && tsc --noEmit -p tsconfig.electron.json`) — exit 0, no output.
- Result: typecheck green; one pre-existing test failure (icon asset, recorded above).

### noesismirror-web — `codex/noesismirror-capability-parity` @ `244346d`

- HEAD confirmed: `244346d` (matches).
- `npm test` (vitest) — exit 0. `Test Files 1 passed (1)`, `Tests 2 passed (2)`.
- Result: **all gates green**.

### Live readback

- `curl -s -o /dev/null -w '%{http_code}\n' https://selemene.tryambakam.space/api/v1/engines/capabilities` (no API key sent) → `404`. Per the brief, `404` means Phase A is not yet deployed and all consumers are currently running their fallback paths; this is expected at this point in the rollout, not a failure.

### Summary

- 6/6 repos verified at the branch/SHA specified in the task brief; all SHAs matched.
- Two pre-existing, out-of-scope failures were observed and recorded verbatim, not fixed: noesis-raycast full lint (13 ESLint errors + Prettier findings, pre-existing per brief), and sankalpa's `build/icon.svg` test (pre-existing per brief, asset deleted in `ca3a0f2`).
- All other gates (contracts, cargo build, ts-engines, urania-137 tests/typecheck, antahkarana containment/truth/secrets/typecheck, sankalpa typecheck, noesismirror-web tests) passed cleanly.
- Live capabilities route returns `404` (not yet deployed) as of this receipt.

### Final-review fix wave (2026-09-27)

- Selemene-engine `e76452b2`: `take_contract_provenance` strips the reserved `__contract_provenance` key before readings are persisted (calculate and workflow paths) and from workflow responses; `capabilities.rs` caches the readiness-derived availability map for `CAPABILITY_READINESS_TTL` (10s), caching both Ok and Err. Receipts: `cargo test -p noesis-api --lib` 103 passed; `capability_route_tests` 5 passed; `integration_tests test_calculate_` 17 passed; workspace build, `cargo fmt --check`, `pnpm run gate:contracts` clean.
- antahkarana `cee7b52`: dead Rust `availability_to_roster_status` removed (it contradicted the TypeScript unknown→`declared`→`idle` mapping); `src/features/roster/model.test.ts` added (8 passing) covering parser, unknown availability, missing `engine_id`, null/404 bodies, and the 404→health-derived fallback. `test:containment` 41 passed; `check:truth` 95 checks; `check:secrets` PASS; `tsc --noEmit` clean.
- Scoped re-review: all three final-review findings ADDRESSED, no new Critical/Important breakage.

### Witness-pipeline consumer probe — two-subject business-partners L2 (2026-09-27)

Purpose: exercise the capability/provenance surface and the witness pipeline end to end with a real two-subject, relationship-aware run (`business-partners`, L2, register `l1_l3`, language `en`). Subjects were the two Thoughtseed co-founders already present in `tests/fixtures/humdes`. Birth data and the reading itself are deliberately not recorded here.

- Engine data: 9 deterministic engines × 2 subjects fetched from `https://selemene.tryambakam.space` (18/18 ok). Production results carry `metadata.backend` but `provenance` is absent — production predates this branch, consistent with the live `404` on `/api/v1/engines/capabilities` recorded above.
- Note for scripted fetchers: the production edge returns `403` to Python's default `User-Agent`; the same requests succeed with a curl-style agent. Not an API-key or quota problem.
- Local branch server (`target/debug/noesis-server`, `RUST_ENV=development`, no Postgres): `/health` ok, 18 engines, 6 workflows; `/api/v1/engines/capabilities` and engine calculate both answer `401` without credentials, as the route tests expect. Authenticated local capture was not performed in this session (no dev API key available without admin). `cargo test -p noesis-api --test capability_route_tests`: 5 passed.
- ts-engines (`bun run src/index.ts`, port 3001): `GET /engines/capabilities` → 200, 6 rows (`tarot`, `i-ching`, `enneagram`, `sacred-geometry`, `sigil-forge`, `raaga`), all `contract_version: v1`, `availability: available`, `runtime_kind: typescript`.
- Pipeline gap found and closed: `IntegratedReadingOrchestrator` never injected engine results into any pass prompt (they were only used for the subject-0 rubric), so live readings ran on names alone. Added `src/orchestrator/engine-facts.ts` (per-subject deterministic facts block, `{{engine_facts}}` placeholder with append fallback) and audit the rubric against all subjects' results. New tests: 3 passing. `renderLocalArtifacts` gained an optional `title`.
- New runner: `packages/witness-pipeline/scripts/business-partners-l2-runner.ts` (stub or `--live`, `--source prod|local`).
- Live run (OpenRouter `anthropic/claude-sonnet-4`; `command-code` had no credits, NVIDIA `meta/llama-3.1-8b-instruct` is EOL since 2026-08-26 and should be replaced in `scripts/lib/multi-llm.ts`): 4 passes, 2,904 words, Folio header at top, all rubric gates pass on every pass (word fit, deterministic facts, layering, guardrails), chart fidelity 0.5 on every pass, source-pack gate `ready` (18 facts), final verification PASS, no romantic or predictive language.
- Pre-existing, unchanged: 3 witness-pipeline tests fail because they hardcode `/Volumes/madara/2026/twc-vault/01-Projects/.../brand-config.yaml`, which now lives only under `04-Archives/...` and is itself malformed YAML (duplicate mapping keys). Rendering for this probe used a trimmed copy of that file's valid head.
- Jev layer (same day, follow-up): typed-judgment pass gate added to witness-pipeline in shadow mode, ported from urania-137 v0.7.1 and the factor 003-jev contract; see `docs/plans/selemene-engine/JEV-GATE.md`. 13 new tests pass. Field Theory resync: 456 bookmarks, 0 new; 12 Jev records. Live shadow run followed once the key was located (stored as `API_KEY` under a `####JEV-TYPESAFE-AI####` header in `~/.claude/.env`; loader now resolves that form; 14 client/gate tests): 4/4 passes judged on `jev-1.13.0` in 274 to 386 ms each, framing and grounding confident and agreeing with the regex rubric, guardrail flagged 2 of 4 sections as predictive where the regex passed (confirmed by inspection), register question never cleared 0.5 confidence. Details in `JEV-GATE.md`.
- Jev retry loop + all-modes matrix (same day): 9 of 10 mode docs ran live on real engine data with Jev shadow and one retry; 40/40 framing and grounding passes; 12/14 retries improved the guardrail score; business-partners rerun left no confident guardrail failure; L0 kundali is predictive by design (28/40 Master Timeline sentences); `partner-synastry.md` fails to parse (legacy frontmatter). One JSON-cycle defect in retry receipts found and fixed with a test. Table and readings in `JEV-GATE.md`.
- Mode-aware guardrail + voice rules + lessons wiring + partner-synastry migration (same day): business-partners first-draft guardrail 0.38–0.66 → 0.81–0.91 with no retry; L0 kundali under `forecast-allowed` 0.07–0.51 → 0.72–0.90; partner-synastry parses and runs (4 passes). Found and fixed: `{{lessons_summary}}` was declared by 1 of 10 templates, so mode lessons reached no prompt; now appended when absent. Tests 130 pass, 3 pre-existing fail. Details in `JEV-GATE.md`.
