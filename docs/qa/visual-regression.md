# Visual Regression Workflow — Admin Web

> ADR-31 | Area: QA | Owner: Tech Lead

## Overview

Screenshot-based regression tests guard the admin shell and core pages against unintentional visual drift. Tests live in `apps/admin-web/tests/visual/` and use Playwright.

## Covered Routes

| Route | Screenshot name |
|---|---|
| `/dashboard` | `dashboard.png` |
| `/users` | `users.png` |
| `/api-keys` | `api-keys.png` |
| `/history-sync` | `history-sync.png` |
| `/analytics` | `analytics.png` |
| `/system` | `system.png` |
| `/audit` | `audit.png` |

Viewport: **1440 × 900** (desktop Chrome). Threshold: ≤ 2% pixel diff ratio.

## Prerequisites

1. Install Playwright browsers: `npx playwright install chromium`
2. Generate an admin auth session file:
   ```bash
   npx playwright codegen --save-storage=tests/visual/auth.json http://localhost:3001/login
   ```
   Log in through the Discord OAuth flow. The resulting `auth.json` stores cookies/localStorage for subsequent test runs.

3. Start the dev server: `npm run dev` (port 3001)

## Capturing a Baseline

Run once after a major design change to commit new reference screenshots:

```bash
cd apps/admin-web
npx playwright test --update-snapshots
```

Baseline images land in `tests/visual/.snapshots/`. **Commit them** — they are the regression reference.

## Running Regression Checks

```bash
cd apps/admin-web
npm run test:visual
# or
npx playwright test
```

A diff report is written to `tests/visual/.report/index.html`.

## CI Integration

Add to your pipeline (example GitHub Actions step):

```yaml
- name: Install Playwright browsers
  run: npx playwright install --with-deps chromium
  working-directory: apps/admin-web

- name: Run visual regression
  run: npm run test:visual
  working-directory: apps/admin-web
  env:
    PLAYWRIGHT_BASE_URL: ${{ secrets.ADMIN_STAGING_URL }}
    PLAYWRIGHT_AUTH_STATE: tests/visual/auth.json
```

The `auth.json` must be a CI secret or re-generated per run via a login script.

## Updating Snapshots After Intentional Changes

After any approved design change, run with `--update-snapshots`, review the diffs in the HTML report, then commit the updated baseline images alongside the code change.
