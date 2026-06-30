# Dodo Payments — Frozen Contracts

> Wave 1.1 output. This file is the single source of truth for every shared
> boundary across the Dodo integration. **Do not edit during Phase 2 builds.**
> Changes to this file during Phase 2 require re-planning per
> `playbooks/multi-agent-boundaries.md`.
>
> Owner: Claude (orchestrator). Reviewers: Codex (UI), Copilot (backend), Gemini (validation).
> Last frozen: 2026-05-05.

---

## § Data

### Existing schema (migration 014, kept as-is)

| Table | Role |
|---|---|
| `users` | identity, tier mirror, Discord OAuth join key |
| `plan_catalog` | canonical plan codes (`free`, `basic`, `premium`, `enterprise`) seeded by 014 |
| `billing_subscriptions` | per-user subscription rows, multi-provider via `provider` column |
| `user_active_plan_resolutions` | view joining the two for read-side queries |

### Migration 020 additions (Dodo-only)

| Column / index | Why |
|---|---|
| `users.dodo_customer_id VARCHAR(128) NULL` | direct user → Dodo customer lookup; populated on first checkout or webhook |
| `uq_users_dodo_customer_id` (partial) | enforces 1:1 user ↔ Dodo customer |
| `plan_catalog.dodo_product_id VARCHAR(128) NULL` | maps `free|basic|premium` plan codes to the product IDs Dodo issues in T06 |
| `uq_plan_catalog_dodo_product_id` (partial) | one Dodo product can only back one plan code |
| `billing_subscriptions.metadata JSONB DEFAULT '{}'::jsonb` | stores raw Dodo payloads, balance snapshots, last webhook id (for idempotency observability) |
| `idx_billing_subscriptions_provider_customer` | webhook-time lookup by `(provider='dodo_payments', provider_customer_id)` |

### Subscription lifecycle (mirrors Dodo's states 1:1)

```
            ┌──────────┐
            │incomplete│  (created, awaiting first payment)
            └────┬─────┘
                 │ payment.succeeded
                 ▼
            ┌──────────┐ ◄── subscription.renewed (no state change)
            │  active  │
            └────┬─────┘
                 │ payment.failed              user cancels
                 │                             from portal
                 ▼                             ▼
            ┌──────────┐                  ┌──────────┐
            │ past_due │ ─── recovers ──► │ canceled │ (period_end honoured)
            └────┬─────┘                  └────┬─────┘
                 │ recovery window expires      │ period_end passes
                 ▼                              ▼
            ┌──────────┐                   ┌──────────┐
            │ expired  │                   │ expired  │
            └──────────┘                   └──────────┘
```

The `status` CHECK constraint from 014 already permits all six states.

### `users.tier` ↔ `billing_subscriptions.plan_id` invariant

`users.tier` is a **mirror**, not the source of truth. The webhook handler
updates `users.tier` whenever the active subscription's plan changes. The view
`user_active_plan_resolutions` is the authoritative read path; `users.tier`
exists only as a denormalised fast-path for hot request middleware.

If they ever diverge, the hourly reconciliation cron (T24) flags it and the
webhook-derived value wins.

### Reconciliation rules (T24 cron)

Once an hour:
1. `GET /subscriptions?status=active` from Dodo → list of `provider_subscription_id`
2. `SELECT … FROM billing_subscriptions WHERE provider='dodo_payments' AND status IN ('trialing','active','past_due')` → local list
3. Diff. Three possible drift classes:
   - **Local-only active** (Dodo says canceled): force `status='canceled'`, set `canceled_at=NOW()`, mirror to `users.tier='free'` (or floor plan)
   - **Dodo-only active** (we missed the webhook): synthesise the missed event, rerun the handler
   - **Plan mismatch** (Dodo says basic, we say premium): trust Dodo, update `plan_id` and `users.tier`
4. Drift count emitted as `metrics.dodo.reconcile_drift{class=…}`. Non-zero → Sentry alert.

### Idempotency

