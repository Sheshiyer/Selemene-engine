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
- Identity readback: `wrangler whoami --json` reported OAuth login for a different default account; its account id was `9d7cec1b5a32b2df8c6cdc1321ccd00b`; profile-targeted Worker calls addressed account `9d9d23b27f32e70ae3afb6a1aa2c0f10`.
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

## Remaining boundaries

- ~~Native Rust/API runtime capability adoption remains a separate slice.~~ **Closed by Slice 2** above (2026-08-31) for the bridge-proxied capability-discovery surface specifically; native (non-bridge-proxied) Rust engines already exist and remain a separate, unfinished capability-discovery concern.
- ~~Python/database-conditional capability reporting remains a separate slice.~~ **Partially closed by Slice 2**: Python sidecar local self-check capability status is done. `database-conditional` `RuntimeKind` reporting remains open for the existing `biofield-capture` engine. The scalar Python health status does not provide a full canonical capability-record inventory.
- Per-engine semantic completion repair remains a separate slice, now begun (partially) for tarot slot `07` only — see Slice 3.
- GitHub Actions immutable SHA pinning (`ISC-217`) remains open.
- No push, publication, deployment, or remote mutation to GitHub issues beyond what was explicitly authorized occurred in Slices 2–3; no deploy occurred.
