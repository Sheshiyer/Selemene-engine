---
project: Selemene ChatGPT plugin
task: Implement customer MCP and prepare public plugin submission
effort: E3
phase: verify
progress: 28/32
mode: algorithm
started: 2026-09-29
updated: 2026-09-30
---

## Problem

The first plugin specification was based on incomplete bridge documentation and assumed Sites hosting. The user requested a parallel deep pass over current source and existing Cloudflare/Railway infrastructure before proceeding.

## Vision

The plugin uses Selemene's actual account, execution, and storage contracts. The implementation plan distinguishes current deployment, newer local work, and unverified operational behavior.

## Out of Scope

- The prior audit made no deployment, merge, or credential changes. The later authorized implementation created a dedicated connector Worker and OAuth KV namespace; its live acceptance remains separate from local verification.
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
- [x] ISC-14: A real MCP server exposes the seven reviewed tool contracts.
- [x] ISC-15: OAuth metadata and PKCE flow are implemented with maintained provider library.
- [x] ISC-16: OAuth customer linkage validates a per-user upstream credential.
- [x] ISC-17: Cross-user identity tests prevent credential or reading mix-ups.
- [x] ISC-18: Calculation tools declare their persistent side effects.
- [x] ISC-19: Input preflight validates actual input schemas.
- [x] ISC-20: Workflow execution reports missing expected engine results.
- [x] ISC-21: Sensitive errors and credential output are bounded and sanitized.
- [x] ISC-22: Report phase authority uses authenticated user phase only.
- [x] ISC-23: Focused phase regression tests exercise authorization behavior.
- [x] ISC-24: Worker typecheck and focused tests pass.
- [x] ISC-25: MCP protocol tests exercise initialization, discovery and calls.
- [x] ISC-26: Deployment source and rollback instructions are recorded.
- [x] ISC-27: A verified live MCP endpoint is configured in the package.
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
- ISC-14..21,24..25: Actual OAuth provider and SDK protocol integration against synthetic upstream and KV; distinguish local assertions from deployed customer acceptance.
- ISC-26..27: Deployment/version receipt and sanitized public metadata/challenge checks, followed by source readback of the packaged MCP URL. These checks do not establish an authenticated ChatGPT connection.

## Features

- Code audit: customer auth, route semantics, capabilities, persistence, reports; ISC-8; parallel.
- Cloudflare audit: profile, gateway, bindings, routes, versions; ISC-4..5; parallel.
- Railway audit: identity, services, deployments, health, source relationship; ISC-6..7 and ISC-9; parallel.
- Reconciliation: source divergence, corrected specification, next steps; ISC-1..3 and ISC-10..12; after audits.

## Decisions

- 2026-09-30: User explicitly authorized submission and website publication. Website PR13 is merged and its automatic production deployment verified. Submission remains incomplete: authenticated portal access and an available individual publisher identity were observed, but public policy facts and reviewer evidence cannot be inferred from infrastructure credentials.

- 2026-09-30: Initial deployment `a1ddbb65` / `9e80e849-b54b-422a-b45a-9efd0598c895` passed public discovery but exposed a browser Origin policy defect. Parent subsequently deployed `c79806c0` / `e8d38136-faa5-426f-abcd-63a5e56d76b7` and verified the Deny path in the in-app browser. Authenticated customer acceptance remains open.

- 2026-09-30: User approved the existing website draft and requested MotionSkin plus MotionSites MCP. Design/content approval is recorded; missing publisher/contact/retention facts remain unresolved, not silently inferred. Selected Particle Field template via live MCP; preserve its five-panel motion with existing Tryambakam sigil and shared companion-page shell. Website draft baseline d9a1127; new visual verification required after redesign.

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

