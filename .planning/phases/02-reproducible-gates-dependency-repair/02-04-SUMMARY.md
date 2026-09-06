---
phase: 02-reproducible-gates-dependency-repair
plan: "04"
status: complete_source_scope
completed: 2026-09-05
baseline_commit: 9b618deff890e7aecfb3dd445eb37fa31dd9565a
requirements_partial: ["GATE-01", "GATE-03", "GATE-05"]
---
# Integrated candidate evidence

Draft PR #1488 consolidates the preserved capability work and scoped recovery repairs. Local canonical gates, all 67 script tests, 93 TS tests, 104 witness tests, admin build/lint/typecheck/tests and 61 Python tests pass. Fresh Node/Python audits have zero vulnerabilities; the optional Rust lockfile yank remains documented. The final bounded source review has no unresolved correctness findings within its stated scope.

Clean remote CI exposed missing gate requirements, admin prerequisite ordering, Linux MediaPipe libraries and a Vercel adapter packaging regression. The repaired jobs and preview pass at 9b618de. The remaining integration failure is reproduced in a new disposable migrated PostgreSQL database: a shared test router survives the Tokio runtime that owns its pool. A test-binary-owned runtime passes the same three cases with database permissions and assertions preserved, in serial and default parallel execution. Full remote CI for that final fixture repair belongs to plan 05.

All eight GitHub controls are reconciled by body-only updates and readback. Their exit checkboxes, open state and labels remain unchanged. The 570 engine issues are untouched. Railway/Cloudflare/Vercel identities, targeted DNS, current production admin observations and the missing public CI smoke URL are reconciled in the infrastructure receipt.

CodeGraph requires forced indexing plus a changed-symbol readback: incremental sync twice claimed freshness while serving stale content. The current capability fixture and Next configuration must resolve from the recovery worktree's own index.

## Remaining scope

Full registry/asset/release authority, immutable deployed artifact equivalence, production API source/schema attribution and rollback proof remain original Wave 0/1/6 obligations. This source-task summary does not complete Phase 2 or any original wave. Main security changes use the concrete critical packet; production promotion remains independently held.
