use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{Error, PgPool, Postgres, QueryBuilder};
use std::collections::BTreeSet;
use uuid::Uuid;

pub struct AdminRepository {
    pool: PgPool,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AdminUserRecord {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub tier: String,
    pub consciousness_level: i32,
    pub experience_points: i32,
    pub last_login_at: Option<DateTime<Utc>>,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub active_key_count: i64,
    pub permissions: Value,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AdminApiKeyRecord {
    pub id: Uuid,
    pub name: Option<String>,
    pub key_prefix: Option<String>,
    pub user_id: Uuid,
    pub user_email: String,
    pub tier: String,
    pub permissions: Value,
    pub consciousness_level: i32,
    pub rate_limit: i32,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct NewApiKeyRecord {
    pub key_hash: String,
    pub name: Option<String>,
    pub key_prefix: String,
    pub user_id: Uuid,
    pub tier: String,
    pub permissions: Value,
    pub consciousness_level: i32,
    pub rate_limit: i32,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HistorySyncUserRecord {
    pub user_id: Uuid,
    pub email: String,
    pub readings_count: i64,
    pub usage_events_count: i64,
    pub last_reading_at: Option<DateTime<Utc>>,
    pub last_event_at: Option<DateTime<Utc>>,
    pub drift_count: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HistorySyncDeviceRecord {
    pub device_id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub tier: String,
    pub is_active: bool,
    pub permission_count: i32,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HistorySyncEventRecord {
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub engine_id: Option<String>,
    pub workflow_id: Option<String>,
    pub status: String,
    pub duration_ms: i32,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AnalyticsSummaryRecord {
    pub requests_total: i64,
    pub success_total: i64,
    pub failure_total: i64,
    pub error_rate_pct: f64,
    pub p95_duration_ms: f64,
    pub avg_duration_ms: f64,
    pub active_users: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AnalyticsTimeseriesPointRecord {
    pub bucket_start: DateTime<Utc>,
    pub request_count: i64,
    pub success_count: i64,
    pub failure_count: i64,
    pub avg_duration_ms: f64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AnalyticsBreakdownRecord {
    pub label: String,
    pub request_count: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AnalyticsTopConsumerRecord {
    pub user_id: Uuid,
    pub user_email: String,
    pub request_count: i64,
    pub failure_count: i64,
    pub avg_duration_ms: f64,
}

#[derive(Debug, sqlx::FromRow)]
struct ExistingApiKeyRecord {
    user_id: Uuid,
    name: Option<String>,
    tier: String,
    permissions: Value,
    consciousness_level: i32,
    rate_limit: i32,
    expires_at: Option<DateTime<Utc>>,
}

impl AdminRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_users(
        &self,
        query: Option<&str>,
        tier: Option<&str>,
        state: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AdminUserRecord>, Error> {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT
                u.id,
                u.email,
                COALESCE(u.full_name, '') AS full_name,
                u.tier,
                u.consciousness_level,
                u.experience_points,
                u.last_login_at,
                u.failed_login_attempts,
                u.locked_until,
                u.created_at,
                u.updated_at,
                COALESCE((
                    SELECT COUNT(*)
                    FROM api_keys k
                    WHERE k.user_id = u.id AND k.is_active = true
                ), 0)::BIGINT AS active_key_count,
                COALESCE((
                    SELECT jsonb_agg(DISTINCT perms.perm)
                    FROM (
                        SELECT jsonb_array_elements_text(k.permissions) AS perm
                        FROM api_keys k
                        WHERE k.user_id = u.id AND k.is_active = true
                    ) perms
                ), '[]'::jsonb) AS permissions
            FROM users u
            WHERE 1=1
            "#,
        );

        if let Some(q) = query.map(str::trim).filter(|q| !q.is_empty()) {
            let pattern = format!("%{}%", q);
            qb.push(" AND (u.email ILIKE ")
                .push_bind(pattern.clone())
                .push(" OR u.full_name ILIKE ")
                .push_bind(pattern)
                .push(")");
        }

        if let Some(t) = tier.map(str::trim).filter(|t| !t.is_empty()) {
            qb.push(" AND u.tier = ").push_bind(t);
        }

        match state.map(|s| s.trim().to_ascii_lowercase()) {
            Some(s) if s == "active" => {
                qb.push(" AND (u.locked_until IS NULL OR u.locked_until <= NOW())");
            }
            Some(s) if s == "locked" => {
                qb.push(" AND (u.locked_until IS NOT NULL AND u.locked_until > NOW())");
            }
            _ => {}
        }

        qb.push(" ORDER BY u.created_at DESC LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(offset);

        qb.build_query_as::<AdminUserRecord>()
            .fetch_all(&self.pool)
            .await
    }

    pub async fn count_users(
        &self,
        query: Option<&str>,
        tier: Option<&str>,
        state: Option<&str>,
    ) -> Result<i64, Error> {
        let mut qb: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT COUNT(*)::BIGINT FROM users u WHERE 1=1");

        if let Some(q) = query.map(str::trim).filter(|q| !q.is_empty()) {
            let pattern = format!("%{}%", q);
            qb.push(" AND (u.email ILIKE ")
                .push_bind(pattern.clone())
                .push(" OR u.full_name ILIKE ")
                .push_bind(pattern)
                .push(")");
        }

        if let Some(t) = tier.map(str::trim).filter(|t| !t.is_empty()) {
            qb.push(" AND u.tier = ").push_bind(t);
        }

        match state.map(|s| s.trim().to_ascii_lowercase()) {
            Some(s) if s == "active" => {
                qb.push(" AND (u.locked_until IS NULL OR u.locked_until <= NOW())");
            }
            Some(s) if s == "locked" => {
                qb.push(" AND (u.locked_until IS NOT NULL AND u.locked_until > NOW())");
            }
            _ => {}
        }

        qb.build_query_scalar::<i64>().fetch_one(&self.pool).await
    }

    pub async fn set_user_lock_state(
        &self,
        user_id: Uuid,
        locked_until: Option<DateTime<Utc>>,
    ) -> Result<Option<(Uuid, Option<DateTime<Utc>>)>, Error> {
        sqlx::query_as::<_, (Uuid, Option<DateTime<Utc>>)>(
            r#"
            UPDATE users
            SET
                locked_until = $2,
                failed_login_attempts = CASE
                    WHEN $2 IS NULL THEN 0
                    ELSE failed_login_attempts
                END,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, locked_until
            "#,
        )
        .bind(user_id)
        .bind(locked_until)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn set_user_tier(
        &self,
        user_id: Uuid,
        tier: &str,
    ) -> Result<Option<(Uuid, String)>, Error> {
        let mut tx = self.pool.begin().await?;

        let updated = sqlx::query_as::<_, (Uuid, String)>(
            r#"
            UPDATE users
            SET tier = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, tier
            "#,
        )
        .bind(user_id)
        .bind(tier)
        .fetch_optional(&mut *tx)
        .await?;

        if updated.is_some() {
            sqlx::query("UPDATE api_keys SET tier = $2 WHERE user_id = $1")
                .bind(user_id)
                .bind(tier)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(updated)
    }

    pub async fn user_exists(&self, user_id: Uuid) -> Result<bool, Error> {
        let exists =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*)::BIGINT FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_one(&self.pool)
                .await?;

        Ok(exists > 0)
    }

    pub async fn set_user_roles_and_permissions(
        &self,
        user_id: Uuid,
        roles: &[String],
        permissions: &[String],
    ) -> Result<Option<i64>, Error> {
        let mut tx = self.pool.begin().await?;

        let exists =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*)::BIGINT FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_one(&mut *tx)
                .await?;

        if exists == 0 {
            tx.rollback().await?;
            return Ok(None);
        }

        let roles_json = serde_json::json!(roles);
        let permissions_json = serde_json::json!(permissions);

        sqlx::query(
            r#"
            INSERT INTO user_profiles (user_id, preferences, created_at, updated_at)
            VALUES (
                $1,
                jsonb_build_object(
                    'admin_roles', $2::jsonb,
                    'admin_permissions', $3::jsonb
                ),
                NOW(),
                NOW()
            )
            ON CONFLICT (user_id)
            DO UPDATE SET
                preferences = jsonb_set(
                    jsonb_set(COALESCE(user_profiles.preferences, '{}'::jsonb), '{admin_roles}', $2::jsonb, true),
                    '{admin_permissions}',
                    $3::jsonb,
                    true
                ),
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(roles_json)
        .bind(permissions_json)
        .execute(&mut *tx)
        .await?;

        let updated_keys = sqlx::query(
            "UPDATE api_keys SET permissions = $2 WHERE user_id = $1 AND is_active = true",
        )
        .bind(user_id)
        .bind(serde_json::json!(permissions))
        .execute(&mut *tx)
        .await?
        .rows_affected() as i64;

        tx.commit().await?;

        Ok(Some(updated_keys))
    }

    pub async fn get_user_tier(&self, user_id: Uuid) -> Result<Option<String>, Error> {
        sqlx::query_scalar::<_, String>("SELECT tier FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn get_effective_permissions(&self, user_id: Uuid) -> Result<Vec<String>, Error> {
        let mut permissions: BTreeSet<String> = BTreeSet::new();

        let profile_permissions = sqlx::query_scalar::<_, Option<Value>>(
            "SELECT preferences -> 'admin_permissions' FROM user_profiles WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(Some(value)) = profile_permissions {
            for permission in parse_permissions_value(&value) {
                permissions.insert(permission);
            }
        }

        let key_permissions = sqlx::query_scalar::<_, Value>(
            "SELECT permissions FROM api_keys WHERE user_id = $1 AND is_active = true",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        for value in key_permissions {
            for permission in parse_permissions_value(&value) {
                permissions.insert(permission);
            }
        }

        Ok(permissions.into_iter().collect())
    }

    pub async fn list_api_keys(
        &self,
        query: Option<&str>,
        user_id: Option<Uuid>,
        active_only: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AdminApiKeyRecord>, Error> {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT
                k.id,
                k.name,
                k.key_prefix,
                k.user_id,
                u.email AS user_email,
                k.tier,
                k.permissions,
                k.consciousness_level,
                k.rate_limit,
                k.created_at,
                k.expires_at,
                k.last_used,
                k.is_active
            FROM api_keys k
            INNER JOIN users u ON u.id = k.user_id
            WHERE 1=1
            "#,
        );

        if let Some(uid) = user_id {
            qb.push(" AND k.user_id = ").push_bind(uid);
        }

        if active_only {
            qb.push(" AND k.is_active = true");
        }

        if let Some(q) = query.map(str::trim).filter(|q| !q.is_empty()) {
            let pattern = format!("%{}%", q);
            qb.push(" AND (u.email ILIKE ")
                .push_bind(pattern.clone())
                .push(" OR k.id::TEXT ILIKE ")
                .push_bind(pattern.clone())
                .push(" OR k.user_id::TEXT ILIKE ")
                .push_bind(pattern.clone())
                .push(" OR k.name ILIKE ")
                .push_bind(pattern)
                .push(")");
        }

        qb.push(" ORDER BY k.created_at DESC LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(offset);

        qb.build_query_as::<AdminApiKeyRecord>()
            .fetch_all(&self.pool)
            .await
    }

    pub async fn count_api_keys(
        &self,
        query: Option<&str>,
        user_id: Option<Uuid>,
        active_only: bool,
    ) -> Result<i64, Error> {
        let mut qb: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT COUNT(*)::BIGINT FROM api_keys k INNER JOIN users u ON u.id = k.user_id WHERE 1=1");

        if let Some(uid) = user_id {
            qb.push(" AND k.user_id = ").push_bind(uid);
        }

        if active_only {
            qb.push(" AND k.is_active = true");
        }

        if let Some(q) = query.map(str::trim).filter(|q| !q.is_empty()) {
            let pattern = format!("%{}%", q);
            qb.push(" AND (u.email ILIKE ")
                .push_bind(pattern.clone())
                .push(" OR k.id::TEXT ILIKE ")
                .push_bind(pattern.clone())
                .push(" OR k.user_id::TEXT ILIKE ")
                .push_bind(pattern.clone())
                .push(" OR k.name ILIKE ")
                .push_bind(pattern)
                .push(")");
        }

        qb.build_query_scalar::<i64>().fetch_one(&self.pool).await
    }

    pub async fn get_api_key(&self, key_id: Uuid) -> Result<Option<AdminApiKeyRecord>, Error> {
        sqlx::query_as::<_, AdminApiKeyRecord>(
            r#"
            SELECT
                k.id,
                k.name,
                k.key_prefix,
                k.user_id,
                u.email AS user_email,
                k.tier,
                k.permissions,
                k.consciousness_level,
                k.rate_limit,
                k.created_at,
                k.expires_at,
                k.last_used,
                k.is_active
            FROM api_keys k
            INNER JOIN users u ON u.id = k.user_id
            WHERE k.id = $1
            "#,
        )
        .bind(key_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create_api_key(
        &self,
        new_key: NewApiKeyRecord,
    ) -> Result<AdminApiKeyRecord, Error> {
        let key_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO api_keys (
                key_hash,
                name,
                key_prefix,
                user_id,
                tier,
                permissions,
                consciousness_level,
                rate_limit,
                expires_at,
                is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true)
            RETURNING id
            "#,
        )
        .bind(new_key.key_hash)
        .bind(new_key.name)
        .bind(new_key.key_prefix)
        .bind(new_key.user_id)
        .bind(new_key.tier)
        .bind(new_key.permissions)
        .bind(new_key.consciousness_level)
        .bind(new_key.rate_limit)
        .bind(new_key.expires_at)
        .fetch_one(&self.pool)
        .await?;

        self.get_api_key(key_id)
            .await?
            .ok_or_else(|| Error::RowNotFound)
    }

    pub async fn revoke_api_key(&self, key_id: Uuid) -> Result<bool, Error> {
        let affected = sqlx::query("UPDATE api_keys SET is_active = false WHERE id = $1")
            .bind(key_id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        Ok(affected > 0)
    }

    pub async fn rotate_api_key(
        &self,
        key_id: Uuid,
        new_key_hash: &str,
        new_key_prefix: &str,
    ) -> Result<Option<AdminApiKeyRecord>, Error> {
        let mut tx = self.pool.begin().await?;

        let existing = sqlx::query_as::<_, ExistingApiKeyRecord>(
            r#"
            SELECT user_id, name, tier, permissions, consciousness_level, rate_limit, expires_at
            FROM api_keys
            WHERE id = $1 AND is_active = true
            "#,
        )
        .bind(key_id)
        .fetch_optional(&mut *tx)
        .await?;

        let Some(existing) = existing else {
            tx.rollback().await?;
            return Ok(None);
        };

        sqlx::query("UPDATE api_keys SET is_active = false WHERE id = $1")
            .bind(key_id)
            .execute(&mut *tx)
            .await?;

        let new_key_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO api_keys (
                key_hash,
                name,
                key_prefix,
                user_id,
                tier,
                permissions,
                consciousness_level,
                rate_limit,
                expires_at,
                is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true)
            RETURNING id
            "#,
        )
        .bind(new_key_hash)
        .bind(existing.name)
        .bind(new_key_prefix)
        .bind(existing.user_id)
        .bind(existing.tier)
        .bind(existing.permissions)
        .bind(existing.consciousness_level)
        .bind(existing.rate_limit)
        .bind(existing.expires_at)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        self.get_api_key(new_key_id).await
    }

    pub async fn list_history_sync_users(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<HistorySyncUserRecord>, Error> {
        sqlx::query_as::<_, HistorySyncUserRecord>(
            r#"
            SELECT
                u.id AS user_id,
                u.email,
                COALESCE(r.readings_count, 0)::BIGINT AS readings_count,
                COALESCE(l.usage_count, 0)::BIGINT AS usage_events_count,
                r.last_reading_at,
                l.last_event_at,
                (COALESCE(l.usage_count, 0) - COALESCE(r.readings_count, 0))::BIGINT AS drift_count
            FROM users u
            LEFT JOIN (
                SELECT user_id, COUNT(*)::BIGINT AS readings_count, MAX(created_at) AS last_reading_at
                FROM readings
                GROUP BY user_id
            ) r ON r.user_id = u.id
            LEFT JOIN (
                SELECT user_id, COUNT(*)::BIGINT AS usage_count, MAX(created_at) AS last_event_at
                FROM usage_logs
                GROUP BY user_id
            ) l ON l.user_id = u.id
            ORDER BY ABS(COALESCE(l.usage_count, 0) - COALESCE(r.readings_count, 0)) DESC, u.created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn count_history_sync_users(&self) -> Result<i64, Error> {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*)::BIGINT FROM users")
            .fetch_one(&self.pool)
            .await
    }

    pub async fn list_history_sync_devices(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<HistorySyncDeviceRecord>, Error> {
        sqlx::query_as::<_, HistorySyncDeviceRecord>(
            r#"
            SELECT
                k.id AS device_id,
                k.user_id,
                u.email AS user_email,
                k.tier,
                k.is_active,
                COALESCE(jsonb_array_length(k.permissions), 0)::INTEGER AS permission_count,
                k.last_used AS last_seen_at,
                k.created_at
            FROM api_keys k
            INNER JOIN users u ON u.id = k.user_id
            ORDER BY COALESCE(k.last_used, k.created_at) DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn count_history_sync_devices(&self) -> Result<i64, Error> {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*)::BIGINT FROM api_keys")
            .fetch_one(&self.pool)
            .await
    }

    pub async fn list_history_sync_events(
        &self,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<HistorySyncEventRecord>, Error> {
        sqlx::query_as::<_, HistorySyncEventRecord>(
            r#"
            SELECT
                l.id AS event_id,
                l.user_id,
                u.email AS user_email,
                l.engine_id,
                l.workflow_id,
                l.status,
                l.duration_ms,
                l.created_at AS occurred_at
            FROM usage_logs l
            INNER JOIN users u ON u.id = l.user_id
            WHERE ($1::TEXT IS NULL OR l.status = $1)
            ORDER BY l.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn count_history_sync_events(&self, status: Option<&str>) -> Result<i64, Error> {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)::BIGINT FROM usage_logs WHERE ($1::TEXT IS NULL OR status = $1)",
        )
        .bind(status)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn analytics_summary(
        &self,
        window_hours: i64,
    ) -> Result<AnalyticsSummaryRecord, Error> {
        sqlx::query_as::<_, AnalyticsSummaryRecord>(
            r#"
            SELECT
                COUNT(*)::BIGINT AS requests_total,
                COUNT(*) FILTER (WHERE status = 'success')::BIGINT AS success_total,
                COUNT(*) FILTER (WHERE status != 'success')::BIGINT AS failure_total,
                COALESCE(
                    ROUND(
                        ((COUNT(*) FILTER (WHERE status != 'success'))::NUMERIC * 100.0)
                        / NULLIF(COUNT(*)::NUMERIC, 0),
                        2
                    ),
                    0
                )::DOUBLE PRECISION AS error_rate_pct,
                COALESCE(PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY duration_ms), 0)::DOUBLE PRECISION AS p95_duration_ms,
                COALESCE(AVG(duration_ms), 0)::DOUBLE PRECISION AS avg_duration_ms,
                COUNT(DISTINCT user_id)::BIGINT AS active_users
            FROM usage_logs
            WHERE created_at >= NOW() - ($1 * INTERVAL '1 hour')
            "#,
        )
        .bind(window_hours)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn analytics_unique_keys(&self, window_hours: i64) -> Result<i64, Error> {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(DISTINCT id)::BIGINT
            FROM api_keys
            WHERE is_active = true
              AND last_used IS NOT NULL
              AND last_used >= NOW() - ($1 * INTERVAL '1 hour')
            "#,
        )
        .bind(window_hours)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn analytics_timeseries(
        &self,
        window_hours: i64,
        bucket: &str,
    ) -> Result<Vec<AnalyticsTimeseriesPointRecord>, Error> {
        let bucket_expr = match bucket {
            "day" => "date_trunc('day', created_at)",
            _ => "date_trunc('hour', created_at)",
        };

        let query = format!(
            r#"
            SELECT
                {bucket_expr} AS bucket_start,
                COUNT(*)::BIGINT AS request_count,
                COUNT(*) FILTER (WHERE status = 'success')::BIGINT AS success_count,
                COUNT(*) FILTER (WHERE status != 'success')::BIGINT AS failure_count,
                COALESCE(AVG(duration_ms), 0)::DOUBLE PRECISION AS avg_duration_ms
            FROM usage_logs
            WHERE created_at >= NOW() - ($1 * INTERVAL '1 hour')
            GROUP BY 1
            ORDER BY 1 ASC
            "#
        );

        sqlx::query_as::<_, AnalyticsTimeseriesPointRecord>(&query)
            .bind(window_hours)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn analytics_engine_breakdown(
        &self,
        window_hours: i64,
        limit: i64,
    ) -> Result<Vec<AnalyticsBreakdownRecord>, Error> {
        sqlx::query_as::<_, AnalyticsBreakdownRecord>(
            r#"
            SELECT
                COALESCE(engine_id, 'unknown') AS label,
                COUNT(*)::BIGINT AS request_count
            FROM usage_logs
            WHERE created_at >= NOW() - ($1 * INTERVAL '1 hour')
            GROUP BY 1
            ORDER BY request_count DESC
            LIMIT $2
            "#,
        )
        .bind(window_hours)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn analytics_workflow_breakdown(
        &self,
        window_hours: i64,
        limit: i64,
    ) -> Result<Vec<AnalyticsBreakdownRecord>, Error> {
        sqlx::query_as::<_, AnalyticsBreakdownRecord>(
            r#"
            SELECT
                COALESCE(workflow_id, '(none)') AS label,
                COUNT(*)::BIGINT AS request_count
            FROM usage_logs
            WHERE created_at >= NOW() - ($1 * INTERVAL '1 hour')
            GROUP BY 1
            ORDER BY request_count DESC
            LIMIT $2
            "#,
        )
        .bind(window_hours)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn analytics_top_consumers(
        &self,
        window_hours: i64,
        limit: i64,
    ) -> Result<Vec<AnalyticsTopConsumerRecord>, Error> {
        sqlx::query_as::<_, AnalyticsTopConsumerRecord>(
            r#"
            SELECT
                l.user_id,
                u.email AS user_email,
                COUNT(*)::BIGINT AS request_count,
                COUNT(*) FILTER (WHERE l.status != 'success')::BIGINT AS failure_count,
                COALESCE(AVG(l.duration_ms), 0)::DOUBLE PRECISION AS avg_duration_ms
            FROM usage_logs l
            INNER JOIN users u ON u.id = l.user_id
            WHERE l.created_at >= NOW() - ($1 * INTERVAL '1 hour')
            GROUP BY l.user_id, u.email
            ORDER BY request_count DESC
            LIMIT $2
            "#,
        )
        .bind(window_hours)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }
}

fn parse_permissions_value(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}
