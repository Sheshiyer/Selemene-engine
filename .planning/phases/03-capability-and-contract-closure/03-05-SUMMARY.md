---
phase: 03-capability-and-contract-closure
plan: "05"
subsystem: typescript-observations
tags: [typescript, self-check, capability-route, bounded-errors]

requires:
  - phase: 03-capability-and-contract-closure
    plan: "04"
    provides: pure capability reducer and registry identity authority
provides:
  - explicit six-row TypeScript observation projection
  - bounded self-check mapping for pass, failure, timeout, malformed and absent evidence
  - internal capability route that never defaults missing checks to available
affects: [03-07, 03-09, 03-11, 03-18]

tech-stack:
  added: []
  patterns:
    - legacy health routes retain compatibility while capability route uses explicit evidence
    - raw self-check errors collapse to bounded reason codes before projection

key-files:
  created:
    - ts-engines/src/server/__tests__/capability-route.test.ts
  modified:
    - ts-engines/src/server/registry.ts
    - ts-engines/src/server/app.ts

key-decisions:
  - "Missing self-check evidence remains declared and never becomes available; existing /health/ready behavior is preserved for compatibility."
  - "TypeScript validation is always advertised unsupported until a real validator exists; calculate remains supported for registered rows."
  - "Capability rows use the canonical six-ID order and internal dependency metadata `typescript:bridge`."

patterns-established:
  - "Timeout, malformed, optional and thrown self-checks map to bounded reason codes without exposing details."
  - "Capability observation tests use fake engines only; no calculation or provider path is invoked."

requirements-completed: [CON-01, CON-02]
requirements-progressed: []

duration: 9min
completed: 2026-09-08
---

# Phase 3: Capability and contract closure — Plan 05 Summary

**TypeScript capability reporting now reflects explicit self-check evidence and leaves absent observations declared.**

## Accomplishments

- Added a canonical six-row projection in registry order: `enneagram`, `i-ching`, `raaga`, `sacred-geometry`, `sigil-forge`, `tarot`.
- Added `runCapabilityObservations` with bounded handling for successful, false, thrown, timeout, malformed and optional-dependency self-checks.
- Changed `/engines/capabilities` to use explicit observations while leaving `/health`, `/health/live`, `/health/engines` and `/health/ready` compatibility behavior intact.
- Added no-provider, no-generation mutation coverage and guaranteed TypeScript `validate: unsupported` metadata.

## Verification Evidence

- `cd ts-engines && bun test src/server/__tests__/registry-authority.test.ts src/server/__tests__/capability-route.test.ts` — 9 passed, 37 assertions.
- `cd ts-engines && bun run typecheck` — exit 0.
- `bunx biome check src/server/registry.ts src/server/app.ts src/server/__tests__/capability-route.test.ts` — passed.
- Timeout matrix completed in ~1 second and raw `/private/provider/secret token` text was absent from the projected receipt.
- No provider, network, database, Railway, Cloudflare, deployment or release mutation occurred.

## Deviations from Plan

None. The legacy health compatibility path remains unchanged by design; only the capability projection gained explicit observation semantics.

## User Setup Required

None.

## Next Phase Readiness

Plan 06 can normalize Python sidecar dependency facts under the same bounded observation vocabulary before the Rust API joins all 19 rows.

---
*Phase: 03-capability-and-contract-closure*
*Plan: 05*
