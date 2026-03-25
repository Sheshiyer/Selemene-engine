# Lessons Learned

## 2026-02-25 — Bootstrap Rule

- For new surface areas (like `apps/admin-web`), land a thin vertical slice first:
  - deployable scaffold,
  - real auth/session handshake,
  - route shell,
  - explicit run/deploy docs.

## 2026-03-25 — Deployment Claim Verification

- Do not imply a change is committed, on `main`, or live until all three checks agree:
  - `git status` shows no local-only delta for the claimed change,
  - `git log` shows the relevant commit on the current branch,
  - the live URL reflects the claimed behavior when inspected directly.
- If any of those checks disagree, state the exact state plainly: local only, committed but undeployed, or deployed but stale.
