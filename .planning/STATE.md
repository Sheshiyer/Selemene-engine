# State

## Current

- **Active milestone**: m1 (P4 + P5 in progress)
- **Open PRs**: 27 (agent-dispatch scaffold queue)
- **PR queue health**: all open PRs are check-green (`SUCCESS/SKIPPED`) and mergeable; 2 remain draft (face-reading/FMRL excluded scope)

## Recent Activity

- **2026-04-28**:
  - Completed smallest repeated CI failure bucket repair across 7 branches (Clippy)
  - Merged `origin/main` baseline into affected branches and pushed reruns
  - Patched branch-local rustdoc clippy lint on V22 compatibility PR branch
  - Confirmed queue is now primarily merge sequencing, not red-check remediation
- **2026-04-26**:
  - Closed PR #567 (1.5M-line scope-contaminated branch — fully redundant via #568, #596, #597, #432)
  - Opened PR #598 — wire witness contract into CI (workflow-parity job)
  - Filed issue #599 → PR #600 (B.1 dispatch test) — homegrown loop fired in 25s, Copilot bot kickoff posted
  - Opened PR #601 — unblock upstream Lint (clippy::iter_kv_map, sort_by_key) + Audit (rustls-webpki bump + 0.101.7 ignores)
  - Initialized `.planning/` (this PR)
- **2026-04-24**: Witness contract test added (#432); agent-dispatch + auto-ready + merge-lane + post-merge pipeline shipped (#568, #596, #597)
- **2026-03-31**: PR #549 — docs cleanup, planning moved to `.context/`
- **2026-02-24**: m1 roadmap published

## Open Blockers

- No global CI blocker currently observed for the open PR queue
- Remaining risk is merge sequencing and scope gating for excluded drafts (#591, #577)

## Next

1. Merge wave 1 (8 `CLEAN` + mergeable PRs)
2. Merge wave 2 (17 `UNSTABLE` but mergeable/check-green PRs)
3. Keep drafts #591 and #577 parked (face-reading/FMRL excluded scope)
4. Branch cleanup pass — 35 no-PR remote branches (Phase A.5)
5. Backfill `phases/P{1,2,3}/VERIFICATION.md` from `.context/planning/_archive-2026-03/`

## Notes

- The full GSD `/gsd:new-project` workflow was bypassed for this brownfield init — the repo's existing `.context/` Substrate documentation provides the deep context that `/gsd:new-project` would gather through questioning. Only `.planning/` skeleton was synthesized to enable `task-master-planner` and the agent-ready dispatch loop.
- `.context/planning/` archived to `.context/planning/_archive-2026-03/` (25 files, untouched, frozen 2026-03-02).
