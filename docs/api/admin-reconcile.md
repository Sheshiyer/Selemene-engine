# Admin Billing Reconciliation Reference

> Requires: `billing-admin` role · Version: 3.3.0

Reconciliation is the process of comparing what Dodo Payments believes about subscriptions and credits against what the Noesis database has recorded. Drift occurs when:
- Webhook deliveries fail or are retried
- Race conditions between payment events and user state updates
- Manual database edits

---

## Drift Detection

### GET /api/v1/admin/billing/reconcile/drift

Returns a report of users/subscriptions where Dodo state diverges from the local database.

**Query Parameters**
| Param | Default | Description |
|---|---|---|
| `limit` | 100 | Max discrepancies to return |
| `severity` | (all) | Filter: `critical` (credits mismatch > 20%), `warning` (status mismatch), `info` (metadata drift) |

**Response**
```json
{
  "drift_count": 3,
  "generated_at": "2026-05-08T05:00:00Z",
  "discrepancies": [
    {
      "user_id": "uuid",
      "email": "user@example.com",
      "dodo_subscription_id": "sub_abc123",
      "severity": "critical",
      "drift_type": "status_mismatch",
      "dodo_status": "active",
      "local_status": "canceled",
      "recommendation": "sync_to_dodo"
    },
    {
      "user_id": "uuid2",
      "severity": "warning",
      "drift_type": "credit_mismatch",
      "dodo_credits": 500,
      "local_credits": 0,
      "recommendation": "reset_local_credits"
    }
  ]
}
```

**Drift types**
| Type | Meaning |
|---|---|
| `status_mismatch` | Dodo subscription status ≠ local `subscriptions.status` |
| `credit_mismatch` | Dodo allocated credits ≠ local `billing_balances.balance_credits` |
| `missing_local` | Dodo subscription exists but no local record |
| `orphaned_local` | Local subscription record with no matching Dodo subscription |
| `metadata_drift` | Non-critical fields diverge (plan name, period dates) |

---

## Reconciliation Run

### POST /api/v1/admin/billing/reconcile/run

Triggers a reconciliation run that syncs local state from Dodo for all detected discrepancies (or a specific user/subscription).

**Request** (all fields optional)
```json
{
  "target_user_id": "uuid",
  "target_subscription_id": "sub_abc123",
  "dry_run": true
}
```

| Field | Default | Description |
|---|---|---|
| `target_user_id` | (all) | Limit reconciliation to one user |
| `target_subscription_id` | (all) | Limit to one subscription |
| `dry_run` | `false` | If `true`, returns what _would_ be changed without writing |

**Response**
```json
{
  "run_id": "recon_20260508_001",
  "started_at": "2026-05-08T05:01:00Z",
  "dry_run": false,
  "users_checked": 180,
  "discrepancies_found": 3,
  "discrepancies_fixed": 3,
  "errors": [],
  "completed_at": "2026-05-08T05:01:04Z"
}
```

Reconciliation is synchronous for small runs (< 200 users). For full-platform runs, the response may include a `run_id` to poll, but currently all runs complete in-request.

---

## Reconciliation Runs History

### GET /api/v1/admin/billing/reconcile/runs

Returns the last N reconciliation runs and their summaries.

> ℹ️ This endpoint was introduced in migration `023_reconcile_runs.sql`. If you see a 500, confirm the migration has been applied.

**Query Parameters**
| Param | Default | Description |
|---|---|---|
| `limit` | 20 | Number of runs to return |

**Response**
```json
{
  "runs": [
    {
      "run_id": "recon_20260508_001",
      "started_at": "2026-05-08T05:01:00Z",
      "completed_at": "2026-05-08T05:01:04Z",
      "users_checked": 180,
      "discrepancies_fixed": 3,
      "triggered_by": "operator",
      "dry_run": false
    }
  ]
}
```

---

## Applying Migration 023

If you hit a 500 on any reconcile endpoint, confirm that migration `023_reconcile_runs.sql` has been applied:

```sql
-- Check if the reconcile_runs table exists
SELECT EXISTS (
  SELECT 1 FROM information_schema.tables
  WHERE table_name = 'reconcile_runs'
);
```

To apply via `psql` against Railway Postgres:

```bash
psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f migrations/023_reconcile_runs.sql
```

Or apply manually in your PostgreSQL editor by copying the contents of `migrations/023_reconcile_runs.sql`.

---

## When to Reconcile

| Situation | Action |
|---|---|
| After a Dodo webhook outage | Run full reconcile (no target filter) |
| User reports wrong credits after upgrade | Run targeted reconcile for their `user_id` |
| After manual DB edits (emergency hotfix) | Run dry-run first, then live |
| Routine health check | Weekly scheduled reconcile (not yet automated — do manually) |
| Before major version releases | Full dry-run pass |

---

## Related

- [Admin Analytics](./admin-analytics.md) — usage summary, billing overview
- [Billing (User-facing)](./billing.md)
- [Runbook: Admin Billing Dashboard](../../runbooks/admin-billing-dashboard.md)
