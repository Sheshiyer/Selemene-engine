# Release v3.3.0 — Billing Operator Surface

> **Tryambakam Noesis · Selemene Engine**
> Production release shipping the complete Dodo Payments admin billing dashboard, schema migrations, and billing-admin role.

---

## What's New

### Admin Billing Dashboard (`apps/admin-web`)

The operator billing surface is now live at `/billing` inside the dedicated admin app (`admin-web`, port 3001 / `144.tryambakam.space`).

| Route | Surface |
|---|---|
| `/billing` | Overview: subscription counts, free users, MRR estimate |
| `/billing/subscriptions` | Paginated Dodo subscription table, status filter |
| `/billing/subscriptions/[id]` | Single subscription detail + force-cancel |
| `/billing/webhook-events` | Last 100 processed webhook events |
| `/billing/reconcile` | Latest drift report + trigger command |
| `/billing/plans` | Plan catalog (read-only) |

Previously this surface existed in `biofield-web` (the user-facing app). It has been moved to the dedicated `admin-web` operator surface where it belongs — `biofield-web` retains no admin routes.

### Backend Admin Billing Endpoints (Rust, `noesis-api`)

All billing endpoints shipped in #683 and are now correctly wired to `admin-web`:

```
GET  /api/v1/admin/billing/overview
GET  /api/v1/admin/billing/subscriptions
GET  /api/v1/admin/billing/subscriptions/:id
POST /api/v1/admin/billing/subscriptions/:id/cancel
GET  /api/v1/admin/billing/webhook-events
GET  /api/v1/admin/billing/reconcile/drift
POST /api/v1/admin/billing/reconcile/run
GET  /api/v1/admin/billing/plans
```

All endpoints are gated behind `admin:billing:read` (or `:cancel` / `:reconcile:trigger` for mutating operations).

### New Role: `billing-admin`

A new role scoped exclusively to the billing surface. Deliberately disjoint from `admin` so billing access does not grant user-admin privileges (and vice versa). `platform-admin` retains full access via `admin:*` wildcard.

Permissions granted by `billing-admin`:
- `admin:billing:read`
- `admin:billing:subscriptions:cancel`
- `admin:billing:reconcile:trigger`

Grant via SQL — see `runbooks/admin-billing-dashboard.md`.

### Schema Migrations Applied to Production

| Migration | Description |
|---|---|
| `010_user_roles_account_state` | `user_roles` + `user_account_state` tables |
| `011_api_key_events` | API key event log |
| `012_usage_partition_maintenance` | Usage log partition maintenance |
| `013_history_sync_schema` | History sync schema |
| `014_plan_catalog_billing_subscriptions` | Plan catalog + `billing_subscriptions` + `user_active_plan_resolutions` view + legacy backfill |
| `020_dodo_payments_columns` | `dodo_customer_id` on users, `dodo_product_id` on plan_catalog, `metadata` on subscriptions |
| `021_processed_webhook_events` | Processed webhook event deduplication table |
| `022_engine_usage_monthly` | Monthly engine usage aggregation |
| `023_reconcile_runs` | Reconcile run persistence — admin dashboard drift surface |

### Runbook

Full operator guide, grant SQL, common tasks, and failure modes:
`runbooks/admin-billing-dashboard.md`

---

## Breaking Changes

None. All schema changes are additive. The billing surface move from `biofield-web` to `admin-web` is internal — no user-facing routes changed.

---

## Packages

| Package | Version | Description |
|---|---|---|
| `noesis-api` | 3.3.0 | Axum HTTP server — admin billing endpoints, billing-admin role resolution |
| `noesis-auth` | 3.3.0 | JWT + API key auth — billing-admin role + permissions expansion |
| `noesis-data` | 3.3.0 | Admin repository — billing overview, subscription CRUD, webhook events, reconcile |
| `noesis-core` | 3.3.0 | Shared traits (no changes) |
| `noesis-orchestrator` | 3.3.0 | Multi-engine orchestration (no changes) |
| `noesis-witness` | 3.3.0 | Witness prompt generation (no changes) |
| `apps/admin-web` | 3.3.0 | Next.js operator dashboard — `/billing` surface, billing nav routes |

---

## Upgrade Notes

If running a self-hosted instance, apply all migrations `010` through `023` in sequence before deploying this release. The `reconcile_runs` table (`023`) is required for the `/billing/reconcile` page to render; without it the page shows "No reconcile run recorded yet" which is correct but requires the table to exist.

Full migration files are in `supabase/migrations/` and `migrations/`.
