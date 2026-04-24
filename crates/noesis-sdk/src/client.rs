//! Noesis API Client — Typed HTTP client for Selemene Engine
//!
//! Provides a high-level async client for calling all 16 engines and 6 workflows.

use crate::{Config, Error, Result};
use noesis_core::{EngineInput, EngineOutput, WorkflowResult};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, instrument, warn};

/// List of all available engines
pub const ENGINES: &[&str] = &[
    "panchanga",
    "numerology",
    "biorhythm",
    "human-design",
    "gene-keys",
    "vimshottari",
    "vedic-clock",
    "biofield",
    "face-reading",
    "nadabrahman",
    "transits",
    // TypeScript engines
    "tarot",
    "i-ching",
    "enneagram",
    "sacred-geometry",
    "sigil-forge",
];

/// List of all available workflows
pub const WORKFLOWS: &[&str] = &[
    "birth-blueprint",
    "daily-practice",
    "decision-support",
    "self-inquiry",
    "creative-expression",
    "full-spectrum",
];

/// Canonical tarot spread identifiers accepted by the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TarotSpread {
    SingleCard,
    ThreeCard,
    CelticCross,
    Horseshoe,
    Relationship,
    Career,
    YesNo,
}

impl TarotSpread {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SingleCard => "single_card",
            Self::ThreeCard => "three_card",
            Self::CelticCross => "celtic_cross",
            Self::Horseshoe => "horseshoe",
            Self::Relationship => "relationship",
            Self::Career => "career",
            Self::YesNo => "yes_no",
        }
    }
}

/// High-level client for the Selemene Engine API.
///
/// Handles authentication, request formatting, and response parsing.
#[derive(Clone)]
pub struct NoesisClient {
    http: Client,
    base_url: String,
    api_key: Option<String>,
    max_retries: u32,
    backoff_ms: u64,
}

impl NoesisClient {
    /// Create a new client with the given configuration.
    pub fn new(config: &Config) -> Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .pool_max_idle_per_host(config.pool_max_idle_per_host)
            .build()
            .map_err(Error::Http)?;

