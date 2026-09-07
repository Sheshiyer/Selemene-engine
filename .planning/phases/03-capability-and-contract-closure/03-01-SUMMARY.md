---
phase: 03-capability-and-contract-closure
plan: 01
subsystem: contracts
tags: [json-schema, capability-registry, locked-environments]

requires:
  - phase: 02-reproducible-gates-dependency-repair
    provides: verified exact-head repository gate and dependency policy
provides:
  - canonical 19-row registry metadata with 12/1/6 runtime classes and 17 public mirrors
  - additive capability item/list schemas and registry-ordered list fixture
  - locked pnpm, Bun and uv environment hydration receipt
affects: [03-02, capability-resolver, api, sdk, cli, admin, witness]

tech-stack:
  added: []
  patterns: [registry identity plus bounded observations, additive strict v1 envelope]

key-files:
  created:
    - contracts/v1/schemas/engine-capability-list.schema.json
    - contracts/v1/fixtures/engine-capability-list.json
  modified:
    - contracts/v1/manifest.json
    - contracts/v1/registries/engines.json
    - contracts/v1/schemas/engine-capability.schema.json

key-decisions:
  - "Keep runtime identity in the 19-row registry and project 17 public mirrors separately."
  - "Represent dependency requirements and calculate/validate/witness support as bounded registry metadata."

patterns-established:
  - "Capability list preserves every identity while availability remains an observed four-state overlay."
  - "TypeScript validation is explicitly unsupported unless a real validator is proven."

requirements-completed: [CON-01, CON-02]

duration: 35min
completed: 2026-09-08
---

# Phase 3: Capability and contract closure — Plan 01 Summary

**A registry-derived capability authority now carries 19 ordered runtime identities, explicit dependency requirements, operation support and a strict 19-row/17-mirror list envelope.**

## Performance

- **Tasks:** 2 completed
- **Files modified:** 5 tracked contract files plus 2 new schema/fixture files

## Accomplishments

- Re-ran the existing frozen hydration commands for pnpm, independent Bun projects and the locked Python project; all required Python imports succeeded and no lockfile changed.
- Extended every registry row with bounded dependency requirements and operation metadata while preserving the 12 native, one database-conditional and six TypeScript split.
- Added the capability-list schema/fixture and registered them in the v1 manifest; the list fixture contains all 19 registry-ordered rows and the 17-public-mirror count.

## Task Commits

1. **Task 1: Hydrate and preflight the locked test environments** — no source changes; receipt captured below.
2. **Task 2: Define the canonical capability list and registry metadata** — `5535d12` (feat(03-01))

## Files Created/Modified

- `contracts/v1/registries/engines.json` - dependency requirements and calculate/validate/Witness operation metadata for all 19 identities.
- `contracts/v1/schemas/engine-capability.schema.json` - bounded reasons, dependency observations and operation support fields.
- `contracts/v1/schemas/engine-capability-list.schema.json` - strict 19-row envelope with count 19 and public mirror count 17.
- `contracts/v1/fixtures/engine-capability-list.json` - registry-ordered declared capability projection.
- `contracts/v1/manifest.json` - registers the new schema and fixture.

## Verification Evidence

- `pnpm install --offline --ignore-scripts --frozen-lockfile` — passed with lockfile unchanged.
- `(cd ts-engines && bun install --frozen-lockfile)` — passed with lockfile unchanged.
- `(cd bridges/cli && bun install --frozen-lockfile)` — passed with lockfile unchanged.
- `(cd python-services && uv sync --locked --extra dev --python 3.12)` — passed.
- `PYTHONPATH=.. uv run --locked --extra dev python -c 'import pytest,httpx,fastapi,numpy,cv2,mediapipe'` — `locked-python-imports: ok`.
- Locked `python -m json.tool` checks passed for manifest, registry, capability item schema, capability-list schema and list fixture.
- Registry inspection returned 19 rows and exactly 16 `witness_eligible=true` rows; `biofield-capture`, `financial-biosensor` and `raaga` are the three explicit exclusions.
- `git diff --check` — passed before commit.

## Decisions Made

- Used `dependency_requirements` on registry rows and `dependency_observations` on capability items so identity stays static while runtime evidence can vary.
- Kept the existing item fixture compatible by making additive fields optional at item level; the new list envelope requires the full canonical shape.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added the missing decision-coverage citations to planning artifacts**
- **Found during:** GSD planning gate before execution
- **Issue:** `gsd-sdk query check.decision-coverage-plan` initially reported five uncovered context decisions.
- **Fix:** Added literal `D-02` through `D-06` references in designated `must_haves` sections without changing the task graph.
- **Files modified:** `03-01-PLAN.md`, `03-02-PLAN.md`, `03-08-PLAN.md`, `03-09-PLAN.md`, `03-21-PLAN.md`
- **Verification:** Decision coverage returned `passed: true`, `covered: 7`, `uncovered: []`.
- **Committed in:** pending planning metadata commit.

**Total deviations:** 1 auto-fixed (planning gate coverage)
**Impact on plan:** No source scope changed; the plan now satisfies the workflow translation gate.

## Issues Encountered

- The configured GSD runtime reports `agents_installed=false`; two delegated gsd-executor attempts stalled without writing files, so execution used the documented inline fallback. No partial executor commits existed.
- The locked Python environment does not include the optional `jsonschema` module; schema verification used the required locked JSON syntax probes and the repository validator remains the next wave's authority.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 02 can extend the validator against this authority. It must update validator row-key expectations for `dependency_requirements` and `operations` and add negative mutation coverage before any runtime producer emits the new fields. No Railway, Cloudflare, database, secret or deployment action was performed.

---
*Phase: 03-capability-and-contract-closure*
*Plan: 03-01*
