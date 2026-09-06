---
phase: 02-reproducible-gates-dependency-repair
plan: "05"
status: complete
completed: 2026-09-06
source_commit: 7a5793d94bc850976f29db9277527e475d7f127b
requirements_partial: ["GATE-05"]
---
# Remote verification and critical protection gate

Draft PR #1488 passes all 16 jobs in GitHub run 33973728459 at source `7a5793d94bc850976f29db9277527e475d7f127b`. GitHub's pull-request merge tree has the same tree object as the source commit. The strict capability route fixture passes with PostgreSQL 16, the regenerated route inventory matches the router, both Linux Python images import, both Python versions pass contracts and live sidecar smoke, the admin job passes, audits pass and CI Gate succeeds.

All eight roadmap control bodies have dated evidence with readback hashes. Their original exit checkboxes, labels and OPEN state are preserved. A fresh GitHub query still returns 570 open W3E issues, 19 engines and exactly 30 unique slots per engine. The existing branches and planning stash remain preserved.

The repository's public `ADMIN_WEB_URL` Actions variable now names the already verified admin origin; its unauthenticated smoke passes. No secret value was read or changed. A plan-sync dry run finds no `docs/planning/*.json` inputs, so the candidate tree supplies no issue mutations even though a main push would start that changed workflow.

The user approved the exact additive main CI rule on 2026-09-06. Ruleset `15597830` was rechecked against the preserved baseline, updated, read back, and resolved against `main`. It now requires strict, up-to-date `CI Gate` from GitHub Actions integration `15368`, while preserving deletion and non-fast-forward protections, branch conditions and the empty bypass list. The reviewed request and rollback body remain in `MAIN-CI-RULE-PROPOSAL.json`. This approval did not authorize merging PR #1488 or deploying. Production promotion remains held because a main merge triggers Railway/Vercel/CD and immutable artifact equivalence, API source/schema identity and executable rollback evidence remain unproven.

Plans 06–07 are independently checked gap plans for the remaining registry and release-receipt authority. Phase 2 and the original Wave 0/1 exits remain open until those plans pass. Waves 2–6 and every engine's individual evidence obligations remain open.