        Ok(Self {
            http,
            base_url: config.api_url.clone(),
            api_key: config.api_key.clone(),
            max_retries: config.max_retries,
            backoff_ms: config.backoff_ms,
        })
    }

    /// Create a builder for more advanced configuration.
    pub fn builder() -> NoesisClientBuilder {
        NoesisClientBuilder::default()
    }

    /// Set the API key for authenticated requests.
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Calculate using a specific engine.
    #[instrument(skip(self, input), fields(engine_id = %engine_id))]
    pub async fn calculate(&self, engine_id: &str, input: EngineInput) -> Result<EngineOutput> {
        let url = format!("{}/api/v1/engines/{}/calculate", self.base_url, engine_id);
        debug!("Calling engine: {}", engine_id);

        self.send_with_retry(|| self.authenticated(self.http.post(&url).json(&input)))
            .await
    }

    /// Calculate a tarot reading with typed spread options.
    #[instrument(skip(self, input, question), fields(spread = %spread.as_str()))]
    pub async fn calculate_tarot(
        &self,
        input: EngineInput,
        question: impl Into<String>,
        spread: TarotSpread,
    ) -> Result<EngineOutput> {
        let mut tarot_input = input;
        let mut options = tarot_input.options;

        options.insert("question".to_string(), serde_json::Value::String(question.into()));
        options.insert(
            "spread".to_string(),
            serde_json::Value::String(spread.as_str().to_string()),
        );

        tarot_input.options = options;
        self.calculate("tarot", tarot_input).await
    }

    /// Execute a multi-engine workflow.
    #[instrument(skip(self, input), fields(workflow_id = %workflow_id))]
    pub async fn workflow(&self, workflow_id: &str, input: EngineInput) -> Result<WorkflowResult> {
        let url = format!("{}/api/v1/workflows/{}/execute", self.base_url, workflow_id);
        debug!("Executing workflow: {}", workflow_id);

        self.send_with_retry(|| self.authenticated(self.http.post(&url).json(&input)))
            .await
    }

    /// List all available engines.
    #[instrument(skip(self))]
    pub async fn list_engines(&self) -> Result<Vec<EngineInfo>> {
        let url = format!("{}/api/v1/engines", self.base_url);
        self.send_with_retry(|| self.http.get(&url)).await
    }

    /// List all available workflows.
    #[instrument(skip(self))]
    pub async fn list_workflows(&self) -> Result<Vec<WorkflowInfo>> {
        let url = format!("{}/api/v1/workflows", self.base_url);
        self.send_with_retry(|| self.http.get(&url)).await
    }

    /// Get past readings for the authenticated user.
    #[instrument(skip(self))]
    pub async fn list_readings(&self, limit: Option<u32>) -> Result<Vec<ReadingRecord>> {
        let mut url = format!("{}/api/v1/readings", self.base_url);
        if let Some(l) = limit {
            url.push_str(&format!("?limit={}", l));
        }

        let wrapper: ReadingsListResponse = self
            .send_with_retry(|| self.authenticated(self.http.get(&url)))
            .await?;
        Ok(wrapper.readings)
    }

    /// Get a specific reading by ID.
    #[instrument(skip(self))]
    pub async fn get_reading(&self, reading_id: &str) -> Result<ReadingRecord> {
        let url = format!("{}/api/v1/readings/{}", self.base_url, reading_id);
        self.send_with_retry(|| self.authenticated(self.http.get(&url)))
            .await
    }

    /// Health check — returns true if the API is reachable.
    pub async fn health(&self) -> bool {
        let url = format!("{}/health/live", self.base_url);
        self.http.get(&url).send().await.is_ok()
    }

    /// Get the authenticated user's server-side profile.
    /// Returns consciousness_level and experience_points from Supabase.
    #[instrument(skip(self))]
    pub async fn get_me(&self) -> Result<UserProfile> {
        let url = format!("{}/api/v1/users/me", self.base_url);
        self.send_with_retry(|| self.authenticated(self.http.get(&url)))
            .await
    }

    /// Update the authenticated user's server-side profile.
    #[instrument(skip(self, payload))]
    pub async fn update_me(&self, payload: &UpdateUserRequest) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/users/me", self.base_url);
        self.send_with_retry(|| self.authenticated(self.http.patch(&url).json(payload)))
            .await
    }

    fn authenticated(&self, request: RequestBuilder) -> RequestBuilder {
        if let Some(ref key) = self.api_key {
            request.header("X-API-Key", key)
        } else {
            request
        }
    }

    async fn send_with_retry<T, F>(&self, mut make_request: F) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
        F: FnMut() -> RequestBuilder,
    {
        let mut attempt = 0u32;

        loop {
            let response = make_request().send().await;

            match response {
                Ok(resp) => {
                    if resp.status().is_server_error() && attempt < self.max_retries {
                        let delay_ms = self.backoff_ms.saturating_mul(1u64 << attempt.min(20));
                        warn!(
                            status = %resp.status(),
                            attempt,
                            max_retries = self.max_retries,
                            delay_ms,
                            "Retriable server response from API"
                        );
                        sleep(Duration::from_millis(delay_ms)).await;
                        attempt += 1;
                        continue;
                    }

                    return self.handle_response(resp).await;
                }
                Err(err)
                    if (err.is_timeout() || err.is_connect() || err.is_request())
                        && attempt < self.max_retries =>
                {
                    let delay_ms = self.backoff_ms.saturating_mul(1u64 << attempt.min(20));
                    warn!(
                        error = %err,
                        attempt,
                        max_retries = self.max_retries,
                        delay_ms,
                        "Retriable HTTP transport error"
                    );
                    sleep(Duration::from_millis(delay_ms)).await;
                    attempt += 1;
                }
                Err(err) => return Err(Error::Http(err)),
            }
        }
    }

    /// Handle API response, parsing JSON or returning an error.
    async fn handle_response<T: for<'de> Deserialize<'de>>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            response.json::<T>().await.map_err(Error::Http)
        } else {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".into());
            Err(Error::Api {
                status: status.as_u16(),
                message,
            })
        }
    }
}

