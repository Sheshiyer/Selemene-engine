---
phase: 03-capability-and-contract-closure
plan: "04"
subsystem: capability-resolution
tags: [rust, resolver, registry, conditional-registration]

requires:
  - phase: 03-capability-and-contract-closure
    plan: "03"
    provides: typed capability/workflow contract readers
provides:
  - pure four-state capability reducer with bounded observation joins
  - canonical registry declaration loader and orchestrator snapshot seam
  - no-pool conditional registration receipt for biofield-capture
affects: [03-05, 03-06, 03-07, 03-09, 03-16]

tech-stack:
  added: []
  patterns:
    - required dependency failure wins; optional dependency failure degrades
    - registration and availability remain separate while every identity is retained

key-files:
  created:
    - crates/noesis-orchestrator/src/capability.rs
  modified:
    - crates/noesis-orchestrator/src/lib.rs
    - crates/noesis-api/src/handlers/admin.rs

key-decisions:
  - "Dependency observations are passed as (engine_id, observation) pairs because the canonical observation schema intentionally keeps dependency observations reusable and does not duplicate engine identity."
  - "No observation leaves a declared row declared; only registered engines with observed required dependencies can become available."
  - "The orchestrator exposes an explicit registration-matrix seam for deterministic conditional tests without reading database configuration."

patterns-established:
  - "All resolver outputs preserve declaration order and the complete 19-row authority."
  - "Resolver error paths reject duplicate, unknown and metadata-conflicting observations before projection."

requirements-completed: [CON-01]
requirements-progressed: [CON-02]

duration: 22min
completed: 2026-09-08
---

# Phase 3: Capability and contract closure — Plan 04 Summary

**The orchestrator now resolves canonical capabilities from bounded registration and dependency observations without database, network or provider access.**

## Accomplishments

- Added `capability.rs` with a pure resolver preserving all declarations and applying deterministic `declared`, `available`, `degraded` and `unavailable` precedence.
- Added canonical registry parsing from `contracts/v1/registries/engines.json` and orchestrator snapshot methods for actual or injected registration matrices.
- Kept `biofield-capture` present with `registered=false` and `declared` availability when native registration runs without a database pool.
- Preserved the existing admin capability initializer by explicitly setting additive metadata fields to `None` until the public API projection consumes the resolver.

## Verification Evidence

- `cargo test -p noesis-orchestrator capability --locked` — 6 passed.
- `env -u DATABASE_URL -u TEST_DATABASE_URL cargo test -p noesis-api database_conditional_registration --locked` — 2 targeted registration tests passed; compilation completed for the API test binary.
- `cargo fmt --all -- --check` and `git diff --check` — passed.
- Resolver tests cover no observations, required failure, optional degradation, successful required dependency, duplicate observation and conflicting registration matrices.
- No database connection, migration, network request, provider call, Railway/Cloudflare action or deployment occurred.

## Deviations from Plan

**1. [Rule 3 - Compatibility] Updated one existing API initializer**
- **Found during:** Task 2 compilation.
- **Issue:** Extending `EngineCapability` with additive metadata made an existing admin initializer fail to compile.
- **Fix:** Set the three new fields to `None`; the admin projection remains a legacy compatibility surface until Plan 07 binds it to the canonical resolver.
- **Verification:** API conditional-registration tests pass.

**2. [Schema boundary] Kept engine identity outside dependency observations**
- **Found during:** Task 1 design review.
- **Issue:** Encoding an engine ID inside `dependency_id` would have violated the canonical dependency ID vocabulary.
- **Fix:** Resolver inputs use `(engine_id, DependencyObservation)` pairs while emitted observations retain canonical IDs.
- **Verification:** Resolver metadata matching and duplicate tests pass.

## User Setup Required

None.

## Next Phase Readiness

Plan 05 can replace TypeScript default-healthy reporting with bounded self-check observations while the Rust resolver remains the sole availability reducer.

---
*Phase: 03-capability-and-contract-closure*
*Plan: 04*
