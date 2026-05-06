# Dodo Payments — Incident Runbook (T28)

> Four scenarios. Each one starts with **how to recognise it** and ends
> with a tabletop check the on-call should run within 15 minutes of
> arriving. Copy-paste commands assume `bash`, `psql`, and `curl` on the
> server, and that `.env` is loaded (e.g. `set -a && source .env && set +a`).

---

## Scenario 1: Webhook outage (signatures rejected)

### Symptoms
- Grafana **Webhook 401 / 400 (auth + freshness rejections)** panel
  spikes above baseline.
- Sentry: bursts of `[dodo-webhook] signature verification failed` from
  biofield-web.
- New Dodo events stop landing in `processed_webhook_events`:
  ```sql
  SELECT MAX(processed_at) FROM processed_webhook_events
   WHERE provider = 'dodo_payments';
  ```
  → result is older than 15 minutes (typical webhook cadence).

### Likely causes (ordered by probability)
1. **Dodo rolled the signing key** without us updating
   `DODO_PAYMENTS_WEBHOOK_KEY` in env.
2. **Forward secret drift** — biofield-web and noesis-api have
   different `DODO_INTERNAL_FORWARD_SECRET` values after a partial
   deploy.
3. **Dodo's webhook endpoint URL** in their dashboard points at a
   stale tunnel / reverted deployment.

### Fix
```bash
# 1. Confirm Dodo can reach us at all (uses the public key, no secret needed).
curl -i https://<your-domain>/api/webhook/dodo-payments  # → 405 Method not allowed = reachable

# 2. Refetch signing key from Dodo and re-set env.
API_KEY=$(grep DODO_PAYMENTS_API_KEY .env | cut -d= -f2)
WEBHOOK_ID=$(curl -s -H "Authorization: Bearer $API_KEY" \
  https://test.dodopayments.com/webhooks | jq -r '.data[0].id')
NEW_SECRET=$(curl -s -H "Authorization: Bearer $API_KEY" \
  https://test.dodopayments.com/webhooks/$WEBHOOK_ID/secret | jq -r '.secret')
echo "DODO_PAYMENTS_WEBHOOK_KEY=$NEW_SECRET"  # paste into .env, then redeploy biofield-web

# 3. If forward-secret drift suspected, regenerate and set on BOTH services:
openssl rand -base64 48
# → set as DODO_INTERNAL_FORWARD_SECRET in noesis-api AND biofield-web env, restart both.

# 4. Trigger a test event from the Dodo dashboard.
#    Verify in Rust logs: "processing webhook" should appear within 5s.
```

### Tabletop check (15 min)
A test webhook from the dashboard reaches the Rust handler with
`outcome=ok` (visible in metrics within 1 minute).

---

## Scenario 2: Postgres drift (local says active, Dodo says canceled)

### Symptoms
- `noesis_dodo_reconcile_drift_total{class="local_only"}` non-zero on
  Grafana.
- User reports: "I cancelled in the Dodo portal but I'm still on
  Premium."
- `billing_subscriptions.status='active'` for a subscription Dodo lists
  as canceled.

### Likely causes
1. **Missed `subscription.cancelled` webhook** during an outage window
   (Scenario 1).
2. **Webhook delivered to a different environment** (staging vs prod)
   due to misconfigured Dodo dashboard URL.

### Fix
```bash
# 1. Identify drift via reconcile bin (read-only mode).
DODO_RECONCILE_FORCE_CANCEL=false cargo run --bin dodo_reconcile

# Look for "local_only_active" > 0 in the JSON output. Note the IDs
# in samples.local_only.

# 2. Re-deliver missed events from the Dodo dashboard:
#    Dashboard → Developers → Webhooks → <our webhook> → "Logs" tab.
#    Filter by event_type=subscription.cancelled, redeliver any with
#    error status. Idempotency makes this safe to retry.

# 3. If Dodo doesn't have a redelivery option for a given drift, force-cancel locally.
DODO_RECONCILE_FORCE_CANCEL=true cargo run --bin dodo_reconcile

# 4. Manual single-row force-cancel (when surgical):
psql $DATABASE_URL -c "
  UPDATE billing_subscriptions
  SET status='canceled', canceled_at=NOW(), updated_at=NOW()
  WHERE provider_subscription_id='sub_REPLACEME';
"
psql $DATABASE_URL -c "
  UPDATE users SET tier='free', updated_at=NOW()
  WHERE id=(SELECT user_id FROM billing_subscriptions
            WHERE provider_subscription_id='sub_REPLACEME');
"
```

### Tabletop check
Re-run `dodo_reconcile` — drift counts return to zero.

---

## Scenario 3: Signing key rotation (planned or breach)

Use this when **`DODO_PAYMENTS_WEBHOOK_KEY` is suspected leaked** (committed by accident, exfiltrated, etc.) — same procedure for planned periodic rotation.

```bash
# 1. Open the webhook in the Dodo dashboard:
#    Developers → Webhooks → click the webhook → "Roll signing key".
#    Copy the new whsec_… value.

# 2. Update env (everywhere — local, staging, prod):
sed -i.bak 's|^DODO_PAYMENTS_WEBHOOK_KEY=.*|DODO_PAYMENTS_WEBHOOK_KEY=whsec_NEW|' .env
# Repeat on each host (or update the secret manager: Railway / Vercel / k8s).

# 3. Redeploy biofield-web (the secret is read at request time by the
#    Standard Webhooks library — a process restart is sufficient).
#    No noesis-api redeploy needed; signature verification is Next.js-side.

# 4. Verify:
#    Dodo dashboard → Webhooks → "Send test event"
#    Watch biofield-web logs for "processing webhook" — should reach
#    Rust within 5 s. If "signature verification failed" still
#    appears, double-check the env var landed (no surrounding quotes,
#    no whitespace).
```

