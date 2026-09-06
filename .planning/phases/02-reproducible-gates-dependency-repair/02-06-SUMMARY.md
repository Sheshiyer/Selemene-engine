---
phase: 02-reproducible-gates-dependency-repair
plan: "06"
subsystem: testing
tags: [registry, contracts, rust, typescript, python]

requires:
  - phase: 02-reproducible-gates-dependency-repair
    plan: "05"
    provides: exact-head green CI and verified strict CI Gate protection on main
provides:
  - versioned 19-row runtime engine registry with 17 public mirror groups
  - fail-closed registry validation and negative mutation fixtures
  - source enumeration parity for native, database-conditional and TypeScript registrations
affects: [02-07-release-receipts, phase-03-capability-contract-closure]

tech-stack:
  added: []
  patterns:
    - machine-readable runtime registry consumed by language-specific tests
    - database-conditional registration tested with a lazy disposable local fixture

key-files:
  created:
    - contracts/v1/registries/engines.json
    - ts-engines/src/server/__tests__/registry-authority.test.ts
  modified:
    - contracts/v1/manifest.json
    - scripts/validate_contracts.py
    - tests/scripts/test_validate_contracts.py
    - crates/noesis-core/tests/contract_v1_authority.rs
    - crates/noesis-orchestrator/src/lib.rs
    - ts-engines/src/server/registry.ts

key-decisions:
  - "The 19-row registry is the canonical identity, class, ownership, public grouping and evidence-axis authority."
  - "Database-conditional registration is proven by parsing a local lazy PostgreSQL fixture without connecting or reading ambient database configuration."
  - "Deployed and operational evidence remains qualified; Plan 02-07 is still required before GATE-05 can close."

patterns-established:
  - "Registry drift gate: Python validates shape and invariants while Rust and TypeScript compare real startup inventories."
  - "Evidence boundary: registry declaration never upgrades deployed or operational status."

requirements-completed: []
requirements-progressed: [GATE-05]

duration: 26min
completed: 2026-09-06
---

# Phase 02 Plan 06: Executable Registry Authority Summary

**A versioned 19-engine authority now fails the canonical gate on identity, grouping, class, owner, issue or evidence drift and is checked against real Rust and TypeScript registration paths.**

## Performance

- **Duration:** 26 min
- **Started:** 2026-09-06T10:23:51Z
- **Completed:** 2026-09-06T10:49:58Z
- **Tasks:** 2
- **Files modified:** 16 including execution evidence and state bookkeeping

## Source Precondition

Execution began from `41908e19c7cce32645d213add29886befb89eeca`, the committed Plan 02-05 evidence that exact-head CI was green and user-approved ruleset 15597830 required strict CI Gate with verified readback. Production promotion remained HOLD throughout Plan 02-06.

## Accomplishments

- Added `contracts/v1/registries/engines.json` with 19 unique runtime IDs, 17 public mirror groups, exact 12 native / 1 database-conditional / 6 TypeScript classes, stable owners, five reused issue IDs per engine and six evidence axes.
- Made `scripts/validate_contracts.py` reject missing or duplicate IDs, wrong grouping or class, missing owners, malformed issue mappings and incomplete evidence cells with actionable messages.
- Bound actual native, bridge and TypeScript startup enumeration to the registry and proved `biofield-capture` absent without database configuration and present with a local lazy fixture.
- Kept deployed and operational statuses evidence-qualified; no row, engine, wave or production promotion is closed from registry declaration alone.

## Task Commits

Each task was committed atomically:

1. **Task 1: Bind registry rows to current runtime evidence** — `351ae45` (feat)
2. **Task 2: Make registry drift fail the existing gate** — `b0f3095` (fix)

## Files Created/Modified

- `contracts/v1/registries/engines.json` — Canonical registry rows, counts, owners, public groupings, issue reuse and six-axis evidence.
- `contracts/v1/manifest.json` — Registers the engine authority with the v1 contract manifest.
- `scripts/validate_contracts.py` — Fails closed on registry structure and invariant drift.
- `tests/scripts/test_validate_contracts.py` — Positive authority check plus negative missing, duplicate, grouping, class, owner and evidence mutations.
- `crates/noesis-core/tests/contract_v1_authority.rs` — Verifies counts, owners, issue reuse, evidence bounds and conditional views.
- `crates/noesis-orchestrator/src/lib.rs` — Compares actual native/bridge registration with authority and exercises conditional registration using a lazy local pool.
- `ts-engines/src/server/registry.ts` — Exposes the same six-engine startup registration used by the server and tests.
- `ts-engines/src/index.ts`, `ts-engines/src/server/app.ts`, `ts-engines/src/server/index.ts` — Route server startup through the shared registration function.
- `ts-engines/src/server/__tests__/registry-authority.test.ts` — Compares actual TypeScript startup IDs and capability runtime kinds with authority.
- `ISA.md` and `02-VERIFICATION.md` — Record only the registry evidence proven by this plan and retain the remaining release gap.

