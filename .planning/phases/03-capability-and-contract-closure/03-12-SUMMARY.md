---
phase: 03-capability-and-contract-closure
plan: "12"
status: complete
completed: 2026-09-08
---

# Plan 03-12 Summary

The focus TypeScript SDK now exposes the protected Rust capability route and keeps its four-engine public surface explicit. It supports bearer or API-key authentication with mutually exclusive headers, rejects unknown engines and known unsupported validation before downstream calls, and preserves consent-first direct-sidecar behavior for local media APIs. Error details are bounded and no raw upstream or credential material is retained.

## Evidence

- `pnpm --filter @selemene/engine-sdk build` passed.
- `pnpm --filter @selemene/engine-sdk test -- tests/engine-client.test.ts` passed: 36 tests.
- `pnpm --filter @selemene/engine-sdk typecheck` passed.
- Tests cover canonical 19-row decoding, exact auth headers, zero-call preflight, protected route selection, consent guards, and safe errors.

No deployment, merge, database, DNS, or provider mutation was performed.
