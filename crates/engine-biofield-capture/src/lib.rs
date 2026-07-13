//! Biofield Capture Engine
//!
//! Resolves a persisted `biofield-capture` reading by `reading_id` and exposes it as a
//! workflow-consumable `EngineOutput` without changing the existing birth-data `biofield` engine.

use async_trait::async_trait;
use chrono::Utc;
use noesis_core::{
    CalculationMetadata, ConsciousnessEngine, EngineError, EngineInput, EngineOutput,
    ValidationResult,
};
use noesis_data::{
    models::reading::Reading, repositories::readings_repository::ReadingsRepository,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

pub const BIOFIELD_CAPTURE_ENGINE_ID: &str = "biofield-capture";
const BIOFIELD_CAPTURE_OPTION_NAMESPACE: &str = "biofield_capture";
const INTERNAL_AUTH_OPTION_NAMESPACE: &str = "_auth";
const REQUIRED_PHASE: u8 = 1;

pub struct BiofieldCaptureEngine {
    pool: PgPool,
}

impl BiofieldCaptureEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn extract_reading_id(input: &EngineInput) -> Result<Uuid, EngineError> {
        let raw = input
            .options
            .get(BIOFIELD_CAPTURE_OPTION_NAMESPACE)
            .and_then(Value::as_object)
            .and_then(|value| value.get("reading_id"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                EngineError::ValidationError(
                    "options.biofield_capture.reading_id is required".to_string(),
                )
            })?;

        Uuid::parse_str(raw).map_err(|_| {
            EngineError::ValidationError(
                "options.biofield_capture.reading_id must be a valid UUID".to_string(),
            )
        })
    }

    fn extract_user_id(input: &EngineInput) -> Result<Uuid, EngineError> {
        let raw = input
            .options
            .get(INTERNAL_AUTH_OPTION_NAMESPACE)
            .and_then(Value::as_object)
            .and_then(|value| value.get("user_id"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                EngineError::AuthError(
                    "Authenticated user context is required for biofield-capture lookup"
                        .to_string(),
                )
            })?;

        Uuid::parse_str(raw).map_err(|_| {
            EngineError::AuthError(
                "Authenticated user context contained an invalid user_id".to_string(),
            )
        })
    }

    fn build_result(reading: &Reading) -> Value {
        json!({
            "available": true,
            "reading_id": reading.id,
            "engine_id": reading.engine_id,
            "created_at": reading.created_at,
            "session_id": reading.input_data.get("session_id").cloned().unwrap_or(Value::Null),
            "analysis_version": reading.result_data.get("analysis_version").cloned().unwrap_or(Value::Null),
            "contract_version": reading.result_data.get("contract_version").cloned().unwrap_or(Value::Null),
            "quality_assessment": reading.result_data.get("quality_assessment").cloned().unwrap_or_else(|| json!({})),
            "input": reading.input_data,
            "analysis": reading.result_data,
        })
    }

    fn build_witness_prompt(reading: &Reading) -> String {
        if let Some(prompt) = reading
            .witness_prompt
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return prompt.to_string();
        }

        let sufficient_quality = reading
            .result_data
            .get("quality_assessment")
            .and_then(|value| value.get("sufficient_quality"))
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let analysis_version = reading
            .result_data
            .get("analysis_version")
            .and_then(Value::as_str)
            .unwrap_or("captured analysis");

        if sufficient_quality {
            format!(
                "What stands out as you witness this captured field state from {}?",
                analysis_version
            )
        } else {
            "What do you notice about the conditions around this capture when you witness them without trying to improve them?".to_string()
        }
    }
}

#[async_trait]
impl ConsciousnessEngine for BiofieldCaptureEngine {
    fn engine_id(&self) -> &str {
        BIOFIELD_CAPTURE_ENGINE_ID
    }

    fn engine_name(&self) -> &str {
        "Biofield Capture"
    }

    fn required_phase(&self) -> u8 {
        REQUIRED_PHASE
    }

    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        let start = std::time::Instant::now();
        let reading_id = Self::extract_reading_id(&input)?;
        let user_id = Self::extract_user_id(&input)?;
        let repository = ReadingsRepository::new(self.pool.clone());

        let reading = repository
            .get_reading(reading_id, user_id)
            .await
            .map_err(|error| {
                EngineError::ServiceUnavailable(format!(
                    "Failed to load persisted biofield capture reading: {error}"
                ))
            })?
            .ok_or_else(|| {
                EngineError::ValidationError(
                    "Persisted biofield-capture reading not found or not accessible".to_string(),
                )
            })?;

