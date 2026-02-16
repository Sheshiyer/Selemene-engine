//! Noesis API Client — Typed HTTP client for Selemene Engine
//!
//! Provides a high-level async client for calling all 16 engines and 6 workflows.

use crate::{Config, Error, Result};
use noesis_core::{EngineInput, EngineOutput, WorkflowResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, instrument};

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

/// High-level client for the Selemene Engine API.
///
/// Handles authentication, request formatting, and response parsing.
#[derive(Clone)]
pub struct NoesisClient {
    http: Client,
    base_url: String,
    api_key: Option<String>,
}

impl NoesisClient {
    /// Create a new client with the given configuration.
    pub fn new(config: &Config) -> Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(Error::Http)?;

        Ok(Self {
            http,
            base_url: config.api_url.clone(),
            api_key: config.api_key.clone(),
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

        let mut request = self.http.post(&url).json(&input);

        if let Some(ref key) = self.api_key {
            request = request.header("X-API-Key", key);
        }

        let response = request.send().await?;
        self.handle_response(response).await
    }

    /// Execute a multi-engine workflow.
    #[instrument(skip(self, input), fields(workflow_id = %workflow_id))]
    pub async fn workflow(
        &self,
        workflow_id: &str,
        input: EngineInput,
    ) -> Result<WorkflowResult> {
        let url = format!(
            "{}/api/v1/workflows/{}/execute",
            self.base_url, workflow_id
        );
        debug!("Executing workflow: {}", workflow_id);

        let mut request = self.http.post(&url).json(&input);

        if let Some(ref key) = self.api_key {
            request = request.header("X-API-Key", key);
        }

        let response = request.send().await?;
        self.handle_response(response).await
    }

    /// List all available engines.
    #[instrument(skip(self))]
    pub async fn list_engines(&self) -> Result<Vec<EngineInfo>> {
        let url = format!("{}/api/v1/engines", self.base_url);
        let response = self.http.get(&url).send().await?;
        self.handle_response(response).await
    }

    /// List all available workflows.
    #[instrument(skip(self))]
    pub async fn list_workflows(&self) -> Result<Vec<WorkflowInfo>> {
        let url = format!("{}/api/v1/workflows", self.base_url);
        let response = self.http.get(&url).send().await?;
        self.handle_response(response).await
    }

    /// Get past readings for the authenticated user.
    #[instrument(skip(self))]
    pub async fn list_readings(&self, limit: Option<u32>) -> Result<Vec<ReadingRecord>> {
        let mut url = format!("{}/api/v1/readings", self.base_url);
        if let Some(l) = limit {
            url.push_str(&format!("?limit={}", l));
        }

        let mut request = self.http.get(&url);
        if let Some(ref key) = self.api_key {
            request = request.header("X-API-Key", key);
        }

        let response = request.send().await?;
        self.handle_response(response).await
    }

    /// Get a specific reading by ID.
    #[instrument(skip(self))]
    pub async fn get_reading(&self, reading_id: &str) -> Result<ReadingRecord> {
        let url = format!("{}/api/v1/readings/{}", self.base_url, reading_id);

        let mut request = self.http.get(&url);
        if let Some(ref key) = self.api_key {
            request = request.header("X-API-Key", key);
        }

        let response = request.send().await?;
        self.handle_response(response).await
    }

    /// Health check — returns true if the API is reachable.
    pub async fn health(&self) -> bool {
        let url = format!("{}/health/live", self.base_url);
        self.http.get(&url).send().await.is_ok()
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
            let message = response.text().await.unwrap_or_else(|_| "Unknown error".into());
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

        let http = Client::builder()
            .timeout(Duration::from_millis(timeout))
            .build()
            .map_err(Error::Http)?;

        Ok(NoesisClient {
            http,
            base_url,
            api_key: self.api_key,
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

/// A stored reading record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingRecord {
    pub id: String,
    pub engine_id: String,
    pub output: EngineOutput,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_builder_defaults() {
        let client = NoesisClient::builder().build().unwrap();
        assert_eq!(client.base_url, "https://selemene.tryambakam.space");
        assert!(client.api_key.is_none());
    }

    #[test]
    fn test_builder_custom() {
        let client = NoesisClient::builder()
            .base_url("http://localhost:8080")
            .api_key("test_key")
            .timeout_ms(5000)
            .build()
            .unwrap();

        assert_eq!(client.base_url, "http://localhost:8080");
        assert_eq!(client.api_key, Some("test_key".into()));
    }
}
