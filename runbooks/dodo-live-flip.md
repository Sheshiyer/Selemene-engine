# Dodo Payments — T29/T30 Staging Dry-Run + Production Live-Mode Flip

> Two-stage release procedure. **Stage 1 (T29)** validates everything
> works against a real $1 charge on staging. **Stage 2 (T30)** flips
> production to live mode and watches for 24 hours.
> **Do not skip stage 1.** Do not run stage 2 same-day as stage 1 — wait
> at least overnight to let edge cases surface.

---

## T29 — Staging dry-run (≈ 90 minutes)

### Prerequisites

- [ ] PR #683 merged to `main`
- [ ] Staging is on the merged commit
- [ ] Tabletop exercise (`runbooks/dodo-tabletop.md`) signed off
- [ ] Smoke test (`runbooks/scripts/dodo-smoke-test.sh`) passed locally
- [ ] You have a real Dodo **live-mode** API key + webhook signing key
      ready in 1Password / Railway secrets / k8s secrets — but **not
      yet deployed**

### Step 1 — Provision live-mode dashboard objects (15 min)

The provisioning script is mode-agnostic. Run it against live mode to
create matching products + entitlement + meter + webhook:

```bash
# Switch CLI to live mode
rm -f ~/.dodopayments/api-key
dodo login              # paste live-mode key, choose "Live Mode"

# Provision
DODO_PROVISION_WEBHOOK_URL=https://<staging-host>/api/webhook/dodo-payments \
  bun run runbooks/scripts/dodo-provision.ts
```

The script will print the live-mode IDs (`pdt_…`, `cde_…`, `mtr_…`,
webhook ID). **These are different from your test-mode IDs.**

Update `runbooks/dodo-dashboard-setup.md` §10 with the live-mode IDs
(commit message: `docs(runbooks): record live-mode provisioning IDs`).

### Step 2 — Update staging Postgres `plan_catalog` (5 min)

```sql
UPDATE plan_catalog SET dodo_product_id='<LIVE_FREE_ID>'    WHERE code='free';
UPDATE plan_catalog SET dodo_product_id='<LIVE_BASIC_ID>'   WHERE code='basic';
UPDATE plan_catalog SET dodo_product_id='<LIVE_PREMIUM_ID>' WHERE code='premium';
```

### Step 3 — Configure staging env (5 min)

In your staging env (Railway / k8s secret), set:

```
DODO_PAYMENTS_API_KEY=sk_live_…
DODO_PAYMENTS_WEBHOOK_KEY=whsec_…   # from the live-mode webhook page
DODO_PAYMENTS_ENV=live
DODO_ENTITLEMENT_WITNESS_CREDITS_ID=<live-ent-id>
DODO_METER_ENGINE_QUERY_ID=<live-meter-id>
DODO_PRODUCT_FREE_ID=<live-free-id>
DODO_PRODUCT_BASIC_ID=<live-basic-id>
DODO_PRODUCT_PREMIUM_ID=<live-premium-id>
DODO_INTERNAL_FORWARD_SECRET=<unchanged or new value>
```

Restart admin-web + noesis-api on staging.

### Step 4 — Verify boot in live mode (5 min)

```bash
# Should log "Dodo billing emitter installed" with env_mode=live
kubectl logs -l app=selemene -c noesis-api --tail=50 | grep "billing emitter"

# Healthy
curl -fsS https://<staging-host>/health/live
```

### Step 5 — End-to-end live-mode purchase (30 min)

1. Open staging in an incognito window
2. Sign up as a new test user (use a **real** card you control, e.g.
   your personal Visa). The card will be charged $1 — refundable.
3. Go to `/pricing`, pick **Basic** ($1 if you set up a test product
   with that price; otherwise the real $9 — your call). Click upgrade.
4. Complete checkout on Dodo's hosted page.
5. Watch the staging logs:
   ```
   [dodo-webhook] processing webhook
   subscription activated user_id=… plan=basic
   ```
6. Refresh the billing page — should show "Witness Basic" with the
   correct period_end and Witness Credits.
7. **Refund yourself** in the Dodo dashboard:
   - Open the Payment row → Refund → full amount
   - Watch staging logs for `payment.refunded` event (log-only in v1,
     no state change)

### Step 6 — Smoke the rest of the surface (15 min)

- [ ] `GET /api/v1/billing/balance` returns `source: "dodo"` with real
      credit numbers
- [ ] `POST /api/v1/billing/portal` returns a working portal URL
- [ ] `dodo_reconcile` (run manually): drift counts are zero
- [ ] Grafana `Dodo Payments Billing` dashboard shows the
      `subscription.active` event in the ingest panel

### Step 7 — Cancel in the portal (15 min)

1. Click "Manage subscription" on the billing page
2. In the Dodo portal, cancel the subscription (immediate, not
   end-of-period)
3. Watch staging logs for `subscription.cancelled` event
4. Verify users.tier flipped to `free`
5. Verify billing_subscriptions.status = `canceled` and `canceled_at`
   is recent

### T29 sign-off

