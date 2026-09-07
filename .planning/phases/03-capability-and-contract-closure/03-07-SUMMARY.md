---
phase: 03-capability-and-contract-closure
plan: "07"
subsystem: api-capability
tags: [rust, axum, capability-contract, authentication, admin-adapter]

requires:
  - phase: 03-capability-and-contract-closure
    plan: "04"
    provides: canonical resolver and registration/dependency observations
  - phase: 03-capability-and-contract-closure
    plan: "05"
    provides: bounded TypeScript capability observations
  - phase: 03-capability-and-contract-closure
    plan: "06"
    provides: bounded Python capability observations
provides:
  - protected authenticated Rust capability envelope at /api/v1/engines/capabilities
  - resolver-backed public and admin capability identity parity
  - retained six-row admin compatibility adapter with separate readiness projection
affects: [03-08, 03-09, 03-10, 03-11, 03-18, 03-21]

tech-stack:
  added: []
  patterns:
    - canonical capability list is validated before serialization
    - authentication remains middleware-owned; handlers perform no provider generation
    - legacy admin arrays filter canonical rows and keep operational readiness separate

key-decisions:
  - "The public route emits the exact registry order, 19 rows and 17 public mirrors under the v1 envelope."
  - "The existing raw six-row admin array remains through Phase 3 for compatibility; its identity and metadata now come from the canonical resolver."
  - "Admin bridge readiness may project availability on the compatibility adapter, while admin analytics and internal targets never enter the public envelope."

requirements-completed: [CON-01, CON-02]
requirements-progressed: []

duration: 8min
completed: 2026-09-08
---

# Phase 3: Capability and contract closure — Plan 07 Summary

**The Rust API now exposes one authenticated canonical capability envelope while retaining the admin compatibility surface.**

## Accomplishments

- Added `GET /api/v1/engines/capabilities`, protected by the existing API v1 authentication middleware.
- Added a shared resolver-backed envelope builder that validates contract version, row count, mirror count, metadata and registry identity before returning JSON.
- Added route coverage for missing credentials and the exact ordered 19-row response, including public field exclusion assertions.
- Updated the admin six-row TypeScript adapter to filter canonical rows and retain bridge readiness as an operational availability projection.
- Preserved `/api/v1/engines` as the named legacy engine-ID adapter.

## Verification Evidence

- `cargo check -p noesis-api --locked` — passed.
- `cargo test -p noesis-api --test capability_route_tests --locked -- --test-threads=1` — 5 passed.
- Public route assertions prove `contract_version: v1`, `count: 19`, `public_mirror_count: 17`, exact registry order, canonical metadata on every row and absence of admin/internal fields.
- Existing admin authentication, permission and six-row compatibility assertions remain green.
- No provider request, database mutation, Railway/Cloudflare action, deployment, merge or release action occurred.

## Deviations from Plan

- The public handler starts with an empty injected observation set. The resolver preserves dependency-driven rows as `declared` until bounded sidecar observations are explicitly supplied; Plans 05 and 06 provide those sidecar observation producers for later integration.
- OpenAPI registration is deferred to Plan 08 so the response schema can be frozen together with canonical error and security components.

## User Setup Required

None.

## Next Phase Readiness

Plan 08 can freeze the route's OpenAPI response/security schema and canonical error boundary without changing route identity.

---
*Phase: 03-capability-and-contract-closure*
*Plan: 07*
