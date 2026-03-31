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

## 2026-02-28 — Vercel Monorepo Binding Rule

- Before advising `Root Directory`, verify the Vercel project is connected to the intended GitHub repository and branch.
- If build logs show very low file counts and "Root Directory does not exist", suspect repo/link mismatch first.
- Preserve env values before destructive resets, then re-import with explicit monorepo root selection.

## 2026-03-02 — Schema Drift Diagnosis Rule

- If API endpoint returns 500 and null checks are clean, immediately test for missing columns/indexes expected by latest code.
- Distinguish **null data shape issues** from **missing schema objects** using `information_schema.columns` before patching query defaults.
- Add compatibility fallbacks in repository layer for optional, recently introduced columns to reduce incident blast radius during phased migrations.

## 2026-03-10 — Panchanga Table Interpretation Rule

- When comparing external Panchanga tables against engine output, do not assume every `upto` time refers to the core nakshatra or tithi itself.
- Distinguish clearly between:
  - nakshatra end time,
  - nakshatra pada transition time,
  - karana sequence windows,
  - yoga transition time.
- If the source is a screenshot or pasted table, resolve row semantics first before calling a mismatch.

## 2026-03-25 — Deployment Claim Verification

- Do not imply a change is committed, on `main`, or live until all three checks agree:
  - `git status` shows no local-only delta for the claimed change,
  - `git log` shows the relevant commit on the current branch,
  - the live URL reflects the claimed behavior when inspected directly.
- If any of those checks disagree, state the exact state plainly: local only, committed but undeployed, or deployed but stale.

## 2026-03-25 — Vercel Settings Change Is Not Deployment Proof

- Treat Vercel settings edits as inert until a fresh production deployment is confirmed and the custom domain resolves to that deployment.
- If the user reports the same 404 after changing `Root Directory` or framework settings, immediately verify:
  - whether the project settings were saved,
  - whether a new production deployment was created after the save,
  - whether the custom domain is attached to this exact project and environment.
