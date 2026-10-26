# Lane N1: contract provenance for the 13 native engines

Date: 2026-10-27. Base: `origin/main` at dcc1a046.

## Problem

Only the six TypeScript bridge engines emit the seven-field contract `provenance` on the calculate envelope. The 13 native Rust engines emit none, so consumer badges (Sankalpa #18, Urania, Raycast) show nothing for them, and a native fallback is invisible to clients.

Staging sample, 2026-10-27, key `ralph-s1-staging`:

| Engine | `metadata.backend` | Envelope provenance | Notes |
|---|---|---|---|
| face-reading, no image | `birth-physiognomy-fallback` | none | engine-owned `result.provenance.fallback_used=false` with a `fallback_reason` set: inconsistent |
| biofield, birth data | `vedic-ephemeris` | none | engine-owned `result.provenance` (S2 shape, not the contract shape) |
| numerology | `native-rust` | none | |
| panchanga | `native-rust` | none | vedic provider fallback lives in `noesis-vedic-api` |
| nadabrahman | `native` | none | |

Seven engines were not sampled (staging key lacks their scopes); the lane audits them in code.

## Why not #1491's approach

#1491 derived a block from `CalculationMetadata` with `fallback_used: false` hard-coded. That would label face-reading's birth-data fallback and biofield's mock path as real. String-sniffing `backend` for "fallback" is fragile for the same reason.

## Design (recommended)

1. **Engines declare fallbacks explicitly.** Add to `noesis_core::CalculationMetadata`, both `#[serde(default)]` so stored readings still deserialize:
   - `fallback_used: bool`
   - `provider_id: Option<String>` (set only when an external provider computed the result, e.g. the vedic API)
   Add `impl Default`. Every construction site (about 45 across 13 engine crates, orchestrator, bridge, api, sdk, tui) sets `fallback_used` deliberately; no blanket `..Default::default()` in engine crates, so each engine's author decision is visible in review.
2. **Per-engine truth table** (the lane fills and tests this):
   - face-reading: `true` when no image or landmark hook failed (matches its `fallback_reason`), else `false`; also fix the engine-owned block so `fallback_used` agrees with `fallback_reason`.
   - biofield: `true` on the mock path, `false` for vedic and real capture (S2 already distinguishes these).
   - biofield-capture: `false` when a persisted reading is found, else error (no silent mock).
   - panchanga, vedic-clock, vimshottari, transits: `true` when the vedic provider fell back (read from `noesis-vedic-api`'s fallback result), `provider_id` set when the external API answered.
   - numerology, biorhythm, human-design, gene-keys, nadabrahman, financial-biosensor: deterministic local computation, `false`, unless the audit finds a fallback branch.
3. **API derives the envelope block for native engines only.** In `From<EngineOutput> for ApiEngineOutputResponse`, when no bridge block was lifted and the engine is native (decided from the orchestrator registry, the same source `capabilities.rs` uses, never from the backend string):
   `runtime_kind: native`, `implementation_version: metadata.engine_version`, `cached: metadata.cached`, `fallback_used: metadata.fallback_used`, `backend_id: metadata.backend`, `provider_id: metadata.provider_id`, `confidence`: omitted (no engine computes one yet; a later lane may add it).
   Bridge engines keep today's behaviour: a malformed sidecar block is still dropped, not replaced (`invalid_contract_provenance_is_dropped_not_leaked` stays green).
4. **Engine-owned `result.provenance`** (face-reading, biofield) stays for backward compatibility; the envelope block becomes the canonical one. Docs say so.
5. Workflow responses: per-engine outputs get the same derived block (they already pass through `strip_workflow_contract_provenance`).

## Lock zones

`crates/noesis-core/src/types.rs` (metadata fields), `crates/noesis-api/src/lib.rs` (envelope derivation), each `crates/engine-*/src` (its own flag), `tests/fixtures/expected_outputs/**` (snapshot refresh). No TypeScript, no contract schema change (the seven fields already exist), no consumer repos.

## Verification

T1 local:
- unit test per engine crate for each fallback branch in the truth table;
- api unit tests: native success → block with `runtime_kind native`, `fallback_used false`; native fallback (face-reading no image) → `true`, `backend_id birth-physiognomy-fallback`; bridge malformed → still none; serialized block has exactly the seven fields and parses with `deny_unknown_fields`;
- `cargo test -p noesis-api --locked` (integration, capability_route, engine_scope, fixture_snapshot after refresh, p4_media), `cargo test --workspace --locked` for the engine crates;
- stored-reading deserialization test: a reading JSON without the new fields still loads.

T2 staging (built from the lane branch as before); staging key widened to `engines:*` for the run:
1. all 13 native engines return an envelope `provenance` with the seven-field shape;
2. face-reading without image → `fallback_used true`; with the synthetic portrait → `false`, `backend_id mediapipe-face-cv`;
3. biofield without birth data or capture → `true`; with birth data → `false`;
4. a vedic engine with the provider forced to fail (staging-only env toggle, reverted after) → `true`;
5. sigil-forge (bridge) unchanged; `GET /readings/{id}` for a native reading carries no reserved key.

T3 production: merge, deploy, probes 1, 2 and 5 with a scoped key minted and revoked.

## Risks

- Snapshot fixtures change for every native engine: expected, reviewed in the diff.
- Stored readings: additive `serde(default)` fields; covered by the deserialization test.
- An engine with an undiscovered fallback branch would report `false`: mitigated by the per-crate audit and the explicit field at every construction site.

## Execution

One lane session (worktree from main), Rust only. This session reviews, drives staging and production, and writes the `n1-native-provenance` receipt. Estimated iterations: 3 (audit and fields, envelope and tests, staging fixes).