- ISC-14..15: BUILD-RECEIPT.md and SECURITY-REVIEW.md; official SDK 1.31.0 implements seven tools, OAuth provider 1.2.1 implements DCR, S256, browser-bound consent, canonical resources and token lifecycle. Public authorization metadata additionally advertises S256 and both supported scopes.
- ISC-16..17: Synthetic full-provider tests link two separate customers, verify per-user upstream keys and encrypted KV values, and execute SDK calls with isolated credentials. No real customer connection is claimed.
- ISC-18..20: Tool metadata declares write effects; local preflight rejects missing/invalid inputs before calculations; strict workflow composition and missing-engine output handling are exercised by integration tests. Persistence is reported as not confirmed.
- ISC-21: Source and synthetic tests cover bounded request/response handling, scope challenges, sanitized upstream failure output, credential isolation and no automatic retries. SECURITY-REVIEW.md records the bounded review and its limits.
- ISC-24..25: BUILD-RECEIPT.md records typecheck and the initial 16 passing tests; independent review reran those 16 successfully. Parent first reported 17, then reported the builder's final 18 passing local tests after the consent-policy repair. Later counts are attributed evidence, not independent reruns by this ISA reviewer.
- ISC-26: DEPLOYMENT-RECEIPT.md records initial source/version/account, dedicated namespace, endpoint, deployment output and containment/rollback procedure. This first deployment has no older working connector version to restore.
- ISC-27: plugins/selemene-engine/mcp.json contains `https://selemene-mcp.sheshnarayan-iyer.workers.dev/mcp`. live-endpoint-check.json records health 200, authorization metadata 200, canonical protected-resource metadata 200, and unauthenticated MCP 401 with canonical challenge. DEPLOYMENT-RECEIPT.md records synthetic DCR 201 and browser form rendering. Completion is limited to endpoint deployment/discovery/configuration, not authenticated usability.

## Outstanding release gates

- **Consent Deny is now verified; authenticated approval remains pending.** Parent reports `c79806c0` deployed as `e8d38136-faa5-426f-abcd-63a5e56d76b7`. In-app-browser/CDP evidence showed the Deny POST carrying the exact canonical Origin and returning 302 to the synthetic loopback callback with `error=access_denied`, state and canonical issuer, plus response `Referrer-Policy: no-referrer`. Callback navigation aborted because no local callback server was running; the observed success is the connector's rejection redirect, not callback receipt. No key entry, token exchange or reading occurred.
- **Connected ChatGPT/customer acceptance remains pending.** Public reachability, DCR and a rendered form do not prove key linkage, token exchange, account-scoped tools, disconnect/reconnect or a consented synthetic reading in the real host.
- **ISC-28 and ISC-29 remain open.** Publisher/controller, country, commerce, contacts and policy commitments require confirmed facts; approved draft design does not resolve missing legal metadata or verify all public listing pages.
- **ISC-31 remains open.** No real reviewer-accessible demonstration recording has been verified.
- **ISC-32 remains open.** No plugin upload, portal review submission, human attestation or publication is claimed.

Progress is 28 of 32 recorded criteria. The checklist's local implementation and public discovery completions do not waive the browser/customer release gates above.

- Authorized release follow-up: website PR13 merged as `df7ced81b80cc7a76e3b4f3c353468124db2a8c8`; Vercel deployment `dpl_Ct5sR7cWonh36vyyree5qj3jTerE` Ready. Twelve live route/asset checks returned200 and matched source bytes (`website-main-live-checks.json`). Authenticated OpenAI portal exposes an individual publisher identity and Upload Plugin, but no Selemene package was uploaded or submitted. ISC-28,29,31,32 remain open.

- Website follow-up: MotionSkin site deployed from tryambakam-space6feceab as dpl_9YMGkwHxcV3C8ZTjtP5jABfyeBuS and promoted to www.tryambakam.space. All5routes and5animation assets returned200 with source byte equality; desktop/mobile/reduced-motion/no-JS/module-failure checks passed. ISC-29 remains open because privacy/terms/support are review drafts awaiting publisher/contact/retention decisions. Landing websiteURL is configured; this does not approve effective legal policies.
