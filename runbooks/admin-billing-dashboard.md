# Admin Billing Dashboard — Operator Guide

> Surfaces the full Dodo Payments operator view: subscriptions, webhook
> ingestion, reconcile drift, plan catalog. Read-only by default; one
> destructive action (force-cancel a subscription locally) gated behind an
> explicit permission and a typed-confirmation modal.

---

## Who can use it

Two permission tiers:

| Role | Can do |
|---|---|
| `billing-admin` | All read endpoints + force-cancel + trigger-reconcile |
| `platform-admin` | Same (via `admin:*` wildcard) |
| `admin` | **Nothing.** Generic admin role does NOT receive billing perms. |

Granting `billing-admin` is a separate explicit step — see "Granting access"
below.

## Routes

Frontend lives in **`apps/admin-web`** (the dedicated operator app on port
3001), under the `(protected)` route group. The end-user `apps/biofield-web`
on port 3000 has no admin surface — it is the engine output area only.

| Path (admin-web) | Purpose |
|---|---|
| `/billing` | Status counts, free user count, MRR estimate |
| `/billing/subscriptions` | Paginated table of every Dodo subscription, status filter |
| `/billing/subscriptions/[id]` | Single sub detail + force-cancel button |
| `/billing/webhook-events` | Last 100 processed webhook events |
| `/billing/reconcile` | Latest drift report + trigger command |
| `/billing/plans` | Plan catalog (read-only) |

Local dev: `cd apps/admin-web && bun run dev` → `http://localhost:3001/billing`

Backend (Rust):

| Method | Path | Permission |
|---|---|---|
| GET | `/api/v1/admin/billing/overview` | `admin:billing:read` |
| GET | `/api/v1/admin/billing/subscriptions` | `admin:billing:read` |
| GET | `/api/v1/admin/billing/subscriptions/:id` | `admin:billing:read` |
| POST | `/api/v1/admin/billing/subscriptions/:id/cancel` | `admin:billing:subscriptions:cancel` |
| GET | `/api/v1/admin/billing/webhook-events` | `admin:billing:read` |
| GET | `/api/v1/admin/billing/reconcile/drift` | `admin:billing:read` |
| POST | `/api/v1/admin/billing/reconcile/run` | `admin:billing:reconcile:trigger` |
| GET | `/api/v1/admin/billing/plans` | `admin:billing:read` |

## Granting access

The auth system stores roles per user. To grant a user the billing-admin
role on production:

```sql
-- Replace with the target user_id
UPDATE users
SET roles = COALESCE(roles, '{}'::text[]) || ARRAY['billing-admin']
WHERE id = '00000000-0000-0000-0000-000000000000';
```

Then have the user sign out + back in. The `/api/v1/admin/session` endpoint
will resolve their permissions on the next admin layout load.

To revoke:

```sql
UPDATE users
SET roles = array_remove(roles, 'billing-admin')
WHERE id = '00000000-0000-0000-0000-000000000000';
```

## Common tasks

### "User says they cancelled but they're still active"

1. `/billing/subscriptions?status=active` → search by user_id (first 8 chars)
2. Click into the row
3. Verify `provider_subscription_id` is set and `status=active`
4. Open Dodo dashboard → confirm sub is cancelled there
5. If yes: click **Force-cancel locally**, type the 8-char prefix to confirm
6. Verify status flips to `canceled` and tier drops to `free` for the user

This does **not** call the Dodo API — it only fixes local state. Dodo-side
cancel must already have happened.

### "Reconcile cron is reporting drift"

1. `/billing/reconcile` → see latest run's `drift_json`
2. If `local_only_active > 0`: legitimate cancellation Dodo did but we
   missed. Use the Subscriptions page to find the row and force-cancel.
3. If `dodo_only_active > 0`: someone exists in Dodo that we don't track.
   Usually a webhook signature failure during signup — check Sentry.

### "I want to run reconcile right now"

1. `/billing/reconcile` → click **Trigger reconcile**
2. Copy the returned shell command (kubectl/Railway/systemd equivalent)
3. Paste into your terminal on the cluster
4. Reload the page after ~30 s — new run will appear

This is intentionally not a one-click trigger. The reconcile bin runs
out-of-process and we don't want a UI button quietly spawning a 30-second
DB-heavy job inside an HTTP handler.

### "What webhooks have we processed today?"

`/billing/webhook-events` shows the last 100 across all providers. Filter
manually by provider in the URL: `?provider=dodo_payments`. Older events
are pruned by maintenance after 90 days.

## Failure modes

| Symptom | Likely cause | Fix |
|---|---|---|
| "Resolving operator permissions…" forever | Backend is unreachable from biofield-web | Check `NEXT_PUBLIC_API_BASE_URL` |
| "Forbidden" page even though user is platform-admin | Stale localStorage session | Sign out + back in to refresh permissions |
| "No reconcile run recorded yet" | Migration 023 not applied OR cron hasn't fired | `psql -c "SELECT 1 FROM reconcile_runs LIMIT 1"`; if missing, apply migration |
| Force-cancel button disabled | Sub has no `provider_subscription_id` | This is a malformed row; investigate via SQL before fixing |

## What's deliberately out of scope (v1)

- **Plan catalog editing** — change via migration + restart only
- **Refund triggering** — use the Dodo dashboard
- **In-process reconcile spawn** — kept out-of-process
- **Per-user metering view** — use the existing user analytics page
- **Webhook event detail / replay** — use the Dodo dashboard's webhook log
