# Dodo Payments — Dashboard Setup Runbook

> **Owner:** noesisX (you).
> **When:** before Wave 2 starts. Wave 1.3 scaffolding can run in parallel; this
> runbook produces the IDs and signing keys that Wave 2 needs.
> **Time estimate:** **5 min via script + 3 min in dashboard.**
> **Mode:** **test mode** by default. Live mode is its own runbook (P3-T29).
>
> Reference: <https://docs.dodopayments.com/features/credit-based-billing> ·
> <https://docs.dodopayments.com/developer-resources/integration-guide> ·
> <https://docs.dodopayments.com/miscellaneous/test-mode-vs-live-mode>
>
> ## Fast path (recommended)
>
> ```bash
> # 1. Put your test-mode secret key in .env
> echo 'DODO_PAYMENTS_API_KEY=sk_test_…' >> .env
> echo 'DODO_PAYMENTS_ENV=test' >> .env
>
> # 2. Run the provisioning script (creates products + entitlement + meter)
> bun run runbooks/scripts/dodo-provision.ts
>
> # 3. Paste the printed `.env` block into .env and run the printed SQL UPDATEs
> # 4. Click through the 3-product entitlement attachment in the dashboard
> #    (per-tier credit settings — script-output prints the URLs)
> ```
>
> The manual sections below remain authoritative for understanding what each
> object means. If you prefer click-by-click over the script, read on.

---

## 0. Login + confirm test mode

1. Sign in at <https://app.dodopayments.com/>.
2. Top-right header → confirm the badge says **TEST MODE** (not LIVE). If it
   says LIVE, click and switch.
3. **Settings → API Keys** → copy the test-mode secret key.
   - Paste into `.env` as `DODO_PAYMENTS_API_KEY=…`
   - Set `DODO_PAYMENTS_ENV=test`

Everything below happens inside test mode. None of it costs money.

---

## 1. Create the three subscription products

For each tier in the table below: **Products → New product → Subscription**.

| Field | Free | Basic | Premium |
|---|---|---|---|
| Name | `Witness Free` | `Witness Basic` | `Witness Premium` |
| Description | "Free access to consciousness engines, 50 credits / month" | "Standard access, 500 credits / month, opt-in overage" | "Pro access, 2 500 credits / month, lowest overage rate, priority queue" |
| Pricing model | Recurring | Recurring | Recurring |
| Price | `0` USD | `9` USD | `29` USD |
| Billing interval | Monthly | Monthly | Monthly |
| Tax behaviour | Inclusive (Dodo MoR handles it) | Inclusive | Inclusive |
| Trial period | None | None | None (set `14` if you want a trial later) |

After **Save** for each product:
- Open the product → top of page shows the product ID like `prod_xxxxxxxx`.
- Paste into `.env`:
  - Free  → `DODO_PRODUCT_FREE_ID=prod_…`
  - Basic → `DODO_PRODUCT_BASIC_ID=prod_…`
  - Premium → `DODO_PRODUCT_PREMIUM_ID=prod_…`

> The numbers are provisional per the plan §1. If you want different prices,
> edit the product before going live; the codebase reads the IDs, not the
> numbers, so price changes don't break anything.

---

## 2. Create the Credit Entitlement "Witness Credits"

**Entitlements → New credit entitlement.**

| Field | Value |
|---|---|
| Name | `Witness Credits` |
| Credit type | **Custom Unit** |
| Unit name | `Credits` (singular noun) |
| Precision | `0` (whole credits only — no fractions) |
| Credit expiry | `30` days |
| **Rollover** | Enable |
| Max rollover percentage | `100%` |
| Rollover timeframe | `1 month` |
| Max rollover count | `1` |
| Default overage behaviour | **Disabled** (per-product override below) |

Save. Copy the entitlement ID (`ent_…`) into `.env`:

```
DODO_ENTITLEMENT_WITNESS_CREDITS_ID=ent_…
```

---

## 3. Create the Usage Meter `noesis.engine_query`

**Usage-based billing → Meters → New meter.**

| Field | Value |
|---|---|
| Name (machine) | `noesis.engine_query` |
| Display name | `Engine Query` |
| Aggregation | **Sum** |
| Linked credit entitlement | `Witness Credits` (from step 2) |
| Meter units per credit | `1` |
| Default unit price | leave at `0` — overage is set per-product |

Save. Copy the meter ID (`meter_…`) into `.env`:

```
DODO_METER_ENGINE_QUERY_ID=meter_…
```

---

## 4. Attach Witness Credits + meter to each product

For each of the three products in step 1: open product → **Entitlements** tab
→ **Attach** → select Witness Credits.

> **Critical:** uncheck **"Import default credit settings"**. This unlocks the
> per-product override fields below. (See ElevenLabs deconstruction page —
> the lower overage rate at higher tiers is what drives upgrades.)

| Setting | Free | Basic | Premium |
|---|---|---|---|
| Credits issued per cycle | `50` | `500` | `2500` |
| Trial credits | `0` | `0` | `0` |
| **Overage behaviour** | **Disabled** (hard cap) | **Bill at billing — opt-in** | **Bill at billing — enabled by default** |
| Overage price per credit (USD) | n/a | `0.030` | `0.015` |
| Low balance threshold | `5` (10 % of 50) | `50` (10 %) | `250` (10 %) |

Then **Meters** tab on the same product → attach `noesis.engine_query`.

Repeat for all 3 products. **Save** each time.

---

## 5. Register the webhook endpoint

**Developers → Webhooks → New webhook.**

