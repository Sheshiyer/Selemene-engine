---
project: Selemene ChatGPT plugin
task: Implement customer MCP and prepare public plugin submission
effort: E3
phase: build
progress: 16/32
mode: algorithm
started: 2026-09-29
updated: 2026-09-30
---

## Problem

The first plugin specification was based on incomplete bridge documentation and assumed Sites hosting. The user requested a parallel deep pass over current source and existing Cloudflare/Railway infrastructure before proceeding.

## Vision

The plugin uses Selemene's actual account, execution, and storage contracts. The implementation plan distinguishes current deployment, newer local work, and unverified operational behavior.

## Out of Scope

- The prior audit made no deployment, merge, or credential changes; implementation remains isolated until verification gates pass.
- No real personal reading, paid generation, database-content inspection, or public plugin publication.
- No alteration of pre-existing source changes or historical root ISA completion records.

## Principles

Source evidence and live evidence answer different questions. A running service is not proof that every engine or external provider workflow is operational.

## Constraints

- Cloudflare reads use the explicit 9d9d profile and account.
- Railway reads target the identified Selemene project and production environment without changing the local link.
- Credential values and personal reading contents must never enter receipts.

## Goal

Implement an account-bound customer MCP server, correct relevant authorization defects, prepare verified website and review materials, and advance the plugin to the submission boundary. Track local build, deployment, archive readiness, draft upload, review submission, and publication separately.

## Criteria

- [x] ISC-1: Local source revision is recorded with Git evidence.
- [x] ISC-2: Current remote main revision is recorded with remote evidence.
- [x] ISC-3: Source divergence is quantified by Git ancestry.
- [x] ISC-4: Cloudflare scoped-account authentication is independently verified.
- [x] ISC-5: Cloudflare Selemene resource evidence is saved.
- [x] ISC-6: Railway account authentication is independently verified.
- [x] ISC-7: Railway deployment evidence is saved with source-linkage limits.
- [x] ISC-8: API and customer identity contracts are documented from implementation.
- [x] ISC-9: Live capability-route behavior is distinguished from source behavior.
- [x] ISC-10: The original plugin specification is corrected against findings.
- [x] ISC-11: The implementation sequence names the actual unresolved gates.
- [x] ISC-12: Anti: audit actions do not mutate production or pre-existing source work.

- [x] ISC-13: Isolated implementation checkout starts from verified current main.
- [ ] ISC-14: A real MCP server exposes the seven reviewed tool contracts.
- [ ] ISC-15: OAuth metadata and PKCE flow are implemented with maintained provider library.
- [ ] ISC-16: OAuth customer linkage validates a per-user upstream credential.
- [ ] ISC-17: Cross-user identity tests prevent credential or reading mix-ups.
- [ ] ISC-18: Calculation tools declare their persistent side effects.
- [ ] ISC-19: Input preflight validates actual input schemas.
- [ ] ISC-20: Workflow execution reports missing expected engine results.
- [ ] ISC-21: Sensitive errors and credential output are bounded and sanitized.
- [x] ISC-22: Report phase authority uses authenticated user phase only.
- [x] ISC-23: Focused phase regression tests exercise authorization behavior.
- [ ] ISC-24: Worker typecheck and focused tests pass.
- [ ] ISC-25: MCP protocol tests exercise initialization, discovery and calls.
- [ ] ISC-26: Deployment source and rollback instructions are recorded.
- [ ] ISC-27: A verified live MCP endpoint is configured in the package.
- [ ] ISC-28: Listing uses confirmed publisher, country and commerce facts.
- [ ] ISC-29: All four public listing URLs are verified for relevant content.
- [x] ISC-30: Five positive and three negative review cases are recorded.
- [ ] ISC-31: A real reviewer-accessible demo recording is verified.
- [ ] ISC-32: Submission portal state and required human attestations are accurately recorded.

## Test Strategy

- ISC-1..3: Git log, ls-remote, GH commit read, and rev-list compare exact revisions.
- ISC-4..7: Scoped provider CLI reads establish identity, resources, and deployments; retain only sanitized metadata.
- ISC-8..9: Source paths and lines plus read-only health/catalog probes establish behavior; no authenticated personal calculation.
- ISC-10..11: Read back the revised specification and cross-check against all three audit reports.
- ISC-12: Review commands, agent scope and final Git status; production changes are excluded.

