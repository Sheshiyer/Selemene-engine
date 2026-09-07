---
phase: 03-capability-and-contract-closure
plan: "15"
subsystem: universal-tool-bridge
tags: [python, fastapi, httpx, authentication, routing, security]

requires:
  - phase: 03-capability-and-contract-closure
    plan: "09"
    provides: protected Rust operation boundary and bounded API errors
provides:
  - typed allowlisted universal-tool descriptors
  - protected Rust engine/workflow dispatch
  - fail-closed identifier, auth and upstream-error handling
affects: [03-16, 03-17, 03-21]

tech-stack:
  added: []
  patterns:
    - canonical registry rows derive public engine calculate descriptors
    - workflow IDs are validated before route construction and Full Spectrum is explicit unsupported
    - injected httpx transport proves exact headers and zero-call rejections

key-decisions:
  - "Direct TypeScript/Python public tools and universal engine validation are removed from the descriptor catalogue."
  - "API keys use X-API-Key and bearer tokens use Authorization: Bearer; simultaneous credentials fail before transport."
  - "Upstream status, body, URL and credential material are collapsed to bounded error codes."

requirements-completed: [CON-02]
requirements-progressed: []

duration: 12min
completed: 2026-09-08
---

# Phase 3: Capability and contract closure — Plan 15 Summary

**The universal tool bridge now dispatches through typed protected Rust descriptors.**

## Accomplishments

- Derived the runtime/public engine catalogue from `contracts/v1/registries/engines.json`.
- Retained only protected Rust health, status, capability, engine calculation and workflow descriptors.
- Preserved five supported workflow IDs and exposed Full Spectrum as visible `unsupported` metadata.
- Added identifier validation for engine/workflow IDs before URL encoding or transport.
- Added exact API-key/bearer credential branches and bounded timeout, non-JSON and upstream error responses.
- Added injected ASGI/httpx tests for 19 runtime identities, 17 public mirrors, Raaga inclusion, no sidecar route, unsupported workflows and zero-call rejection.

## Verification Evidence

- `cd python-services && PYTHONPATH=.. uv run --locked --extra dev python -m pytest ../bridges/universal-tool-server/tests -q` — 6 passed.
- No provider, database, Railway, Cloudflare, deployment, merge or release mutation occurred.

## Deviations from Plan

- The bridge keeps a compatibility `engine_calculate` request name, but resolves it through the typed public descriptor and never accepts a caller-supplied path.
- The universal bridge does not execute a live capability probe; canonical capability observations remain the protected API producer's responsibility.

## User Setup Required

None.

## Next Phase Readiness

Plan 16 can make workflow producer outcomes lossless and route the existing synthesis modules explicitly.

---
*Phase: 03-capability-and-contract-closure*
*Plan: 15*