| Check | Pass? |
|---|---|
| Live-mode dashboard objects created | ☐ |
| Staging boots in live mode without errors | ☐ |
| Real card charge → tier flipped within 30s | ☐ |
| Refund processed without state corruption | ☐ |
| Balance endpoint returns Dodo source | ☐ |
| Portal session opens and works | ☐ |
| Reconcile cron run shows zero drift | ☐ |
| Cancel-from-portal → tier dropped to free | ☐ |

**All 8 boxes must be checked before T30.** If any fail, file an issue
and **do not** proceed to production flip until resolved.

Hold for at least 12 hours after T29 completes. Watch for delayed
webhooks / overnight cron runs / Sentry quiet.

---

## T30 — Production live-mode flip (≈ 60 minutes work + 24-hour watch)

### Pre-flight

- [ ] T29 sign-off complete with all 8 boxes ✓
- [ ] At least 12 hours since T29 completed
- [ ] Sentry on staging quiet for those 12 hours
- [ ] On-call rotation knows the flip is happening
- [ ] Statuspage / customer-facing comms ready (low risk, but standby)

### Step 1 — Provision production webhook URL (10 min)

If production uses a different domain than staging, register a
new webhook in the Dodo dashboard (live mode) pointing at production:

```bash
DODO_PROVISION_WEBHOOK_URL=https://<prod-host>/api/webhook/dodo-payments \
  bun run runbooks/scripts/dodo-provision.ts
```

Capture the **production-specific** webhook signing key (`whsec_…`).
The product/entitlement/meter IDs are the same as T29 (live-mode is a
single account namespace).

### Step 2 — Update production secrets (10 min)

Set in production secret store (NOT in `.env` checked in):

```
DODO_PAYMENTS_API_KEY=sk_live_…              # same as T29
DODO_PAYMENTS_WEBHOOK_KEY=whsec_…             # from step 1, prod-specific
DODO_PAYMENTS_ENV=live
DODO_ENTITLEMENT_WITNESS_CREDITS_ID=<live>
DODO_METER_ENGINE_QUERY_ID=<live>
DODO_PRODUCT_FREE_ID=<live>
DODO_PRODUCT_BASIC_ID=<live>
DODO_PRODUCT_PREMIUM_ID=<live>
DODO_INTERNAL_FORWARD_SECRET=<openssl rand -base64 48>
```

### Step 3 — Update production `plan_catalog` (2 min)

Same UPDATE statements as T29, but against the production DB. Use a
read-only check first to confirm you're connected to the right one:

```sql
SELECT current_database(), inet_server_addr();   -- should NOT be staging
```

### Step 4 — Restart production (5 min)

Roll-restart noesis-api + admin-web. Watch logs for the boot
message: `"Dodo billing emitter installed" env_mode=live`.

### Step 5 — First real customer charge (passive, watch only)

Wait for an organic customer upgrade. Don't stage one. Watch:

- [ ] Sentry quiet for 24 hours
- [ ] First `subscription.active` reaches the `noesis_dodo_webhook_received_total{outcome="ok"}` panel
- [ ] First `noesis_dodo_usage_emit_total{outcome="success"}` lands

### Step 6 — Enable reconcile auto-correction (after 1 week)

After 7 days of clean drift reports, edit
`k8s/base/dodo-reconcile-cronjob.yaml`:

```yaml
- name: DODO_RECONCILE_FORCE_CANCEL
  value: "true"   # was "false"
```

`kubectl apply -k k8s/base` and wait for the next run to demonstrate
auto-correction works (you can intentionally drift one row in
production by cancelling a test sub in Dodo without webhook
delivery — though most cancels will deliver fine).

### T30 sign-off

| Check | Pass? |
|---|---|
| Production secrets updated, restart logged | ☐ |
| First real `subscription.active` processed | ☐ |
| 24 h Sentry quiet | ☐ |
| First `usage.engine_query` emit succeeded | ☐ |
| Reconcile cron green for 7 days | ☐ |
| Auto-cancel mode enabled and verified | ☐ |

When all 6 boxes are checked, billing is **live and trusted**.
Commit a `chore(billing): close T30 — live mode stable` marker so the
ledger is clear.

---

## Rollback

If anything goes catastrophically wrong during T29 or T30:

1. **Set `DODO_PAYMENTS_ENV=test` in env** and restart. The boot-time
   emitter installer will switch the API base URL back to test mode.
   Live mode webhooks will start failing signature verification (test-
   mode signing key) — this is the desired fail-safe; events queue on
   Dodo's side and can be replayed after fix.
2. **Stop the reconcile cron**: `kubectl scale --replicas=0` on the
   CronJob to prevent any auto-cancellation while you investigate.
3. **Communicate**: post in #incident-billing channel with timestamp,
   what broke, and "rolling back to test mode."
4. **Investigate**: use `runbooks/dodo-incident.md` scenario 4
   (extended outage) once you're back on test mode and the bleeding
   has stopped.

The fail-safe is designed so that **you can never accidentally double-
charge a customer by toggling env back and forth**. Live and test
modes have separate Dodo customer ID namespaces; switching to test
mode just means new charges aren't possible until you switch back.
