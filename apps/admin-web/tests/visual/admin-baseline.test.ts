import { test, expect } from "@playwright/test";

/**
 * Admin web visual regression baseline.
 *
 * These tests capture screenshots of key protected routes and compare them
 * against a committed baseline. They require the admin-web dev server to be
 * running on PLAYWRIGHT_BASE_URL (default: http://localhost:3001) and a valid
 * admin session cookie/localStorage token.
 *
 * To capture a fresh baseline:
 *   pnpm exec playwright test --update-snapshots
 *
 * To run regression checks:
 *   pnpm exec playwright test
 *
 * CI: see docs/qa/visual-regression.md for the documented workflow.
 */

const ROUTES = [
  { path: "/dashboard", name: "dashboard" },
  { path: "/users", name: "users" },
  { path: "/api-keys", name: "api-keys" },
  { path: "/history-sync", name: "history-sync" },
  { path: "/analytics", name: "analytics" },
  { path: "/system", name: "system" },
  { path: "/audit", name: "audit" }
] as const;

/** Storage state path; generate with `playwright codegen --save-storage=auth.json` */
const AUTH_STATE = process.env.PLAYWRIGHT_AUTH_STATE ?? "./tests/visual/auth.json";

test.use({ storageState: AUTH_STATE });

for (const route of ROUTES) {
  test(`${route.name} — visual baseline`, async ({ page }) => {
    await page.goto(route.path);
    // Wait for shell to settle past the loading skeleton
    await page.waitForSelector(".shell-main-grid", { timeout: 8000 });
    await expect(page).toHaveScreenshot(`${route.name}.png`, {
      fullPage: false,
      maxDiffPixelRatio: 0.02
    });
  });
}
