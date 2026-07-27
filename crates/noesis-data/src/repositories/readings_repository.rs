use crate::models::reading::{NewReading, Reading, ReadingSyncRecord};
use sqlx::{Error, PgPool};
use uuid::Uuid;

pub struct ReadingsRepository {
    pool: PgPool,
}

impl ReadingsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert a new reading and return its generated UUID.
    pub async fn save_reading(&self, reading: &NewReading) -> Result<Uuid, Error> {
        match self.save_reading_with_history_sync(reading).await {
            Ok(id) => Ok(id),
            Err(err) if missing_optional_reading_schema(&err) => {
                self.save_reading_legacy(reading).await
            }
            Err(err) => Err(err),
        }
    }

    async fn save_reading_with_history_sync(&self, reading: &NewReading) -> Result<Uuid, Error> {
        let mut tx = self.pool.begin().await?;

        let source_device_id = if let Some(client_device_id) = reading.client_device_id.as_deref() {
            Some(
                sqlx::query_scalar::<_, Uuid>(
                    r#"
                    INSERT INTO user_devices (
                        user_id, client_device_id, platform, app_version, last_client_event_id, last_seen_at
                    )
                    VALUES ($1, $2, $3, $4, $5, NOW())
                    ON CONFLICT (user_id, client_device_id)
                    DO UPDATE SET
                        platform = COALESCE(EXCLUDED.platform, user_devices.platform),
                        app_version = COALESCE(EXCLUDED.app_version, user_devices.app_version),
                        last_client_event_id = COALESCE(EXCLUDED.last_client_event_id, user_devices.last_client_event_id),
                        last_seen_at = NOW(),
                        updated_at = NOW()
                    RETURNING id
                    "#,
                )
                .bind(reading.user_id)
                .bind(client_device_id)
                .bind(reading.device_platform.as_deref())
                .bind(reading.device_app_version.as_deref())
                .bind(reading.client_event_id.as_deref())
                .fetch_one(&mut *tx)
                .await?,
            )
        } else {
            None
        };

        let (row_id, sync_cursor) = if reading.client_event_id.is_some() {
            sqlx::query_as::<_, (Uuid, i64)>(
                r#"
                INSERT INTO readings (
                    user_id, engine_id, workflow_id, input_hash,
                    input_data, result_data, witness_prompt,
                    consciousness_level, calculation_time_ms,
                    client_event_id, source_device_id, claimed_source_client
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                ON CONFLICT (user_id, client_event_id) WHERE client_event_id IS NOT NULL
                DO UPDATE SET
                    source_device_id = COALESCE(EXCLUDED.source_device_id, readings.source_device_id),
                    claimed_source_client = COALESCE(
                        EXCLUDED.claimed_source_client,
                        readings.claimed_source_client
                    )
                RETURNING id, sync_cursor
                "#,
            )
            .bind(reading.user_id)
            .bind(&reading.engine_id)
            .bind(&reading.workflow_id)
            .bind(&reading.input_hash)
            .bind(&reading.input_data)
            .bind(&reading.result_data)
            .bind(&reading.witness_prompt)
            .bind(reading.consciousness_level)
            .bind(reading.calculation_time_ms)
            .bind(reading.client_event_id.as_deref())
            .bind(source_device_id)
            .bind(reading.claimed_source_client.as_deref())
            .fetch_one(&mut *tx)
            .await?
        } else {
            sqlx::query_as::<_, (Uuid, i64)>(
                r#"
                INSERT INTO readings (
                    user_id, engine_id, workflow_id, input_hash,
                    input_data, result_data, witness_prompt,
                    consciousness_level, calculation_time_ms,
                    client_event_id, source_device_id, claimed_source_client
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NULL, $10, $11)
                RETURNING id, sync_cursor
                "#,
            )
            .bind(reading.user_id)
            .bind(&reading.engine_id)
            .bind(&reading.workflow_id)
            .bind(&reading.input_hash)
            .bind(&reading.input_data)
            .bind(&reading.result_data)
            .bind(&reading.witness_prompt)
            .bind(reading.consciousness_level)
            .bind(reading.calculation_time_ms)
            .bind(source_device_id)
            .bind(reading.claimed_source_client.as_deref())
            .fetch_one(&mut *tx)
            .await?
        };

        if let Some(device_id) = source_device_id {
            sqlx::query(
                r#"
                INSERT INTO history_sync_state (
                    user_id, device_id, last_synced_cursor, last_client_event_id, last_synced_at, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, NOW(), NOW(), NOW())
                ON CONFLICT (user_id, device_id)
                DO UPDATE SET
                    last_synced_cursor = GREATEST(history_sync_state.last_synced_cursor, EXCLUDED.last_synced_cursor),
                    last_client_event_id = COALESCE(EXCLUDED.last_client_event_id, history_sync_state.last_client_event_id),
                    last_synced_at = NOW(),
                    updated_at = NOW()
                "#,
            )
            .bind(reading.user_id)
            .bind(device_id)
            .bind(sync_cursor)
            .bind(reading.client_event_id.as_deref())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(row_id)
    }

    async fn save_reading_legacy(&self, reading: &NewReading) -> Result<Uuid, Error> {
        let row = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO readings (
                user_id, engine_id, workflow_id, input_hash,
                input_data, result_data, witness_prompt,
                consciousness_level, calculation_time_ms
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id
            "#,
        )
        .bind(reading.user_id)
        .bind(&reading.engine_id)
        .bind(&reading.workflow_id)
        .bind(&reading.input_hash)
        .bind(&reading.input_data)
        .bind(&reading.result_data)
        .bind(&reading.witness_prompt)
        .bind(reading.consciousness_level)
        .bind(reading.calculation_time_ms)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Fetch a single reading by ID, scoped to a user.
    pub async fn get_reading(&self, id: Uuid, user_id: Uuid) -> Result<Option<Reading>, Error> {
        sqlx::query_as::<_, Reading>("SELECT * FROM readings WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
    }

    /// List readings for a user, with optional engine filter and pagination.
    pub async fn list_readings(
        &self,
        user_id: Uuid,
        engine_id: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Reading>, Error> {
        if let Some(eid) = engine_id {
            sqlx::query_as::<_, Reading>(
                r#"
                SELECT * FROM readings
                WHERE user_id = $1 AND engine_id = $2
                ORDER BY created_at DESC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(user_id)
            .bind(eid)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, Reading>(
                r#"
                SELECT * FROM readings
                WHERE user_id = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        }
    }

    /// Count readings per engine for a user. Returns (engine_id, count) pairs.
    pub async fn count_by_engine(&self, user_id: Uuid) -> Result<Vec<(String, i64)>, Error> {
        sqlx::query_as::<_, (String, i64)>(
            "SELECT engine_id, COUNT(*) as count FROM readings WHERE user_id = $1 GROUP BY engine_id ORDER BY count DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Count readings for a user, with optional engine filter.
    pub async fn count_readings(
        &self,
        user_id: Uuid,
        engine_id: Option<&str>,
    ) -> Result<i64, Error> {
        if let Some(eid) = engine_id {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM readings WHERE user_id = $1 AND engine_id = $2",
            )
            .bind(user_id)
            .bind(eid)
            .fetch_one(&self.pool)
            .await
        } else {
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM readings WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(&self.pool)
                .await
        }
    }

    pub async fn list_readings_after_cursor(
        &self,
        user_id: Uuid,
        after_cursor: i64,
        limit: i64,
    ) -> Result<Vec<ReadingSyncRecord>, Error> {
        sqlx::query_as::<_, ReadingSyncRecord>(
            r#"
            SELECT
                id,
                sync_cursor,
                user_id,
                engine_id,
                workflow_id,
                client_event_id,
                created_at
            FROM readings
            WHERE user_id = $1
              AND sync_cursor > $2
            ORDER BY sync_cursor ASC
            LIMIT $3
            "#,
        )
        .bind(user_id)
        .bind(after_cursor)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }
}

fn missing_optional_reading_schema(err: &Error) -> bool {
    let Some(db_err) = err.as_database_error() else {
        return false;
    };

    match db_err.code().as_deref() {
        Some("42P01") => {
            let message = db_err.message().to_ascii_lowercase();
            message.contains("user_devices") || message.contains("history_sync_state")
        }
        Some("42703") => {
            let message = db_err.message().to_ascii_lowercase();
            message.contains("client_event_id")
                || message.contains("source_device_id")
                || message.contains("sync_cursor")
                || message.contains("claimed_source_client")
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::reading::NewReading;
    use crate::repositories::user_repository::UserRepository;
    use serde_json::json;
    use sqlx::postgres::PgPoolOptions;

    #[tokio::test]
    async fn save_reading_is_idempotent_when_client_event_id_is_present() {
        let database_url = match std::env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(_) => {
                eprintln!("Skipping DB integration test: DATABASE_URL not set");
                return;
            }
        };

        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        let has_schema: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_name = 'readings'
                  AND column_name = 'client_event_id'
            ) AND EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_name = 'user_devices'
            ) AND EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_name = 'history_sync_state'
            )
            "#,
        )
        .fetch_one(&pool)
        .await
        .expect("schema probe should succeed");

        if !has_schema {
            eprintln!("Skipping DB integration test: history sync schema not migrated");
            return;
        }

        let user_repo = UserRepository::new(pool.clone());
        let readings_repo = ReadingsRepository::new(pool.clone());
        let email = format!("history-sync-{}@example.com", Uuid::new_v4());

        let user = user_repo
            .create_user(&email, "test_password_hash", "History Sync Test User")
            .await
            .expect("Failed to create test user");

        let client_event_id = format!("evt-{}", Uuid::new_v4());
        let reading = NewReading {
            user_id: user.id,
            engine_id: "panchanga".to_string(),
            workflow_id: None,
            input_hash: "same-input".to_string(),
            input_data: json!({"birth_date":"1991-08-13"}),
            result_data: json!({"ok":true}),
            witness_prompt: Some("test witness".to_string()),
            consciousness_level: 1,
            calculation_time_ms: Some(42.0),
            client_event_id: Some(client_event_id.clone()),
            client_device_id: Some("ios-phone-1".to_string()),
            device_platform: Some("ios".to_string()),
            device_app_version: Some("1.0.0".to_string()),
            claimed_source_client: Some("urania".to_string()),
        };

        let first_id = readings_repo
            .save_reading(&reading)
            .await
            .expect("first save should succeed");
        let second_id = readings_repo
            .save_reading(&reading)
            .await
            .expect("second save should be idempotent");

        assert_eq!(first_id, second_id);

        let matching_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM readings WHERE user_id = $1 AND client_event_id = $2",
        )
        .bind(user.id)
        .bind(&client_event_id)
        .fetch_one(&pool)
        .await
        .expect("count query should succeed");
        assert_eq!(matching_count, 1);

        let claimed_source: Option<String> =
            sqlx::query_scalar("SELECT claimed_source_client FROM readings WHERE id = $1")
                .bind(first_id)
                .fetch_one(&pool)
                .await
                .expect("claimed source query should succeed");
        assert_eq!(claimed_source.as_deref(), Some("urania"));

        let delta = readings_repo
            .list_readings_after_cursor(user.id, 0, 10)
            .await
            .expect("cursor query should succeed");
        assert!(!delta.is_empty());
        assert_eq!(delta[0].id, first_id);
        assert_eq!(
            delta[0].client_event_id.as_deref(),
            Some(client_event_id.as_str())
        );

        sqlx::query("DELETE FROM history_sync_state WHERE user_id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .expect("cleanup history_sync_state");
        sqlx::query("DELETE FROM readings WHERE user_id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .expect("cleanup readings");
        sqlx::query("DELETE FROM user_devices WHERE user_id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .expect("cleanup user_devices");
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .expect("cleanup users");
    }
}
