use crate::models::reading::{NewReading, Reading};
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
    pub async fn get_reading(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Reading>, Error> {
        sqlx::query_as::<_, Reading>(
            "SELECT * FROM readings WHERE id = $1 AND user_id = $2",
        )
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
    pub async fn count_by_engine(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<(String, i64)>, Error> {
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
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM readings WHERE user_id = $1",
            )
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
        }
    }
}