        if reading.engine_id != BIOFIELD_CAPTURE_ENGINE_ID {
            return Err(EngineError::ValidationError(format!(
                "reading_id does not reference a {} reading",
                BIOFIELD_CAPTURE_ENGINE_ID
            )));
        }

        Ok(EngineOutput {
            engine_id: BIOFIELD_CAPTURE_ENGINE_ID.to_string(),
            result: Self::build_result(&reading),
            witness_prompt: Self::build_witness_prompt(&reading),
            consciousness_level: reading.consciousness_level.max(0) as u8,
            metadata: CalculationMetadata {
                calculation_time_ms: start.elapsed().as_secs_f64() * 1000.0,
                backend: "persisted-reading".to_string(),
                precision_achieved: "Persisted".to_string(),
                cached: false,
                timestamp: Utc::now(),
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        })
    }

    async fn validate(&self, output: &EngineOutput) -> Result<ValidationResult, EngineError> {
        let has_reading_id = output
            .result
            .get("reading_id")
            .and_then(Value::as_str)
            .is_some();
        let has_analysis = output.result.get("analysis").is_some();

        Ok(ValidationResult {
            valid: output.engine_id == BIOFIELD_CAPTURE_ENGINE_ID && has_reading_id && has_analysis,
            confidence: if has_reading_id && has_analysis {
                1.0
            } else {
                0.0
            },
            messages: if has_reading_id && has_analysis {
                vec!["biofield-capture output shape is valid".to_string()]
            } else {
                vec!["biofield-capture output is missing required fields".to_string()]
            },
        })
    }

