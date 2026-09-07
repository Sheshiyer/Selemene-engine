---
phase: 03-capability-and-contract-closure
plan: "08"
subsystem: api-errors-openapi
tags: [rust, errors, redaction, openapi, security]

requires:
  - phase: 03-capability-and-contract-closure
    plan: "07"
    provides: protected canonical capability route
provides:
  - OpenAPI documentation for canonical capability envelope and security schemes
  - recursive credential redaction for error responses and Sentry messages
  - bounded Biofield upstream error projection
affects: [03-09, 03-10, 03-11, 03-13, 03-21]

tech-stack:
  added: []
  patterns:
    - canonical capability contract types expose ToSchema only with the openapi feature
    - API-key and bearer security remain distinct OpenAPI schemes
    - upstream diagnostics cross the API boundary only through allowlisted fields

key-decisions:
  - "Public OpenAPI advertises the canonical capability route and excludes admin/internal fields."
  - "Credential-like key/value material is redacted recursively before response serialization and telemetry capture."
  - "Biofield bridge failures expose only stable reason/status fields; quality responses retain bounded numeric/boolean assessment fields."

requirements-completed: [CON-02]
requirements-progressed: []

duration: 9min
completed: 2026-09-08
---

# Phase 3: Capability and contract closure — Plan 08 Summary

**The authenticated capability boundary is now described in OpenAPI and upstream error details are bounded before they leave the API.**

## Accomplishments

- Added OpenAPI schema support for the capability contract and registered the protected capability path with bearer and `X-API-Key` security alternatives.
- Added regression coverage for the route, response schema reference, required public fields and admin/internal field exclusion.
- Added recursive redaction of credential-like values in `ErrorMapper` response details, messages and Sentry capture text.
- Removed Biofield bridge URLs, raw upstream bodies, parser diagnostics and arbitrary upstream messages from public error details.
- Kept the quality rejection response useful through an allowlisted numeric/boolean assessment projection.

## Verification Evidence

- `cargo fmt --all` — passed.
- `cargo test -p noesis-api --lib error_mapper::tests --locked` — 7 passed.
- `cargo test -p noesis-api --test biofield_capture_proxy --locked -- --test-threads=1` — 10 passed.
- `cargo test -p noesis-api --test openapi_schema_tests --test auth_rate_limit_openapi_tests --test capability_route_tests --locked -- --test-threads=1` — 12 passed.
- Existing error snapshots and API error handling remain compatible; no provider, database, Railway, Cloudflare, deployment or release mutation occurred.

## Deviations from Plan

- Stable error-code expansion for future unsupported-operation and workflow-specific failures remains in the later operation/workflow plans; this slice closes the observed redaction and OpenAPI gaps without inventing new runtime errors.

## User Setup Required

None.

## Next Phase Readiness

Plan 09 can converge bridge operation metadata, validation truth, path safety and zero-call routing against this protected, redacted boundary.

---
*Phase: 03-capability-and-contract-closure*
*Plan: 08*
