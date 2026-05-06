#!/usr/bin/env bun
// Dodo Payments — idempotent provisioning script.
//
// Creates the Selemene billing topology against Dodo's REST API:
//   3 subscription products (Free / Basic / Premium)
//   1 credit entitlement (Witness Credits)
//   1 usage meter         (noesis.engine_query)
//   1 webhook              (only if DODO_PROVISION_WEBHOOK_URL is set)
//
// Reads DODO_PAYMENTS_API_KEY + DODO_PAYMENTS_ENV from .env at repo root.
// Re-running is safe: the script lists existing objects and skips anything
// with a matching name.
//
// Usage:
//   bun run runbooks/scripts/dodo-provision.ts
//   DODO_PROVISION_WEBHOOK_URL=https://staging.example.com/api/webhook/dodo-payments \
//     bun run runbooks/scripts/dodo-provision.ts
//
// Prints an .env block at the end. Copy those values into your local .env
// (and the SQL UPDATE block into your Postgres) to finish provisioning.
//
// What this script does NOT do (must be done in dashboard, ~3 min total):
//   • Attach Witness Credits + the meter to each product with per-tier
//     credit settings (50 / 500 / 2 500 credits per cycle, overage rates).
//     The Dodo API does not currently expose this attachment surface;
//     it is dashboard-only as of 2026-05.

import { readFileSync, existsSync } from "node:fs";
import { homedir } from "node:os";
import { resolve, join } from "node:path";

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

interface ProductSpec {
  envVar: string;            // e.g. "DODO_PRODUCT_FREE_ID"
  planCode: string;          // matches plan_catalog.code
  name: string;              // dashboard-visible
  description: string;
  priceCents: number;        // 0 = free
}

const PRODUCTS: ProductSpec[] = [
  {
    envVar: "DODO_PRODUCT_FREE_ID",
    planCode: "free",
    name: "Witness Free",
    description: "Free access to consciousness engines, 50 Witness Credits / month",
    priceCents: 0,
  },
  {
    envVar: "DODO_PRODUCT_BASIC_ID",
    planCode: "basic",
    name: "Witness Basic",
    description: "Standard access, 500 Witness Credits / month, opt-in overage at $0.030/credit",
    priceCents: 900,
  },
  {
    envVar: "DODO_PRODUCT_PREMIUM_ID",
    planCode: "premium",
    name: "Witness Premium",
    description: "Pro access, 2 500 Witness Credits / month, lowest overage rate $0.015/credit, priority queue",
    priceCents: 2900,
  },
];

const ENTITLEMENT_NAME = "Witness Credits";
const METER_NAME = "noesis.engine_query";
const METER_EVENT_NAME = "noesis.engine_query";

const WEBHOOK_EVENTS = [
  "subscription.active",
  "subscription.updated",
  "subscription.on_hold",
  "subscription.cancelled",
  "subscription.failed",
  "payment.succeeded",
  "payment.failed",
  "credit.added",
  "credit.deducted",
  "credit.balance_low",
  "credit.overage_charged",
];

// ---------------------------------------------------------------------------
// Bootstrapping
// ---------------------------------------------------------------------------

function loadDotEnv(): Record<string, string> {
  const path = resolve(import.meta.dir, "../../.env");
  if (!existsSync(path)) return {};
  const out: Record<string, string> = {};
  for (const raw of readFileSync(path, "utf-8").split("\n")) {
    const line = raw.trim();
    if (!line || line.startsWith("#")) continue;
    const eq = line.indexOf("=");
    if (eq === -1) continue;
    out[line.slice(0, eq).trim()] = line.slice(eq + 1).trim();
  }
  return out;
}

/**
 * Fallback: read the credentials the `dodo` CLI stores after `dodo login`.
 * File format: { "<mode>": "<api_key>" } where mode is e.g. "test_mode" or
 * "live_mode". Returns null if missing or unparseable.
 */
function loadCliCreds(): { apiKey: string; mode: "test" | "live" } | null {
  const path = join(homedir(), ".dodopayments", "api-key");
  if (!existsSync(path)) return null;
  try {
    const json = JSON.parse(readFileSync(path, "utf-8")) as Record<string, string>;
    const [modeKey, apiKey] = Object.entries(json)[0] ?? [];
    if (!modeKey || !apiKey) return null;
    const mode = modeKey === "live_mode" ? "live" : "test";
    return { apiKey, mode };
  } catch {
    return null;
  }
}

