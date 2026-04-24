use crate::models::biofield::{
    BiofieldBaseline, BiofieldBaselineSummaryRecord, BiofieldCaptureArtifact, BiofieldExport,
    BiofieldSession, NewBiofieldBaseline, NewBiofieldCaptureArtifact, NewBiofieldExport,
    NewBiofieldSession, BIOFIELD_SESSION_STATUS_ACTIVE, BIOFIELD_SESSION_STATUS_CLOSED,
};
use crate::models::reading::Reading;
use sqlx::{Error, PgPool};
use uuid::Uuid;

pub struct BiofieldRepository {
    pool: PgPool,
}

impl BiofieldRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_session(
        &self,
        session: &NewBiofieldSession,
    ) -> Result<BiofieldSession, Error> {
        sqlx::query_as::<_, BiofieldSession>(
            r#"
            INSERT INTO biofield_sessions (
                user_id, status, client_device_id, viewer_version, notes
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(session.user_id)
        .bind(BIOFIELD_SESSION_STATUS_ACTIVE)
        .bind(session.client_device_id.as_deref())
        .bind(session.viewer_version.as_deref())
        .bind(session.notes.as_deref())
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_session(
        &self,
        session_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<BiofieldSession>, Error> {
        sqlx::query_as::<_, BiofieldSession>(
            "SELECT * FROM biofield_sessions WHERE id = $1 AND user_id = $2",
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn close_session(
        &self,
        session_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<BiofieldSession>, Error> {
        sqlx::query_as::<_, BiofieldSession>(
            r#"
            UPDATE biofield_sessions
            SET status = $3,
                closed_at = NOW()
            WHERE id = $1
              AND user_id = $2
              AND status = $4
            RETURNING *
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .bind(BIOFIELD_SESSION_STATUS_CLOSED)
        .bind(BIOFIELD_SESSION_STATUS_ACTIVE)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create_artifact(
        &self,
        artifact: &NewBiofieldCaptureArtifact,
    ) -> Result<BiofieldCaptureArtifact, Error> {
        sqlx::query_as::<_, BiofieldCaptureArtifact>(
            r#"
            INSERT INTO biofield_capture_artifacts (
                session_id, reading_id, artifact_kind, storage_path,
                mime_type, byte_size, capture_metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(artifact.session_id)
        .bind(artifact.reading_id)
        .bind(&artifact.artifact_kind)
        .bind(&artifact.storage_path)
        .bind(&artifact.mime_type)
        .bind(artifact.byte_size)
        .bind(&artifact.capture_metadata)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_session_artifacts(
        &self,
        session_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<BiofieldCaptureArtifact>, Error> {
        sqlx::query_as::<_, BiofieldCaptureArtifact>(
            r#"
            SELECT artifact.*
            FROM biofield_capture_artifacts AS artifact
            INNER JOIN biofield_sessions AS session
                ON session.id = artifact.session_id
            WHERE artifact.session_id = $1
              AND session.user_id = $2
            ORDER BY artifact.created_at DESC
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn link_artifact_to_reading(
        &self,
        artifact_id: Uuid,
        reading_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<BiofieldCaptureArtifact>, Error> {
        sqlx::query_as::<_, BiofieldCaptureArtifact>(
            r#"
            UPDATE biofield_capture_artifacts AS artifact
            SET reading_id = $2
            FROM biofield_sessions AS session
            INNER JOIN readings
                ON readings.id = $2
            WHERE artifact.id = $1
              AND artifact.session_id = session.id
              AND session.user_id = $3
              AND readings.user_id = $3
            RETURNING artifact.*
            "#,
        )
        .bind(artifact_id)
        .bind(reading_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_reading_artifacts(
        &self,
        reading_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<BiofieldCaptureArtifact>, Error> {
        sqlx::query_as::<_, BiofieldCaptureArtifact>(
            r#"
            SELECT artifact.*
            FROM biofield_capture_artifacts AS artifact
            INNER JOIN biofield_sessions AS session
                ON session.id = artifact.session_id
            INNER JOIN readings
                ON readings.id = artifact.reading_id
            WHERE artifact.reading_id = $1
              AND session.user_id = $2
              AND readings.user_id = $2
            ORDER BY artifact.created_at DESC
            "#,
        )
        .bind(reading_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_baseline_summary(
        &self,
        baseline_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<BiofieldBaselineSummaryRecord>, Error> {
        sqlx::query_as::<_, BiofieldBaselineSummaryRecord>(
            r#"
            SELECT
                baseline.id,
                baseline.user_id,
                baseline.name,
                baseline.notes,
                baseline.created_at,
                baseline.updated_at,
                COUNT(link.reading_id)::BIGINT AS reading_count
            FROM biofield_baselines AS baseline
            LEFT JOIN biofield_baseline_readings AS link
                ON link.baseline_id = baseline.id
            WHERE baseline.id = $1
              AND baseline.user_id = $2
            GROUP BY baseline.id
            "#,
        )
        .bind(baseline_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_baseline_readings(
        &self,
        baseline_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<Reading>, Error> {
        sqlx::query_as::<_, Reading>(
            r#"
            SELECT readings.*
            FROM biofield_baseline_readings AS link
            INNER JOIN biofield_baselines AS baseline
                ON baseline.id = link.baseline_id
            INNER JOIN readings
                ON readings.id = link.reading_id
            WHERE link.baseline_id = $1
              AND baseline.user_id = $2
              AND readings.user_id = $2
            ORDER BY link.sort_order ASC, readings.created_at ASC
            "#,
        )
        .bind(baseline_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create_baseline(
        &self,
        baseline: &NewBiofieldBaseline,
        reading_ids: &[Uuid],
    ) -> Result<BiofieldBaseline, Error> {
        let mut tx = self.pool.begin().await?;

        let baseline_record = sqlx::query_as::<_, BiofieldBaseline>(
            r#"
            INSERT INTO biofield_baselines (user_id, name, notes)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(baseline.user_id)
        .bind(&baseline.name)
        .bind(baseline.notes.as_deref())
        .fetch_one(&mut *tx)
        .await?;

        for (index, reading_id) in reading_ids.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO biofield_baseline_readings (baseline_id, reading_id, sort_order)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(baseline_record.id)
            .bind(reading_id)
            .bind(index as i32)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(baseline_record)
    }

    pub async fn list_baselines(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<BiofieldBaselineSummaryRecord>, Error> {
        sqlx::query_as::<_, BiofieldBaselineSummaryRecord>(
            r#"
            SELECT
                baseline.id,
                baseline.user_id,
                baseline.name,
                baseline.notes,
                baseline.created_at,
                baseline.updated_at,
                COUNT(link.reading_id)::BIGINT AS reading_count
            FROM biofield_baselines AS baseline
            LEFT JOIN biofield_baseline_readings AS link
                ON link.baseline_id = baseline.id
            WHERE baseline.user_id = $1
            GROUP BY baseline.id
            ORDER BY baseline.updated_at DESC, baseline.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create_export(&self, export: &NewBiofieldExport) -> Result<BiofieldExport, Error> {
        sqlx::query_as::<_, BiofieldExport>(
            r#"
            INSERT INTO biofield_exports (
                user_id, reading_id, baseline_id, export_format, file_name,
                storage_path, mime_type, byte_size
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(export.user_id)
        .bind(export.reading_id)
        .bind(export.baseline_id)
        .bind(&export.export_format)
        .bind(&export.file_name)
        .bind(&export.storage_path)
        .bind(&export.mime_type)
        .bind(export.byte_size)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_export(
        &self,
        export_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<BiofieldExport>, Error> {
        sqlx::query_as::<_, BiofieldExport>(
            "SELECT * FROM biofield_exports WHERE id = $1 AND user_id = $2",
        )
        .bind(export_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::biofield::{
        NewBiofieldBaseline, NewBiofieldExport, BIOFIELD_CAPTURE_ARTIFACT_SOURCE_IMAGE,
    };
    use crate::models::reading::NewReading;
    use crate::repositories::readings_repository::ReadingsRepository;
    use crate::repositories::user_repository::UserRepository;
    use serde_json::json;
    use sqlx::postgres::PgPoolOptions;
    use std::fs;
    use std::path::PathBuf;

    const BIOFIELD_SCHEMA_LOCK_ID: i64 = 20_260_405_017;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace crate dir")
            .parent()
            .expect("workspace root")
            .to_path_buf()
    }

    async fn ensure_biofield_schema(pool: &PgPool) {
        let schema_exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_name = 'biofield_sessions'
            ) AND EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_name = 'biofield_capture_artifacts'
            ) AND EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_name = 'biofield_baselines'
            ) AND EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_name = 'biofield_baseline_readings'
            ) AND EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_name = 'biofield_exports'
            )
            "#,
        )
        .fetch_one(pool)
        .await
        .expect("schema probe should succeed");

        if schema_exists {
            return;
        }

        sqlx::query("SELECT pg_advisory_lock($1)")
            .bind(BIOFIELD_SCHEMA_LOCK_ID)
            .execute(pool)
            .await
            .expect("schema lock should succeed");

        let migration_017 =
            fs::read_to_string(repo_root().join("migrations/017_biofield_sessions.sql"))
                .expect("root migration 017");
        let migration_018 =
            fs::read_to_string(repo_root().join("migrations/018_biofield_baselines.sql"))
                .expect("root migration 018");
        let migration_019 =
            fs::read_to_string(repo_root().join("migrations/019_biofield_exports.sql"))
                .expect("root migration 019");

        let apply_result = async {
            let schema_exists_after_lock: bool = sqlx::query_scalar(
                r#"
                SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.tables
                    WHERE table_name = 'biofield_sessions'
                ) AND EXISTS (
                    SELECT 1
                    FROM information_schema.tables
                    WHERE table_name = 'biofield_capture_artifacts'
                ) AND EXISTS (
                    SELECT 1
                    FROM information_schema.tables
                    WHERE table_name = 'biofield_baselines'
                ) AND EXISTS (
                    SELECT 1
                    FROM information_schema.tables
                    WHERE table_name = 'biofield_baseline_readings'
                ) AND EXISTS (
                    SELECT 1
                    FROM information_schema.tables
                    WHERE table_name = 'biofield_exports'
                )
                "#,
            )
            .fetch_one(pool)
            .await
            .expect("schema probe after lock should succeed");

            if schema_exists_after_lock {
                return;
            }

            sqlx::raw_sql(&migration_017)
                .execute(pool)
                .await
                .expect("biofield migration 017 should apply to test database");
            sqlx::raw_sql(&migration_018)
                .execute(pool)
                .await
                .expect("biofield migration 018 should apply to test database");
            sqlx::raw_sql(&migration_019)
                .execute(pool)
                .await
                .expect("biofield migration 019 should apply to test database");
        }
        .await;

        let unlock_result = sqlx::query("SELECT pg_advisory_unlock($1)")
            .bind(BIOFIELD_SCHEMA_LOCK_ID)
            .execute(pool)
            .await;
        unlock_result.expect("schema unlock should succeed");

        apply_result
    }

    async fn connect_test_db() -> Option<PgPool> {
        let database_url =
            match std::env::var("DATABASE_URL").or_else(|_| std::env::var("TEST_DATABASE_URL")) {
                Ok(url) => url,
                Err(_) => {
                    eprintln!("Skipping DB integration test: DATABASE_URL not set");
                    return None;
                }
            };

        let pool = match PgPoolOptions::new()
            .max_connections(2)
            .connect(&database_url)
            .await
        {
            Ok(pool) => pool,
            Err(err) => {
                eprintln!("Skipping DB integration test: could not connect: {err}");
                return None;
            }
        };

        ensure_biofield_schema(&pool).await;

        Some(pool)
    }

    #[tokio::test]
    async fn biofield_repository_creates_gets_and_closes_user_scoped_session() {
        let Some(pool) = connect_test_db().await else {
            return;
        };

        let user_repo = UserRepository::new(pool.clone());
        let biofield_repo = BiofieldRepository::new(pool.clone());
        let email = format!("biofield-session-{}@example.com", Uuid::new_v4());

        let user = user_repo
            .create_user(&email, "test_password_hash", "Biofield Session Test User")
            .await
            .expect("Failed to create test user");

        let mut new_session = NewBiofieldSession::new(user.id);
        new_session.client_device_id = Some("desktop-browser-1".to_string());
        new_session.viewer_version = Some("web-0.1.0".to_string());

        let session = biofield_repo
            .create_session(&new_session)
            .await
            .expect("session should be created");

        assert_eq!(session.user_id, user.id);
        assert_eq!(session.status, BIOFIELD_SESSION_STATUS_ACTIVE);
        assert_eq!(
            session.client_device_id.as_deref(),
            Some("desktop-browser-1")
        );
        assert_eq!(session.viewer_version.as_deref(), Some("web-0.1.0"));
        assert!(session.closed_at.is_none());

        let fetched = biofield_repo
            .get_session(session.id, user.id)
            .await
            .expect("session lookup should succeed")
            .expect("session should exist");
        assert_eq!(fetched.id, session.id);

        let missing = biofield_repo
            .get_session(session.id, Uuid::new_v4())
            .await
            .expect("cross-user lookup should not error");
        assert!(missing.is_none());

        let closed = biofield_repo
            .close_session(session.id, user.id)
            .await
            .expect("session close should succeed")
            .expect("session should still exist for owner");
        assert_eq!(closed.id, session.id);
        assert_eq!(closed.status, BIOFIELD_SESSION_STATUS_CLOSED);
        assert!(closed.closed_at.is_some());

        let refetched = biofield_repo
            .get_session(session.id, user.id)
            .await
            .expect("closed session lookup should succeed")
            .expect("closed session should exist");
        assert_eq!(refetched.status, BIOFIELD_SESSION_STATUS_CLOSED);
        assert!(refetched.closed_at.is_some());

        let second_close = biofield_repo
            .close_session(session.id, user.id)
            .await
            .expect("second close should not error");
        assert!(second_close.is_none());

        sqlx::query("DELETE FROM biofield_sessions WHERE id = $1")
            .bind(session.id)
            .execute(&pool)
            .await
            .expect("cleanup biofield_sessions");
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .expect("cleanup users");
    }

    #[tokio::test]
    async fn biofield_repository_creates_lists_and_links_artifacts() {
        let Some(pool) = connect_test_db().await else {
            return;
        };

        let user_repo = UserRepository::new(pool.clone());
        let readings_repo = ReadingsRepository::new(pool.clone());
        let biofield_repo = BiofieldRepository::new(pool.clone());
        let email = format!("biofield-artifact-{}@example.com", Uuid::new_v4());

        let user = user_repo
            .create_user(&email, "test_password_hash", "Biofield Artifact Test User")
            .await
            .expect("Failed to create test user");

        let session = biofield_repo
            .create_session(&NewBiofieldSession::new(user.id))
            .await
            .expect("session should be created");

        let artifact = biofield_repo
            .create_artifact(&NewBiofieldCaptureArtifact {
                session_id: session.id,
                reading_id: None,
                artifact_kind: BIOFIELD_CAPTURE_ARTIFACT_SOURCE_IMAGE.to_string(),
                storage_path: format!("biofield/{}/capture.jpg", session.id),
                mime_type: "image/jpeg".to_string(),
                byte_size: 12_345,
                capture_metadata: json!({
                    "width": 640,
                    "height": 480
                }),
            })
            .await
            .expect("artifact should be created");

        let session_artifacts = biofield_repo
            .list_session_artifacts(session.id, user.id)
            .await
            .expect("session artifact listing should succeed");
        assert_eq!(session_artifacts.len(), 1);
        assert_eq!(session_artifacts[0].id, artifact.id);
        assert!(session_artifacts[0].reading_id.is_none());

        let reading_id = readings_repo
            .save_reading(&NewReading {
                user_id: user.id,
                engine_id: "biofield-capture".to_string(),
                workflow_id: None,
                input_hash: format!("biofield-input-{}", Uuid::new_v4()),
                input_data: json!({
                    "session_id": session.id,
                    "artifact_id": artifact.id
                }),
                result_data: json!({
                    "quality": { "sufficient_quality": true }
                }),
                witness_prompt: None,
                consciousness_level: 0,
                calculation_time_ms: Some(24.0),
                client_event_id: None,
                client_device_id: None,
                device_platform: None,
                device_app_version: None,
            })
            .await
            .expect("reading should be created");

        let linked_artifact = biofield_repo
            .link_artifact_to_reading(artifact.id, reading_id, user.id)
            .await
            .expect("artifact linkage should succeed")
            .expect("artifact should be linked");
        assert_eq!(linked_artifact.reading_id, Some(reading_id));

        let reading_artifacts = biofield_repo
            .list_reading_artifacts(reading_id, user.id)
            .await
            .expect("reading artifact listing should succeed");
        assert_eq!(reading_artifacts.len(), 1);
        assert_eq!(reading_artifacts[0].id, artifact.id);
        assert_eq!(reading_artifacts[0].reading_id, Some(reading_id));

        sqlx::query("DELETE FROM biofield_capture_artifacts WHERE session_id = $1")
            .bind(session.id)
            .execute(&pool)
            .await
            .expect("cleanup biofield_capture_artifacts");
        sqlx::query("DELETE FROM biofield_sessions WHERE id = $1")
            .bind(session.id)
            .execute(&pool)
            .await
            .expect("cleanup biofield_sessions");
        sqlx::query("DELETE FROM readings WHERE id = $1")
            .bind(reading_id)
            .execute(&pool)
            .await
            .expect("cleanup readings");
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .expect("cleanup users");
    }

    #[tokio::test]
    async fn biofield_repository_lists_baseline_readings_and_gets_exports() {
        let Some(pool) = connect_test_db().await else {
            return;
        };

        let user_repo = UserRepository::new(pool.clone());
        let readings_repo = ReadingsRepository::new(pool.clone());
        let biofield_repo = BiofieldRepository::new(pool.clone());
        let email = format!("biofield-export-{}@example.com", Uuid::new_v4());

        let user = user_repo
            .create_user(&email, "test_password_hash", "Biofield Export Test User")
            .await
            .expect("Failed to create test user");

        let session = biofield_repo
            .create_session(&NewBiofieldSession::new(user.id))
            .await
            .expect("session should be created");

        let first_reading_id = readings_repo
            .save_reading(&NewReading {
                user_id: user.id,
                engine_id: "biofield-capture".to_string(),
                workflow_id: None,
                input_hash: format!("biofield-export-input-{}", Uuid::new_v4()),
                input_data: json!({
                    "session_id": session.id,
                    "content_type": "image/jpeg"
                }),
                result_data: json!({
                    "analysis_version": "stub-metrics/v1",
                    "metrics": { "light_quanta_density": 40.0 },
                    "quality_assessment": { "sufficient_quality": true }
                }),
                witness_prompt: None,
                consciousness_level: 0,
                calculation_time_ms: Some(8.0),
                client_event_id: None,
                client_device_id: None,
                device_platform: None,
                device_app_version: None,
            })
            .await
            .expect("first reading should be created");

        let second_reading_id = readings_repo
            .save_reading(&NewReading {
                user_id: user.id,
                engine_id: "biofield-capture".to_string(),
                workflow_id: None,
                input_hash: format!("biofield-export-input-{}", Uuid::new_v4()),
                input_data: json!({
                    "session_id": session.id,
                    "content_type": "image/jpeg"
                }),
                result_data: json!({
                    "analysis_version": "stub-metrics/v1",
                    "metrics": { "light_quanta_density": 35.0 },
                    "quality_assessment": { "sufficient_quality": true }
                }),
                witness_prompt: None,
                consciousness_level: 0,
                calculation_time_ms: Some(8.0),
                client_event_id: None,
                client_device_id: None,
                device_platform: None,
                device_app_version: None,
            })
            .await
            .expect("second reading should be created");

        let baseline = biofield_repo
            .create_baseline(
                &NewBiofieldBaseline {
                    user_id: user.id,
                    name: "Reference baseline".to_string(),
                    notes: Some("Created for repository verification".to_string()),
                },
                &[first_reading_id, second_reading_id],
            )
            .await
            .expect("baseline should be created");

        let baseline_readings = biofield_repo
            .list_baseline_readings(baseline.id, user.id)
            .await
            .expect("baseline readings should list");
        assert_eq!(baseline_readings.len(), 2);
        assert_eq!(baseline_readings[0].id, first_reading_id);
        assert_eq!(baseline_readings[1].id, second_reading_id);

        let export = biofield_repo
            .create_export(&NewBiofieldExport {
                user_id: user.id,
                reading_id: first_reading_id,
                baseline_id: Some(baseline.id),
                export_format: "json".to_string(),
                file_name: "biofield-reading.json".to_string(),
                storage_path: format!("biofield/{}/exports/export.json", session.id),
                mime_type: "application/json".to_string(),
                byte_size: 1024,
            })
            .await
            .expect("export should be created");

        let fetched_export = biofield_repo
            .get_export(export.id, user.id)
            .await
            .expect("export fetch should succeed")
            .expect("export should exist");
        assert_eq!(fetched_export.reading_id, first_reading_id);
        assert_eq!(fetched_export.baseline_id, Some(baseline.id));
        assert_eq!(fetched_export.export_format, "json");

        let missing_export = biofield_repo
            .get_export(export.id, Uuid::new_v4())
            .await
            .expect("cross-user export fetch should not error");
        assert!(missing_export.is_none());

        sqlx::query("DELETE FROM biofield_exports WHERE id = $1")
            .bind(export.id)
            .execute(&pool)
            .await
            .expect("cleanup biofield_exports");
        sqlx::query("DELETE FROM biofield_baseline_readings WHERE baseline_id = $1")
            .bind(baseline.id)
            .execute(&pool)
            .await
            .expect("cleanup biofield_baseline_readings");
        sqlx::query("DELETE FROM biofield_baselines WHERE id = $1")
            .bind(baseline.id)
            .execute(&pool)
            .await
            .expect("cleanup biofield_baselines");
        sqlx::query("DELETE FROM readings WHERE id = $1 OR id = $2")
            .bind(first_reading_id)
            .bind(second_reading_id)
            .execute(&pool)
            .await
            .expect("cleanup readings");
        sqlx::query("DELETE FROM biofield_sessions WHERE id = $1")
            .bind(session.id)
            .execute(&pool)
            .await
            .expect("cleanup biofield_sessions");
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .expect("cleanup users");
    }
}
