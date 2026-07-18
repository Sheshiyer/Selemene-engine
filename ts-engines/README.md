# Noesis TS Engines

Bun + Elysia HTTP sidecar providing 6 TypeScript consciousness engines:

- raaga (Carnatic audio therapy, production-ready)
- sigil-forge (intention → method + optional image)
- tarot, i-ching, enneagram, sacred-geometry

Registered at runtime and reached from Rust via `noesis-bridge` (HTTP proxy).

**Port:** 3001 (override with `PORT` env)

## P1 W1 Context (Media Contracts Focus)

This setup enables end-to-end contract testing for the media/embodiment extensions in P1 Wave 1.

**References (MUST READ before work):**
- `docs/plans/engine-integration/p1-w1-worker-bootstrap-packet.md`
- `docs/plans/engine-integration/p1-w1-validation-gate-checklist.md`
- `docs/plans/engine-integration/EXECUTION-STATUS.md`
- `docs/plans/engine-integration/detailed-task-list.md`
- The three extraction files (anti-drift):
  - `resources-and-assets.md` (existing raaga/sigil strength)
  - `gaps-and-improvements.md` (media contracts, provider abstraction, capture lifecycle missing)
  - `goal-understanding.md` (two-prong, local-first + explicit consent, focus 4 engines first)
- Main plan: `selemene-sankalpa-full-integration-swarm-plan.md`
- Engine specs: `docs/engines/{raaga,sigil-forge}.md`
- Contract work (T-002..): `crates/noesis-core/src/types.rs` (media extensions for image_data, audio_ref, consent_token, generated outputs, etc.)

**Do not implement engine logic here.** This is for runnable dev environment so contracts (EngineInput/Output media shapes, raaga audio output, sigil image via providers) can be roundtripped and validated.

Local-first + consent invariant (from goal-understanding.md + Sankalpa ISA):
- These are **backend** services.
- Sankalpa (desktop) owns safe capture, explicit opt-in UI, local preview (e.g. PIP for biofield), consent_token before any network call.
- Never auto-upload; backend escalation only on user consent.
- No secrets in client paths.

## Prerequisites
- Bun >= 1.0 (`bun --version`)
- (optional for full) Rust noesis-api for bridge tests

## Setup & Run (TS Server)

```bash
cd ts-engines

# Install (once)
bun install

# Run (watch for dev)
bun run dev

# Or production mode
bun run start

# With custom port
PORT=3002 bun run dev
```

On start you see the banner with registered engines and endpoints.

## Verification Commands (for contract testing)

```bash
# Health (liveness + engines)
curl -s http://localhost:3001/health | jq
curl -s http://localhost:3001/health/live
curl -s http://localhost:3001/health/ready

# List engines
curl -s http://localhost:3001/engines | jq

# Example raaga calculate (extend with audio_ref / strudel fields per T-005 once frozen)
curl -X POST http://localhost:3001/engines/raaga/calculate \
  -H 'Content-Type: application/json' \
  -d '{
    "consciousness_level": 2,
    "parameters": { "melakarta": 1, "dosh": "vata" }
  }' | jq

# Sigil (image path will use provider abstraction per T-003)
curl -X POST http://localhost:3001/engines/sigil-forge/calculate \
  -H 'Content-Type: application/json' \
  -d '{
    "consciousness_level": 2,
    "parameters": { "intention": "I witness my patterns clearly", "method": "rose-wheel" }
  }' | jq
```

Run tests (exercises server + contracts):
```bash
bun test
```

See `tests/integration.test.ts` for calculate roundtrips.

## Integration with noesis-api / Bridge (for full e2e)

