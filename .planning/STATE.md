# State

## Current

- **Active milestone**: m1 (P4 + P5 in progress)
- **Open PRs**: 27 (agent-dispatch scaffold queue)
- **PR queue health**: all open PRs are check-green (`SUCCESS/SKIPPED`) and mergeable; 2 remain draft (face-reading/FMRL excluded scope)

## Recent Activity

- **2026-05-04**:
  - Opened PR #652 — v3.1.0: security audits, input validation hardening, usage analytics & migration guide
  - Branch cleanup: deleted 66 merged `agent/issue-*` branches (66 deleted, 8 retained as unmerged)
  - Updated `noesis-admin` ruleset to exclude `refs/heads/agent/**` from deletion protection
  - Main now at PR #651 (V22-W1-S2-09 merged); V22-W1 numerology+biorhythm work fully landed

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

- None active. 8 unmerged agent branches with genuine work (see below).

## Unmerged Agent Branches (8 remaining)

| Branch | Work | Status |
|--------|------|--------|
| `agent/issue-373-v22-w1-s2-06` | `ForecastDay` aesthetic + spiritual fields | **Needs PR** — fields absent from main |
| `agent/issue-374-v22-w1-s2-07` | Compatibility biorhythm calculation | Likely redundant — compat types already in main |
| `agent/issue-361-v22-w1-s1-01` | Numerology `types.rs` extraction | Evaluate against main |
| `agent/issue-368-v22-w1-s2-01` | Biorhythm `types.rs` extraction | Evaluate against main |
| `agent/issue-237-p4-w1-s2-05` | TS bridge golden output fixtures | P4 work — needs PR |
| `agent/issue-242-p4-w1-s3-02` | Workflow-level duration metrics | P4 work — needs PR |
| `agent/issue-263-p4-w2-s3-01` | 60-minute auth soak test (k6) | P4/P5 work — needs PR |
| `agent/issue-338-p5-w3-s1-02` | Final cargo audit + dep fixes | Covered by PR #652 — verify/close |

## Next

1. PR `agent/issue-373` (ForecastDay secondary cycles) — genuine missing feature
2. Evaluate issues 361, 368, 374 for redundancy vs. main
3. PR issues 237, 242, 263 (P4 gate items)
4. Verify issue-338 fully covered by PR #652 and close
5. Gate D: p95 SLO + load validation
6. Gate E: canary + rollback drill, release checklist signoff

## Notes

- The full GSD `/gsd:new-project` workflow was bypassed for this brownfield init — the repo's existing `.context/` Substrate documentation provides the deep context that `/gsd:new-project` would gather through questioning. Only `.planning/` skeleton was synthesized to enable `task-master-planner` and the agent-ready dispatch loop.
- `.context/planning/` archived to `.context/planning/_archive-2026-03/` (25 files, untouched, frozen 2026-03-02).