### Tabletop check
A real `subscription.updated` event from a customer reaches our DB with
`outcome=ok` after rotation.

---

## Scenario 4: Manual reconcile after extended outage

When the system has been off for **more than 90 minutes** (longer than
the typical Dodo retry window), some events may have aged out of the
retry queue. Drift will accumulate and must be fixed manually.

```bash
# 1. Quantify the gap.
psql $DATABASE_URL -c "
  SELECT MAX(processed_at) AS last_event,
         NOW() - MAX(processed_at) AS gap
  FROM processed_webhook_events
  WHERE provider = 'dodo_payments';
"

# 2. Pull every active subscription from Dodo and diff against local.
cargo run --bin dodo_reconcile  # read-only first, no DB mutations

# 3. For each drift, redeliver from the Dodo dashboard:
#    Webhooks → click webhook → Logs → filter by status=failed, event time within the gap.
#    Click "Resend" on each.

# 4. After redelivery completes (give it 10 minutes), re-run reconcile:
cargo run --bin dodo_reconcile
# All drift counts should be 0.

# 5. If Dodo can't redeliver some events (too old), force-cancel
#    locally per Scenario 2. Affected users may see incorrect tier
#    during the gap window — investigate with the user_active_plan_resolutions view:
psql $DATABASE_URL -c "
  SELECT u.email, u.tier, r.plan_code, r.current_period_end
  FROM users u
  LEFT JOIN user_active_plan_resolutions r ON r.user_id = u.id
  WHERE u.tier <> r.plan_code
     OR (u.tier='free' AND r.plan_code IS NULL)
  LIMIT 50;
"
```

### Tabletop check
Reconcile reports zero drift, and the diagnostic SQL above returns no
mismatched rows.

---

## Quick reference card (print this)

| What broke? | First command |
|---|---|
| Webhooks rejected en masse | `cargo run --bin dodo_reconcile` (verify gap), then refetch `whsec_…` from Dodo |
| User says "I cancelled but still paying" | `psql -c "SELECT * FROM billing_subscriptions WHERE provider_subscription_id='sub_…'"` |
| Suspected key leak | Roll signing key in Dodo dashboard → update env → redeploy biofield-web |
| Long outage recovery | `cargo run --bin dodo_reconcile`, then redeliver from Dodo dashboard logs |

## Where to look

| Signal | Location |
|---|---|
| Inbound webhook flow | `noesis_dodo_webhook_received_total` (Grafana → Dodo Payments Billing dashboard) |
| Outbound emit success | `noesis_dodo_usage_emit_total{outcome="success"}` |
| DB state | `billing_subscriptions`, `processed_webhook_events`, `user_active_plan_resolutions` view |
| Logs (Rust) | tag `events_forward` or `subscription activated` / `subscription cancelled` |
| Logs (Next.js) | tag `[dodo-webhook]` |
| Sentry | category `dodo.usage_emit` for outbound failures |

---

## Setup notes (one-time)

### Import the Grafana dashboard

```bash
# From repo root, with $GRAFANA_URL and $GRAFANA_API_TOKEN set:
curl -X POST "$GRAFANA_URL/api/dashboards/db" \
  -H "Authorization: Bearer $GRAFANA_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d @monitoring/grafana/dashboards/dodo-billing.json
```

Or via the UI: Grafana → Dashboards → New → Import → upload
`monitoring/grafana/dashboards/dodo-billing.json`. Pick the Prometheus
datasource that scrapes the `noesis-api` `/metrics` endpoint.

### Schedule the reconcile cron

**k8s** — apply the manifest:
```bash
kubectl apply -f k8s/base/dodo-reconcile-cronjob.yaml
kubectl get cronjob dodo-reconcile      # verify
kubectl get jobs --selector=job=dodo-reconcile   # see runs
```

**Railway** — add to your project settings → Cron Jobs:
- **Schedule**: `13 * * * *`  (offset from the top-of-hour stampede)
- **Command**: `/app/dodo_reconcile`
- **Environment**: same secrets as the main service (`DODO_PAYMENTS_API_KEY`,
  `DATABASE_URL`, `DODO_PAYMENTS_ENV`)
- **Override**: `DODO_RECONCILE_FORCE_CANCEL=false` for the first week.

**Bare host / systemd timer**:
```ini
# /etc/systemd/system/dodo-reconcile.timer
[Unit]
Description=Dodo reconcile hourly
[Timer]
OnCalendar=hourly
Persistent=true
[Install]
WantedBy=timers.target

# /etc/systemd/system/dodo-reconcile.service
[Unit]
Description=Dodo subscription reconciliation
After=network-online.target
[Service]
Type=oneshot
EnvironmentFile=/etc/selemene/dodo.env
ExecStart=/usr/local/bin/dodo_reconcile
User=selemene
```

Check the first few runs with `journalctl -u dodo-reconcile.service`.

### Promote reconcile to auto-correct mode

After **7 days** of clean drift reports, flip the env var so
`local_only_active` rows are auto-cancelled instead of just reported.
See `runbooks/dodo-live-flip.md` step 6 for the full procedure.
