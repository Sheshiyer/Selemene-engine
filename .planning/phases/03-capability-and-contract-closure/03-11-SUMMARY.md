---
phase: 03-capability-and-contract-closure
plan: "11"
status: complete
completed: 2026-09-08
---

# Plan 03-11 Summary

The general TypeScript SDK now uses the shared runtime decoder for protected capability responses and canonical workflow outcomes. Authentication configuration rejects API-key/bearer ambiguity, requests use exact headers, profile updates use PATCH, resource identifiers are encoded, and malformed or sensitive upstream bodies become bounded `SelemeneError` details. Existing legacy list and workflow method shapes remain available for compatibility; canonical access is exposed through `listCapabilities` and `executeWorkflowOutcome`.

## Evidence

- `pnpm --filter @noesis/sdk test -- --runInBand` passed: 16 tests.
- `pnpm --filter @noesis/sdk typecheck` passed.
- Tests cover the 19-row/17-mirror envelope, shared workflow decoder, auth ambiguity, PATCH, malformed bodies, and safe errors.

No deployment, merge, database, DNS, or provider mutation was performed.