/// Builder for NoesisClient with advanced options.
#[derive(Default)]
pub struct NoesisClientBuilder {
    base_url: Option<String>,
    api_key: Option<String>,
    timeout_ms: u64,
    max_retries: Option<u32>,
    backoff_ms: Option<u64>,
    pool_max_idle_per_host: Option<usize>,
}

impl NoesisClientBuilder {
    /// Set the API base URL.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set the API key.
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set the request timeout in milliseconds.
    pub fn timeout_ms(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// Set maximum retry attempts for retriable errors.
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = Some(retries);
        self
    }

    /// Set base backoff in milliseconds for exponential retry strategy.
    pub fn backoff_ms(mut self, ms: u64) -> Self {
        self.backoff_ms = Some(ms);
        self
    }

    /// Set maximum idle pooled connections per host.
    pub fn pool_max_idle_per_host(mut self, pool_size: usize) -> Self {
        self.pool_max_idle_per_host = Some(pool_size);
        self
    }

    /// Build the client.
    pub fn build(self) -> Result<NoesisClient> {
        let base_url = self
            .base_url
            .unwrap_or_else(|| "https://selemene.tryambakam.space".into());

        let timeout = if self.timeout_ms > 0 {
            self.timeout_ms
        } else {
            30_000
        };

        let max_retries = self.max_retries.unwrap_or(3);
        let backoff_ms = self.backoff_ms.unwrap_or(200);
        let pool_max_idle_per_host = self.pool_max_idle_per_host.unwrap_or(16);

        let http = Client::builder()
            .timeout(Duration::from_millis(timeout))
            .pool_max_idle_per_host(pool_max_idle_per_host)
            .build()
            .map_err(Error::Http)?;

        Ok(NoesisClient {
            http,
            base_url,
            api_key: self.api_key,
            max_retries,
            backoff_ms,
        })
    }
}

/// Information about an available engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub required_phase: u8,
}

/// Information about an available workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub engine_ids: Vec<String>,
}

/// Birth location shape returned by GET /api/v1/users/me.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBirthLocation {
    pub lat: f64,
    pub lng: f64,
    #[serde(default)]
    pub name: Option<String>,
}

/// Server-side user profile from GET /api/v1/users/me.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub email: String,
    pub full_name: String,
    pub tier: String,
    pub consciousness_level: i32,
    pub experience_points: i32,
    #[serde(default)]
    pub birth_date: Option<String>,
    #[serde(default)]
    pub birth_time: Option<String>,
    #[serde(default)]
    pub birth_location: Option<UserBirthLocation>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub preferences: serde_json::Value,
}

/// Request payload for PATCH /api/v1/users/me.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_location_lat: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_location_lng: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_location_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferences: Option<serde_json::Value>,
}

impl UpdateUserRequest {
    pub fn is_empty(&self) -> bool {
        self.full_name.is_none()
            && self.email.is_none()
            && self.birth_date.is_none()
            && self.birth_time.is_none()
            && self.birth_location_lat.is_none()
            && self.birth_location_lng.is_none()
            && self.birth_location_name.is_none()
            && self.timezone.is_none()
            && self.preferences.is_none()
    }
}

