# Billing API Reference

> Version: 3.3.0 · Payment processor: Dodo Payments

Noesis uses a credit-based billing model. Users purchase credits via Dodo Payments, credits are debited per engine call, and the subscription tier controls rate limits and feature access.

## Authentication

All billing endpoints require a valid JWT Bearer token or `X-API-Key` header for the authenticated user. Admin billing endpoints additionally require the `billing-admin` role — see [admin-analytics.md](./admin-analytics.md).

---

## User-Facing Endpoints

### GET /api/v1/billing/balance

Returns the current credit balance and tier for the authenticated user.

**Response**
```json
{
  "user_id": "uuid",
  "balance_credits": 4200,
  "tier": "pro",
  "tier_display": "Pro",
  "reset_at": "2026-06-01T00:00:00Z"
}
```

| Field | Type | Description |
|---|---|---|
| `balance_credits` | integer | Remaining credits in the current period |
| `tier` | string | `free` \| `starter` \| `pro` \| `enterprise` |
| `reset_at` | ISO-8601 | Next credit reset date (if applicable) |

---

### GET /api/v1/billing/subscription

Returns the current Dodo subscription status.

**Response**
```json
{
  "subscription_id": "sub_abc123",
  "status": "active",
  "plan_id": "plan_pro_monthly",
  "plan_name": "Pro Monthly",
  "current_period_start": "2026-05-01T00:00:00Z",
  "current_period_end": "2026-06-01T00:00:00Z",
  "cancel_at_period_end": false
}
```

Possible `status` values: `active`, `trialing`, `past_due`, `canceled`, `paused`, `incomplete`.

---

### POST /api/v1/billing/checkout

Initiate a plan upgrade via Dodo Payments. Returns a hosted checkout URL.

**Request**
```json
{
  "plan_id": "plan_pro_monthly",
  "success_url": "https://myapp.com/billing/success",
  "cancel_url": "https://myapp.com/billing/cancel"
}
```

**Response**
```json
{
  "checkout_url": "https://checkout.dodo.com/c/abc123",
  "session_id": "cs_abc123",
  "expires_at": "2026-05-09T06:00:00Z"
}
```

Redirect the user to `checkout_url`. On completion, Dodo posts a webhook to Noesis which updates the subscription and credits automatically.

---

### GET /api/v1/billing/portal

Returns a Dodo customer portal URL where the user can update payment methods, download invoices, or cancel.

**Response**
```json
{
  "portal_url": "https://billing.dodo.com/p/abc123",
  "expires_at": "2026-05-09T06:00:00Z"
}
```

Portal sessions expire after 15 minutes. Do not cache — always fetch fresh.

---

## Credit Consumption

Credits are debited per successful engine calculation. Failed calls (4xx/5xx from the engine) do not consume credits.

| Engine type | Credits per call |
|---|---|
| Single engine (`/engines/{id}/calculate`) | 1 |
| Workflow (`/workflows/{id}/execute`) | N where N = number of engines in workflow |
| Witness interpret (`/witness/interpret`) | 2 |

Free tier: 50 credits/month. Starter: 500. Pro: 5000. Enterprise: custom.

---

## Tier Limits

| Tier | Rate limit | Concurrency | Features |
|---|---|---|---|
| `free` | 10 req/min | 1 | Engine + workflow calls |
| `starter` | 60 req/min | 3 | + Readings history |
| `pro` | 300 req/min | 10 | + Biofield, Witness |
| `enterprise` | Unlimited | Unlimited | + Admin API access |

---

## Webhook Events

Dodo sends webhook events to `/api/v1/billing/events` (internal, not user-callable). Events are logged and viewable by billing admins at `GET /api/v1/admin/billing/webhook-events`.

Handled event types:
- `subscription.created`
- `subscription.updated`
- `subscription.deleted`
- `payment.succeeded`
- `payment.failed`
- `invoice.payment_succeeded`

---

## Error Codes

| HTTP | Error code | Meaning |
|---|---|---|
| 402 | `INSUFFICIENT_CREDITS` | Balance is 0 — recharge or upgrade |
| 402 | `SUBSCRIPTION_REQUIRED` | Feature requires paid tier |
| 429 | `RATE_LIMIT_EXCEEDED` | Request rate above tier limit |
| 503 | `BILLING_UNAVAILABLE` | Dodo API unreachable (retry) |

---

## Related

- [Admin Analytics & Billing](./admin-analytics.md) — operator overview, subscriptions, reconciliation
- [Admin Reconciliation](./admin-reconcile.md) — drift detection and repair
