# Admin Analytics & Billing Reference

> Requires: `billing-admin` role · Version: 3.3.0

All endpoints under `/api/v1/admin/billing/` and `/api/v1/admin/usage/` require the `billing-admin` role. Use `GET /api/v1/admin/session` to verify your effective permissions.

To grant `billing-admin`:
```sql
-- Run in PostgreSQL against Railway Postgres
INSERT INTO user_roles (user_id, role)
SELECT id, 'billing-admin' FROM users WHERE email = 'your@email.com'
ON CONFLICT DO NOTHING;
```

---

## Billing Overview

### GET /api/v1/admin/billing/overview

High-level subscription health dashboard.

**Response**
```json
{
  "mrr_usd_estimate": 4200.00,
  "status_counts": [
    {"status": "active", "count": 142},
    {"status": "trialing", "count": 23},
    {"status": "past_due", "count": 4},
    {"status": "canceled", "count": 11}
  ],
  "free_users": 891,
  "as_of": "2026-05-08T05:00:00Z"
}
```

| Field | Type | Description |
|---|---|---|
| `mrr_usd_estimate` | float | Sum of active subscription plan prices (estimate — does not account for discounts) |
| `status_counts` | array | Breakdown of subscriptions by Dodo status |
| `free_users` | integer | Users with no active subscription |

---

## Subscriptions

### GET /api/v1/admin/billing/subscriptions

Paginated list of all subscriptions.

**Query Parameters**
| Param | Default | Description |
|---|---|---|
| `status` | (all) | Filter by status: `active`, `trialing`, `past_due`, `canceled` |
| `page` | 1 | Page number (1-indexed) |
| `per_page` | 50 | Results per page (max 200) |

**Response**
```json
{
  "subscriptions": [
    {
      "id": "sub_abc123",
      "user_id": "uuid",
      "user_email": "user@example.com",
      "plan_id": "plan_pro_monthly",
      "status": "active",
      "current_period_end": "2026-06-01T00:00:00Z",
      "cancel_at_period_end": false
    }
  ],
  "total": 180,
  "page": 1,
  "per_page": 50
}
```

---

### POST /api/v1/admin/billing/subscriptions/{id}/cancel

Force-cancel a subscription immediately (bypasses the normal period-end cancellation).

**Path param**: `id` is the Dodo subscription ID (`sub_...`)

**Request**: empty body or `{"reason": "operator-initiated"}`

**Response**: `204 No Content` on success.

⚠️ This cancels immediately — the user loses access at the next credit check. Only use when there's a billing dispute or policy violation.

---

## Webhook Events

### GET /api/v1/admin/billing/webhook-events

Returns the last 100 processed Dodo webhook events.

**Query Parameters**
| Param | Default | Description |
|---|---|---|
| `limit` | 100 | Max events to return (max 500) |
| `event_type` | (all) | Filter by event type (e.g., `subscription.created`) |

**Response**
```json
{
  "events": [
    {
      "id": "evt_abc123",
      "event_type": "subscription.updated",
      "status": "processed",
      "received_at": "2026-05-08T04:30:00Z",
      "processed_at": "2026-05-08T04:30:01Z",
      "payload_summary": {"subscription_id": "sub_abc123", "new_status": "active"}
    }
  ]
}
```

---

## Usage & Analytics

### GET /api/v1/admin/usage/summary

Aggregated usage across all users for a time window.

**Query Parameters**
| Param | Default | Description |
|---|---|---|
| `range_days` | 30 | Lookback window in days (integer, **not** a float/bigint) |
| `engine_limit` | 10 | Top N engines to include |
| `top_users_limit` | 10 | Top N users by usage |

> ⚠️ **Type gotcha**: `range_days` must be an integer. The underlying SQL uses `::integer` cast — sending a float or bigint may produce a type error. Always pass whole numbers (e.g., `30`, not `30.0`).

**Response**
```json
{
  "range_days": 30,
  "total_calculations": 48291,
  "unique_users": 312,
  "top_engines": [
    {"engine_id": "panchanga", "call_count": 12043},
    {"engine_id": "human-design", "call_count": 9821}
  ],
  "top_users": [
    {"user_id": "uuid", "email": "user@example.com", "call_count": 2341}
  ],
  "credits_consumed": 51892
}
```

---

### GET /api/v1/admin/analytics/summary

Platform-wide aggregated analytics (broader than usage — includes session counts, error rates, latency percentiles).

**Query Parameters**: same as `/usage/summary`

**Response**
```json
{
  "range_days": 30,
  "total_requests": 62004,
  "error_rate_pct": 0.4,
  "p50_latency_ms": 12,
  "p95_latency_ms": 89,
  "p99_latency_ms": 340,
  "unique_users": 312,
  "new_registrations": 47
}
```

---

### GET /api/v1/admin/analytics/top-consumers

Top API consumers by call count.

**Query Parameters**
| Param | Default | Description |
|---|---|---|
| `limit` | 20 | Number of users to return |
| `range_days` | 30 | Lookback window |

**Response**
```json
{
  "consumers": [
    {
      "user_id": "uuid",
      "email": "power@user.com",
      "total_calls": 9823,
      "credits_consumed": 10204,
      "tier": "enterprise"
    }
  ]
}
```

---

## Plans

### GET /api/v1/admin/billing/plans

Returns the configured plan catalog from Dodo.

**Response**
```json
{
  "plans": [
    {
      "plan_id": "plan_free",
      "name": "Free",
      "price_usd": 0,
      "credits_per_month": 50,
      "rate_limit_rpm": 10
    },
    {
      "plan_id": "plan_starter_monthly",
      "name": "Starter",
      "price_usd": 9.00,
      "credits_per_month": 500,
      "rate_limit_rpm": 60
    },
    {
      "plan_id": "plan_pro_monthly",
      "name": "Pro",
      "price_usd": 29.00,
      "credits_per_month": 5000,
      "rate_limit_rpm": 300
    }
  ]
}
```

---

## Common Failure Modes

| Symptom | Likely Cause | Fix |
|---|---|---|
| 500 on `/admin/billing/overview` | `user_roles` CHECK constraint missing `billing-admin` | Run migration `023_reconcile_runs.sql` and check `user_roles` constraint |
| 500 on `/admin/usage/summary` | `range_days` passed as bigint/float from Postgres | Ensure the SQL cast is `::integer` not `::bigint` (see migration `023`) |
| 404 on `/api/v1/admin/billing/*` | Service deployed without billing feature flag | Confirm `ENABLE_BILLING=true` in Railway env vars |
| Empty `status_counts` | Dodo webhook events not being ingested | Check `/admin/billing/webhook-events` for `failed` events |

---

## Related

- [Billing (User-facing)](./billing.md)
- [Reconciliation](./admin-reconcile.md)
- [Runbook: Admin Billing Dashboard](../../runbooks/admin-billing-dashboard.md)
