---
phase: 03-capability-and-contract-closure
plan: "18"
status: complete
completed: 2026-09-08
---

# Plan 03-18 Summary

The admin web now consumes the canonical protected capability envelope through the shared `@selemene/engine-sdk` decoder. The engines, engine detail, and bridge surfaces use the 19-row runtime catalogue, expose the exact 17-row public mirror projection, and retain declared/degraded/available/unavailable state semantics. API parsing bounds malformed upstream responses to a stable error code.

## Evidence

- `pnpm --filter @selemene/engine-sdk build` passed and generated non-empty `dist/index.js` and `dist/index.d.ts`.
- `pnpm --filter @noesis/witness-pipeline build` passed.
- `pnpm --filter @noesis/sdk build` passed.
- `pnpm --filter admin-web typecheck` passed.
- `pnpm --filter admin-web exec node --test src/lib/engine-capability.test.mjs` passed (2 tests).

No deployment, merge, database, DNS, or provider mutation was performed.
