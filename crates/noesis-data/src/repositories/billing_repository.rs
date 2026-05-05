//! Billing repository — Dodo webhook handler's persistence layer.
//!
//! Owns: idempotency (`processed_webhook_events`), `billing_subscriptions`
//! upserts, `users.dodo_customer_id` + `users.tier` mirror updates, and
//! `plan_catalog` lookups by Dodo product ID.

use crate::models::subscription::{BillingSubscription, PlanCatalogEntry};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow, PgPool};
use uuid::Uuid;

pub const PROVIDER_DODO: &str = "dodo_payments";

/// Slim projection of `user_active_plan_resolutions` for the balance proxy.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ActivePlanResolution {
    pub user_id: Uuid,
    pub plan_code: String,
    pub provider_customer_id: Option<String>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub cancel_at_period_end: bool,
}

pub struct BillingRepository {
    pool: PgPool,
}

impl BillingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Cheap pre-flight check: has this webhook_id already been fully
    /// processed? Used as the FIRST gate before dispatch so duplicate
    /// deliveries from Dodo's retry chain short-circuit cleanly.
    pub async fn webhook_event_already_processed(&self, webhook_id: &str) -> Result<bool, Error> {
        let row: Option<(i64,)> = sqlx::query_as(
            r#"SELECT 1::bigint FROM processed_webhook_events WHERE webhook_id = $1 LIMIT 1"#,
        )
        .bind(webhook_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.is_some())
    }

    /// Record a webhook delivery as processed. Called only AFTER dispatch
    /// succeeds, so transient dispatch failures (DB blip, etc.) don't
    /// poison-pill future retries — Dodo retries the event, the row isn't
    /// there yet, dispatch runs again, and only the successful run records
    /// idempotency.
    ///
    /// Returns `Ok(true)` if this call inserted the row, `Ok(false)` if a
    /// concurrent delivery beat us (still safe — both ran an idempotent
    /// dispatch and only one INSERT lands due to PK).
    pub async fn record_webhook_event_processed(
        &self,
        webhook_id: &str,
        provider: &str,
        event_type: &str,
    ) -> Result<bool, Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO processed_webhook_events (webhook_id, provider, event_type)
            VALUES ($1, $2, $3)
            ON CONFLICT (webhook_id) DO NOTHING
            "#,
        )
        .bind(webhook_id)
        .bind(provider)
        .bind(event_type)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() == 1)
    }

    /// Look up a plan catalog entry by Dodo product ID (set in
    /// `plan_catalog.dodo_product_id` post-provisioning).
    pub async fn find_plan_by_dodo_product_id(
        &self,
        product_id: &str,
    ) -> Result<Option<PlanCatalogEntry>, Error> {
        sqlx::query_as::<_, PlanCatalogEntry>(
            r#"
            SELECT id, code, display_name, description, is_active, metadata,
                   dodo_product_id, created_at, updated_at
            FROM plan_catalog
            WHERE dodo_product_id = $1
            LIMIT 1
            "#,
        )
        .bind(product_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Look up a plan catalog entry by canonical plan code (`free`, `basic`,
    /// `premium`, `enterprise`). Used by the checkout route to translate a
    /// user's selected tier into a Dodo product ID.
    pub async fn find_plan_by_code(&self, code: &str) -> Result<Option<PlanCatalogEntry>, Error> {
        sqlx::query_as::<_, PlanCatalogEntry>(
            r#"
            SELECT id, code, display_name, description, is_active, metadata,
                   dodo_product_id, created_at, updated_at
            FROM plan_catalog
            WHERE code = $1 AND is_active = true
            LIMIT 1
            "#,
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
    }

    /// Read the user's email + current Dodo customer ID for checkout. The
    /// email is required so the Dodo session pre-fills the customer form.
    pub async fn get_user_for_checkout(
        &self,
        user_id: Uuid,
    ) -> Result<Option<(String, Option<String>)>, Error> {
        let row: Option<(String, Option<String>)> =
            sqlx::query_as(r#"SELECT email, dodo_customer_id FROM users WHERE id = $1 LIMIT 1"#)
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row)
    }

    /// Read the current calendar month's engine call count for a user.
    /// Used by the free-tier quota gate. Returns 0 if no row exists yet.
    pub async fn get_monthly_engine_count(&self, user_id: Uuid) -> Result<i64, Error> {
        let row: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT count
            FROM engine_usage_monthly
            WHERE user_id = $1
              AND period_start = date_trunc('month', NOW())::date
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|(c,)| c).unwrap_or(0))
    }

    /// Atomic upsert: creates the row at count=1 on first call this month,
    /// increments otherwise. Returns the new count.
    pub async fn increment_monthly_engine_count(&self, user_id: Uuid) -> Result<i64, Error> {
        let row: (i64,) = sqlx::query_as(
            r#"
            INSERT INTO engine_usage_monthly (user_id, period_start, count, updated_at)
            VALUES ($1, date_trunc('month', NOW())::date, 1, NOW())
            ON CONFLICT (user_id, period_start) DO UPDATE
            SET count = engine_usage_monthly.count + 1,
                updated_at = NOW()
            RETURNING count
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0)
    }

    /// Hot-path lookup for the usage emitter. Returns `Ok(None)` for users
    /// who have never checked out (free tier — no Dodo customer to bill).
    pub async fn find_user_dodo_customer_id(&self, user_id: Uuid) -> Result<Option<String>, Error> {
        let row: Option<(Option<String>,)> =
            sqlx::query_as(r#"SELECT dodo_customer_id FROM users WHERE id = $1 LIMIT 1"#)
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.and_then(|(maybe,)| maybe))
    }

    /// Tier + period_end + dodo_customer_id from the `user_active_plan_resolutions`
    /// view (defined in migration 014). Used by the balance proxy to combine
    /// local subscription state with Dodo's credit balance.
    ///
    /// Accepts both `dodo_payments` and `legacy` provider rows — legacy rows
    /// are the seed data from migration 014's backfill (one per existing
    /// user, mirroring `users.tier`). Dodo-issued rows take priority when
    /// both exist for the same user.
    pub async fn find_active_plan_resolution(
        &self,
        user_id: Uuid,
    ) -> Result<Option<ActivePlanResolution>, Error> {
        sqlx::query_as::<_, ActivePlanResolution>(
            r#"
            SELECT
                user_id,
                plan_code,
                provider_customer_id,
                current_period_end,
                cancel_at_period_end
            FROM user_active_plan_resolutions
            WHERE user_id = $1
              AND provider IN ('dodo_payments', 'legacy')
            ORDER BY CASE provider
                       WHEN 'dodo_payments' THEN 0
                       ELSE 1
                     END
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Reverse user lookup for events that only carry `customer_id` (no
    /// metadata). Used for renewals and credit events.
    pub async fn find_user_id_by_dodo_customer_id(
        &self,
        customer_id: &str,
    ) -> Result<Option<Uuid>, Error> {
        let row: Option<(Uuid,)> =
            sqlx::query_as(r#"SELECT id FROM users WHERE dodo_customer_id = $1 LIMIT 1"#)
                .bind(customer_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.map(|(id,)| id))
    }

    /// First-checkout wiring — links the user to their newly-created Dodo
    /// customer. Idempotent: setting the same value twice is a no-op.
    pub async fn set_user_dodo_customer_id(
        &self,
        user_id: Uuid,
        customer_id: &str,
    ) -> Result<(), Error> {
        sqlx::query(r#"UPDATE users SET dodo_customer_id = $2, updated_at = NOW() WHERE id = $1"#)
            .bind(user_id)
            .bind(customer_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Mirror the active plan onto `users.tier`. Authoritative state lives in
    /// `billing_subscriptions`; this column is a denormalised fast-path for
    /// hot middleware.
    pub async fn set_user_tier(&self, user_id: Uuid, tier: &str) -> Result<(), Error> {
        sqlx::query(r#"UPDATE users SET tier = $2, updated_at = NOW() WHERE id = $1"#)
            .bind(user_id)
            .bind(tier)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Upsert a subscription row keyed by (provider, provider_subscription_id).
    /// Used by `subscription.active` and `subscription.updated` handlers.
    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_subscription(
        &self,
        user_id: Uuid,
        plan_id: Uuid,
        provider_customer_id: &str,
        provider_subscription_id: &str,
        status: &str,
        cancel_at_period_end: bool,
        current_period_start: Option<DateTime<Utc>>,
        current_period_end: Option<DateTime<Utc>>,
        metadata: serde_json::Value,
    ) -> Result<BillingSubscription, Error> {
        sqlx::query_as::<_, BillingSubscription>(
            r#"
            INSERT INTO billing_subscriptions (
                user_id, plan_id, provider, provider_customer_id,
                provider_subscription_id, status, cancel_at_period_end,
                current_period_start, current_period_end, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (provider, provider_subscription_id)
            WHERE provider_subscription_id IS NOT NULL
            DO UPDATE SET
                user_id              = EXCLUDED.user_id,
                plan_id              = EXCLUDED.plan_id,
                provider_customer_id = EXCLUDED.provider_customer_id,
                status               = EXCLUDED.status,
                cancel_at_period_end = EXCLUDED.cancel_at_period_end,
                current_period_start = EXCLUDED.current_period_start,
                current_period_end   = EXCLUDED.current_period_end,
                metadata             = billing_subscriptions.metadata || EXCLUDED.metadata,
                updated_at           = NOW()
            RETURNING id, user_id, plan_id, provider, provider_customer_id,
                      provider_subscription_id, status, cancel_at_period_end,
                      current_period_start, current_period_end, canceled_at,
                      created_at, updated_at, metadata
            "#,
        )
        .bind(user_id)
        .bind(plan_id)
        .bind(PROVIDER_DODO)
        .bind(provider_customer_id)
        .bind(provider_subscription_id)
        .bind(status)
        .bind(cancel_at_period_end)
        .bind(current_period_start)
        .bind(current_period_end)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await
    }

    /// Mark a subscription cancelled. Honours `cancel_at_period_end=true` →
    /// keeps `status='active'` until period_end; immediate cancel sets
    /// `status='canceled'` + `canceled_at=NOW()`.
    pub async fn cancel_subscription(
        &self,
        provider_subscription_id: &str,
        cancel_at_period_end: bool,
    ) -> Result<Option<BillingSubscription>, Error> {
        let new_status = if cancel_at_period_end {
            "active"
        } else {
            "canceled"
        };
        sqlx::query_as::<_, BillingSubscription>(
            r#"
            UPDATE billing_subscriptions
            SET status               = $2,
                cancel_at_period_end = $3,
                canceled_at          = CASE WHEN $3 THEN canceled_at ELSE NOW() END,
                updated_at           = NOW()
            WHERE provider = $4 AND provider_subscription_id = $1
            RETURNING id, user_id, plan_id, provider, provider_customer_id,
                      provider_subscription_id, status, cancel_at_period_end,
                      current_period_start, current_period_end, canceled_at,
                      created_at, updated_at, metadata
            "#,
        )
        .bind(provider_subscription_id)
        .bind(new_status)
        .bind(cancel_at_period_end)
        .bind(PROVIDER_DODO)
        .fetch_optional(&self.pool)
        .await
    }

    /// List provider_subscription_ids for all currently-active rows owned by
    /// `provider`. Used by the reconciliation cron to diff against the
    /// authoritative side (Dodo) and detect drift.
    pub async fn list_active_provider_subscription_ids(
        &self,
        provider: &str,
    ) -> Result<Vec<String>, Error> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT provider_subscription_id
            FROM billing_subscriptions
            WHERE provider = $1
              AND status IN ('trialing', 'active', 'past_due')
              AND canceled_at IS NULL
              AND provider_subscription_id IS NOT NULL
            "#,
        )
        .bind(provider)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|(s,)| s).collect())
    }

    /// Force a subscription identified by provider_subscription_id into the
    /// canceled state. Used by reconciliation when Dodo reports the row as
    /// canceled but our local state still says active.
    pub async fn force_cancel_subscription(
        &self,
        provider_subscription_id: &str,
    ) -> Result<Option<BillingSubscription>, Error> {
        sqlx::query_as::<_, BillingSubscription>(
            r#"
            UPDATE billing_subscriptions
            SET status = 'canceled',
                canceled_at = COALESCE(canceled_at, NOW()),
                cancel_at_period_end = false,
                updated_at = NOW()
            WHERE provider = $2 AND provider_subscription_id = $1
            RETURNING id, user_id, plan_id, provider, provider_customer_id,
                      provider_subscription_id, status, cancel_at_period_end,
                      current_period_start, current_period_end, canceled_at,
                      created_at, updated_at, metadata
            "#,
        )
        .bind(provider_subscription_id)
        .bind(PROVIDER_DODO)
        .fetch_optional(&self.pool)
        .await
    }

    /// Push a subscription into past_due (renewal failed, in dunning window).
    pub async fn set_subscription_past_due(
        &self,
        provider_subscription_id: &str,
    ) -> Result<Option<BillingSubscription>, Error> {
        sqlx::query_as::<_, BillingSubscription>(
            r#"
            UPDATE billing_subscriptions
            SET status = 'past_due', updated_at = NOW()
            WHERE provider = $2 AND provider_subscription_id = $1
            RETURNING id, user_id, plan_id, provider, provider_customer_id,
                      provider_subscription_id, status, cancel_at_period_end,
                      current_period_start, current_period_end, canceled_at,
                      created_at, updated_at, metadata
            "#,
        )
        .bind(provider_subscription_id)
        .bind(PROVIDER_DODO)
        .fetch_optional(&self.pool)
        .await
    }
}
