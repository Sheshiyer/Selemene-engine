---
phase: 03-capability-and-contract-closure
plan: "03"
subsystem: cross-language-contracts
tags: [rust, typescript, sdk, strict-decoder]

requires:
  - phase: 03-capability-and-contract-closure
    plan: "02"
    provides: strict v1 capability/workflow schemas, fixtures and validator
provides:
  - strict Rust capability-list and workflow-outcome readers
  - sole shared TypeScript runtime decoder and guards
  - fixture-driven Rust, SDK and independent Bun compatibility tests
affects: [03-03b, 03-04, 03-05, 03-07, 03-11, 03-12, 03-18, 03-19]

tech-stack:
  added: []
  patterns:
    - deny-unknown-fields Rust wire types with explicit semantic validation methods
    - one shared TypeScript decoder reused by pnpm SDK and independent Bun engines

key-files:
  created: []
  modified:
    - crates/noesis-core/src/contract.rs
    - crates/noesis-core/tests/contract_v1_authority.rs
    - packages/noesis-engine-sdk/src/contract-v1.ts
    - packages/noesis-engine-sdk/src/index.ts
    - packages/noesis-engine-sdk/tests/contract-v1.test.ts
    - ts-engines/src/types/engine.ts
    - ts-engines/tests/contract-v1.test.ts

key-decisions:
  - "Legacy item fixtures remain accepted through optional additive metadata, while canonical list readers require metadata on all 19 rows."
  - "Workflow outcomes conserve requested IDs and treat engine_results as an equality-checked compatibility alias."
  - "Runtime decoding lives only in packages/noesis-engine-sdk/src/contract-v1.ts; ts-engines carries type declarations and imports that source authority directly."

patterns-established:
  - "Strict shape checks precede semantic set/status checks in both maintained readers."
  - "No runtime producer or route emits the new fields until later plans consume the reader boundary."

requirements-completed: [CON-01, CON-02]
requirements-progressed: []

duration: 18min
completed: 2026-09-08
---

# Phase 3: Capability and contract closure — Plan 03 Summary

**Rust and TypeScript now share strict, additive readers for the canonical capability list and workflow outcomes before producer migration begins.**

## Accomplishments

- Added typed Rust enums and structures for bounded capability reasons, dependency observations, operation support, 19-row capability lists, workflow outputs, failure receipts, execution status and synthesis status.
- Preserved old v1 item compatibility with optional additive fields while `EngineCapabilityList::validate` requires canonical metadata and exact 19/17 envelope counts.
- Added `WorkflowOutcome::validate` for requested/output/failure conservation, duplicate detection, legacy alias equality and synthesis truth.
- Implemented the sole runtime TypeScript decoder with strict unknown-field rejection, bounded literals, format checks, conservation checks, status checks and sensitive failure-message rejection.
- Exported decoder functions and type guards from the public engine SDK; `ts-engines` tests import this exact source reader instead of creating a duplicate decoder.

## Verification Evidence

- `cargo test -p noesis-core --test contract_v1_authority --locked` — 12 passed.
- `(cd ts-engines && bun test tests/contract-v1.test.ts && bun run typecheck)` — 4 passed; typecheck exit 0.
- `pnpm --filter @selemene/engine-sdk test -- tests/contract-v1.test.ts` — 4 passed, 19 assertions.
- `pnpm --filter @selemene/engine-sdk typecheck` — exit 0 for source and contract tests.
- `cargo fmt --all -- --check` and `git diff --check` — passed.

## Tests Added

- Rust round-trip and semantic tests for all four workflow fixtures, canonical metadata presence, unknown fields and alias drift.
- SDK decoder acceptance for 19 capabilities and partial outcomes, plus unknown-field and alias mutations.
- Independent Bun reader reuse, incompatible-version rejection and unknown-field rejection.

## Deviations from Plan

None. No route, producer, provider, database, deployment or external state was changed.

## User Setup Required

None.

## Next Phase Readiness

Plan 03b can declare locked workspace links for actual pnpm consumers and build the generated public SDK artifacts before downstream SDK, admin, Witness and verification plans execute.

---
*Phase: 03-capability-and-contract-closure*
*Plan: 03*
