---
phase: 03-capability-and-contract-closure
plan: "02"
subsystem: contracts
tags: [json-schema, validator, workflow-outcomes, mutation-tests]

requires:
  - phase: 03-capability-and-contract-closure
    plan: "01"
    provides: additive capability authority, registry metadata and locked environment preflight
provides:
  - strict workflow outcome schema and complete/partial/failed/unsupported fixtures
  - fail-closed capability and workflow invariant validation
  - mutation coverage for identity, metadata, conservation and synthesis truth
affects: [03-03, capability-resolver, api, sdk, cli, admin, witness]

tech-stack:
  added: [jsonschema==4.26.0, referencing==0.36.2]
  patterns:
    - schema validation followed by canonical registry and workflow conservation checks
    - legacy aliases are accepted only when byte-equal to canonical fields

key-files:
  created:
    - contracts/v1/schemas/workflow-outcome.schema.json
    - contracts/v1/fixtures/workflow-outcome-complete.json
    - contracts/v1/fixtures/workflow-outcome-partial.json
    - contracts/v1/fixtures/workflow-outcome-failed.json
    - contracts/v1/fixtures/workflow-outcome-unsupported.json
  modified:
    - contracts/v1/manifest.json
    - scripts/validate_contracts.py
    - tests/scripts/test_validate_contracts.py
    - python-services/pyproject.toml
    - python-services/uv.lock

key-decisions:
  - "Workflow outcomes conserve every requested engine across keyed outputs and bounded failures; synthesis status cannot claim powered content after failure or unsupported execution."
  - "The validator runs in the locked Python project and rejects schema, registry, projection, dependency, operation and workflow drift before runtime readers consume fixtures."
  - "jsonschema and referencing are explicit locked dev dependencies because the repository validator imports them and ambient Python is forbidden by the plan."

patterns-established:
  - "Authority-first validation: local schemas establish shape, then canonical registry and set invariants establish meaning."
  - "Compatibility aliases remain additive and must equal the canonical engine output map."

requirements-completed: [CON-01, CON-02]
requirements-progressed: []

duration: 78min
completed: 2026-09-08
---

# Phase 3: Capability and contract closure — Plan 02 Summary

**The v1 authority now rejects capability drift and false workflow success before any runtime producer consumes the new contracts.**

## Performance

- **Tasks:** 2 completed
- **Files modified:** 10 tracked or created files, plus the locked Python dependency graph

## Accomplishments

- Added and registered a strict workflow outcome envelope with deterministic complete, partial, failed and unsupported fixtures. Each fixture conserves every requested engine across success and failure receipts.
- Extended the repository validator to enforce manifest registration, schema validity, exact 19-row and 17-mirror authority, 12/1/6 runtime classes, bounded dependencies and reasons, operation support, workflow conservation, execution/synthesis status consistency and legacy alias equality.
- Added 8 mutation tests covering capability identity and operation drift, dependency metadata, workflow conservation, alias divergence and powered synthesis after unsupported execution.
- Added `jsonschema==4.26.0` and `referencing==0.36.2` to the locked Python dev group because the validator requires these libraries; no ambient interpreter or fallback project is used.

## Task Commits

1. **Task 1: Add strict workflow outcome schema and fixtures** — included in the Plan 02 source commit below.
2. **Task 2: Extend validator and negative mutation coverage** — included in the Plan 02 source commit below.

## Verification Evidence

- `cd python-services && uv run --locked --extra dev python ../scripts/validate_contracts.py` — `contract authority v1 valid: schemas=8 fixtures=10 registries=1 engines=19`.
- `cd python-services && PYTHONPATH=.. uv run --locked --extra dev python -m pytest ../tests/scripts/test_validate_contracts.py -q --tb=line` — `148 passed in 72.83s`.
- Targeted repaired mutation — `test_capability_list_rejects_missing_or_reordered_identity` — `1 passed in 2.30s`.
- `git diff --check` — passed before commit.
- No network, provider, database, Railway, Cloudflare, GitHub setting, deployment or release action ran.

## Decisions Made

- Schema validation intentionally runs before custom semantic checks, so malformed list sizes receive the JSON Schema `too short` receipt while structurally valid but reordered lists receive the canonical order error.
- Financial-biosensor validation remains explicitly unsupported and is reflected identically in the registry and capability-list fixture.
- Workflow `engine_results` remains a compatibility alias only; divergence is a hard validation error.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added locked schema-validation dependencies**
- **Found during:** Task 2 validator execution.
- **Issue:** The existing locked Python project did not declare `jsonschema` or `referencing`, although `scripts/validate_contracts.py` imports them and the plan requires locked execution.
- **Fix:** Added exact dev pins with `uv add --dev` and refreshed `python-services/uv.lock`; the offline add path was unavailable, so the package index was used only to resolve these declared development dependencies.
- **Verification:** Locked validator and all 148 validator tests pass.
- **Impact:** The dependency graph changes only in the Python dev group; runtime service dependencies and production profiles are unchanged.

**2. [Rule 3 - Test receipt alignment] Accepted schema-first missing-row failure**
- **Found during:** Full mutation suite.
- **Issue:** Removing a capability row fails the list schema's minimum-items rule before the custom registry-order message.
- **Fix:** The mutation assertion now accepts the explicit schema `too short` receipt while retaining checks for custom order/count errors.
- **Verification:** Full suite passes with 148 tests.

**Total deviations:** 2 auto-fixed; no scope expansion.

## Issues Encountered

- Two delegated GSD executor attempts previously stalled without filesystem progress; the documented inline executor fallback was used and produced the verified source receipts.
- The current CodeGraph index is a prior Phase 2 receipt and will be forcibly rebuilt after the next source wave; no stale index is treated as current proof.

## User Setup Required

None. No external service configuration is required.

## Next Phase Readiness

Plan 03 can add Rust, TypeScript and SDK readers against the canonical capability-list and workflow outcome authorities. The validator and locked test environment now provide the first producer-independent blocking gate.

---
*Phase: 03-capability-and-contract-closure*
*Plan: 03-02*