/// A stored reading record from the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingRecord {
    pub id: String,
    #[serde(default)]
    pub user_id: Option<String>,
    pub engine_id: String,
    #[serde(default)]
    pub workflow_id: Option<String>,
    pub result_data: serde_json::Value,
    #[serde(default)]
    pub witness_prompt: Option<String>,
    #[serde(default)]
    pub consciousness_level: i16,
    #[serde(default)]
    pub calculation_time_ms: Option<f64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Wrapper for the paginated readings list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingsListResponse {
    pub readings: Vec<ReadingRecord>,
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_engines_list() {
        assert_eq!(ENGINES.len(), 16);
        assert!(ENGINES.contains(&"panchanga"));
        assert!(ENGINES.contains(&"tarot"));
    }

    #[test]
    fn test_workflows_list() {
        assert_eq!(WORKFLOWS.len(), 6);
        assert!(WORKFLOWS.contains(&"daily-practice"));
    }

    #[test]
    fn test_tarot_spread_strings() {
        assert_eq!(TarotSpread::SingleCard.as_str(), "single_card");
        assert_eq!(TarotSpread::ThreeCard.as_str(), "three_card");
        assert_eq!(TarotSpread::CelticCross.as_str(), "celtic_cross");
        assert_eq!(TarotSpread::Horseshoe.as_str(), "horseshoe");
        assert_eq!(TarotSpread::Relationship.as_str(), "relationship");
        assert_eq!(TarotSpread::Career.as_str(), "career");
        assert_eq!(TarotSpread::YesNo.as_str(), "yes_no");
    }

    #[test]
    fn test_builder_defaults() {
        let client = NoesisClient::builder().build().unwrap();
        assert_eq!(client.base_url, "https://selemene.tryambakam.space");
        assert!(client.api_key.is_none());
        assert_eq!(client.max_retries, 3);
        assert_eq!(client.backoff_ms, 200);
    }

    #[test]
    fn test_builder_custom() {
        let client = NoesisClient::builder()
            .base_url("http://localhost:8080")
            .api_key("test_key")
            .timeout_ms(5000)
            .max_retries(4)
            .backoff_ms(120)
            .pool_max_idle_per_host(8)
            .build()
            .unwrap();

        assert_eq!(client.base_url, "http://localhost:8080");
        assert_eq!(client.api_key, Some("test_key".into()));
        assert_eq!(client.max_retries, 4);
        assert_eq!(client.backoff_ms, 120);
    }

    #[tokio::test]
    async fn test_retry_on_503_until_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/engines/numerology/calculate"))
            .and(header("x-api-key", "test-key"))
            .respond_with(ResponseTemplate::new(503))
            .up_to_n_times(3)
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/api/v1/engines/numerology/calculate"))
            .and(header("x-api-key", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "engine_id": "numerology",
                "timestamp": "2026-03-03T00:00:00Z",
                "result": {},
                "witness_prompt": "ok",
                "consciousness_level": 1,
                "metadata": {
                    "calculation_time_ms": 10.0,
                    "backend": "native",
                    "precision_achieved": "standard",
                    "cached": false,
                    "timestamp": "2026-03-03T00:00:00Z",
                    "engine_version": "3.0.0"
                }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = NoesisClient::builder()
            .base_url(server.uri())
            .api_key("test-key")
            .max_retries(3)
            .backoff_ms(5)
            .build()
            .unwrap();

        let input = EngineInput {
            birth_data: None,
            current_time: chrono::Utc::now(),
            location: None,
            precision: noesis_core::Precision::Standard,
            options: std::collections::HashMap::new(),
        };

        let output = client.calculate("numerology", input).await.unwrap();
        assert_eq!(output.engine_id, "numerology");
    }

    #[tokio::test]
    async fn test_retry_respects_backoff_timing() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/engines"))
            .respond_with(ResponseTemplate::new(503))
            .up_to_n_times(2)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/v1/engines"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .expect(1)
            .mount(&server)
            .await;

        let client = NoesisClient::builder()
            .base_url(server.uri())
            .max_retries(2)
            .backoff_ms(30)
            .build()
            .unwrap();

        let started = std::time::Instant::now();
        let _ = client.list_engines().await.unwrap();
        let elapsed_ms = started.elapsed().as_millis() as u64;

        // Expected minimum ~30ms + 60ms exponential backoff.
        assert!(elapsed_ms >= 85, "elapsed_ms={elapsed_ms}");
    }
}
