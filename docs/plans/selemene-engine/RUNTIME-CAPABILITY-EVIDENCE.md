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

## Remaining boundaries

- Native Rust/API runtime capability adoption remains a separate slice.
- Python/database-conditional capability reporting remains a separate slice.
- Per-engine semantic completion repair remains a separate slice.
- GitHub Actions immutable SHA pinning (`ISC-217`) remains open.
- No push, publication, deployment, or remote mutation occurred in this slice.