## Features

- Code audit: customer auth, route semantics, capabilities, persistence, reports; ISC-8; parallel.
- Cloudflare audit: profile, gateway, bindings, routes, versions; ISC-4..5; parallel.
- Railway audit: identity, services, deployments, health, source relationship; ISC-6..7 and ISC-9; parallel.
- Reconciliation: source divergence, corrected specification, next steps; ISC-1..3 and ISC-10..12; after audits.

## Decisions

- 2026-09-30: User explicitly requested execution toward public submission. Reopened this ISA with stable criteria13–32; prior audit evidence retained.
- 2026-09-30: Main password-login handler returns410 AUTH_RETIRED. Use secure per-user API-key consent linking, not fabricated password login or shared owner credentials.

- 2026-09-30: User explicitly requested three-way fanout; workers inspect code, Cloudflare and Railway independently.
- 2026-09-30: refined: existing Cloudflare/Railway hosting is the intended implementation context. Missing Sites guides are not a project-level blocker.
- 2026-09-30: Root ISA describes historical completed app work; preserve it and articulate this plugin investigation locally.
- 2026-09-30: Twelve evidence criteria cover this bounded audit. Expanding to 32 would manufacture implementation acceptance work outside this turn's scope.
- 2026-09-30: Advisor helper attempted via Inference.ts; provider rejected with usage-credit requirement. No advisor approval is claimed.

## Changelog

- 2026-09-30 | conjectured: existing bridge documentation could directly specify input validation and customer readings.
  refuted by: implementation inspection found output validation, minimal info metadata, persistence, and owner-token gateway behavior.
  learned: plugin design must derive from current handlers and independently verified deployment.
  criterion now: ISC-8 and ISC-10 require source-derived corrections before implementation.

## Verification

- ISC-1: Git log — local HEAD `3e9ecb61e3f4a4b40ac624a5ecefa5753876b26f`; AUDIT-SUMMARY.md source-state table.
- ISC-2: Git remote/GitHub read — main `ebd97fe940a3454b0425ff60d56003b07337564e`.
- ISC-3: Git rev-list — exact output `2 18`; ancestry limits explained in summary.
- ISC-4: Wrangler and scoped account read — authenticated exact 9d9d account; CLOUDFLARE-AUDIT.md identity section.
- ISC-5: Report readback — CLOUDFLARE-AUDIT.md resource versions, bindings, routes and live-script comparison.
- ISC-6: Railway CLI — authenticated Mage Narayan; checkout unlinked; RAILWAY-AUDIT.md identity section.
- ISC-7: Railway provider read — seven running services, source identifiers and missing root commitHash explicitly recorded.
- ISC-8: Source review — CODE-AUDIT.md handler line references, exact-main comparisons, 12 focused synthetic tests passed.
- ISC-9: HTTP probes — root capabilities 404; TS capabilities 200 with six rows; source-local root route explicitly differentiated.
- ISC-10: Specification readback — revised PLUGIN-SPEC.md replaces Sites assumption, input/output validation mistake and persistence claim.
- ISC-11: Specification readback — seven-step implementation order lists identity, source parity, protocol, deployment and connected-reading gates.
- ISC-12: Command/agent audit and Git status — only plugin documentation created/rewritten; original modified-file list retained. No production mutation performed.

Implementation reopened with thirty-two stable criteria. The original twelve investigation criteria remain complete; later criteria require their own source, test, live, or developer evidence. Overall submission remains incomplete.

- ISC-13: Managed worktree selemene-chatgpt-plugin, branch codex/selemene-chatgpt-plugin starts at ebd97fe940a3454b0425ff60d56003b07337564e.
- ISC-22..23: Commit 45b04a75 and PHASE-REPAIR.md; actual report handlers tested through JWT middleware with ten HTTP authorization scenarios. Local source/test evidence only; not deployed.
- ISC-30: plugins/selemene-engine/plugin.json contains five positive and three negative cases; REVIEW-CASES.md marks all connected-host executions Not run.