Inbound webhooks dedupe on Dodo's `webhook-id` header via Redis `SETNX
billing:webhook:{webhook-id}=1 EX 86400`. Hits are returned `200 {dedup: true}`
without state mutation. The Redis key TTL is 24 h; the reconciliation cron
covers anything older than that.

---

## § API

### Inbound webhooks (Dodo → admin-web → noesis-api)

The Next.js adaptor verifies Standard Webhooks signatures (`webhook-id`,
`webhook-signature`, `webhook-timestamp` headers) and forwards the verified
JSON to the Rust internal endpoint. We subscribe to **8 event types**:

| Event | Trigger | What we do |
|---|---|---|
| `subscription.active` | first successful payment | upsert `billing_subscriptions`, set `users.tier`, `users.dodo_customer_id` |
| `subscription.updated` | any subscription field change | re-sync from payload (period_end, plan, cancel_at_period_end) |
| `subscription.on_hold` | renewal failed, in dunning | `status='past_due'`, surface banner via T26 |
| `subscription.cancelled` | user cancelled (still in period) | `cancel_at_period_end=true`, keep `status='active'` until period_end |
| `subscription.failed` | mandate creation failed | `status='incomplete'`, log + alert |
| `payment.succeeded` | renewal payment captured | metric only; subscription state unchanged |
| `payment.failed` | renewal payment declined | metric only; `subscription.on_hold` does the state change |
| `credit.added` / `credit.deducted` / `credit.balance_low` / `credit.overage_charged` | credit-pool lifecycle | invalidate balance cache (T17), emit metric, send T20 modal trigger if applicable |

> Counted as 8 categories above; the credit-pool variants share one route.

### Forward contract (Next.js → Rust)

`POST /internal/billing/events`

Headers:
```
Content-Type: application/json
X-Forward-Secret: <DODO_INTERNAL_FORWARD_SECRET>
```

Body:
```json
{
  "webhook_id": "msg_29...",                  // Dodo webhook-id header (idempotency key)
  "webhook_timestamp": "1714867200",          // unix seconds
  "event_type": "subscription.active",
  "payload": { /* raw verified Dodo body */ }
}
```

Responses:
- `200 {"status":"ok"}` — handled
- `200 {"status":"dedup"}` — already processed (idempotent replay)
- `401` — bad/missing forward secret
- `400` — malformed body
- `422` — known event type but payload missing required field
- `500` — unrecoverable handler error (Sentry breadcrumb attached, Dodo will retry)

### Outbound: usage event ingestion (noesis-api → Dodo)

`POST https://{api_base}/usage-events/ingest` with bearer `DODO_PAYMENTS_API_KEY`.

Single-event payload shape (we batch 1 per call in v1):
```json
{
  "events": [{
    "event_id": "noesis_engine_<user_uuid>_<engine_id>_<unix_nanos>",
    "customer_id": "<users.dodo_customer_id>",
    "event_name": "noesis.engine_query",
    "timestamp": "2026-05-05T10:00:00.000Z",
    "metadata": {
      "engine_id": "panchanga",
      "tier": "premium",
      "internal_user_id": "<users.id>"
    }
  }]
}
```

Retry: 2 attempts, exponential backoff (200 ms, 1 s). Final failure → Sentry
breadcrumb + `metrics.dodo.usage_emit_failed` counter. **Never blocks** the
engine response.

### Customer balance proxy

`GET /api/v1/billing/balance` (authenticated, admin-web → noesis-api)

Rust handler:
1. Look up `users.dodo_customer_id` for the JWT subject.
2. If absent (Free tier never checked out) → return `{"credits_remaining": <free_quota>, "tier": "free", "source": "tier_default"}`.
3. Else `GET /credit-entitlements/get-customer-balance?customer_id=…&entitlement_id=…` from Dodo.
4. Cache 60 s in Redis (`billing:balance:{customer_id}`); invalidate on `credit.added`/`credit.deducted` webhook.

Response:
```json
{
  "credits_remaining": 2347,
  "credits_total": 2500,
  "period_end": "2026-06-01T00:00:00Z",
  "overage_enabled": true,
  "tier": "premium",
  "source": "dodo"
}
```

### Checkout creation

`POST /api/billing/checkout` (Next.js route, App Router)

Request:
```json
{ "plan_code": "premium" }
```

The route:
1. Reads JWT cookie → `selemene_user_id` UUID + email
2. Looks up `dodo_product_id` for `plan_code` (fail 400 if absent — means dashboard not yet provisioned)
3. Calls `client.checkoutSessions.create({ product_cart, customer, return_url, metadata: { selemene_user_id } })`
4. Returns `{ "checkout_url": "..." }` for the SPA to redirect to

The `metadata.selemene_user_id` is what closes the loop on the inbound
`subscription.active` webhook so we know which `users` row to attach
`dodo_customer_id` to.

