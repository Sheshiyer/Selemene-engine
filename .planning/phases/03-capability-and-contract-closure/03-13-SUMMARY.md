---
phase: 03-capability-and-contract-closure
plan: "13"
subsystem: cli-transport
tags: [bun, typescript, cli, authentication, diagnostics]

requires:
  - phase: 03-capability-and-contract-closure
    plan: "09"
    provides: explicit operation support and bounded API boundary
provides:
  - typed CLI API-key/bearer transport
  - bounded CLI HTTP errors and health reasons
  - config validation for mutually exclusive credentials
affects: [03-14, 03-15]

tech-stack:
  added: []
  patterns:
    - API keys use X-API-Key and bearer tokens use Authorization exclusively
    - malformed/empty/plain-text responses collapse to stable bounded errors
    - ambiguous credentials fail before fetch

key-decisions:
  - "The CLI configuration accepts either apiKey or bearerToken and rejects both together."
  - "Health diagnostics expose request_failed/timeout reasons rather than raw URL, socket or parser text."
  - "The existing CLI dependency and lockfile surface stays unchanged."

requirements-completed: [CON-02]
requirements-progressed: []

duration: 8min
completed: 2026-09-08
---

# Phase 3: Capability and contract closure — Plan 13 Summary

**CLI credentials and transport now match the protected Rust authentication boundary.**

## Accomplishments

- Added mutually exclusive `apiKey` and `bearerToken` configuration fields.
- Changed CLI API-key transport to `X-API-Key`; bearer transport remains `Authorization: Bearer`.
- Added bounded error parsing for empty, malformed, non-JSON and upstream responses.
- Added bounded health failure reasons and passed typed auth through check/doctor probes.
- Added recording tests for header exclusivity, zero-call ambiguous auth and safe errors.

## Verification Evidence

- `cd bridges/cli && bun test src/core/http.test.ts` — 5 passed.
- `cd bridges/cli && bun run typecheck` — passed.
- Existing CLI scripts and lockfiles remain unchanged apart from the additive test script.
- No provider, database, Railway, Cloudflare, deployment, merge or release mutation occurred.

## Deviations from Plan

- Capability catalogue retrieval remains in the generator boundary (Plan 14); this slice limits itself to shared CLI transport and diagnostics so credential handling is established first.

## User Setup Required

None.

## Next Phase Readiness

Plan 14 can generate CLI/LangChain tools from the protected Rust capability contract using this typed transport.

---
*Phase: 03-capability-and-contract-closure*
*Plan: 13*
