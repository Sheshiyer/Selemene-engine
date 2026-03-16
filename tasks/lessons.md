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

## 2026-03-15 — Runtime Scrub vs Repo Scrub Rule

- Do not equate "removed from the active runtime path" with "fully scrubbed from the repository."
- After decommissioning a dependency, explicitly audit four residue surfaces before claiming it is scrubbed:
  - runtime config and env vars,
  - deployment scripts and compose files,
  - operator-facing docs,
  - optional crates or historical records that should be marked retained.
- Final reports must separate:
  - fully removed from active runtime,
  - intentionally retained for history or optional integrations,
  - stale residue that still needs cleanup.

## 2026-03-15 — API Key Verification Surface Rule

- When a live API key test fails on a branded production domain, distinguish edge/network denial from application auth before concluding the key is bad.
- If the public domain is fronted by Cloudflare or similar, retry against the direct app origin documented in deployment files when available.
- If `GET /api/v1/engines/{id}/info` returns `401`, treat it as an auth failure first, not a phase/tier failure, because `info` is authenticated but not phase-gated in this codebase.
- When the user provides a replacement credential, rerun the same minimal auth probe first (`/api/v1/engines`) before spending time on full engine payload analysis.
