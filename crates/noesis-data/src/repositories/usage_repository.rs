use sqlx::{Error, PgPool};
use uuid::Uuid;

pub struct UsageRepository {
    pool: PgPool,
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
}