### Customer portal

`POST /api/billing/portal` (Next.js route)

1. Look up `users.dodo_customer_id`. 404 if absent.
2. Call `client.customers.createCustomerPortalSession({ customer_id })`
3. Return `{ "portal_url": "..." }`.

---

## Frozen at: 2026-05-05

Any deviation from this document during Phase 2 is a contract drift and must
trigger a re-planning loop, not a silent edit.

---

## §Admin (added 2026-05-06)

Operator-facing dashboard endpoints for the Dodo Payments integration. All
routes are mounted under `/api/v1/admin/billing/*` and require the standard
`auth_middleware`. Permission gating is per-endpoint (not group-wide).

### Permission matrix

| Permission | Granted to role | Use |
|---|---|---|
| `admin:billing:read` | `billing-admin`, implied by any other `admin:billing:*` perm, or by `admin:*` | All read-only endpoints below |
| `admin:billing:subscriptions:cancel` | `billing-admin`, `admin:*` | `POST /admin/billing/subscriptions/:id/cancel` |
| `admin:billing:reconcile:trigger` | `billing-admin`, `admin:*` | `POST /admin/billing/reconcile/run` |

The existing `admin` and `platform-admin` roles do NOT receive billing
permissions by default; grant the new `billing-admin` role explicitly.
`platform-admin` retains everything via the `admin:*` wildcard.

### Endpoints

| Method | Path | Permission | Returns |
|---|---|---|---|
| GET | `/api/v1/admin/billing/overview` | `admin:billing:read` | `{ status_counts[], free_users, mrr_usd_estimate }` |
| GET | `/api/v1/admin/billing/subscriptions?status=…&limit=…&offset=…` | `admin:billing:read` | `{ items[], total, limit, offset }` |
| GET | `/api/v1/admin/billing/subscriptions/:id` | `admin:billing:read` | `{ subscription }` |
| POST | `/api/v1/admin/billing/subscriptions/:id/cancel` | `admin:billing:subscriptions:cancel` | `{ ok, subscription_id }` |
| GET | `/api/v1/admin/billing/webhook-events?provider=…&limit=…` | `admin:billing:read` | `{ items[], limit }` |
| GET | `/api/v1/admin/billing/reconcile/drift` | `admin:billing:read` | `{ latest? }` |
| POST | `/api/v1/admin/billing/reconcile/run` | `admin:billing:reconcile:trigger` | `{ command, note }` (returns shell one-liner; no in-process spawn) |
| GET | `/api/v1/admin/billing/plans` | `admin:billing:read` | `{ items[] }` |

### Storage additions

Migration `023_reconcile_runs.sql` adds:

```sql
CREATE TABLE reconcile_runs (
  id              BIGSERIAL PRIMARY KEY,
  started_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  finished_at     TIMESTAMPTZ,
  force_cancel    BOOLEAN     NOT NULL DEFAULT FALSE,
  drift_json      JSONB       NOT NULL DEFAULT '{}'::jsonb,
  error           TEXT
);
```

`bin/dodo_reconcile` now writes one row per execution; the admin
`/reconcile/drift` endpoint reads `ORDER BY started_at DESC LIMIT 1`.

### Out of scope (v1)

- No write surface for `plan_catalog` (change via migration only)
- No refund triggering (refund via Dodo dashboard)
- Reconcile trigger is informational only (returns the kubectl/cli command;
  the bin runs out-of-process under cron infrastructure)

### Frontend surface

Route group: `apps/admin-web/app/(protected)/billing/*` (port 3001 in dev).
Admin-web is the dedicated operator app. The end-user engine output area
lives in the `sankalpa` repo after `apps/biofield-web` was retired.

The existing `(protected)/layout.tsx` already gates routes via
`requiredPermissionForPath()` + `hasPermission()`. The billing routes are
wired into:

- `NAV_ITEMS` — adds `{ href: "/billing", permission: "admin:billing:read" }`
- `ROUTE_META["/billing"]` — header eyebrow/title/summary
- `requiredPermissionForPath()` in `src/lib/permissions.ts` — maps
  `/billing/*` → `admin:billing:read`
- `hasPermission()` parent-grant — holding any explicit `admin:billing:*`
  perm grants the broader read permission

Direct client-side fetch via the existing `src/lib/api.ts` (8 new
`*AdminBilling*` exports). Types live in `src/types/admin.ts`. No BFF
proxy needed.
