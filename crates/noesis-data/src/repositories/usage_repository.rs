use chrono::{Datelike, NaiveDate, TimeZone, Utc};
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

pub struct AdminUsageDailyPoint {
    pub day: String,
    pub request_count: i64,
}

pub struct AdminUsageTierDistribution {
    pub tier: String,
    pub request_count: i64,
}

pub struct UsageWindowSummary {
    pub total: i64,
    pub success: i64,
    pub failure: i64,
}

pub struct UsageDateRange {
    pub start: chrono::DateTime<Utc>,
    pub end: chrono::DateTime<Utc>,
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

    /// Get usage totals for one UTC day.
    pub async fn get_daily_usage(
        &self,
        user_id: Uuid,
        date: NaiveDate,
    ) -> Result<UsageWindowSummary, Error> {
        let start = Utc
            .with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
            .single()
            .expect("valid day start");
        let end = start + chrono::Duration::days(1);

        self.get_window_usage(user_id, UsageDateRange { start, end })
            .await
    }

    /// Get usage totals for one UTC month (based on the provided date's year/month).
    pub async fn get_monthly_summary(
        &self,
        user_id: Uuid,
        month: NaiveDate,
    ) -> Result<UsageWindowSummary, Error> {
        let start = Utc
            .with_ymd_and_hms(month.year(), month.month(), 1, 0, 0, 0)
            .single()
            .expect("valid month start");

        let (end_year, end_month) = if month.month() == 12 {
            (month.year() + 1, 1)
        } else {
            (month.year(), month.month() + 1)
        };

        let end = Utc
            .with_ymd_and_hms(end_year, end_month, 1, 0, 0, 0)
            .single()
            .expect("valid month end");

        self.get_window_usage(user_id, UsageDateRange { start, end })
            .await
    }