## Verification Evidence

All database-sensitive commands ran with `DATABASE_URL` and `TEST_DATABASE_URL` absent or explicitly removed.

| Command | Result |
|---|---|
| `codegraph status .` and structural trace of registration symbols | Index up to date: 863 files, 14,909 nodes, 36,292 edges; traced native, bridge and API conditional registration paths |
| Task 1 JSON proof script | `ids=19 groups=17 classes={'native': 12, 'database-conditional': 1, 'typescript': 6} axes=6` |
| `python3 scripts/validate_contracts.py` | `contract authority v1 valid: schemas=6 fixtures=5 registries=1 engines=19` |
| `python3 -m pytest tests/scripts/test_validate_contracts.py -q` | 23 passed in 3.34s |
| `cargo test -p noesis-core --test contract_v1_authority --locked` | 9 passed; 0 failed; 0 ignored |
| `cargo test -p noesis-orchestrator --lib --locked` | 97 passed; 0 failed; 0 ignored |
| `bun test src/server/__tests__/registry-authority.test.ts` | 1 passed; 0 failed |
| `bun run lint` | Exit 0; two pre-existing unused suppression warnings in `sigil-forge/engine.test.ts` |
| `bun run typecheck` | Exit 0 |
| `bun test` | 94 passed; 0 failed; 295 assertions |
| `env -u DATABASE_URL -u TEST_DATABASE_URL pnpm run gate` | Exit 0: 76 script tests, 9 core authority tests, 4 OpenAPI tests, 16 API calculate tests, 35 engine SDK tests, 11 noesis SDK tests, 36 verification tests and 94 TypeScript tests; all builds/typechecks passed |

## Decisions Made

- Runtime rows and their evidence references are canonical for registry identity and drift checks; language-specific source remains the executable comparison target.
- The public mirror count excludes `financial-biosensor` because it is a composed runtime surface and `biofield-capture` because it is an operational conditional runtime.
- The conditional harness uses `PgPoolOptions::connect_lazy` with `postgres://127.0.0.1:1/noesis_registry_fixture`. It parses configuration but performs no connection or query.
- `requirements-completed` stays empty because GATE-05 also requires Plan 02-07 release-receipt and asset authority.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Extracted TypeScript startup registration into a testable function**
- **Found during:** Task 2 (Make registry drift fail the existing gate)
- **Issue:** The six server registrations lived only as top-level entrypoint side effects, so a source-enumeration test would either start a listener or duplicate the list it was meant to verify.
- **Fix:** Added `registerTypeScriptRuntimeEngines` and routed both the entrypoint and authority test through it.
- **Files modified:** `ts-engines/src/index.ts`, `ts-engines/src/server/registry.ts`, `ts-engines/src/server/app.ts`, `ts-engines/src/server/index.ts`
- **Verification:** TypeScript typecheck, 94 tests and the canonical gate pass.
- **Committed in:** `b0f3095`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The extraction is a source-preserving test seam needed to compare the actual startup list; calculation and provider behavior did not change.

## Issues Encountered

- An edit invocation initially resolved against the primary checkout. The exact five touched source files and one newly created test were identified, restored immediately from their clean pre-edit state, and re-applied only inside the isolated recovery worktree. No primary-checkout commit or unrelated file changed.
- Biome was first invoked from the repository root and used the wrong formatting context. The affected TypeScript files were immediately rewritten with the package-local configuration; `bun run lint` then exited zero.
- The installed GSD state handler expected a Performance Metrics table that this recovered `STATE.md` did not contain and initially misplaced the first row after scaffolding. The canonical table was added and the generated row was corrected before commit.

## Known Stubs

- `ts-engines/src/index.ts:7` and `:8` retain pre-existing “stub” labels in the server banner comments for Sacred Geometry and Sigil Forge. Both engines are registered and exercised by the passing TypeScript suite; these comments did not block the registry authority goal.
- `crates/noesis-orchestrator/src/lib.rs:574` through `:579` retain a pre-existing readiness-check TODO/placeholder. Plan 02-06 reads registration membership only and does not claim that this broader health implementation is complete.

## User Setup Required

None — no external service configuration is required.

## Remaining Evidence and Next Phase Readiness

- GATE-05 remains Partial until Plan 02-07 supplies release-receipt and asset authority.
- Native, conditional and full Python capability/consumer catalogue conversion remains Phase 3 scope.
- Production promotion remains HOLD. No deployment, cloud/DNS, schema/data or production database mutation occurred.
- All 570 engine issue implementations and semantic closures remain later-phase work.

## Self-Check: PASSED

- Created registry, TypeScript authority test and summary files exist.
- Task commits `351ae45` and `b0f3095` are present in repository history.
- ISA progress recomputes to 326 checked criteria out of 330 total.
- Documentation diff passes `git diff --check`.

---
*Phase: 02-reproducible-gates-dependency-repair*
*Completed: 2026-09-06*
