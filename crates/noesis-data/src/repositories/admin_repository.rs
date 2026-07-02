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
    pub created_by_user_id: Option<Uuid>,
    pub tier: String,
    pub permissions: Value,
    pub consciousness_level: i32,
    pub rate_limit: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub rotated_from_key_id: Option<Uuid>,
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

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SystemWorkflowSnapshotRecord {
    pub workflow_id: String,
    pub request_count: i64,
    pub failure_count: i64,
    pub last_seen_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuditEventRecord {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub actor_user_id: Uuid,
    pub actor_email: String,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub result: String,
    pub duration_ms: i32,
    pub engine_id: Option<String>,
    pub workflow_id: Option<String>,
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
        match self
            .list_users_with_account_state(query, tier, state, limit, offset)
            .await
        {
            Ok(records) => Ok(records),
            Err(err) if missing_admin_schema_tables(&err) => {
                self.list_users_legacy(query, tier, state, limit, offset)
                    .await
            }
            Err(err) => Err(err),
        }
    }

    async fn list_users_with_account_state(
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
                COALESCE(uas.locked_until, u.locked_until) AS locked_until,
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
            LEFT JOIN user_account_state uas ON uas.user_id = u.id
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
                qb.push(
                    " AND (COALESCE(uas.locked_until, u.locked_until) IS NULL OR COALESCE(uas.locked_until, u.locked_until) <= NOW())",
                );
            }
            Some(s) if s == "locked" => {
                qb.push(
                    " AND (COALESCE(uas.locked_until, u.locked_until) IS NOT NULL AND COALESCE(uas.locked_until, u.locked_until) > NOW())",
                );
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

    async fn list_users_legacy(
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
        match self
            .count_users_with_account_state(query, tier, state)
            .await
        {
            Ok(count) => Ok(count),
            Err(err) if missing_admin_schema_tables(&err) => {
                self.count_users_legacy(query, tier, state).await
            }
            Err(err) => Err(err),
        }
    }

    async fn count_users_with_account_state(
        &self,
        query: Option<&str>,
        tier: Option<&str>,
        state: Option<&str>,
    ) -> Result<i64, Error> {
        let mut qb: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT COUNT(*)::BIGINT FROM users u LEFT JOIN user_account_state uas ON uas.user_id = u.id WHERE 1=1");

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
                qb.push(
                    " AND (COALESCE(uas.locked_until, u.locked_until) IS NULL OR COALESCE(uas.locked_until, u.locked_until) <= NOW())",
                );
            }
            Some(s) if s == "locked" => {
                qb.push(
                    " AND (COALESCE(uas.locked_until, u.locked_until) IS NOT NULL AND COALESCE(uas.locked_until, u.locked_until) > NOW())",
                );
            }
            _ => {}
        }

        qb.build_query_scalar::<i64>().fetch_one(&self.pool).await
    }

    async fn count_users_legacy(
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
        match self
            .set_user_lock_state_with_account_state(user_id, locked_until)
            .await
        {
            Ok(result) => Ok(result),
            Err(err) if missing_admin_schema_tables(&err) => {
                self.set_user_lock_state_legacy(user_id, locked_until).await
            }
            Err(err) => Err(err),
        }
    }

    async fn set_user_lock_state_with_account_state(
        &self,
        user_id: Uuid,
        locked_until: Option<DateTime<Utc>>,
    ) -> Result<Option<(Uuid, Option<DateTime<Utc>>)>, Error> {
        let mut tx = self.pool.begin().await?;

        let updated = sqlx::query_as::<_, (Uuid, Option<DateTime<Utc>>)>(
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
        .fetch_optional(&mut *tx)
        .await?;

        let Some((updated_user_id, updated_locked_until)) = updated else {
            tx.rollback().await?;
            return Ok(None);
        };

        let state = if updated_locked_until.is_some() {
            "locked"
        } else {
            "active"
        };

        sqlx::query(
            r#"
            INSERT INTO user_account_state (user_id, state, locked_until, updated_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (user_id)
            DO UPDATE SET
                state = EXCLUDED.state,
                locked_until = EXCLUDED.locked_until,
                updated_at = NOW()
            "#,
        )
        .bind(updated_user_id)
        .bind(state)
        .bind(updated_locked_until)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(Some((updated_user_id, updated_locked_until)))
    }

    async fn set_user_lock_state_legacy(
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

            if let Err(err) =
                sync_user_active_plan_subscription(&mut tx, user_id, tier, Utc::now()).await
            {
                if !missing_plan_billing_schema(&err) {
                    return Err(err);
                }
            }
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
        match self
            .set_user_roles_and_permissions_with_table(user_id, roles, permissions)
            .await
        {
            Ok(result) => Ok(result),
            Err(err) if missing_admin_schema_tables(&err) => {
                self.set_user_roles_and_permissions_legacy(user_id, roles, permissions)
                    .await
            }
            Err(err) => Err(err),
        }
    }

    async fn set_user_roles_and_permissions_with_table(
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

        sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        if !roles.is_empty() {
            sqlx::query(
                r#"
                INSERT INTO user_roles (user_id, role)
                SELECT $1, role
                FROM UNNEST($2::TEXT[]) AS role
                "#,
            )
            .bind(user_id)
            .bind(roles)
            .execute(&mut *tx)
            .await?;
        }

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

    async fn set_user_roles_and_permissions_legacy(
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

    pub async fn replace_user_roles_from_cloudflare(
        &self,
        user_id: Uuid,
        roles: &[String],
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        for role in roles {
            sqlx::query(
                r#"
                INSERT INTO user_roles (user_id, role)
                VALUES ($1, $2)
                ON CONFLICT (user_id, role) DO NOTHING
                "#,
            )
            .bind(user_id)
            .bind(role)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_user_tier(&self, user_id: Uuid) -> Result<Option<String>, Error> {
        sqlx::query_scalar::<_, String>("SELECT tier FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn get_effective_permissions(&self, user_id: Uuid) -> Result<Vec<String>, Error> {
        match self
            .get_effective_permissions_with_roles_table(user_id)
            .await
        {
            Ok(permissions) => Ok(permissions),
            Err(err) if missing_admin_schema_tables(&err) => {
                self.get_effective_permissions_legacy(user_id).await
            }
            Err(err) => Err(err),
        }
    }

    async fn get_effective_permissions_with_roles_table(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<String>, Error> {
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

        let role_rows =
            sqlx::query_scalar::<_, String>("SELECT role FROM user_roles WHERE user_id = $1")
                .bind(user_id)
                .fetch_all(&self.pool)
                .await?;

        for role in role_rows {
            for permission in permissions_for_admin_role(&role) {
                permissions.insert(permission.to_string());
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

    async fn get_effective_permissions_legacy(&self, user_id: Uuid) -> Result<Vec<String>, Error> {
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

        let query_result = qb
            .build_query_as::<AdminApiKeyRecord>()
            .fetch_all(&self.pool)
            .await;
        match query_result {
            Ok(rows) => Ok(rows),
            Err(err) if missing_api_keys_optional_columns(&err) => {
                self.list_api_keys_legacy(query, user_id, active_only, limit, offset)
                    .await
            }
            Err(err) => Err(err),
        }
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

        let count_result = qb.build_query_scalar::<i64>().fetch_one(&self.pool).await;
        match count_result {
            Ok(total) => Ok(total),
            Err(err) if missing_api_keys_optional_columns(&err) => {
                self.count_api_keys_legacy(query, user_id, active_only)
                    .await
            }
            Err(err) => Err(err),
        }
    }

    pub async fn get_api_key(&self, key_id: Uuid) -> Result<Option<AdminApiKeyRecord>, Error> {
        let key_result = sqlx::query_as::<_, AdminApiKeyRecord>(
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
        .await;

        match key_result {
            Ok(key) => Ok(key),
            Err(err) if missing_api_keys_optional_columns(&err) => {
                self.get_api_key_legacy(key_id).await
            }
            Err(err) => Err(err),
        }
    }

    pub async fn create_api_key(
        &self,
        new_key: NewApiKeyRecord,
    ) -> Result<AdminApiKeyRecord, Error> {
        match self.create_api_key_with_events(new_key.clone()).await {
            Ok(record) => Ok(record),
            Err(err) if missing_api_key_lifecycle_schema_or_events(&err) => {
                self.create_api_key_legacy(new_key).await
            }
            Err(err) => Err(err),
        }
    }

    async fn create_api_key_with_events(
        &self,
        new_key: NewApiKeyRecord,
    ) -> Result<AdminApiKeyRecord, Error> {
        let NewApiKeyRecord {
            key_hash,
            name,
            key_prefix,
            user_id,
            created_by_user_id,
            tier,
            permissions,
            consciousness_level,
            rate_limit,
            expires_at,
            rotated_from_key_id,
        } = new_key;
        let event_name = name.clone();
        let event_key_prefix = key_prefix.clone();
        let event_rotated_from_key_id = rotated_from_key_id;

        let mut tx = self.pool.begin().await?;

        let key_id_result = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO api_keys (
                key_hash,
                name,
                key_prefix,
                user_id,
                created_by_user_id,
                tier,
                permissions,
                consciousness_level,
                rate_limit,
                expires_at,
                rotated_from_key_id,
                is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, true)
            RETURNING id
            "#,
        )
        .bind(key_hash.clone())
        .bind(name)
        .bind(key_prefix)
        .bind(user_id)
        .bind(created_by_user_id)
        .bind(tier.clone())
        .bind(permissions.clone())
        .bind(consciousness_level)
        .bind(rate_limit)
        .bind(expires_at)
        .bind(rotated_from_key_id)
        .fetch_one(&mut *tx)
        .await;

        let key_id = match key_id_result {
            Ok(id) => id,
            Err(err) => {
                tx.rollback().await?;
                return Err(err);
            }
        };

        sqlx::query(
            r#"
            INSERT INTO api_key_events (key_id, actor_user_id, event_type, metadata, created_at)
            VALUES ($1, $2, 'created', $3, NOW())
            "#,
        )
        .bind(key_id)
        .bind(created_by_user_id)
        .bind(serde_json::json!({
            "name": event_name,
            "key_prefix": event_key_prefix,
            "rotated_from_key_id": event_rotated_from_key_id,
        }))
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        self.get_api_key(key_id)
            .await?
            .ok_or_else(|| Error::RowNotFound)
    }

    async fn create_api_key_legacy(
        &self,
        new_key: NewApiKeyRecord,
    ) -> Result<AdminApiKeyRecord, Error> {
        let NewApiKeyRecord {
            key_hash,
            name,
            key_prefix,
            user_id,
            created_by_user_id: _,
            tier,
            permissions,
            consciousness_level,
            rate_limit,
            expires_at,
            rotated_from_key_id: _,
        } = new_key;

        let key_id_result = sqlx::query_scalar::<_, Uuid>(
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
        .bind(key_hash.clone())
        .bind(name)
        .bind(key_prefix)
        .bind(user_id)
        .bind(tier.clone())
        .bind(permissions.clone())
        .bind(consciousness_level)
        .bind(rate_limit)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await;

        let key_id = match key_id_result {
            Ok(id) => id,
            Err(err) if missing_api_keys_optional_columns(&err) => {
                sqlx::query_scalar::<_, Uuid>(
                    r#"
                    INSERT INTO api_keys (
                        key_hash,
                        user_id,
                        tier,
                        permissions,
                        consciousness_level,
                        rate_limit,
                        expires_at,
                        is_active
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, true)
                    RETURNING id
                    "#,
                )
                .bind(key_hash)
                .bind(user_id)
                .bind(tier)
                .bind(permissions)
                .bind(consciousness_level)
                .bind(rate_limit)
                .bind(expires_at)
                .fetch_one(&self.pool)
                .await?
            }
            Err(err) => return Err(err),
        };

        self.get_api_key(key_id)
            .await?
            .ok_or_else(|| Error::RowNotFound)
    }

    pub async fn revoke_api_key(
        &self,
        key_id: Uuid,
        actor_user_id: Option<Uuid>,
    ) -> Result<bool, Error> {
        match self.revoke_api_key_with_events(key_id, actor_user_id).await {
            Ok(revoked) => Ok(revoked),
            Err(err) if missing_api_key_lifecycle_schema_or_events(&err) => {
                self.revoke_api_key_legacy(key_id).await
            }
            Err(err) => Err(err),
        }
    }

    async fn revoke_api_key_with_events(
        &self,
        key_id: Uuid,
        actor_user_id: Option<Uuid>,
    ) -> Result<bool, Error> {
        let mut tx = self.pool.begin().await?;

        let affected = sqlx::query(
            "UPDATE api_keys SET is_active = false, revoked_at = NOW(), revoked_by_user_id = $2 WHERE id = $1 AND is_active = true",
        )
        .bind(key_id)
        .bind(actor_user_id)
        .execute(&mut *tx)
        .await?
        .rows_affected();

        if affected == 0 {
            tx.rollback().await?;
            return Ok(false);
        }

        sqlx::query(
            r#"
            INSERT INTO api_key_events (key_id, actor_user_id, event_type, metadata, created_at)
            VALUES ($1, $2, 'revoked', '{}'::jsonb, NOW())
            "#,
        )
        .bind(key_id)
        .bind(actor_user_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(affected > 0)
    }

    async fn revoke_api_key_legacy(&self, key_id: Uuid) -> Result<bool, Error> {
        let affected = sqlx::query("UPDATE api_keys SET is_active = false WHERE id = $1")
            .bind(key_id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        Ok(affected > 0)
    }

    pub async fn delete_api_key(&self, key_id: Uuid) -> Result<bool, Error> {
        let affected = sqlx::query("DELETE FROM api_keys WHERE id = $1")
            .bind(key_id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        Ok(affected > 0)
    }

    /// Purge revoked API keys older than `days` days. Returns the number deleted.
    /// Falls back to `created_at` for legacy rows where `revoked_at` is NULL.
    pub async fn purge_revoked_api_keys(&self, days: i64) -> Result<u64, Error> {
        let affected = sqlx::query(
            "DELETE FROM api_keys \
             WHERE is_active = false \
               AND COALESCE(revoked_at, created_at) < NOW() - ($1 || ' days')::interval",
        )
        .bind(days)
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(affected)
    }

    pub async fn rotate_api_key(
        &self,
        key_id: Uuid,
        new_key_hash: &str,
        new_key_prefix: &str,
        actor_user_id: Option<Uuid>,
    ) -> Result<Option<AdminApiKeyRecord>, Error> {
        match self
            .rotate_api_key_with_events(key_id, new_key_hash, new_key_prefix, actor_user_id)
            .await
        {
            Ok(record) => Ok(record),
            Err(err) if missing_api_key_lifecycle_schema_or_events(&err) => {
                self.rotate_api_key_legacy(key_id, new_key_hash).await
            }
            Err(err) => Err(err),
        }
    }

    async fn rotate_api_key_with_events(
        &self,
        key_id: Uuid,
        new_key_hash: &str,
        new_key_prefix: &str,
        actor_user_id: Option<Uuid>,
    ) -> Result<Option<AdminApiKeyRecord>, Error> {
        let mut tx = self.pool.begin().await?;

        let existing_result = sqlx::query_as::<_, ExistingApiKeyRecord>(
            r#"
            SELECT user_id, name, tier, permissions, consciousness_level, rate_limit, expires_at
            FROM api_keys
            WHERE id = $1 AND is_active = true
            "#,
        )
        .bind(key_id)
        .fetch_optional(&mut *tx)
        .await;

        let existing = match existing_result {
            Ok(record) => record,
            Err(err) if missing_api_keys_optional_columns(&err) => {
                tx.rollback().await?;
                return self.rotate_api_key_legacy(key_id, new_key_hash).await;
            }
            Err(err) => {
                tx.rollback().await?;
                return Err(err);
            }
        };

        let Some(existing) = existing else {
            tx.rollback().await?;
            return Ok(None);
        };
        let replacement_name = existing.name.clone();

        sqlx::query(
            "UPDATE api_keys SET is_active = false, revoked_at = NOW(), revoked_by_user_id = $2 WHERE id = $1",
        )
            .bind(key_id)
            .bind(actor_user_id)
            .execute(&mut *tx)
            .await?;

        let new_key_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO api_keys (
                key_hash,
                name,
                key_prefix,
                user_id,
                created_by_user_id,
                tier,
                permissions,
                consciousness_level,
                rate_limit,
                expires_at,
                rotated_from_key_id,
                is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, true)
            RETURNING id
            "#,
        )
        .bind(new_key_hash)
        .bind(existing.name)
        .bind(new_key_prefix)
        .bind(existing.user_id)
        .bind(actor_user_id)
        .bind(existing.tier)
        .bind(existing.permissions)
        .bind(existing.consciousness_level)
        .bind(existing.rate_limit)
        .bind(existing.expires_at)
        .bind(key_id)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO api_key_events (key_id, actor_user_id, event_type, metadata, created_at)
            VALUES ($1, $2, 'rotated', $3, NOW())
            "#,
        )
        .bind(key_id)
        .bind(actor_user_id)
        .bind(serde_json::json!({
            "replacement_key_id": new_key_id,
            "replacement_key_prefix": new_key_prefix,
        }))
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO api_key_events (key_id, actor_user_id, event_type, metadata, created_at)
            VALUES ($1, $2, 'created', $3, NOW())
            "#,
        )
        .bind(new_key_id)
        .bind(actor_user_id)
        .bind(serde_json::json!({
            "name": replacement_name,
            "key_prefix": new_key_prefix,
            "rotated_from_key_id": key_id,
        }))
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        self.get_api_key(new_key_id).await
    }

    async fn list_api_keys_legacy(
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
                NULL::TEXT AS name,
                NULL::TEXT AS key_prefix,
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

    async fn count_api_keys_legacy(
        &self,
        query: Option<&str>,
        user_id: Option<Uuid>,
        active_only: bool,
    ) -> Result<i64, Error> {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT COUNT(*)::BIGINT FROM api_keys k INNER JOIN users u ON u.id = k.user_id WHERE 1=1",
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
                .push_bind(pattern)
                .push(")");
        }

        qb.build_query_scalar::<i64>().fetch_one(&self.pool).await
    }

    async fn get_api_key_legacy(&self, key_id: Uuid) -> Result<Option<AdminApiKeyRecord>, Error> {
        sqlx::query_as::<_, AdminApiKeyRecord>(
            r#"
            SELECT
                k.id,
                NULL::TEXT AS name,
                NULL::TEXT AS key_prefix,
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

    async fn rotate_api_key_legacy(
        &self,
        key_id: Uuid,
        new_key_hash: &str,
    ) -> Result<Option<AdminApiKeyRecord>, Error> {
        let mut tx = self.pool.begin().await?;

        let existing = sqlx::query_as::<_, (Uuid, String, Value, i32, i32, Option<DateTime<Utc>>)>(
            r#"
            SELECT user_id, tier, permissions, consciousness_level, rate_limit, expires_at
            FROM api_keys
            WHERE id = $1 AND is_active = true
            "#,
        )
        .bind(key_id)
        .fetch_optional(&mut *tx)
        .await?;

        let Some((user_id, tier, permissions, consciousness_level, rate_limit, expires_at)) =
            existing
        else {
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
                user_id,
                tier,
                permissions,
                consciousness_level,
                rate_limit,
                expires_at,
                is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, true)
            RETURNING id
            "#,
        )
        .bind(new_key_hash)
        .bind(user_id)
        .bind(tier)
        .bind(permissions)
        .bind(consciousness_level)
        .bind(rate_limit)
        .bind(expires_at)
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

    pub async fn ping(&self) -> Result<bool, Error> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| true)
    }

    pub async fn system_workflow_snapshots(
        &self,
        window_hours: i64,
    ) -> Result<Vec<SystemWorkflowSnapshotRecord>, Error> {
        sqlx::query_as::<_, SystemWorkflowSnapshotRecord>(
            r#"
            SELECT
                l.workflow_id,
                COUNT(*)::BIGINT AS request_count,
                COUNT(*) FILTER (WHERE l.status != 'success')::BIGINT AS failure_count,
                MAX(l.created_at) AS last_seen_at
            FROM usage_logs l
            WHERE l.workflow_id IS NOT NULL
              AND l.created_at >= NOW() - ($1 * INTERVAL '1 hour')
            GROUP BY l.workflow_id
            ORDER BY request_count DESC, l.workflow_id ASC
            "#,
        )
        .bind(window_hours)
        .fetch_all(&self.pool)
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list_audit_events(
        &self,
        actor: Option<&str>,
        action: Option<&str>,
        result: Option<&str>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditEventRecord>, Error> {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT
                l.id AS event_id,
                l.created_at AS occurred_at,
                l.user_id AS actor_user_id,
                u.email AS actor_email,
                CASE
                    WHEN l.workflow_id IS NOT NULL THEN 'workflow.execute'
                    WHEN l.engine_id IS NOT NULL THEN 'engine.calculate'
                    ELSE 'request.execute'
                END AS action,
                CASE
                    WHEN l.workflow_id IS NOT NULL THEN 'workflow'
                    WHEN l.engine_id IS NOT NULL THEN 'engine'
                    ELSE 'user'
                END AS target_type,
                COALESCE(l.workflow_id, l.engine_id, l.user_id::TEXT) AS target_id,
                l.status AS result,
                l.duration_ms,
                l.engine_id,
                l.workflow_id
            FROM usage_logs l
            INNER JOIN users u ON u.id = l.user_id
            WHERE 1=1
            "#,
        );

        if let Some(actor_filter) = actor.map(str::trim).filter(|value| !value.is_empty()) {
            let pattern = format!("%{}%", actor_filter);
            qb.push(" AND (u.email ILIKE ")
                .push_bind(pattern.clone())
                .push(" OR l.user_id::TEXT ILIKE ")
                .push_bind(pattern)
                .push(")");
        }

        if let Some(action_filter) = action.map(str::trim).filter(|value| !value.is_empty()) {
            qb.push(" AND (CASE WHEN l.workflow_id IS NOT NULL THEN 'workflow.execute' WHEN l.engine_id IS NOT NULL THEN 'engine.calculate' ELSE 'request.execute' END = ")
                .push_bind(action_filter)
                .push(")");
        }

        if let Some(result_filter) = result.map(str::trim).filter(|value| !value.is_empty()) {
            qb.push(" AND l.status = ").push_bind(result_filter);
        }

        if let Some(from_ts) = from {
            qb.push(" AND l.created_at >= ").push_bind(from_ts);
        }

        if let Some(to_ts) = to {
            qb.push(" AND l.created_at < ").push_bind(to_ts);
        }

        qb.push(" ORDER BY l.created_at DESC LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(offset);

        qb.build_query_as::<AuditEventRecord>()
            .fetch_all(&self.pool)
            .await
    }

    pub async fn count_audit_events(
        &self,
        actor: Option<&str>,
        action: Option<&str>,
        result: Option<&str>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Result<i64, Error> {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT COUNT(*)::BIGINT FROM usage_logs l INNER JOIN users u ON u.id = l.user_id WHERE 1=1",
        );

        if let Some(actor_filter) = actor.map(str::trim).filter(|value| !value.is_empty()) {
            let pattern = format!("%{}%", actor_filter);
            qb.push(" AND (u.email ILIKE ")
                .push_bind(pattern.clone())
                .push(" OR l.user_id::TEXT ILIKE ")
                .push_bind(pattern)
                .push(")");
        }

        if let Some(action_filter) = action.map(str::trim).filter(|value| !value.is_empty()) {
            qb.push(" AND (CASE WHEN l.workflow_id IS NOT NULL THEN 'workflow.execute' WHEN l.engine_id IS NOT NULL THEN 'engine.calculate' ELSE 'request.execute' END = ")
                .push_bind(action_filter)
                .push(")");
        }

        if let Some(result_filter) = result.map(str::trim).filter(|value| !value.is_empty()) {
            qb.push(" AND l.status = ").push_bind(result_filter);
        }

        if let Some(from_ts) = from {
            qb.push(" AND l.created_at >= ").push_bind(from_ts);
        }

        if let Some(to_ts) = to {
            qb.push(" AND l.created_at < ").push_bind(to_ts);
        }

        qb.build_query_scalar::<i64>().fetch_one(&self.pool).await
    }

    pub async fn get_audit_event(&self, event_id: Uuid) -> Result<Option<AuditEventRecord>, Error> {
        sqlx::query_as::<_, AuditEventRecord>(
            r#"
            SELECT
                l.id AS event_id,
                l.created_at AS occurred_at,
                l.user_id AS actor_user_id,
                u.email AS actor_email,
                CASE
                    WHEN l.workflow_id IS NOT NULL THEN 'workflow.execute'
                    WHEN l.engine_id IS NOT NULL THEN 'engine.calculate'
                    ELSE 'request.execute'
                END AS action,
                CASE
                    WHEN l.workflow_id IS NOT NULL THEN 'workflow'
                    WHEN l.engine_id IS NOT NULL THEN 'engine'
                    ELSE 'user'
                END AS target_type,
                COALESCE(l.workflow_id, l.engine_id, l.user_id::TEXT) AS target_id,
                l.status AS result,
                l.duration_ms,
                l.engine_id,
                l.workflow_id
            FROM usage_logs l
            INNER JOIN users u ON u.id = l.user_id
            WHERE l.id = $1
            "#,
        )
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_audit_actions(&self) -> Result<Vec<String>, Error> {
        sqlx::query_scalar::<_, String>(
            r#"
            SELECT DISTINCT
                CASE
                    WHEN workflow_id IS NOT NULL THEN 'workflow.execute'
                    WHEN engine_id IS NOT NULL THEN 'engine.calculate'
                    ELSE 'request.execute'
                END AS action
            FROM usage_logs
            ORDER BY action ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }
}

fn missing_api_keys_optional_columns(err: &Error) -> bool {
    let Some(db_err) = err.as_database_error() else {
        return false;
    };

    if db_err.code().as_deref() != Some("42703") {
        return false;
    }

    let message = db_err.message().to_ascii_lowercase();
    message.contains("api_keys") && (message.contains("name") || message.contains("key_prefix"))
}

fn missing_admin_schema_tables(err: &Error) -> bool {
    let Some(db_err) = err.as_database_error() else {
        return false;
    };

    if db_err.code().as_deref() != Some("42P01") {
        return false;
    }

    let message = db_err.message().to_ascii_lowercase();
    message.contains("user_roles") || message.contains("user_account_state")
}

fn missing_api_key_lifecycle_schema_or_events(err: &Error) -> bool {
    let Some(db_err) = err.as_database_error() else {
        return false;
    };

    match db_err.code().as_deref() {
        Some("42P01") => {
            let message = db_err.message().to_ascii_lowercase();
            message.contains("api_key_events")
        }
        Some("42703") => {
            let message = db_err.message().to_ascii_lowercase();
            message.contains("created_by_user_id")
                || message.contains("revoked_at")
                || message.contains("revoked_by_user_id")
                || message.contains("revoked_reason")
                || message.contains("rotated_from_key_id")
        }
        _ => false,
    }
}

fn permissions_for_admin_role(role: &str) -> &'static [&'static str] {
    match role {
        "viewer" => &[
            "basic:access",
            "admin:analytics:read",
            "admin:system:read",
            "admin:audit:list",
        ],
        "support" => &[
            "basic:access",
            "admin:analytics:read",
            "admin:users:list",
            "admin:users:read",
            "admin:history-sync:read",
        ],
        "admin" => &[
            "basic:access",
            "admin:analytics:read",
            "admin:users:list",
            "admin:users:read",
            "admin:history-sync:read",
            "admin:keys:list",
            "admin:keys:create",
            "admin:keys:revoke",
            "admin:keys:rotate",
            "admin:users:tier:update",
        ],
        "platform-admin" => &[
            "basic:access",
            "admin:*",
            "admin:analytics:read",
            "admin:users:list",
            "admin:users:read",
            "admin:users:roles:update",
            "admin:history-sync:read",
            "admin:keys:list",
            "admin:keys:create",
            "admin:keys:revoke",
            "admin:keys:rotate",
            "admin:users:tier:update",
        ],
        "billing-admin" => &[
            "basic:access",
            "admin:billing:read",
            "admin:billing:subscriptions:cancel",
            "admin:billing:reconcile:trigger",
            "admin:analytics:read",
            "admin:users:list",
            "admin:users:read",
        ],
        _ => &["basic:access"],
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

async fn sync_user_active_plan_subscription(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    user_id: Uuid,
    tier: &str,
    effective_start: DateTime<Utc>,
) -> Result<(), Error> {
    let plan_code = normalize_plan_code(tier);
    let display_name = plan_display_name(tier);
    let provider_subscription_id = format!("internal:{user_id}:{plan_code}");

    let plan_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO plan_catalog (code, display_name, description)
        VALUES ($1, $2, $3)
        ON CONFLICT (code)
        DO UPDATE SET
            display_name = EXCLUDED.display_name,
            description = COALESCE(plan_catalog.description, EXCLUDED.description),
            is_active = true,
            updated_at = NOW()
        RETURNING id
        "#,
    )
    .bind(&plan_code)
    .bind(&display_name)
    .bind(format!("Canonical plan row for tier '{tier}'"))
    .fetch_one(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE billing_subscriptions
        SET
            status = 'canceled',
            canceled_at = COALESCE(canceled_at, NOW()),
            cancel_at_period_end = false,
            updated_at = NOW()
        WHERE user_id = $1
          AND status IN ('trialing', 'active', 'past_due')
          AND canceled_at IS NULL
        "#,
    )
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO billing_subscriptions (
            user_id,
            plan_id,
            provider,
            provider_customer_id,
            provider_subscription_id,
            status,
            cancel_at_period_end,
            current_period_start,
            current_period_end
        )
        VALUES ($1, $2, 'internal', NULL, $3, 'active', false, $4, NULL)
        "#,
    )
    .bind(user_id)
    .bind(plan_id)
    .bind(provider_subscription_id)
    .bind(effective_start)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

fn normalize_plan_code(tier: &str) -> String {
    let trimmed = tier.trim();
    if trimmed.is_empty() {
        "free".to_string()
    } else {
        trimmed.to_ascii_lowercase()
    }
}

fn plan_display_name(tier: &str) -> String {
    let trimmed = tier.trim();
    if trimmed.is_empty() {
        "Free".to_string()
    } else {
        let lower = trimmed.to_ascii_lowercase();
        let mut chars = lower.chars();
        match chars.next() {
            Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
            None => "Free".to_string(),
        }
    }
}

fn missing_plan_billing_schema(err: &Error) -> bool {
    let Some(db_err) = err.as_database_error() else {
        return false;
    };

    match db_err.code().as_deref() {
        Some("42P01") => {
            let message = db_err.message().to_ascii_lowercase();
            message.contains("plan_catalog")
                || message.contains("billing_subscriptions")
                || message.contains("user_active_plan_resolutions")
        }
        Some("42703") => {
            let message = db_err.message().to_ascii_lowercase();
            message.contains("plan_id")
                || message.contains("provider_subscription_id")
                || message.contains("plan_code")
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace crate dir")
            .parent()
            .expect("workspace root")
            .to_path_buf()
    }

    #[test]
    fn role_permissions_include_platform_admin_controls() {
        let perms = permissions_for_admin_role("platform-admin");
        assert!(perms.contains(&"admin:*"));
        assert!(perms.contains(&"admin:users:roles:update"));
        assert!(perms.contains(&"admin:keys:rotate"));
    }

    #[test]
    fn migration_010_exists_in_root_and_supabase() {
        let root_sql =
            fs::read_to_string(repo_root().join("migrations/010_user_roles_account_state.sql"))
                .expect("root migration 010");
        let supabase_sql = fs::read_to_string(
            repo_root().join("supabase/migrations/20260313000010_010_user_roles_account_state.sql"),
        )
        .expect("supabase migration 010");

        for sql in [&root_sql, &supabase_sql] {
            assert!(sql.contains("CREATE TABLE IF NOT EXISTS user_roles"));
            assert!(sql.contains("CREATE TABLE IF NOT EXISTS user_account_state"));
            assert!(sql.contains("INSERT INTO user_roles"));
            assert!(sql.contains("INSERT INTO user_account_state"));
        }
    }

    #[test]
    fn migration_011_exists_in_root_and_supabase() {
        let root_sql = fs::read_to_string(repo_root().join("migrations/011_api_key_events.sql"))
            .expect("root migration 011");
        let supabase_sql = fs::read_to_string(
            repo_root().join("supabase/migrations/20260313000011_011_api_key_events.sql"),
        )
        .expect("supabase migration 011");

        for sql in [&root_sql, &supabase_sql] {
            assert!(sql.contains("CREATE TABLE IF NOT EXISTS api_key_events"));
            assert!(sql.contains("ADD COLUMN IF NOT EXISTS created_by_user_id"));
            assert!(sql.contains("ADD COLUMN IF NOT EXISTS revoked_at"));
            assert!(sql.contains("ADD COLUMN IF NOT EXISTS rotated_from_key_id"));
            assert!(sql.contains("INSERT INTO api_key_events"));
        }
    }

    #[test]
    fn migration_012_and_partition_check_script_exist() {
        let root_sql =
            fs::read_to_string(repo_root().join("migrations/012_usage_partition_maintenance.sql"))
                .expect("root migration 012");
        let supabase_sql = fs::read_to_string(
            repo_root()
                .join("supabase/migrations/20260313000012_012_usage_partition_maintenance.sql"),
        )
        .expect("supabase migration 012");
        let script = fs::read_to_string(repo_root().join("scripts/check_usage_log_partitions.sh"))
            .expect("partition check script");

        for sql in [&root_sql, &supabase_sql] {
            assert!(sql.contains("CREATE OR REPLACE FUNCTION ensure_usage_log_partitions"));
            assert!(sql.contains("to_regclass"));
            assert!(sql.contains("CREATE TABLE %I PARTITION OF usage_logs"));
        }

        assert!(script.contains("ensure_usage_log_partitions"));
        assert!(script.contains("ALERT: usage_logs partition maintenance failed"));
    }
}
