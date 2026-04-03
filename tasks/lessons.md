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

## 2026-03-27 — Live Callback Verification Before Auth Claims

- For frontend auth fixes, do not stop at local tests plus `git push`.
- Re-check the live login button’s generated OAuth authorize URL on the exact production/custom domain before claiming the fix is live.
- If the live callback path or redirect URI still reflects old behavior, treat it as a deployment-state problem first, not a new code regression.

## 2026-03-28 — Custom Domain vs Direct Origin CORS Rule

- When localhost CORS fails against a deployed API, test both:
  - the custom/public API domain,
  - the direct platform origin (`*.up.railway.app`, etc.).
- If the direct origin returns `Access-Control-Allow-Origin` but the custom domain does not, treat it as an edge/domain-layer issue, not an application CORS bug.
- For local debugging, prefer the direct service URL until the custom-domain edge behavior is fixed.

## 2026-03-28 — Failure Mode Re-Classification Rule

- When one blocking failure is removed, immediately re-classify the next failure from fresh evidence instead of continuing to reason from the old root cause.
- For auth flows specifically:
  - CORS failure means the request never reached the application logic.
  - callback-page `internal error` means the request reached the application and failed deeper in the exchange or persistence path.
- After a user reports "CORS is gone but still broken", jump straight to:
  - callback network response body/status,
  - live application logs,
  - deploy/runtime config parity.
