//! Persistence for background reading jobs (migration 043).
//!
//! A job is `running` until its writer records `succeeded` with the full
//! response or `failed` with a reason. A job whose writer stopped without
//! either (the process restarted) is closed as failed once it has gone
//! quiet for longer than any single pass can take.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{Error, PgPool, Row};
use uuid::Uuid;

pub struct AssetJobRepository {
    pool: PgPool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssetJob {
    pub id: Uuid,
    pub user_id: String,
    pub mode: String,
    pub status: String,
    pub completed_passes: i32,
    pub total_passes: i32,
    pub current_pass: Option<String>,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Why `start` did not create a job.
#[derive(Debug)]
pub enum StartJobError {
    /// The user already has a job running; one at a time keeps cost bounded.
    AlreadyRunning(Uuid),
    Database(Error),
}

impl From<Error> for StartJobError {
    fn from(error: Error) -> Self {
        Self::Database(error)
    }
}

const JOB_COLUMNS: &str = "id, user_id, mode, status, completed_passes, total_passes, \
    current_pass, result, error, created_at, updated_at";

fn job_from_row(row: &sqlx::postgres::PgRow) -> Result<AssetJob, Error> {
    Ok(AssetJob {
        id: row.try_get("id")?,
        user_id: row.try_get("user_id")?,
        mode: row.try_get("mode")?,
        status: row.try_get("status")?,
        completed_passes: row.try_get("completed_passes")?,
        total_passes: row.try_get("total_passes")?,
        current_pass: row.try_get("current_pass")?,
        result: row.try_get("result")?,
        error: row.try_get("error")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

impl AssetJobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Close jobs that have gone quiet for longer than `stale_after_secs`:
    /// their writer is gone, so they will never finish.
    async fn close_stale(&self, user_id: &str, stale_after_secs: i64) -> Result<(), Error> {
        sqlx::query(
            r#"
            UPDATE asset_jobs
               SET status = 'failed',
                   error = 'The reading stopped before it finished. Start it again.',
                   updated_at = NOW()
             WHERE user_id = $1
               AND status = 'running'
               AND updated_at < NOW() - make_interval(secs => $2)
            "#,
        )
        .bind(user_id)
        .bind(stale_after_secs as f64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Start a job for `user_id` unless one of theirs is still running.
    pub async fn start(
        &self,
        user_id: &str,
        mode: &str,
        total_passes: i32,
        stale_after_secs: i64,
    ) -> Result<AssetJob, StartJobError> {
        self.close_stale(user_id, stale_after_secs).await?;
        let mut tx = self.pool.begin().await?;
        // Serialise starts per user so two requests cannot both pass the check.
        sqlx::query("SELECT pg_advisory_xact_lock(hashtext('asset_jobs:' || $1))")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        let running: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM asset_jobs WHERE user_id = $1 AND status = 'running' LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?;
        if let Some(id) = running {
            return Err(StartJobError::AlreadyRunning(id));
        }
        let row = sqlx::query(&format!(
            "INSERT INTO asset_jobs (id, user_id, mode, status, total_passes) \
             VALUES ($1, $2, $3, 'running', $4) RETURNING {JOB_COLUMNS}"
        ))
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(mode)
        .bind(total_passes)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(job_from_row(&row)?)
    }

    /// Record that `completed` passes are done and `current` is being written.
    pub async fn progress(
        &self,
        id: Uuid,
        completed: i32,
        current: Option<&str>,
    ) -> Result<(), Error> {
        sqlx::query(
            "UPDATE asset_jobs SET completed_passes = $2, current_pass = $3, updated_at = NOW() \
             WHERE id = $1 AND status = 'running'",
        )
        .bind(id)
        .bind(completed)
        .bind(current)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn succeed(&self, id: Uuid, result: &Value) -> Result<(), Error> {
        sqlx::query(
            "UPDATE asset_jobs SET status = 'succeeded', completed_passes = total_passes, \
             current_pass = NULL, result = $2, updated_at = NOW() WHERE id = $1 AND status = 'running'",
        )
        .bind(id)
        .bind(result)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn fail(&self, id: Uuid, error: &str) -> Result<(), Error> {
        sqlx::query(
            "UPDATE asset_jobs SET status = 'failed', current_pass = NULL, error = $2, \
             updated_at = NOW() WHERE id = $1 AND status = 'running'",
        )
        .bind(id)
        .bind(error)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// The job if it exists and belongs to `user_id`; stale jobs are closed first.
    pub async fn get_for_user(
        &self,
        id: Uuid,
        user_id: &str,
        stale_after_secs: i64,
    ) -> Result<Option<AssetJob>, Error> {
        self.close_stale(user_id, stale_after_secs).await?;
        let row = sqlx::query(&format!(
            "SELECT {JOB_COLUMNS} FROM asset_jobs WHERE id = $1 AND user_id = $2"
        ))
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        row.as_ref().map(job_from_row).transpose()
    }
}