    fn cache_key(&self, input: &EngineInput) -> String {
        let reading_id = input
            .options
            .get(BIOFIELD_CAPTURE_OPTION_NAMESPACE)
            .and_then(Value::as_object)
            .and_then(|value| value.get("reading_id"))
            .and_then(Value::as_str)
            .unwrap_or("missing-reading-id");
        let user_id = input
            .options
            .get(INTERNAL_AUTH_OPTION_NAMESPACE)
            .and_then(Value::as_object)
            .and_then(|value| value.get("user_id"))
            .and_then(Value::as_str)
            .unwrap_or("missing-user-id");

        let mut hasher = Sha256::new();
        hasher.update(BIOFIELD_CAPTURE_ENGINE_ID.as_bytes());
        hasher.update(reading_id.as_bytes());
        hasher.update(user_id.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noesis_data::{models::reading::NewReading, repositories::user_repository::UserRepository};
    use serde_json::json;
    use sqlx::postgres::PgPoolOptions;

    fn dummy_pool() -> PgPool {
        PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://localhost/noesis_test")
            .expect("lazy pool should build")
    }

    fn capture_input(reading_id: Option<&str>, user_id: Option<&str>) -> EngineInput {
        let mut options = serde_json::Map::new();
        if let Some(reading_id) = reading_id {
            options.insert(
                BIOFIELD_CAPTURE_OPTION_NAMESPACE.to_string(),
                json!({ "reading_id": reading_id }),
            );
        }
        if let Some(user_id) = user_id {
            options.insert(
                INTERNAL_AUTH_OPTION_NAMESPACE.to_string(),
                json!({ "user_id": user_id }),
            );
        }

        EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: noesis_core::Precision::Standard,
            options: options.into_iter().collect(),
        }
    }

    async fn connect_test_pool() -> Option<PgPool> {
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

    async fn create_user(pool: &PgPool, label: &str) -> noesis_data::models::user::User {
        let repo = UserRepository::new(pool.clone());
        let email = format!(
            "biofield-capture-engine-{label}-{}@example.com",
            Uuid::new_v4()
        );
        repo.create_user(
            &email,
            "test_password_hash",
            "Biofield Capture Engine Test User",
        )
        .await
        .expect("test user should be created")
    }

    async fn create_reading(pool: &PgPool, user_id: Uuid, engine_id: &str) -> Uuid {
        let repo = ReadingsRepository::new(pool.clone());
        repo.save_reading(&NewReading {
            user_id,
            engine_id: engine_id.to_string(),
            workflow_id: None,
            input_hash: format!("bfce-{}", Uuid::new_v4().simple()),
            input_data: json!({
                "session_id": Uuid::new_v4(),
                "content_type": "image/jpeg"
            }),
            result_data: json!({
                "contract_version": "biofield-cv/v1",
                "analysis_version": "stub-metrics/v1",
                "quality_assessment": { "sufficient_quality": true },
                "metrics": { "light_quanta_density": 42.0 }
            }),
            witness_prompt: Some(
                "What do you notice as you witness this stored capture?".to_string(),
            ),
            consciousness_level: 1,
            calculation_time_ms: Some(7.5),
            client_event_id: None,
            client_device_id: None,
            device_platform: None,
            device_app_version: None,
        })
        .await
        .expect("reading should be created")
    }

    #[tokio::test]
    async fn missing_reading_id_is_validation_error() {
        let engine = BiofieldCaptureEngine::new(dummy_pool());
        let input = capture_input(None, Some(&Uuid::new_v4().to_string()));

        let error = engine
            .calculate(input)
            .await
            .expect_err("missing reading_id should fail");
        assert!(matches!(error, EngineError::ValidationError(_)));
        assert!(error
            .to_string()
            .contains("options.biofield_capture.reading_id"));
    }

    #[tokio::test]
    async fn missing_user_context_is_auth_error() {
        let engine = BiofieldCaptureEngine::new(dummy_pool());
        let input = capture_input(Some(&Uuid::new_v4().to_string()), None);

        let error = engine
            .calculate(input)
            .await
            .expect_err("missing auth should fail");
        assert!(matches!(error, EngineError::AuthError(_)));
    }

    #[tokio::test]
    async fn malformed_reading_id_is_validation_error() {
        let engine = BiofieldCaptureEngine::new(dummy_pool());
        let input = capture_input(Some("not-a-uuid"), Some(&Uuid::new_v4().to_string()));

        let error = engine
            .calculate(input)
            .await
            .expect_err("bad reading_id should fail");
        assert!(matches!(error, EngineError::ValidationError(_)));
    }

    #[tokio::test]
    async fn resolves_persisted_biofield_capture_reading() {
        let Some(pool) = connect_test_pool().await else {
            return;
        };

        let user = create_user(&pool, "owner").await;
        let reading_id = create_reading(&pool, user.id, BIOFIELD_CAPTURE_ENGINE_ID).await;
        let engine = BiofieldCaptureEngine::new(pool.clone());
        let input = capture_input(Some(&reading_id.to_string()), Some(&user.id.to_string()));

        let output = engine
            .calculate(input)
            .await
            .expect("reading lookup should succeed");
        assert_eq!(output.engine_id, BIOFIELD_CAPTURE_ENGINE_ID);
        assert_eq!(output.result["reading_id"], reading_id.to_string());
        assert_eq!(
            output.result["analysis"]["analysis_version"],
            "stub-metrics/v1"
        );
        assert!(!output.witness_prompt.trim().is_empty());

        sqlx::query("DELETE FROM readings WHERE id = $1")
            .bind(reading_id)
            .execute(&pool)
            .await
            .expect("cleanup reading");
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .expect("cleanup user");
    }

    #[tokio::test]
    async fn rejects_foreign_user_reading_lookup() {
        let Some(pool) = connect_test_pool().await else {
            return;
        };

        let owner = create_user(&pool, "owner-foreign").await;
        let other = create_user(&pool, "other-foreign").await;
        let reading_id = create_reading(&pool, owner.id, BIOFIELD_CAPTURE_ENGINE_ID).await;
        let engine = BiofieldCaptureEngine::new(pool.clone());
        let input = capture_input(Some(&reading_id.to_string()), Some(&other.id.to_string()));

        let error = engine
            .calculate(input)
            .await
            .expect_err("foreign reading should fail");
        assert!(matches!(error, EngineError::ValidationError(_)));
        assert!(error.to_string().contains("not found or not accessible"));

        sqlx::query("DELETE FROM readings WHERE id = $1")
            .bind(reading_id)
            .execute(&pool)
            .await
            .expect("cleanup reading");
        sqlx::query("DELETE FROM users WHERE id = $1 OR id = $2")
            .bind(owner.id)
            .bind(other.id)
            .execute(&pool)
            .await
            .expect("cleanup users");
    }

    #[tokio::test]
    async fn rejects_non_biofield_capture_reading() {
        let Some(pool) = connect_test_pool().await else {
            return;
        };

        let user = create_user(&pool, "wrong-engine").await;
        let reading_id = create_reading(&pool, user.id, "numerology").await;
        let engine = BiofieldCaptureEngine::new(pool.clone());
        let input = capture_input(Some(&reading_id.to_string()), Some(&user.id.to_string()));

        let error = engine
            .calculate(input)
            .await
            .expect_err("wrong engine_id should fail");
        assert!(matches!(error, EngineError::ValidationError(_)));
        assert!(error.to_string().contains(BIOFIELD_CAPTURE_ENGINE_ID));

        sqlx::query("DELETE FROM readings WHERE id = $1")
            .bind(reading_id)
            .execute(&pool)
            .await
            .expect("cleanup reading");
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .expect("cleanup user");
    }
}
