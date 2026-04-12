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

## 2026-01-31 — Wave 1 Retrospective Lessons

### Sequential Gate Mapping (NOT King Wen)
- Human Design uses 360°/64 = 5.625° sequential mapping, NOT the I-Ching King Wen philosophical ordering.
- Using King Wen order produces completely wrong charts. Professional HD software (MMI, Jovian Archive) all use sequential.
- Validated across 16 reference charts with 100% accuracy.

### Swiss Ephemeris Initialization
- Swiss Ephemeris requires synchronous init with a file path. Must initialize once at startup, share via `Arc`.
- Ephemeris data files must be present before any engine using Swiss Eph can start.

### Consciousness Level Gating at Orchestrator Layer
- Auth middleware checks happen at the HTTP boundary, but internal workflow calls between engines bypass HTTP.
- Enforce consciousness level at orchestrator layer so internal engine-to-engine calls still respect tier access.

### Witness Prompt Philosophy
- All prompts must use non-prescriptive inquiry format. Never advice, never commands.
- Automated tests enforce the language contract (no "must", "should", "do this").

### Cache Strategy for Immutable Calculations
- Birth data calculations are immutable (same input → same output), so infinite TTL for L3 disk cache is appropriate.
- Only L1 in-memory cache needs LRU eviction for memory management.

### Prometheus Metrics in Tests
- Multiple test cases registering the same Prometheus metrics causes panics.
- Solve with `OnceLock` singleton pattern for the router — metrics register exactly once.

### HD Center Serialization
- Normalize HashMap-keyed center data to sorted array output for consistent JSON serialization across test/prod.

### Performance Baselines (Wave 1)
- HD engine: 1.31ms (target <100ms, 76x faster)
- Gene Keys: 0.012ms (target <50ms, 4166x faster)
- Vimshottari: <1ms (target <200ms, 200x faster)
- API p95: <100ms (target <500ms)

## 2026-02-11 — Sprint 2 Cleanup Lessons
- Deleted 40 files (legacy/, archive/, deprecated scripts) with zero compilation errors or test regressions.
- Always verify no workspace references before deleting directories (`Cargo.toml` workspace members list).
- Old `.claude/` session files accumulate — periodically clean.

## 2026-02-16 — Doc Consistency Standards

### Vocabulary Guardrails
- Preferred terms: `authorship`, `reflection`, `inquiry`, `witness`, `synthesis`, `mirrors`.
- Disallowed in user-facing copy: `journey`, `manifesting`, `vibration`.
- Imperative wording OK only for technical/operational content (commands, auth, runbook steps).

### Link Policy
- API runtime examples: `https://selemene.tryambakam.space`
- Parent field context: `https://tryambakam.space`
- Somatic callouts: `https://1319.tryambakam.space` (only in embodiment contexts)

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
