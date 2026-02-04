use sqlx::{PgPool, Error};
use uuid::Uuid;
use chrono::{Utc, DateTime, NaiveDate, NaiveTime};
use crate::models::user::{User, UserProfile};

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
        .bind("Free") // Default tier
        .bind(0)      // Default consciousness level
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, Error> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, Error> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

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
            RETURNING *
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
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(profile)
    }

    pub async fn get_profile(&self, user_id: Uuid) -> Result<Option<UserProfile>, Error> {
         sqlx::query_as::<_, UserProfile>(
            "SELECT * FROM user_profiles WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn update_user(
        &self, 
        user_id: Uuid, 
        full_name: Option<String>, 
        email: Option<String>
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
            "#
        )
        .bind(user_id)
        .bind(full_name)
        .bind(email)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;
        
        Ok(user)
    }

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
            RETURNING *
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

    pub async fn set_password_reset_token(&self, email: &str, token: &str, expires_at: DateTime<Utc>) -> Result<(), Error> {
        sqlx::query(
            "UPDATE users SET reset_token = $1, reset_token_expires_at = $2 WHERE email = $3"
        )
        .bind(token)
        .bind(expires_at)
        .bind(email)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_user_by_reset_token(&self, token: &str) -> Result<Option<User>, Error> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE reset_token = $1 AND reset_token_expires_at > $2"
        )
        .bind(token)
        .bind(Utc::now())
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn update_password(&self, user_id: Uuid, password_hash: &str) -> Result<(), Error> {
         sqlx::query(
            "UPDATE users SET password_hash = $1, reset_token = NULL, reset_token_expires_at = NULL WHERE id = $2"
        )
        .bind(password_hash)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