| Field | Value |
|---|---|
| URL (test mode) | `https://<your-staging-host>/api/webhook/dodo-payments` |
| Description | `Selemene admin-web inbound webhook` |
| Status | Active |

> For local development: use a public tunnel (`ngrok http 3001` or `cloudflared`)
> and paste that URL. Re-edit when the tunnel rotates.

Subscribe to exactly these **8 event categories** (the credit-pool variants
share a route in our handler):

- [ ] `subscription.active`
- [ ] `subscription.updated`
- [ ] `subscription.on_hold`
- [ ] `subscription.cancelled`
- [ ] `subscription.failed`
- [ ] `payment.succeeded`
- [ ] `payment.failed`
- [ ] `credit.added` · `credit.deducted` · `credit.balance_low` · `credit.overage_charged` (4 events, one route)

Save. Open the webhook → copy the **Signing Secret** (`whsec_…`):

```
DODO_PAYMENTS_WEBHOOK_KEY=whsec_…
```

Then generate a forward secret for the Next.js → Rust hop:

```bash
openssl rand -base64 48
```

Paste into `.env`:

```
DODO_INTERNAL_FORWARD_SECRET=<output>
```

---

## 6. Fire a test event and capture the fixture

In the same webhook detail page → **Send test event** → choose
`subscription.active`.

While Wave 1.3 is live, the Next.js stub at
`app/api/webhook/dodo-payments/route.ts` will return 200 and log the request.
You should see "received forwarded billing event (Wave 1.3 stub)" in the Rust
logs only **after** T11 ships and connects the adaptor — for now the test
event is just confirming the URL is reachable.

If the test event reports **delivery failed** in the dashboard, your URL is
unreachable. Re-check the tunnel / staging deploy.

> The signed fixture for replay-attack tests in T23 must come from a real
> Dodo test event. Once T11 lands, capture the raw POST body + the three
> `webhook-*` headers and save under `tests/dodo/fixtures/test_event.json`.

---

## 7. Sanity-check via API

With `DODO_PAYMENTS_API_KEY` set in your shell:

```bash
# List products — should return 3 with the IDs from step 1
curl -s -H "Authorization: Bearer $DODO_PAYMENTS_API_KEY" \
  https://test.dodopayments.com/v1/products | jq '.data[] | {id, name}'

# Confirm the entitlement exists
curl -s -H "Authorization: Bearer $DODO_PAYMENTS_API_KEY" \
  https://test.dodopayments.com/v1/credit-entitlements | jq '.data[] | {id, name, unit_name}'

# Confirm the meter exists
curl -s -H "Authorization: Bearer $DODO_PAYMENTS_API_KEY" \
  https://test.dodopayments.com/v1/meters | jq '.data[] | {id, name, aggregation}'
```

If any of those return an empty array, the corresponding step did not save.
Re-do that step.

---

## 8. Test-mode payment instruments (for E2E later)

When you reach Wave 2 and run a real checkout:

| Instrument | Number / ID | Notes |
|---|---|---|
| Card (succeed) | `4242 4242 4242 4242` | Any future expiry, any CVC |
| Card (decline) | `4000 0000 0000 0002` | Triggers `payment.failed` |
| Card (3DS challenge) | `4000 0027 6000 3184` | Use to test SCA flow |
| UPI (India, succeed) | `success@upi` | For the Indian-market path |
| UPI (decline) | `failure@upi` | |

See <https://docs.dodopayments.com/miscellaneous/testing-process> for the
full matrix.

---

## 9. Done — what you should now have in `.env`

```env
DODO_PAYMENTS_API_KEY=sk_test_…
DODO_PAYMENTS_WEBHOOK_KEY=whsec_…
DODO_PAYMENTS_ENV=test
DODO_PAYMENTS_RETURN_URL=http://localhost:3001/billing?status=success

DODO_PRODUCT_FREE_ID=prod_…
DODO_PRODUCT_BASIC_ID=prod_…
DODO_PRODUCT_PREMIUM_ID=prod_…
DODO_ENTITLEMENT_WITNESS_CREDITS_ID=ent_…
DODO_METER_ENGINE_QUERY_ID=meter_…

DODO_INTERNAL_FORWARD_SECRET=<openssl-output>
```

When all 11 are non-empty, Wave 2 implementation can proceed against real
Dodo objects.

---

## 10. Update plan_catalog with the Dodo product IDs

Once you have the three product IDs, run this SQL against your Postgres to
finish wiring the dashboard objects to the database:

```sql
UPDATE plan_catalog
SET dodo_product_id = '<DODO_PRODUCT_FREE_ID>'    WHERE code = 'free';
UPDATE plan_catalog
SET dodo_product_id = '<DODO_PRODUCT_BASIC_ID>'   WHERE code = 'basic';
UPDATE plan_catalog
SET dodo_product_id = '<DODO_PRODUCT_PREMIUM_ID>' WHERE code = 'premium';
```

The checkout route (T12) reads this column to map `plan_code` → Dodo product
ID. Until these UPDATE statements run, /api/billing/checkout returns 400.

---

## Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| Test event shows "URL unreachable" | Tunnel down or staging not deployed | Restart `ngrok http 3001` or redeploy admin-web staging |
| `cargo run` warns "DODO_PAYMENTS_API_KEY is set without webhook signing key" | webhook key not in `.env` | Re-do step 5 |
| `cargo run` rejects `DODO_PAYMENTS_ENV` | typo or wrong value | Set exactly `test` or `live` |
| Curl returns 401 | Wrong API key, or you switched to live mode by mistake | Re-fetch test-mode key from Settings → API Keys |
| Curl returns empty product list but dashboard shows them | API key is from a different brand | Make sure the key matches the brand the products are under |
