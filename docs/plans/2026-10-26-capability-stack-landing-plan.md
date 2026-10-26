# Capability stack landing plan (PRs #1486, #1487, #1491)

Date: 2026-10-26. Base: `origin/main` at 72f65e25 (post somatic lanes S1–S6).

## 1. Finding: the stack is mostly already on main

The three PRs are stacked (#1491 → #1487 → #1486 → main) and each is 43 commits behind main. A direct rebase is the wrong move because main already contains most of the lower two, carried in by #1489 (2026-09-08) and the somatic lanes:

| Piece | PR that introduced it | On main today |
|---|---|---|
| TS `GET /engines/capabilities` (registry-derived, self-check availability) | #1486 | yes (`ts-engines/src/server/app.ts:130`), 53 residual lines differ |
| `baseline_registry.test.ts`, `health.test.ts` | #1486 | yes, 7 residual test lines |
| Admin `GET /api/v1/admin/engines/capabilities` + `capability_route_tests.rs` | #1487 | yes, 77 + 169 residual lines |
| Sidecar `capability_status` in both `/health` | #1487 | yes (S1 rewrote mediapipe health; 19 residual lines) |
| Tarot `provenance` + test | #1487 | yes, identical |
| `provenance.confidence` in Rust contract and engine-sdk types | #1491 | yes (S4 and #1506), but **not** in `contracts/v1/schemas/provenance.schema.json` |
| Bridge carries `__contract_provenance`; API lifts and strips it | #1491 | yes (S4: `take_contract_provenance`, `strip_workflow_contract_provenance`) |
| Public `GET /api/v1/engines/capabilities` (API-key surface, native + bridge rows, 10 s readiness cache) + `capabilities.rs` collector | #1491 | **no** |
| engine-sdk `capabilities()` 0.2.0, noesis-sdk-ts `listCapabilities()` 3.4.0 | #1491 | **no** (engine-sdk is 0.1.0, noesis-sdk-ts 3.3.1) |
| `contract_v1_authority.rs`, fixture `engine-result.json` with confidence | #1491 | **no** |

Also: #1491's branch carries four commits (e2ca2ce0, 3e9ecb61, cad2a6b7, f1ba0180 "wip … paused at 3/6") that are witness-pipeline work, 70 files, unrelated to capabilities. They must not ride along.

## 2. Decision: rebuild as one PR, close the stack as superseded

- New branch `capability/consumer-surface-v2` from `origin/main`.
- Cherry-pick only the capability commits of #1491, in order: 5d7d682b, 8a4127cd, 54d088de, c24e5306, 23f48d28, e76452b2 (skip the docs-only ones 7b043097, fb745f61, 53c6e331, 4a20a01b, eba705df; re-write their receipts fresh). Expect conflicts in `crates/noesis-api/src/lib.rs`, `crates/noesis-bridge/src/lib.rs`, `crates/noesis-core/src/contract.rs`, `packages/noesis-engine-sdk/src/contract-v1.ts`: resolve by keeping main's lift/strip implementation and taking only the public route, collector, SDK methods, schema and fixtures from the picks.
- Then diff the residuals of #1486/#1487 against main (listed above) and apply only what is still missing: extra registry/admin test cases and the `ts-engines/src/types/engine.ts` additive fields. Nothing else.
- Split the four witness-pipeline commits onto `witness/reading-repair-wip` for a separate decision (they include a WIP commit; not for main as is).
- Close #1486 and #1487 with a comment pointing at the v2 PR and the table above. Close #1491 the same way once v2 is open.

Lock zones: `crates/noesis-api/**`, `crates/noesis-bridge/**`, `crates/noesis-core/**`, `contracts/v1/**`, `packages/noesis-engine-sdk/**`, `packages/noesis-sdk-ts/**` → the v2 lane only. `ts-engines/**` → additive only. No consumer repos.

Provenance rule still applies: exactly seven fields; `deny_unknown_fields` drops the block otherwise (S4 added tests for this; keep them green).

Scope rule: the public capabilities route sits in the same auth group as `GET /engines` (basic:access), not behind an engine scope. Staging runs `ENGINE_SCOPE_MODE=enforce`, which proves this.

## 3. Verification tiers (no GitHub Actions)

T1 local (in the v2 worktree):
- `cargo test -p noesis-api --locked --test capability_route_tests --test integration_tests --test p4_media_engines_tests --test fixture_snapshot_tests --test engine_scope_tests`
- `cargo test -p noesis-core --locked --test contract_v1_authority`
- `cargo test -p noesis-bridge --locked`
- `cd ts-engines && bun run typecheck && bun test`
- `cd packages/noesis-engine-sdk && bun test`; `cd packages/noesis-sdk-ts && bun test`
- route inventory check: `docs/baseline/api-route-inventory.json` gains the public route (the inventory test fails otherwise).

T2 staging (Railway project 11eedde4, env 3397696a), built from the lane branch via staged source switch + accept-deploy, both `Selemene-engine` and `ts-engines`:
1. `GET /api/v1/engines/capabilities` with `ralph-s1-staging` → 200, `count` = 19, each row has contract version v1, runtime kind, availability; the six TypeScript rows report `available`; sidecar-backed rows reflect `/health/dependencies`.
2. Same call with no key → 401; with a key lacking basic:access → 403 (enforce mode).
3. Stop ts-engines briefly is NOT done; instead compare the bridge rows against `GET ts-engines/engines/capabilities` directly, they must agree.
4. Two calls within 10 s → second served from cache (log line or timing), a third after 10 s re-probes.
5. Regression: calculate sigil-forge with `generate_image:false` → seven-field provenance still lifted into the envelope and absent from `result`; `GET /api/v1/readings/{id}` for a persisted reading has no `__contract_provenance`.
6. Schema: validate `contracts/v1/fixtures/engine-result.json` against `provenance.schema.json` with confidence present and absent.
Receipt: `docs/plans/somatic-ralph-state.json` gains lane `c1-capabilities`.

T3 production: after merge, main deploy green, `/health/ready`, then probes 1 and 5 once with a scoped prod key (minted and revoked as before). SDK publish is a separate operator step.

## 4. Consumers (after T3)

Urania-137, noesis-raycast, Antahkarana and Sankalpa each have a consumer PR that degrades to `declared` on 404, so they can be reviewed in parallel but must bump to engine-sdk 0.2.0 / noesis-sdk-ts 3.4.0 only after those are published. One session per repo, read-only until the SDK versions exist.

## 5. Sessions

- C1 `capability/consumer-surface-v2` (fresh worktree from main): steps in §2 and T1, opens the PR. This session drives T2/T3 and merges.
- C0 `witness/reading-repair-wip`: branch split only, no PR until reviewed.
- Consumer lanes: after T3.

Estimated iterations: C1 3–4 (one for conflicts, one for tests, one for staging fixes).