    /// Get per-engine usage breakdown for a user in a fixed date range.
    pub async fn get_engine_breakdown(
        &self,
        user_id: Uuid,
        date_range: UsageDateRange,
    ) -> Result<Vec<UsageEngineBreakdown>, Error> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT
                COALESCE(NULLIF(engine_id, ''), 'unknown') AS engine_id,
                COUNT(*)::BIGINT AS request_count
            FROM usage_logs
            WHERE user_id = $1
              AND created_at >= $2
              AND created_at < $3
            GROUP BY COALESCE(NULLIF(engine_id, ''), 'unknown')
            ORDER BY request_count DESC, engine_id ASC
            "#,
        )
        .bind(user_id)
        .bind(date_range.start)
        .bind(date_range.end)
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

    /// Get top users by request count in a fixed date range.
    pub async fn get_top_users(
        &self,
        limit: i64,
        date_range: UsageDateRange,
    ) -> Result<Vec<AdminUsageTopUser>, Error> {
        let rows = sqlx::query_as::<_, (Uuid, String, i64)>(
            r#"
            SELECT
                l.user_id,
                u.email,
                COUNT(*)::BIGINT AS request_count
            FROM usage_logs l
            INNER JOIN users u ON u.id = l.user_id
            WHERE l.created_at >= $1
              AND l.created_at < $2
            GROUP BY l.user_id, u.email
            ORDER BY request_count DESC, u.email ASC
            LIMIT $3
            "#,
        )
        .bind(date_range.start)
        .bind(date_range.end)
        .bind(limit.max(1))
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

    async fn get_window_usage(
        &self,
        user_id: Uuid,
        date_range: UsageDateRange,
    ) -> Result<UsageWindowSummary, Error> {
        let row = sqlx::query_as::<_, (i64, i64, i64)>(
            r#"
            SELECT
                COUNT(*)::BIGINT AS total,
                COUNT(*) FILTER (WHERE status = 'success')::BIGINT AS success,
                COUNT(*) FILTER (WHERE status = 'failure')::BIGINT AS failure
            FROM usage_logs
            WHERE user_id = $1
              AND created_at >= $2
              AND created_at < $3
            "#,
        )
        .bind(user_id)
        .bind(date_range.start)
        .bind(date_range.end)
        .fetch_one(&self.pool)
        .await?;

        Ok(UsageWindowSummary {
            total: row.0,
            success: row.1,
            failure: row.2,
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
              AND created_at >= NOW() - make_interval(hours => $2::integer)
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
            WHERE created_at >= NOW() - make_interval(hours => $1::integer)
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
            WHERE created_at >= NOW() - make_interval(hours => $1::integer)
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

    /// Get daily request counts over the last `range_days` days.
    pub async fn admin_daily_series(
        &self,
        range_days: i64,
    ) -> Result<Vec<AdminUsageDailyPoint>, Error> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT
                to_char(date_trunc('day', created_at), 'YYYY-MM-DD') AS day,
                COUNT(*)::BIGINT AS request_count
            FROM usage_logs
            WHERE created_at >= NOW() - make_interval(days => $1::integer)
            GROUP BY date_trunc('day', created_at)
            ORDER BY date_trunc('day', created_at) ASC
            "#,
        )
        .bind(range_days)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(day, request_count)| AdminUsageDailyPoint { day, request_count })
            .collect())
    }

    /// Get request distribution by user tier over the last `window_hours`.
    pub async fn admin_tier_distribution(
        &self,
        window_hours: i64,
    ) -> Result<Vec<AdminUsageTierDistribution>, Error> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT
                COALESCE(NULLIF(lower(u.tier), ''), 'unknown') AS tier,
                COUNT(*)::BIGINT AS request_count
            FROM usage_logs l
            INNER JOIN users u ON u.id = l.user_id
            WHERE l.created_at >= NOW() - make_interval(hours => $1::integer)
            GROUP BY COALESCE(NULLIF(lower(u.tier), ''), 'unknown')
            ORDER BY request_count DESC, tier ASC
            "#,
        )
        .bind(window_hours)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(tier, request_count)| AdminUsageTierDistribution {
                tier,
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
            WHERE l.created_at >= NOW() - make_interval(hours => $1::integer)
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    async fn seed_user(pool: &PgPool, id: Uuid, email: &str, tier: &str) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, full_name, tier, consciousness_level)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(id)
        .bind(email)
        .bind("test_hash")
        .bind("Test User")
        .bind(tier)
        .bind(1_i32)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn seed_usage(
        pool: &PgPool,
        user_id: Uuid,
        engine_id: Option<&str>,
        status: &str,
        created_at: chrono::DateTime<Utc>,
    ) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO usage_logs (user_id, engine_id, workflow_id, status, duration_ms, created_at)
            VALUES ($1, $2, NULL, $3, 120, $4)
            "#,
        )
        .bind(user_id)
        .bind(engine_id)
        .bind(status)
        .bind(created_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "requires SQLx test database"]
    async fn get_daily_usage_returns_expected_counts(pool: PgPool) -> Result<(), Error> {
        let repo = UsageRepository::new(pool.clone());
        let user_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();

        seed_user(&pool, user_id, "daily@example.com", "free").await?;
        seed_user(&pool, other_user_id, "other-daily@example.com", "free").await?;

        seed_usage(
            &pool,
            user_id,
            Some("panchanga"),
            "success",
            Utc.with_ymd_and_hms(2026, 4, 15, 1, 0, 0)
                .single()
                .expect("valid ts"),
        )
        .await?;
        seed_usage(
            &pool,
            user_id,
            Some("panchanga"),
            "failure",
            Utc.with_ymd_and_hms(2026, 4, 15, 2, 0, 0)
                .single()
                .expect("valid ts"),
        )
        .await?;
        seed_usage(
            &pool,
            user_id,
            Some("biorhythm"),
            "success",
            Utc.with_ymd_and_hms(2026, 4, 15, 3, 0, 0)
                .single()
                .expect("valid ts"),
        )
        .await?;

        // Outside requested day
        seed_usage(
            &pool,
            user_id,
            Some("biorhythm"),
            "success",
            Utc.with_ymd_and_hms(2026, 4, 16, 3, 0, 0)
                .single()
                .expect("valid ts"),
        )
        .await?;

        // Different user on same day
        seed_usage(
            &pool,
            other_user_id,
            Some("panchanga"),
            "success",
            Utc.with_ymd_and_hms(2026, 4, 15, 4, 0, 0)
                .single()
                .expect("valid ts"),
        )
        .await?;

        let summary = repo
            .get_daily_usage(
                user_id,
                NaiveDate::from_ymd_opt(2026, 4, 15).expect("valid date"),
            )
            .await?;

        assert_eq!(summary.total, 3);
        assert_eq!(summary.success, 2);
        assert_eq!(summary.failure, 1);

        Ok(())
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "requires SQLx test database"]
    async fn get_monthly_summary_returns_expected_counts(pool: PgPool) -> Result<(), Error> {
        let repo = UsageRepository::new(pool.clone());
        let user_id = Uuid::new_v4();

        seed_user(&pool, user_id, "monthly@example.com", "pro").await?;

        seed_usage(
            &pool,
            user_id,
            Some("panchanga"),
            "success",
            Utc.with_ymd_and_hms(2026, 4, 1, 0, 10, 0)
                .single()
                .expect("valid ts"),
        )
        .await?;
        seed_usage(
            &pool,
            user_id,
            Some("biorhythm"),
            "failure",
            Utc.with_ymd_and_hms(2026, 4, 30, 23, 59, 0)
                .single()
                .expect("valid ts"),
        )
        .await?;

        // Outside April
        seed_usage(
            &pool,
            user_id,
            Some("panchanga"),
            "success",
            Utc.with_ymd_and_hms(2026, 3, 31, 23, 59, 0)
                .single()
                .expect("valid ts"),
        )
        .await?;

        let summary = repo
            .get_monthly_summary(
                user_id,
                NaiveDate::from_ymd_opt(2026, 4, 1).expect("valid date"),
            )
            .await?;

        assert_eq!(summary.total, 2);
        assert_eq!(summary.success, 1);
        assert_eq!(summary.failure, 1);

        Ok(())
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "requires SQLx test database"]
    async fn get_engine_breakdown_returns_expected_rows(pool: PgPool) -> Result<(), Error> {
        let repo = UsageRepository::new(pool.clone());
        let user_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();

        seed_user(&pool, user_id, "breakdown@example.com", "free").await?;
        seed_user(&pool, other_user_id, "breakdown-other@example.com", "free").await?;

        seed_usage(
            &pool,
            user_id,
            Some("panchanga"),
            "success",
            Utc.with_ymd_and_hms(2026, 4, 10, 9, 0, 0)
                .single()
                .expect("valid ts"),
        )
        .await?;
        seed_usage(
            &pool,
            user_id,
            Some("panchanga"),
            "success",
            Utc.with_ymd_and_hms(2026, 4, 11, 9, 0, 0)
                .single()
                .expect("valid ts"),
        )
        .await?;
        seed_usage(
            &pool,
            user_id,
            Some("biorhythm"),
            "success",
            Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0)
                .single()
                .expect("valid ts"),
        )
        .await?;
        seed_usage(
            &pool,
            user_id,
            None,
            "success",
            Utc.with_ymd_and_hms(2026, 4, 13, 9, 0, 0)
                .single()
                .expect("valid ts"),
        )
        .await?;

        // Different user and out-of-range rows must not be included
        seed_usage(
            &pool,
            other_user_id,
            Some("panchanga"),
            "success",
            Utc.with_ymd_and_hms(2026, 4, 14, 9, 0, 0)
                .single()
                .expect("valid ts"),
        )
        .await?;
        seed_usage(
            &pool,
            user_id,
            Some("panchanga"),
            "success",
            Utc.with_ymd_and_hms(2026, 5, 1, 0, 0, 0)
                .single()
                .expect("valid ts"),
        )
        .await?;

        let rows = repo
            .get_engine_breakdown(
                user_id,
                UsageDateRange {
                    start: Utc
                        .with_ymd_and_hms(2026, 4, 1, 0, 0, 0)
                        .single()
                        .expect("valid start"),
                    end: Utc
                        .with_ymd_and_hms(2026, 5, 1, 0, 0, 0)
                        .single()
                        .expect("valid end"),
                },
            )
            .await?;

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].engine_id, "panchanga");
        assert_eq!(rows[0].request_count, 2);
        assert_eq!(rows[1].engine_id, "biorhythm");
        assert_eq!(rows[1].request_count, 1);
        assert_eq!(rows[2].engine_id, "unknown");
        assert_eq!(rows[2].request_count, 1);

        Ok(())
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "requires SQLx test database"]
    async fn get_top_users_returns_ranked_rows(pool: PgPool) -> Result<(), Error> {
        let repo = UsageRepository::new(pool.clone());
        let user_a = Uuid::new_v4();
        let user_b = Uuid::new_v4();
        let user_c = Uuid::new_v4();

        seed_user(&pool, user_a, "a-top@example.com", "pro").await?;
        seed_user(&pool, user_b, "b-top@example.com", "free").await?;
        seed_user(&pool, user_c, "c-top@example.com", "free").await?;

        for _ in 0..5 {
            seed_usage(
                &pool,
                user_a,
                Some("panchanga"),
                "success",
                Utc.with_ymd_and_hms(2026, 4, 20, 8, 0, 0)
                    .single()
                    .expect("valid ts"),
            )
            .await?;
        }
        for _ in 0..3 {
            seed_usage(
                &pool,
                user_b,
                Some("biorhythm"),
                "success",
                Utc.with_ymd_and_hms(2026, 4, 20, 8, 0, 0)
                    .single()
                    .expect("valid ts"),
            )
            .await?;
        }
        seed_usage(
            &pool,
            user_c,
            Some("human-design"),
            "success",
            Utc.with_ymd_and_hms(2026, 4, 20, 8, 0, 0)
                .single()
                .expect("valid ts"),
        )
        .await?;

        // Out-of-range usage should not affect results.
        seed_usage(
            &pool,
            user_c,
            Some("human-design"),
            "success",
            Utc.with_ymd_and_hms(2026, 5, 2, 8, 0, 0)
                .single()
                .expect("valid ts"),
        )
        .await?;

        let top = repo
            .get_top_users(
                2,
                UsageDateRange {
                    start: Utc
                        .with_ymd_and_hms(2026, 4, 1, 0, 0, 0)
                        .single()
                        .expect("valid start"),
                    end: Utc
                        .with_ymd_and_hms(2026, 5, 1, 0, 0, 0)
                        .single()
                        .expect("valid end"),
                },
            )
            .await?;

        assert_eq!(top.len(), 2);
        assert_eq!(top[0].user_id, user_a);
        assert_eq!(top[0].request_count, 5);
        assert_eq!(top[1].user_id, user_b);
        assert_eq!(top[1].request_count, 3);

        Ok(())
    }
}