const dotenv = loadDotEnv();
const cliCreds = loadCliCreds();
// Strip empty-string values from .env so they don't shadow other sources.
// (.env.example ships with `DODO_PAYMENTS_API_KEY=` empty; an empty value
// should fall through to the CLI fallback.)
const dotenvNonEmpty = Object.fromEntries(
  Object.entries(dotenv).filter(([, v]) => v !== ""),
);
const processEnvNonEmpty = Object.fromEntries(
  Object.entries(process.env).filter(([, v]) => v !== "" && v !== undefined),
);
const env = {
  ...(cliCreds
    ? { DODO_PAYMENTS_API_KEY: cliCreds.apiKey, DODO_PAYMENTS_ENV: cliCreds.mode }
    : {}),
  ...dotenvNonEmpty,
  ...processEnvNonEmpty,
};

const apiKey = env.DODO_PAYMENTS_API_KEY;
const envMode = env.DODO_PAYMENTS_ENV ?? "test";
const webhookUrl = env.DODO_PROVISION_WEBHOOK_URL ?? "";

if (!apiKey) {
  console.error("DODO_PAYMENTS_API_KEY not found.");
  console.error("Either:");
  console.error("  • run `dodo login` (CLI stores creds at ~/.dodopayments/api-key), or");
  console.error("  • add DODO_PAYMENTS_API_KEY=<key> to .env at repo root.");
  process.exit(1);
}
const credSource = process.env.DODO_PAYMENTS_API_KEY
  ? "process.env"
  : dotenv.DODO_PAYMENTS_API_KEY
    ? ".env"
    : "dodo CLI (~/.dodopayments/api-key)";
console.log(`▶ Using credentials from: ${credSource}`);
if (envMode !== "test" && envMode !== "live") {
  console.error(`DODO_PAYMENTS_ENV must be 'test' or 'live', got '${envMode}'.`);
  process.exit(1);
}

const baseUrl =
  envMode === "live"
    ? "https://live.dodopayments.com"
    : "https://test.dodopayments.com";

console.log(`▶ Provisioning against ${baseUrl} (${envMode} mode)`);
console.log("");

// ---------------------------------------------------------------------------
// HTTP helper
// ---------------------------------------------------------------------------

