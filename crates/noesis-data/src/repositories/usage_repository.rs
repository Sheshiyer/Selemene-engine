use sqlx::{Error, PgPool};
use uuid::Uuid;

pub struct UsageRepository {
    pool: PgPool,
}

pub struct UserUsageSummary {
    pub daily_total: i64,
    pub daily_success: i64,
    pub daily_failure: i64,
    pub monthly_total: i64,
    pub monthly_success: i64,
    pub monthly_failure: i64,
}

pub struct UsageEngineBreakdown {
    pub engine_id: String,
    pub request_count: i64,
}

pub struct AdminUsageSummary {
    pub total: i64,
    pub success: i64,
    pub failure: i64,
    pub active_users: i64,
}

pub struct AdminUsageTopUser {
    pub user_id: Uuid,
    pub user_email: String,
    pub request_count: i64,
}

impl UsageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Log a usage event to the partitioned usage_logs table.
    pub async fn log_usage(
        &self,
        user_id: Uuid,
        engine_id: Option<&str>,
        workflow_id: Option<&str>,
        status: &str,
        duration_ms: i32,
    ) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO usage_logs (user_id, engine_id, workflow_id, status, duration_ms)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(user_id)
        .bind(engine_id)
        .bind(workflow_id)
        .bind(status)
        .bind(duration_ms)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get daily (24h) and monthly (30d) usage counts for a single user.
    pub async fn user_usage_summary(&self, user_id: Uuid) -> Result<UserUsageSummary, Error> {
        let row = sqlx::query_as::<_, (i64, i64, i64, i64, i64, i64)>(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '24 hours')::BIGINT AS daily_total,
                COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '24 hours' AND status = 'success')::BIGINT AS daily_success,
                COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '24 hours' AND status = 'failure')::BIGINT AS daily_failure,
                COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '30 days')::BIGINT AS monthly_total,
                COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '30 days' AND status = 'success')::BIGINT AS monthly_success,
                COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '30 days' AND status = 'failure')::BIGINT AS monthly_failure
            FROM usage_logs
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(UserUsageSummary {
            daily_total: row.0,
            daily_success: row.1,
            daily_failure: row.2,
            monthly_total: row.3,
            monthly_success: row.4,
            monthly_failure: row.5,
        })
    }

    /// Get per-engine request breakdown for a user over the last `window_hours`.
    pub async fn user_engine_breakdown(
        &self,
        user_id: Uuid,
        window_hours: i64,
        limit: i64,
    ) -> Result<Vec<UsageEngineBreakdown>, Error> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT
                COALESCE(NULLIF(engine_id, ''), 'unknown') AS engine_id,
                COUNT(*)::BIGINT AS request_count
            FROM usage_logs
            WHERE user_id = $1
              AND created_at >= NOW() - make_interval(hours => $2)
            GROUP BY COALESCE(NULLIF(engine_id, ''), 'unknown')
            ORDER BY request_count DESC, engine_id ASC
            LIMIT $3
            "#,
        )
        .bind(user_id)
        .bind(window_hours)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(engine_id, request_count)| UsageEngineBreakdown {
                engine_id,
                request_count,
            })
            .collect())
    }

    /// Get aggregate usage summary over the last `window_hours`.
    pub async fn admin_usage_summary(&self, window_hours: i64) -> Result<AdminUsageSummary, Error> {
        let row = sqlx::query_as::<_, (i64, i64, i64, i64)>(
            r#"
            SELECT
                COUNT(*)::BIGINT AS total,
                COUNT(*) FILTER (WHERE status = 'success')::BIGINT AS success,
                COUNT(*) FILTER (WHERE status = 'failure')::BIGINT AS failure,
                COUNT(DISTINCT user_id)::BIGINT AS active_users
            FROM usage_logs
            WHERE created_at >= NOW() - make_interval(hours => $1)
            "#,
        )
        .bind(window_hours)
        .fetch_one(&self.pool)
        .await?;

        Ok(AdminUsageSummary {
            total: row.0,
            success: row.1,
            failure: row.2,
            active_users: row.3,
        })
    }

    /// Get global per-engine request breakdown over the last `window_hours`.
    pub async fn admin_engine_breakdown(
        &self,
        window_hours: i64,
        limit: i64,
    ) -> Result<Vec<UsageEngineBreakdown>, Error> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT
                COALESCE(NULLIF(engine_id, ''), 'unknown') AS engine_id,
                COUNT(*)::BIGINT AS request_count
            FROM usage_logs
            WHERE created_at >= NOW() - make_interval(hours => $1)
            GROUP BY COALESCE(NULLIF(engine_id, ''), 'unknown')
            ORDER BY request_count DESC, engine_id ASC
            LIMIT $2
            "#,
        )
        .bind(window_hours)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(engine_id, request_count)| UsageEngineBreakdown {
                engine_id,
                request_count,
            })
            .collect())
    }

    /// Get top users by request count over the last `window_hours`.
    pub async fn admin_top_users(
        &self,
        window_hours: i64,
        limit: i64,
    ) -> Result<Vec<AdminUsageTopUser>, Error> {
        let rows = sqlx::query_as::<_, (Uuid, String, i64)>(
            r#"
            SELECT
                l.user_id,
                u.email,
                COUNT(*)::BIGINT AS request_count
            FROM usage_logs l
            INNER JOIN users u ON u.id = l.user_id
            WHERE l.created_at >= NOW() - make_interval(hours => $1)
            GROUP BY l.user_id, u.email
            ORDER BY request_count DESC, u.email ASC
            LIMIT $2
            "#,
        )
        .bind(window_hours)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(user_id, user_email, request_count)| AdminUsageTopUser {
                user_id,
                user_email,
                request_count,
            })
            .collect())
    }
}
