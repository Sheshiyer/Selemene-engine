---
phase: 03-capability-and-contract-closure
plan: "17"
subsystem: hermes-bridge
tags: [python, hermes, tools, authentication, workflows]

requires:
  - phase: 03-capability-and-contract-closure
    plan: "16"
    provides: five producer-backed workflows and canonical workflow outcome semantics
provides:
  - canonical 17-mirror Hermes tool catalogue
  - protected Rust dispatch with auth and identifier preflight
  - explicit Full Spectrum unsupported metadata
affects: [03-20, 03-21]

tech-stack:
  added: []
  patterns:
    - registry-derived public mirror catalogue with runtime/public counts
    - protected Rust URL construction only after allowlist validation
    - bounded Hermes tool errors with exact credential headers

key-decisions:
  - "Hermes advertises 17 public mirrors including raaga; financial-biosensor and biofield-capture remain runtime identities outside the mirror projection."
  - "Full Spectrum remains visible in function metadata with status unsupported and is rejected before HTTP."
  - "Hermes supports either X-API-Key or Authorization: Bearer, never both, and never exposes upstream body text."

requirements-completed: [CON-02]
requirements-progressed: []

duration: 10min
completed: 2026-09-08
---

# Phase 3: Capability and contract closure — Plan 17 Summary

**Hermes now uses the canonical public mirror and protected workflow boundary.**

## Accomplishments

- Loaded runtime and public engine identity from the v1 registry at tool-catalogue construction.
- Added Raaga to the 17 public mirrors and excluded only Financial Biosensor and Biofield Capture.
- Added canonical capability metadata and explicit Full Spectrum `unsupported` status.
- Restricted engine and workflow dispatch to protected Rust routes with identifier validation before URL encoding.
- Added bearer support as a distinct credential branch while preserving exact `X-API-Key` behavior.
- Bounded timeout, transport, status and malformed-response errors and added recording tests.

## Verification Evidence

- `cd python-services && PYTHONPATH=.. uv run --locked --extra dev python -m pytest ../bridges/hermes/tests -q` — 7 passed.
- No model/provider, database, Railway, Cloudflare, deployment, merge or release mutation occurred.

## Deviations from Plan

- The five supported workflow IDs are maintained as an explicit ordered projection because Hermes cannot import Rust `WorkflowRegistry`; the producer remains authoritative in Plan 16.
- Hermes keeps compatibility meta-tool names while routing them through the protected Rust API and canonical support checks.

## User Setup Required

None.

## Next Phase Readiness

The remaining SDK, admin, Witness and API consumer plans can converge on the same shared decoder and producer outcome.

---
*Phase: 03-capability-and-contract-closure*
*Plan: 17*
