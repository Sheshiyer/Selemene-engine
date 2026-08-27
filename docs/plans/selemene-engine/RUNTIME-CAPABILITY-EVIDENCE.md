# Selemene Runtime Capability Evidence

Date: 2026-08-27
Branch: `codex/selemene-runtime-capability`
Scope: TypeScript sidecar, Rust/API, and Python sidecar runtime capability adoption plus a pinned local TypeScript verifier dependency; no engine algorithms, providers, credentials, databases, deployments, remotes, GitHub issues, or external services are changed by this source slice.

## Boundary

This slice begins the post-contract runtime layer by exposing registered TypeScript engines as live `contracts/v1` capability records. It derives each record from the actual sidecar registry and existing per-engine self-checks, so registered metadata is no longer the only runtime discovery surface.

The Rust/API slice adds a matching authenticated `contracts/v1` capability discovery surface for the main API router. It derives rows from the live orchestrator registry and classifies native Rust engines, TypeScript bridge engines, and database-conditional engines without invoking providers, sidecars, databases, or remotes.

The Python sidecar slice extends health responses with contract-v1 capability records derived from import/self-check state only. It keeps the Railway-facing health status as `healthy` or `degraded`, while capability availability carries the more precise `available`, `degraded`, or `unavailable` state.

## Behavior

- `GET /engines/capabilities` returns `{ capabilities, count }`.
- Each capability uses contract version `v1`, runtime kind `typescript`, display name, required phase, implementation version, and an explicit dependencies array.
- Registered engines with passing self-checks report `availability: "available"`.
- Registered engines with failing self-checks report `availability: "unavailable"` instead of being advertised as live.
- The endpoint does not call providers, generation APIs, databases, remotes, or deployment surfaces.
- `ts-engines` now declares and locks `typescript@5.6.3` so `bun run typecheck` resolves from the project lock instead of a floating `bunx` registry lookup.
- `GET /api/v1/engines/capabilities` returns authenticated main-API capability rows with contract version `v1`, display name, runtime kind, availability, dependencies, and required phase.
- Main-API native Rust engines report `runtime_kind: "native"` and `availability: "available"`.
- Main-API TypeScript bridge engines report `runtime_kind: "typescript"`, `availability: "declared"`, and `dependencies: ["ts-engines"]`.
- Main-API database-conditional engines report `runtime_kind: "database-conditional"` and `dependencies: ["postgres"]`.
- Python Biofield CV `/health` includes `capabilities` with `runtime_kind: "python"` and dependency checks for `opencv`, `numpy`, and `mediapipe`.
- Python Biofield CV reports capability `available` when all checks pass, `degraded` when optional MediaPipe is absent, and `unavailable` when required OpenCV or NumPy is absent.
- Python MediaPipe Face Mesh `/health` includes `capabilities` with `runtime_kind: "python"` and dependency check `mediapipe`.
- Python MediaPipe Face Mesh reports capability `available` when MediaPipe is present and `unavailable` when absent.

## HITL operational gate recovery

This operational step repaired the production smoke credential boundary after human authorization. The secret value was never printed, read back, or written to the repository.

- Railway project: `robust-adventure` (`11eedde4-41e6-4f51-b86b-cf77111cf592`), environment `production` (`702b945e-2c66-4d5a-bae1-4c67ea14c3bb`), service `Selemene-engine`.
- A fresh scoped smoke API key was created in production Postgres with name `ci-smoke-gh-actions-20260827T173428Z`.
- Local production smoke preflight against `https://selemene.tryambakam.space` passed: 7 checks, 0 failed.
- GitHub secret `SMOKE_TEST_API_KEY` was updated at `2026-08-27T17:34:53Z`.
- GitHub Actions run `33048319592` (`CD - Build & Deploy`) was rerun and live-read back as `status: completed`, `conclusion: success`, `headSha: ae3e2cef402bd0cf28d1c7a102800d215cc5f2c2`, `updatedAt: 2026-08-27T17:36:34Z`.
- The rerun `API Smoke Tests` job passed, including `Run API smoke runner` and `Verify mode contract on the deployed engine`.

## TDD receipts

### RED

- `bun test tests/baseline_registry.test.ts tests/health.test.ts`: failed because `/engines/capabilities` returned `404`.

### GREEN

- `bun install --frozen-lockfile`: passed; installed locked `typescript@5.6.3`.
- `bun run typecheck && bun test tests/baseline_registry.test.ts tests/health.test.ts`: typecheck passed; 9 tests passed, 0 failed.
- `bun test`: 92 tests passed, 0 failed.
- `cargo fmt --package noesis-api && cargo test -p noesis-api --test capability_route_tests`: formatted `noesis-api`; 1 test passed, 0 failed.
- `env PYTHONPATH="$PWD" uv run --no-project --python 3.11 --with fastapi --with pydantic python -c '...'`: direct Python sidecar health capability probe passed for Biofield available/degraded/unavailable and MediaPipe available/unavailable states.
- `env PYTHONPATH="$PWD" uv run --no-project --python 3.11 --with fastapi --with pydantic --with pytest --with httpx python -m pytest tests/test_biofield_health.py tests/test_mediapipe_health.py -q --noconftest`: 17 tests passed, 0 failed; 1 Starlette/TestClient deprecation warning.

### Broader local check

- `cargo test -p noesis-api`: compiled and passed the unit/OpenAPI/asset lanes reached before billing E2E; failed in `billing_e2e_tests` because all four webhook tests timed out connecting to local Postgres (`PoolTimedOut`). This failure is outside the new capability route path; rerun with a reachable local E2E Postgres before using the full package result as a release gate.
- `PYTHONPATH=. python -m pytest tests/test_biofield_health.py tests/test_mediapipe_health.py -q`: collection failed before tests because host Python had no `cv2` module. A later full `uv run --python 3.11 --extra dev ...` attempt was interrupted after slow CV wheel downloads left no usable FastAPI/CV environment; generated `.venv` and `uv.lock` artifacts were removed. The focused router-only health tests above verify this source slice without requiring the full CV analyze stack.

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
- Live deployment commit for root and TypeScript services after the gate-recovery merge: `ae3e2cef402bd0cf28d1c7a102800d215cc5f2c2`, which predates this local capability slice.

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

## Remaining boundaries

- Native Rust/API runtime capability adoption remains a separate slice.
- Python/database-conditional capability reporting remains a separate slice.
- Per-engine semantic completion repair remains a separate slice.
- GitHub Actions immutable SHA pinning (`ISC-217`) remains open.
- No push, publication, deployment, or remote mutation is included in this source slice.
