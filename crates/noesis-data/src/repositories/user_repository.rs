use crate::models::user::{User, UserProfile};
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use sqlx::{Error, PgPool, Postgres};
use uuid::Uuid;

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        full_name: &str,
    ) -> Result<User, Error> {
        match self
            .create_user_with_active_plan(email, password_hash, full_name)
            .await
        {
            Ok(user) => Ok(user),
            Err(err) if missing_plan_billing_schema(&err) => {
                self.create_user_legacy(email, password_hash, full_name)
                    .await
            }
            Err(err) => Err(err),
        }
    }

    async fn create_user_with_active_plan(
        &self,
        email: &str,
        password_hash: &str,
        full_name: &str,
    ) -> Result<User, Error> {
        let mut tx = self.pool.begin().await?;
        let created_at = Utc::now();
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, email, password_hash, full_name, tier, consciousness_level, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(email)
        .bind(password_hash)
        .bind(full_name)
        .bind("free") // Default tier
        .bind(0)      // Default consciousness level
        .bind(created_at)
        .bind(created_at)
        .fetch_one(&mut *tx)
        .await?;

        sync_user_plan_subscription(&mut tx, user.id, &user.tier, created_at, "internal").await?;
        tx.commit().await?;

        Ok(user)
    }

    async fn create_user_legacy(
        &self,
        email: &str,
        password_hash: &str,
        full_name: &str,
    ) -> Result<User, Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, email, password_hash, full_name, tier, consciousness_level, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(email)
        .bind(password_hash)
        .bind(full_name)
        .bind("free")
        .bind(0)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, Error> {
        let primary = sqlx::query_as::<_, User>(
            r#"
            SELECT
                id,
                email,
                COALESCE(password_hash, '') AS password_hash,
                COALESCE(full_name, '') AS full_name,
                COALESCE(tier, 'free') AS tier,
                COALESCE(consciousness_level, 0) AS consciousness_level,
                COALESCE(experience_points, 0) AS experience_points,
                created_at,
                updated_at,
                reset_token,
                reset_token_expires_at,
                last_login_at,
                COALESCE(failed_login_attempts, 0) AS failed_login_attempts,
                locked_until
            FROM users
            WHERE lower(btrim(email)) = lower(btrim($1))
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await;

        let user = match primary {
            Ok(user) => user,
            Err(primary_err) => {
                let legacy = sqlx::query_as::<_, User>(
                    r#"
                    SELECT
                        id,
                        email,
                        password_hash::text AS password_hash,
                        ''::text AS full_name,
                        'Free'::text AS tier,
                        0::integer AS consciousness_level,
                        0::integer AS experience_points,
                        NOW() AS created_at,
                        NOW() AS updated_at,
                        NULL::text AS reset_token,
                        NULL::timestamptz AS reset_token_expires_at,
                        NULL::timestamptz AS last_login_at,
                        0::integer AS failed_login_attempts,
                        NULL::timestamptz AS locked_until
                    FROM users
                    WHERE lower(btrim(email)) = lower(btrim($1))
                    "#,
                )
                .bind(email)
                .fetch_optional(&self.pool)
                .await;

                match legacy {
                    Ok(user) => user,
                    Err(_) => return Err(primary_err),
                }
            }
        };

        Ok(user)
    }

    pub async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, Error> {
        let primary = sqlx::query_as::<_, User>(
            r#"
            SELECT
                id,
                email,
                COALESCE(password_hash, '') AS password_hash,
                COALESCE(full_name, '') AS full_name,
                COALESCE(tier, 'free') AS tier,
                COALESCE(consciousness_level, 0) AS consciousness_level,
                COALESCE(experience_points, 0) AS experience_points,
                created_at,
                updated_at,
                reset_token,
                reset_token_expires_at,
                last_login_at,
                COALESCE(failed_login_attempts, 0) AS failed_login_attempts,
                locked_until
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await;

        let user = match primary {
            Ok(user) => user,
            Err(primary_err) => {
                let legacy = sqlx::query_as::<_, User>(
                    r#"
                    SELECT
                        id,
                        email,
                        password_hash::text AS password_hash,
                        ''::text AS full_name,
                        'Free'::text AS tier,
                        0::integer AS consciousness_level,
                        0::integer AS experience_points,
                        NOW() AS created_at,
                        NOW() AS updated_at,
                        NULL::text AS reset_token,
                        NULL::timestamptz AS reset_token_expires_at,
                        NULL::timestamptz AS last_login_at,
                        0::integer AS failed_login_attempts,
                        NULL::timestamptz AS locked_until
                    FROM users
                    WHERE id = $1
                    "#,
                )
                .bind(id)
                .fetch_optional(&self.pool)
                .await;

                match legacy {
                    Ok(user) => user,
                    Err(_) => return Err(primary_err),
                }
            }
        };

        Ok(user)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_profile(
        &self,
        user_id: Uuid,
        birth_date: Option<NaiveDate>,
        birth_time: Option<NaiveTime>,
        birth_location_lat: Option<f64>,
        birth_location_lng: Option<f64>,
        birth_location_name: Option<String>,
        timezone: Option<String>,
        preferences: serde_json::Value,
    ) -> Result<UserProfile, Error> {
        let profile = sqlx::query_as::<_, UserProfile>(
            r#"
            INSERT INTO user_profiles (
                user_id, birth_date, birth_time, birth_location_lat, birth_location_lng, 
                birth_location_name, timezone, preferences, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING user_id, birth_date, birth_time,
                      CAST(birth_location_lat AS FLOAT8) AS birth_location_lat,
                      CAST(birth_location_lng AS FLOAT8) AS birth_location_lng,
                      birth_location_name, timezone, preferences, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(birth_date)
        .bind(birth_time)
        .bind(birth_location_lat)
        .bind(birth_location_lng)
        .bind(birth_location_name)
        .bind(timezone)
        .bind(preferences)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(profile)
    }

    pub async fn get_profile(&self, user_id: Uuid) -> Result<Option<UserProfile>, Error> {
        sqlx::query_as::<_, UserProfile>(
            r#"
            SELECT user_id, birth_date, birth_time,
                   CAST(birth_location_lat AS FLOAT8) AS birth_location_lat,
                   CAST(birth_location_lng AS FLOAT8) AS birth_location_lng,
                   birth_location_name, timezone, preferences, created_at, updated_at
            FROM user_profiles WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn resolve_active_plan_code(&self, user_id: Uuid) -> Result<Option<String>, Error> {
        match sqlx::query_scalar::<_, String>(
            "SELECT plan_code FROM user_active_plan_resolutions WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        {
            Ok(plan) => Ok(plan),
            Err(err) if missing_plan_billing_schema(&err) => sqlx::query_scalar::<_, String>(
                "SELECT COALESCE(NULLIF(LOWER(BTRIM(tier)), ''), 'free') FROM users WHERE id = $1",
            )
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await,
            Err(err) => Err(err),
        }
    }

    pub async fn update_user(
        &self,
        user_id: Uuid,
        full_name: Option<String>,
        email: Option<String>,
    ) -> Result<User, Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users 
            SET 
                full_name = COALESCE($2, full_name),
                email = COALESCE($3, email),
                updated_at = $4
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(full_name)
        .bind(email)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_profile(
        &self,
        user_id: Uuid,
        birth_date: Option<NaiveDate>,
        birth_time: Option<NaiveTime>,
        birth_location_lat: Option<f64>,
        birth_location_lng: Option<f64>,
        birth_location_name: Option<String>,
        timezone: Option<String>,
        preferences: Option<serde_json::Value>,
    ) -> Result<UserProfile, Error> {
        // We use COALESCE for optional updates.
        // Note: For JSONB, COALESCE works to replace the whole object if provided.
        // Merging JSONB would require jsonb_concat or similar. Here we assume full replacement of preferences field if provided.

        // We try to UPDATE. If no row exists, we should probably CREATE one?
        // Or assume profile exists. Let's assume profile exists or we handle that in logic.
        // Actually, upsert (INSERT ... ON CONFLICT DO UPDATE) is safer for 1:1 profiles.

        let profile = sqlx::query_as::<_, UserProfile>(
            r#"
            INSERT INTO user_profiles (
                user_id, birth_date, birth_time, birth_location_lat, birth_location_lng, 
                birth_location_name, timezone, preferences, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, COALESCE($8, '{}'::jsonb), $9, $9)
            ON CONFLICT (user_id) DO UPDATE SET
                birth_date = COALESCE(EXCLUDED.birth_date, user_profiles.birth_date),
                birth_time = COALESCE(EXCLUDED.birth_time, user_profiles.birth_time),
                birth_location_lat = COALESCE(EXCLUDED.birth_location_lat, user_profiles.birth_location_lat),
                birth_location_lng = COALESCE(EXCLUDED.birth_location_lng, user_profiles.birth_location_lng),
                birth_location_name = COALESCE(EXCLUDED.birth_location_name, user_profiles.birth_location_name),
                timezone = COALESCE(EXCLUDED.timezone, user_profiles.timezone),
                preferences = CASE WHEN $8 IS NOT NULL THEN $8 ELSE user_profiles.preferences END,
                updated_at = $9
            RETURNING user_id, birth_date, birth_time,
                      CAST(birth_location_lat AS FLOAT8) AS birth_location_lat,
                      CAST(birth_location_lng AS FLOAT8) AS birth_location_lng,
                      birth_location_name, timezone, preferences, created_at, updated_at
            "#
        )
        .bind(user_id)
        .bind(birth_date)
        .bind(birth_time)
        .bind(birth_location_lat)
        .bind(birth_location_lng)
        .bind(birth_location_name)
        .bind(timezone)
        .bind(preferences)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(profile)
    }

    pub async fn set_password_reset_token(
        &self,
        email: &str,
        token: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<bool, Error> {
        let result = sqlx::query(
            "UPDATE users SET reset_token = $1, reset_token_expires_at = $2 WHERE lower(btrim(email)) = lower(btrim($3))",
        )
        .bind(token)
        .bind(expires_at)
        .bind(email)
        .execute(&self.pool)
        .await?;

        // Return true if any row was updated (user found)
        Ok(result.rows_affected() > 0)
    }

    pub async fn find_user_by_reset_token(&self, token: &str) -> Result<Option<User>, Error> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE reset_token = $1 AND reset_token_expires_at > $2",
        )
        .bind(token)
        .bind(Utc::now())
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn update_password(&self, user_id: Uuid, password_hash: &str) -> Result<(), Error> {
        sqlx::query(
            "UPDATE users SET password_hash = $1, reset_token = NULL, reset_token_expires_at = NULL, updated_at = $2 WHERE id = $3"
        )
        .bind(password_hash)
        .bind(Utc::now())
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn find_or_create_cloudflare_user(
        &self,
        email: &str,
        cf_sub: &str,
    ) -> Result<User, Error> {
        let normalized_email = email.trim().to_ascii_lowercase();
        let full_name = normalized_email
            .split('@')
            .next()
            .filter(|name| !name.is_empty())
            .unwrap_or("Cloudflare User");

        let mut tx = self.pool.begin().await?;
        let existing = sqlx::query_as::<_, User>(
            r#"
            SELECT
                id,
                email,
                COALESCE(password_hash, '') AS password_hash,
                COALESCE(full_name, '') AS full_name,
                COALESCE(tier, 'free') AS tier,
                COALESCE(consciousness_level, 0) AS consciousness_level,
                COALESCE(experience_points, 0) AS experience_points,
                created_at,
                updated_at,
                reset_token,
                reset_token_expires_at,
                last_login_at,
                COALESCE(failed_login_attempts, 0) AS failed_login_attempts,
                locked_until
            FROM users
            WHERE lower(btrim(email)) = lower(btrim($1))
            "#,
        )
        .bind(&normalized_email)
        .fetch_optional(&mut *tx)
        .await?;

        let user = if let Some(user) = existing {
            user
        } else {
            sqlx::query_as::<_, User>(
                r#"
                INSERT INTO users (id, email, password_hash, full_name, tier, consciousness_level, created_at, updated_at)
                VALUES ($1, $2, $3, $4, 'free', 0, NOW(), NOW())
                RETURNING *
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(&normalized_email)
            .bind(format!("cf-access:{}", cf_sub))
            .bind(full_name)
            .fetch_one(&mut *tx)
            .await?
        };

        sqlx::query(
            r#"
            INSERT INTO user_account_state (user_id, state)
            VALUES ($1, 'active')
            ON CONFLICT (user_id) DO NOTHING
            "#,
        )
        .bind(user.id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(user)
    }

    /// Update last_login_at timestamp and reset failed login tracking.
    /// Called on every successful login.
    pub async fn update_last_login(&self, user_id: Uuid) -> Result<(), Error> {
        sqlx::query(
            "UPDATE users SET last_login_at = $1, failed_login_attempts = 0, locked_until = NULL WHERE id = $2",
        )
        .bind(Utc::now())
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Atomically increment failed login attempts and lock account if threshold reached.
    /// Returns (new_attempt_count, locked_until).
    pub async fn increment_failed_login(
        &self,
        user_id: Uuid,
        lock_until: DateTime<Utc>,
    ) -> Result<(i32, Option<DateTime<Utc>>), Error> {
        let row = sqlx::query_as::<_, (i32, Option<DateTime<Utc>>)>(
            r#"
            UPDATE users SET
                failed_login_attempts = failed_login_attempts + 1,
                locked_until = CASE
                    WHEN failed_login_attempts + 1 >= 5 THEN $1
                    ELSE locked_until
                END
            WHERE id = $2
            RETURNING failed_login_attempts, locked_until
            "#,
        )
        .bind(lock_until)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Update password for an authenticated user (change-password flow).
    /// Does NOT clear reset_token — that's for the reset-password flow only.
    pub async fn update_password_authenticated(
        &self,
        user_id: Uuid,
        password_hash: &str,
    ) -> Result<(), Error> {
        sqlx::query("UPDATE users SET password_hash = $1, updated_at = $2 WHERE id = $3")
            .bind(password_hash)
            .bind(Utc::now())
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Auto-populate user profile from birth_data on first engine calculation.
    /// Uses upsert — safe to call on every request (no-ops if profile already exists
    /// with the same data). Also updates the user's `full_name` if currently a placeholder.
    #[allow(clippy::too_many_arguments)]
    pub async fn ensure_profile_from_birth_data(
        &self,
        user_id: Uuid,
        name: Option<&str>,
        date: &str,
        time: Option<&str>,
        latitude: f64,
        longitude: f64,
        timezone: &str,
    ) -> Result<(), Error> {
        // Parse date
        let birth_date = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok();
        // Parse time (supports HH:MM and HH:MM:SS)
        let birth_time = time.and_then(|t| {
            NaiveTime::parse_from_str(t, "%H:%M:%S")
                .or_else(|_| NaiveTime::parse_from_str(t, "%H:%M"))
                .ok()
        });

        // Upsert profile — only fills in missing fields, never overwrites existing data
        sqlx::query(
            r#"
            INSERT INTO user_profiles (
                user_id, birth_date, birth_time, birth_location_lat, birth_location_lng,
                timezone, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (user_id) DO UPDATE SET
                birth_date = COALESCE(user_profiles.birth_date, EXCLUDED.birth_date),
                birth_time = COALESCE(user_profiles.birth_time, EXCLUDED.birth_time),
                birth_location_lat = COALESCE(user_profiles.birth_location_lat, EXCLUDED.birth_location_lat),
                birth_location_lng = COALESCE(user_profiles.birth_location_lng, EXCLUDED.birth_location_lng),
                timezone = COALESCE(user_profiles.timezone, EXCLUDED.timezone),
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(birth_date)
        .bind(birth_time)
        .bind(latitude)
        .bind(longitude)
        .bind(timezone)
        .execute(&self.pool)
        .await?;

        // Update user's full_name if they have a placeholder name and birth_data provides one
        if let Some(name) = name {
            if !name.is_empty() {
                sqlx::query(
                    r#"
                    UPDATE users SET full_name = $1, updated_at = NOW()
                    WHERE id = $2 AND (full_name IS NULL OR full_name LIKE 'User %')
                    "#,
                )
                .bind(name)
                .bind(user_id)
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    pub async fn add_experience(
        &self,
        user_id: Uuid,
        amount: i32,
        action: &str,
    ) -> Result<User, Error> {
        // Log the progression event
        sqlx::query(
            "INSERT INTO progression_logs (user_id, xp_amount, action_type, metadata) 
             VALUES ($1, $2, $3, $4)",
        )
        .bind(user_id)
        .bind(amount)
        .bind(action)
        .bind(serde_json::json!({
            "timestamp": Utc::now().to_rfc3339()
        }))
        .execute(&self.pool)
        .await?;

        // Atomic update of user XP and level calculation
        // Level logic is dynamic, but we might want to store it if complex.
        // For now, we update XP. The Level is derived from XP in the application layer usually,
        // or we can add a generated column. But per requirements, just XP tracking is key.
        let updated_user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users 
            SET 
                experience_points = COALESCE(experience_points, 0) + $1,
                updated_at = $2
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(amount)
        .bind(Utc::now())
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_user)
    }

    /// Count readings and promote consciousness_level if the user qualifies
    /// for a higher phase. Also cascades to api_keys table.
    ///
    /// Thresholds: 5→1, 15→2, 40→3, 80→4, 150→5
    pub async fn promote_consciousness_level(&self, user_id: Uuid) -> Result<Option<i32>, Error> {
        // Count total readings for this user
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM readings WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        let reading_count = row.0 as u32;

        // Compute earned level from reading count
        let earned_level: i32 = match reading_count {
            n if n >= 150 => 5,
            n if n >= 80 => 4,
            n if n >= 40 => 3,
            n if n >= 15 => 2,
            n if n >= 5 => 1,
            _ => 0,
        };

        // Only promote (never demote) — take max of current and earned
        let updated = sqlx::query_scalar::<_, i32>(
            r#"
            UPDATE users
            SET consciousness_level = GREATEST(consciousness_level, $1),
                updated_at = $2
            WHERE id = $3
              AND consciousness_level < $1
            RETURNING consciousness_level
            "#,
        )
        .bind(earned_level)
        .bind(Utc::now())
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        // If promoted, cascade to api_keys
        if let Some(new_level) = updated {
            sqlx::query(
                "UPDATE api_keys SET consciousness_level = $1 WHERE user_id = $2 AND consciousness_level < $1",
            )
            .bind(new_level)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

            // Log the promotion event
            sqlx::query(
                "INSERT INTO progression_logs (user_id, xp_amount, action_type, metadata) VALUES ($1, $2, $3, $4)",
            )
            .bind(user_id)
            .bind(0)
            .bind("level_promotion")
            .bind(serde_json::json!({
                "new_level": new_level,
                "reading_count": reading_count,
                "timestamp": Utc::now().to_rfc3339()
            }))
            .execute(&self.pool)
            .await?;
        }

        Ok(updated)
    }
}

async fn sync_user_plan_subscription(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    user_id: Uuid,
    tier: &str,
    effective_start: DateTime<Utc>,
    provider: &str,
) -> Result<(), Error> {
    let plan_code = normalize_plan_code(tier);
    let display_name = plan_display_name(tier);
    let provider_subscription_id = format!("{provider}:{user_id}:{plan_code}");

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
        VALUES ($1, $2, $3, NULL, $4, 'active', false, $5, NULL)
        "#,
    )
    .bind(user_id)
    .bind(plan_id)
    .bind(provider)
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
    use crate::repositories::admin_repository::AdminRepository;
    use sqlx::postgres::PgPoolOptions;

    async fn connect_test_pool() -> Option<sqlx::PgPool> {
        let database_url = match std::env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(_) => {
                eprintln!("Skipping DB integration test: DATABASE_URL not set");
                return None;
            }
        };

        match PgPoolOptions::new()
            .max_connections(2)
            .connect(&database_url)
            .await
        {
            Ok(pool) => Some(pool),
            Err(err) => {
                eprintln!(
                    "Skipping DB integration test: failed to connect to test database: {err}"
                );
                None
            }
        }
    }

    #[tokio::test]
    async fn add_experience_writes_progression_log_with_generated_id() {
        let Some(pool) = connect_test_pool().await else {
            return;
        };

        let repo = UserRepository::new(pool.clone());
        let email = format!("progression-default-{}@example.com", Uuid::new_v4());

        let user = repo
            .create_user(&email, "test_password_hash", "Progression Test User")
            .await
            .expect("Failed to create test user");

        let updated = repo
            .add_experience(user.id, 7, "calculation")
            .await
            .expect("add_experience should succeed with progression_logs UUID default");

        assert_eq!(updated.experience_points, user.experience_points + 7);

        let progression_id: Uuid = sqlx::query_scalar(
            "SELECT id FROM progression_logs WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(user.id)
        .fetch_one(&pool)
        .await
        .expect("Expected a progression_logs row with generated UUID id");

        assert_ne!(
            progression_id,
            Uuid::nil(),
            "Generated UUID must not be nil"
        );

        // Cleanup test data
        sqlx::query("DELETE FROM progression_logs WHERE user_id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .expect("Failed to cleanup progression_logs");

        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .expect("Failed to cleanup users");
    }

    #[tokio::test]
    async fn active_plan_resolution_stays_unambiguous_after_tier_update() {
        let Some(pool) = connect_test_pool().await else {
            return;
        };

        let has_schema: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM information_schema.tables WHERE table_name = 'plan_catalog'
            ) AND EXISTS (
                SELECT 1 FROM information_schema.tables WHERE table_name = 'billing_subscriptions'
            ) AND EXISTS (
                SELECT 1 FROM information_schema.views WHERE table_name = 'user_active_plan_resolutions'
            )
            "#,
        )
        .fetch_one(&pool)
        .await
        .expect("schema probe should succeed");

        if !has_schema {
            eprintln!("Skipping DB integration test: billing plan schema not migrated");
            return;
        }

        let repo = UserRepository::new(pool.clone());
        let admin_repo = AdminRepository::new(pool.clone());
        let email = format!("plan-resolution-{}@example.com", Uuid::new_v4());

        let user = repo
            .create_user(&email, "test_password_hash", "Plan Resolution Test User")
            .await
            .expect("Failed to create test user");

        let initial_plan = repo
            .resolve_active_plan_code(user.id)
            .await
            .expect("active plan should resolve")
            .expect("active plan row should exist");
        assert_eq!(initial_plan, "free");

        let initial_active_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM billing_subscriptions
            WHERE user_id = $1
              AND status IN ('trialing', 'active', 'past_due')
              AND canceled_at IS NULL
            "#,
        )
        .bind(user.id)
        .fetch_one(&pool)
        .await
        .expect("active subscription count should succeed");
        assert_eq!(initial_active_count, 1);

        let updated = admin_repo
            .set_user_tier(user.id, "premium")
            .await
            .expect("tier update should succeed")
            .expect("user should exist");
        assert_eq!(updated.1, "premium");

        let resolved_plan = repo
            .resolve_active_plan_code(user.id)
            .await
            .expect("updated active plan should resolve")
            .expect("updated active plan row should exist");
        assert_eq!(resolved_plan, "premium");

        let active_count_after: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM billing_subscriptions
            WHERE user_id = $1
              AND status IN ('trialing', 'active', 'past_due')
              AND canceled_at IS NULL
            "#,
        )
        .bind(user.id)
        .fetch_one(&pool)
        .await
        .expect("updated active subscription count should succeed");
        assert_eq!(active_count_after, 1);

        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .expect("Failed to cleanup users");
    }
}