The Rust noesis-api (and bridge) proxies TS engines at `TS_ENGINES_URL` (default http://localhost:3001).

See:
- `crates/noesis-bridge/src/lib.rs` (BridgeEngine::raaga, sigil etc.)
- `crates/noesis-api/src/handlers/` for engine routes

To test full:
```bash
# Terminal A: ts-engines
cd ts-engines && bun run dev

# Terminal B: (requires DB for full api; use --no-default-features or mocks for contracts)
export TS_ENGINES_URL=http://localhost:3001
cargo run --bin noesis-server
# then POST to http://localhost:8080/api/v1/engines/raaga/calculate
```

## Python Biofield Sidecar (for biofield capture contracts T-004)

See sibling `python-services/README.md` (and full `docs/PYTHON_SIDECAR_GUIDE.md`).

Biofield CV (port 8002) produces the 11-metric authoritative shape for capture lifecycle (requested/uploaded/analyzed/persisted + quality + consent).

It is called from `crates/noesis-api/src/biofield_client.rs` (via `PythonServiceClient` in bridge) when `PYTHON_BIOFIELD_URL=http://localhost:8002`.

**Aligns with:** Sankalpa `biofieldDomain.ts` (local preview) + dual-path note in `resources-and-assets.md` and `gaps-and-improvements.md`. Do not conflate server CV with client local.

## Environment

- `PORT` (TS default 3001)
- No other secrets for basic contract smoke. Image providers (NVIDIA etc) for sigil are in `src/utils/nvidia-image.ts` (env for keys when testing real gen).

## For Sankalpa + Consent Testing

Use this server + python sidecar only after explicit consent in Sankalpa UI (camera/file/audio opt-in, consent_token in future EngineInput). See `goal-understanding.md` two-prong + local-first.

Local media contracts test here; full Sankalpa surfaces in P5.

## CI / Validation

These commands + `bun test` + health/ready must be green for P1 W1 gate (see validation checklist).

**P1 W1 post-gate verification (local run evidence):**
- `cd ts-engines && bun install && bun run typecheck || echo "pre-exist (see CI)"`
- `cd ts-engines && bun test` → 61 pass / 1 pre-exist timeout (sigil image no-key)
- Server: `bun run start` (PORT=3001) + curls for raaga (strudel_ratios present per T-005), sigil (provider scaffolding T-003)
- For full frozen media contracts (image_data, consent, generated_* on top-level EngineInput/Output): use worktree `cd .worktrees/T-002-copilot/ts-engines && bun run ...` (matches P1W1-CONTRACTS-FROZEN.md)
- Sample media input roundtrip (scaffolding; full in worktree): POST with parameters containing image_data-like for compat; see bootstrap-packet + resources-and-assets.md (raaga/sigil ready) + gaps-and-improvements.md (no prior e2e) + goal-understanding.md (two-prong). Full self-contained harness (4 engines, consent guards, curls, TS/sh): docs/plans/engine-integration/ext-contract-harness.md (T-024 fail-open)

Example evidence (2026-07-17 exec):
```
{"status":"healthy","engines":[... "raaga","sigil-forge"...]}
raaga calc: {"engine_id":"raaga", "result": {"melakarta":{"name":"Kanakangi"}, "strudel_ratios":[1,1.05..., ... ] , ... }}
sigil calc: {"engine_id":"sigil-forge", "result":{"method":{...}, "generated_image":null }, ...}
```
See .github/workflows/test.yml (enhanced for T-020 Wave2: ts-engines typecheck + P1W1 media contract smoke raaga/strudel + sigil via provider; consent samples) + p1-w1-validation-gate-checklist.md local dev section. Cites p1-w1-worker-bootstrap-packet.md + resources-and-assets.md + gaps-and-improvements.md + goal-understanding.md + P1W1-CONTRACTS-FROZEN.md + detailed-task-list.md + EXECUTION-STATUS.
 
**Last updated for P1 W1 + T-020 Wave2 start:** 2026-07-17 (post-gate #899 + T-020 enhance)
 
See EXECUTION-STATUS.md for current wave status. All work refs the 3 extraction files + FROZEN + bootstrap. Minimal per T-020.

