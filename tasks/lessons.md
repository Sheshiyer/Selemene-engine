# Lessons Learned

## 2026-02-25 — Bootstrap Rule

- For new surface areas (like `apps/admin-web`), land a thin vertical slice first:
  - deployable scaffold,
  - real auth/session handshake,
  - route shell,
  - explicit run/deploy docs.

## 2026-02-28 — Incident Verification Rule

- After user reports "still broken", do not assume local fixes are live.
- Re-verify production URLs and behavior immediately before proposing next actions.
- For Next.js on Vercel, prefer `next.config` redirects over `vercel.json` route rules for app routing behavior.
- Provide a crisp "what URL should work right now" answer before deeper debugging.
