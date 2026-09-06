---
phase: 02-reproducible-gates-dependency-repair
status: gaps_found
verified: 2026-09-06T10:47:50+00:00
source_commit: b0f309581540a1b707b86cdf8741a1a0e4f0f46a
---
# Phase 2 verification

| Requirement | Status | Evidence or gap |
|---|---|---|
| GATE-01 | Pass | Complete 16-job run 33973728459 passes at source 7a5793d; the PR merge tree equals the source tree |
| GATE-02 | Pass locally | Node production and development audit: zero findings; Rust: zero advisories, with yank disposition |
| GATE-03 | Pass | Frozen Node install, uv.lock and Python audit pass; Python 3.11/3.12 contracts/smoke and both Linux image imports pass remotely at 19b8082 |
| GATE-04 | Source pass | Railway schema and dry runs; Cloudflare 9d9d ownership and targeted DNS browser proof |
| GATE-05 | Partial | Plan 06 registry authority passes at b0f3095: exact 19/17 and 12/1/6 counts, fail-closed mutations, real Rust/TS enumeration and local conditional registration. Plan 07 release-receipt/asset authority remains outstanding |
| GATE-06 | Pass | User-approved additive CI rule applied to ruleset 15597830 and resolved for main; rollback body retained; production promotion HOLD remains separate |

Plan 06 began from source precondition 41908e19c7cce32645d213add29886befb89eeca, where exact-head CI was green and ruleset 15597830 required strict CI Gate with verified readback. The Plan 06 candidate passes `python3 scripts/validate_contracts.py`, 23 validator tests, 9 core authority tests, 97 orchestrator tests, 94 TypeScript tests, TypeScript lint/typecheck and the complete `pnpm run gate`.

No phase completion transition is permitted while GATE-05 is partial. Plans 01–06 have bounded summaries or committed evidence, including the applied main-protection decision and executable registry authority. Plan 07 still owns release-receipt/asset authority. Native/conditional capability closure and all 570 engine issues remain later phase work. Production promotion remains HOLD.
