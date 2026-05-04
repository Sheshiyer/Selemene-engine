# Admin Web Rollout & Canary Checklist

> ADR-34 | Area: Docs | Owner: Tech Lead
> Prepared for: admin dashboard redesign (P4 / ADR-18 → ADR-34)

---

## Pre-Deployment Verification

### Build & Static Analysis
- [ ] `npm run build` passes with zero errors in `apps/admin-web/`
- [ ] `npm run typecheck` clean
- [ ] `npm run lint` clean
- [ ] No new `console.error` calls introduced in production code paths

### Functional Smoke (staging)
- [ ] Login via Discord OAuth completes and redirects to `/dashboard`
- [ ] All 7 nav routes render without JS errors: Dashboard, Users, API Keys, History Sync, Analytics, System, Audit
- [ ] Command palette opens (Cmd/Ctrl+K), filters items, navigates on Enter, closes on Escape
- [ ] Drawer opens and closes on Users and API Keys pages
- [ ] Bulk selection and clear work on at least one table page
- [ ] Sign-out clears token and returns to `/login`

### Accessibility (pre-launch)
- [ ] a11y-checklist.md items marked ✅ are confirmed in staging (see `docs/qa/a11y-checklist.md`)
- [ ] Keyboard-only sign-in → dashboard navigation completes
- [ ] Command palette keyboard journey works (Cmd+K → Arrow → Enter)
- [ ] No critical WCAG 2.1 AA violations from axe-core browser extension on Dashboard and Users routes

### Visual Regression
- [ ] Playwright baseline captured against staging: `npx playwright test --update-snapshots`
- [ ] No unexpected diffs on Chrome 1440 × 900 (see `docs/qa/visual-regression.md`)

---

## Telemetry Verification (ADR-30)

- [ ] `POST /api/telemetry` responds 200 on staging
- [ ] Command palette select events appear in Railway log stream within 30 s of interaction
- [ ] Sign-out event logged: `{ name: "admin_sign_out" }`
- [ ] Core Web Vitals reported in log stream (LCP, CLS, FID) after first page load

---

## Canary Rollout Plan

### Phase 1 — Internal (0 → 10% of admin sessions)
- Deploy to Railway production
- Enable for internal operators only (bypass traffic split or use feature-flag user list)
- Monitor Railway log stream for: `5xx` spikes, `admin_sign_out` rate, telemetry volume
- Canary window: **48 hours**

### Phase 2 — Broadened (10% → 50%)
- Confirm no P0/P1 regressions from Phase 1
- Confirm visual regression baseline matches production screenshots
- Canary window: **48 hours**

### Phase 3 — Full rollout (50% → 100%)
- Final sign-off from Tech Lead
- Confirm ADR-30 through ADR-33 acceptance criteria all ✅
- Merge canary flag / enable for all sessions

---

## Rollback Procedure

1. Revert to previous Railway deployment via the Railway dashboard (Deployments → rollback to last known good)
2. Or: `git revert <merge-commit-sha>` on `main` + force-deploy
3. Confirm via `/health/ready` that the API is healthy
4. Post incident note in #engineering with: root cause, affected window, rollback time

---

## Final Acceptance Sign-off

| ADR | Owner | Acceptance criteria met | Notes |
|---|---|---|---|
| ADR-30 (Telemetry) | Backend Eng | ☐ | Events visible in Railway logs |
| ADR-31 (Visual regression) | Tech Lead | ☐ | Playwright baseline committed |
| ADR-32 (A11y QA) | Tech Lead | ☐ | Keyboard journeys confirmed |
| ADR-33 (Design polish) | Frontend Eng | ☐ | Consistency pass reviewed |
| ADR-34 (Rollout) | Tech Lead | ☐ | This checklist completed |

> Final acceptance: Tech Lead signs off after all rows above are ✅ and Phase 3 rollout is complete.
