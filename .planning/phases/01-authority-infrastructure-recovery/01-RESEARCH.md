# Phase 1 research — 2026-09-05

Confidence: high for captured local/provider metadata, explicitly limited for runtime identity, DNS and semantic completeness. Research used repository source, git history/stash inspection, live GitHub/Railway/Cloudflare APIs, dependency audit feeds and current official configuration documentation.

## Reusable authority

Use existing contracts/v1, the three recovered plan files and the live issue IDs. Do not introduce another acceptance spec. The stashed ISA contains all 170 checkout criteria plus later gate/contract history. A stable-ID coverage comparison precedes restoration.

## Findings and implications

1. Current source at 9a05f5c includes two unmerged capability slices; CI #1486 has a formatter error. Finish prerequisite gates before broad engine semantics.
2. The actual admin capability handler enumerates `state.bridge().engines()`. Native Rust and database-conditional `biofield-capture` still need explicit capability coverage.
3. Railway SUCCESS can lack a source commit. Record service IDs, digests and unavailable fields; do not infer a release revision from health.
4. Cloudflare profile 9d9d proves account/zone/Worker ownership, but DNS read returns 10000. Pattern-memory is declared but returns 10007. Missing permission and missing resource are different outcomes.
5. TS/Python watch configuration appears under `deploy.watch`; official Railway docs place it under `build.watchPatterns`. TS source builder and effective Dockerfile builder differ. Repair source only after config review; separately verify deployed effect.
6. Node production dependency audit is red. Rust advisories are clear but a yanked version needs disposition; Python local pip is vulnerable and production Python resolution is unpinned.
7. CodeGraph index and symbol/context probes succeed. Cross-file matching is best effort; compiler/tests remain required.

## Independent plan review

A bounded noesis-plan advisor reviewed the supplied factual dossier and returned CONCERNS allowing continuation with tighter gates: explicit acceptance ownership, deployment falsifiers, and separate source/merge/deploy evidence. Its mistaken phrase `570×30` is rejected: live corpus is 19×30=570. It did not inspect code or execute tests. Earlier Observe/Hands calls failed and were not accepted as completed work.

## Primary sources

- docs/plans/selemene-engine/RECOVERY-2026-09-05.md — exact local/remote evidence and limitations.
- docs/plans/selemene-engine/INFRASTRUCTURE-MAP.json — sanitized live resource/deployment inventory.
- docs/plans/selemene-engine/ENGINE-ISSUE-INDEX.json — all existing engine issue keys.
- https://docs.railway.com/reference/config-as-code — documented build/watch fields and configuration precedence.
- https://developers.cloudflare.com/workers/wrangler/configuration/ — account/config/binding declarations.
- https://developers.cloudflare.com/fundamentals/api/how-to/make-api-calls/ — scoped API authorization.

## Do not hand-roll

Use GSD parsers/validators for executable phase structure, CodeGraph for structural lookup, package managers for locked dependency resolution and provider APIs/CLIs for resource readback. No manual source-state invention, no bulk stash apply and no extra GitHub issue swarm.