async function dodo<T>(method: "GET" | "POST", path: string, body?: unknown): Promise<T> {
  const res = await fetch(`${baseUrl}${path}`, {
    method,
    headers: {
      Authorization: `Bearer ${apiKey}`,
      "Content-Type": "application/json",
    },
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
  const text = await res.text();
  let parsed: unknown = text;
  try {
    parsed = JSON.parse(text);
  } catch {
    /* keep as text */
  }
  if (!res.ok) {
    console.error(`  ✗ ${method} ${path} → HTTP ${res.status}`);
    console.error(`    ${typeof parsed === "string" ? parsed : JSON.stringify(parsed, null, 2)}`);
    throw new Error(`Dodo ${method} ${path} failed: ${res.status}`);
  }
  return parsed as T;
}

// ---------------------------------------------------------------------------
// Helpers — list-then-create idempotency
// ---------------------------------------------------------------------------

interface PaginatedItems<T> {
  items: T[];
}

interface DodoProduct {
  product_id: string;
  name: string;
}
interface DodoEntitlement {
  id: string;
  name: string;
}
interface DodoMeter {
  id: string;
  name: string;
}
interface DodoWebhook {
  id: string;
  url: string;
}

async function findProductByName(name: string): Promise<DodoProduct | null> {
  // Page through up to 5 pages of 100; should cover every realistic case
  for (let page = 0; page < 5; page++) {
    const data = await dodo<PaginatedItems<DodoProduct>>(
      "GET",
      `/products?page_number=${page}&page_size=100`,
    );
    const hit = data.items.find((p) => p.name === name);
    if (hit) return hit;
    if (data.items.length < 100) return null;
  }
  return null;
}

async function findEntitlementByName(name: string): Promise<DodoEntitlement | null> {
  for (let page = 0; page < 5; page++) {
    const data = await dodo<PaginatedItems<DodoEntitlement>>(
      "GET",
      `/credit-entitlements?page_number=${page}&page_size=100`,
    );
    const hit = data.items.find((e) => e.name === name);
    if (hit) return hit;
    if (data.items.length < 100) return null;
  }
  return null;
}

async function findMeterByName(name: string): Promise<DodoMeter | null> {
  for (let page = 0; page < 5; page++) {
    const data = await dodo<PaginatedItems<DodoMeter>>(
      "GET",
      `/meters?page_number=${page}&page_size=100`,
    );
    const hit = data.items.find((m) => m.name === name);
    if (hit) return hit;
    if (data.items.length < 100) return null;
  }
  return null;
}

interface WebhooksList {
  data: DodoWebhook[];
}

async function findWebhookByUrl(url: string): Promise<DodoWebhook | null> {
  const data = await dodo<WebhooksList>("GET", "/webhooks?limit=100");
  return data.data.find((w) => w.url === url) ?? null;
}

// ---------------------------------------------------------------------------
// Step 1: Products
// ---------------------------------------------------------------------------

async function provisionProducts(): Promise<Record<string, string>> {
  console.log("◆ Products");
  const ids: Record<string, string> = {};

  for (const spec of PRODUCTS) {
    const existing = await findProductByName(spec.name);
    if (existing) {
      console.log(`  ✓ already exists: ${spec.name} → ${existing.product_id}`);
      ids[spec.envVar] = existing.product_id;
      continue;
    }

    const created = await dodo<DodoProduct>("POST", "/products", {
      name: spec.name,
      description: spec.description,
      tax_category: "saas",
      price: {
        type: "recurring_price",
        currency: "USD",
        price: spec.priceCents,
        discount: 0,
        purchasing_power_parity: false,
        payment_frequency_count: 1,
        payment_frequency_interval: "Month",
        subscription_period_count: 1,
        subscription_period_interval: "Month",
        trial_period_days: 0,
      },
    });
    console.log(`  + created: ${spec.name} → ${created.product_id}`);
    ids[spec.envVar] = created.product_id;
  }
  console.log("");
  return ids;
}

// ---------------------------------------------------------------------------
// Step 2: Credit Entitlement (Witness Credits)
// ---------------------------------------------------------------------------

async function provisionEntitlement(): Promise<string> {
  console.log("◆ Credit Entitlement");
  const existing = await findEntitlementByName(ENTITLEMENT_NAME);
  if (existing) {
    console.log(`  ✓ already exists: ${ENTITLEMENT_NAME} → ${existing.id}`);
    console.log("");
    return existing.id;
  }
  const created = await dodo<DodoEntitlement>("POST", "/credit-entitlements", {
    name: ENTITLEMENT_NAME,
    unit: "Witness Credits",
    precision: 0,
    expires_after_days: 30,
    rollover_enabled: true,
    rollover_percentage: 100,
    rollover_timeframe_count: 1,
    rollover_timeframe_interval: "Month",
    max_rollover_count: 1,
    overage_enabled: false,
  });
  console.log(`  + created: ${ENTITLEMENT_NAME} → ${created.id}`);
  console.log("");
  return created.id;
}

// ---------------------------------------------------------------------------
// Step 3: Usage Meter (noesis.engine_query)
// ---------------------------------------------------------------------------

async function provisionMeter(): Promise<string> {
  console.log("◆ Usage Meter");
  const existing = await findMeterByName(METER_NAME);
  if (existing) {
    console.log(`  ✓ already exists: ${METER_NAME} → ${existing.id}`);
    console.log("");
    return existing.id;
  }
  const created = await dodo<DodoMeter>("POST", "/meters", {
    name: METER_NAME,
    event_name: METER_EVENT_NAME,
    aggregation: { type: "sum", key: "amount" },
    measurement_unit: "queries",
  });
  console.log(`  + created: ${METER_NAME} → ${created.id}`);
  console.log("");
  return created.id;
}

// ---------------------------------------------------------------------------
// Step 4: Webhook (optional — only if URL provided)
// ---------------------------------------------------------------------------

async function provisionWebhook(): Promise<string | null> {
  console.log("◆ Webhook");
  if (!webhookUrl) {
    console.log("  ⚠ DODO_PROVISION_WEBHOOK_URL not set — skipping.");
    console.log("    For local dev, run `dodo wh listen` instead (creates an");
    console.log("    auto-webhook pointing at Dodo's relay server).");
    console.log("    For staging/prod, set DODO_PROVISION_WEBHOOK_URL and re-run.");
    console.log("");
    return null;
  }
  const existing = await findWebhookByUrl(webhookUrl);
  if (existing) {
    console.log(`  ✓ already exists: ${webhookUrl} → ${existing.id}`);
    console.log(`    Signing key: open the webhook in dashboard to copy it.`);
    console.log("");
    return existing.id;
  }
  const created = await dodo<DodoWebhook>("POST", "/webhooks", {
    url: webhookUrl,
    description: "Selemene biofield-web inbound webhook (provisioned by script)",
    filter_types: WEBHOOK_EVENTS,
    disabled: false,
  });
  console.log(`  + created: ${webhookUrl} → ${created.id}`);
  console.log("    ⚠ The signing key is NOT in the API response. Open the");
  console.log("       webhook in the dashboard and copy the signing secret");
  console.log("       (whsec_…) into DODO_PAYMENTS_WEBHOOK_KEY in your .env.");
  console.log("");
  return created.id;
}

// ---------------------------------------------------------------------------
// Run
// ---------------------------------------------------------------------------

const productIds = await provisionProducts();
const entitlementId = await provisionEntitlement();
const meterId = await provisionMeter();
const webhookId = await provisionWebhook();

console.log("═══════════════════════════════════════════════════════════════");
console.log("✔ Provisioning complete. Paste the lines below into your .env:");
console.log("═══════════════════════════════════════════════════════════════");
console.log("");
for (const [key, value] of Object.entries(productIds)) {
  console.log(`${key}=${value}`);
}
console.log(`DODO_ENTITLEMENT_WITNESS_CREDITS_ID=${entitlementId}`);
console.log(`DODO_METER_ENGINE_QUERY_ID=${meterId}`);
if (webhookId) {
  console.log(`# webhook ID ${webhookId} — copy the signing secret from the dashboard:`);
  console.log(`DODO_PAYMENTS_WEBHOOK_KEY=whsec_…  # ← paste from dashboard`);
}
console.log("");
console.log("Then run this SQL against your Postgres to wire plan_catalog:");
console.log("");
console.log(`UPDATE plan_catalog SET dodo_product_id='${productIds.DODO_PRODUCT_FREE_ID}'    WHERE code='free';`);
console.log(`UPDATE plan_catalog SET dodo_product_id='${productIds.DODO_PRODUCT_BASIC_ID}'   WHERE code='basic';`);
console.log(`UPDATE plan_catalog SET dodo_product_id='${productIds.DODO_PRODUCT_PREMIUM_ID}' WHERE code='premium';`);
console.log("");
console.log("═══════════════════════════════════════════════════════════════");
console.log("REMAINING DASHBOARD WORK (~3 min, can't be scripted):");
console.log("═══════════════════════════════════════════════════════════════");
console.log("");
console.log("Open each of the 3 products in the Dodo dashboard:");
console.log(`  https://app.dodopayments.com/products/edit?id=${productIds.DODO_PRODUCT_FREE_ID}`);
console.log(`  https://app.dodopayments.com/products/edit?id=${productIds.DODO_PRODUCT_BASIC_ID}`);
console.log(`  https://app.dodopayments.com/products/edit?id=${productIds.DODO_PRODUCT_PREMIUM_ID}`);
console.log("");
console.log("For each product:");
console.log("  1. Entitlements tab → Attach → select 'Witness Credits'");
console.log("  2. UNCHECK 'Import default credit settings'");
console.log("  3. Set per-tier values:");
console.log("       Free:    50 credits / cycle, overage DISABLED");
console.log("       Basic:   500 credits / cycle, overage opt-in, $0.030/credit");
console.log("       Premium: 2 500 credits / cycle, overage on, $0.015/credit");
console.log("  4. Meters tab → Attach → select 'noesis.engine_query'");
console.log("");
console.log("Done.");
