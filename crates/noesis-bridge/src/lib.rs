//! Noesis Bridge -- HTTP adapter for TypeScript consciousness engines
//!
//! Wraps TypeScript engines (running as Bun HTTP servers) behind the
//! `ConsciousnessEngine` trait so the orchestrator can treat all engines uniformly.

use std::sync::Arc;

use async_trait::async_trait;
use sha2::{Digest, Sha256};
use tracing::{debug, warn};

pub use noesis_core::{
    ConsciousnessEngine, EngineError, EngineInput, EngineOutput, ValidationResult,
};

// ---------------------------------------------------------------------------
// BridgeEngine
// ---------------------------------------------------------------------------

/// HTTP adapter that proxies trait calls to a TypeScript engine running on Bun.
///
/// Each instance targets a single engine endpoint on the Bun server.
/// All HTTP errors are mapped to `EngineError::BridgeError`.
pub struct BridgeEngine {
    engine_id: String,
    engine_name: String,
    required_phase: u8,
    base_url: String,
    client: reqwest::Client,
}

impl BridgeEngine {
    /// Create a new bridge to a TypeScript engine.
    ///
    /// * `engine_id`      - Unique engine identifier (e.g. `"tarot"`)
    /// * `engine_name`    - Human-readable name (e.g. `"Tarot"`)
    /// * `required_phase` - Minimum consciousness phase (0-5)
    /// * `base_url`       - Root URL of the Bun server (e.g. `"http://localhost:3001"`)
    pub fn new(
        engine_id: impl Into<String>,
        engine_name: impl Into<String>,
        required_phase: u8,
        base_url: impl Into<String>,
    ) -> Self {
        Self {
            engine_id: engine_id.into(),
            engine_name: engine_name.into(),
            required_phase,
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl ConsciousnessEngine for BridgeEngine {
    fn engine_id(&self) -> &str {
        &self.engine_id
    }

    fn engine_name(&self) -> &str {
        &self.engine_name
    }

    fn required_phase(&self) -> u8 {
        self.required_phase
    }

    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        let url = format!(
            "{}/engines/{}/calculate",
            self.base_url, self.engine_id
        );

        debug!(engine = %self.engine_id, %url, "bridge calculate request");

        let response = self
            .client
            .post(&url)
            .json(&input)
            .send()
            .await
            .map_err(|e| EngineError::BridgeError(format!(
                "HTTP request to {} failed: {}", url, e
            )))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!(
                engine = %self.engine_id,
                %status,
                %body,
                "bridge calculate returned non-2xx"
            );
            return Err(EngineError::BridgeError(format!(
                "Engine {} returned {}: {}", self.engine_id, status, body
            )));
        }

        response.json::<EngineOutput>().await.map_err(|e| {
            EngineError::BridgeError(format!(
                "Failed to deserialize EngineOutput from {}: {}", self.engine_id, e
            ))
        })
    }

    async fn validate(&self, output: &EngineOutput) -> Result<ValidationResult, EngineError> {
        let url = format!(
            "{}/engines/{}/validate",
            self.base_url, self.engine_id
        );

        debug!(engine = %self.engine_id, %url, "bridge validate request");

        let response = self
            .client
            .post(&url)
            .json(output)
            .send()
            .await
            .map_err(|e| EngineError::BridgeError(format!(
                "HTTP request to {} failed: {}", url, e
            )))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!(
                engine = %self.engine_id,
                %status,
                %body,
                "bridge validate returned non-2xx"
            );
            return Err(EngineError::BridgeError(format!(
                "Engine {} validate returned {}: {}", self.engine_id, status, body
            )));
        }

        response.json::<ValidationResult>().await.map_err(|e| {
            EngineError::BridgeError(format!(
                "Failed to deserialize ValidationResult from {}: {}", self.engine_id, e
            ))
        })
    }

    fn cache_key(&self, input: &EngineInput) -> String {
        let input_json = serde_json::to_string(input).unwrap_or_default();
        let raw = format!("{}:{}", self.engine_id, input_json);
        let hash = format!("{:x}", Sha256::digest(raw.as_bytes()));
        format!("{}:{}", self.engine_id, hash)
    }
}

// ---------------------------------------------------------------------------
// BridgeManager
// ---------------------------------------------------------------------------

/// Manages the set of TypeScript engines accessible through the HTTP bridge.
///
/// Provides factory construction of `BridgeEngine` instances for the five
/// TypeScript-based consciousness engines and a health-check endpoint.
pub struct BridgeManager {
    base_url: String,
    engines: Vec<Arc<dyn ConsciousnessEngine>>,
}

impl BridgeManager {
    /// Create a new manager pointing at the given Bun server root URL.
    ///
    /// Instantiates `BridgeEngine` wrappers for all five TypeScript engines.
    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url: String = base_url.into();

        let engines: Vec<Arc<dyn ConsciousnessEngine>> = vec![
            Arc::new(BridgeEngine::new("tarot", "Tarot", 0, &base_url)),
            Arc::new(BridgeEngine::new("i-ching", "I Ching", 1, &base_url)),
            Arc::new(BridgeEngine::new("enneagram", "Enneagram", 1, &base_url)),
            Arc::new(BridgeEngine::new(
                "sacred-geometry",
                "Sacred Geometry",
                2,
                &base_url,
            )),
            Arc::new(BridgeEngine::new("sigil-forge", "Sigil Forge", 3, &base_url)),
        ];

        Self { base_url, engines }
    }

    /// Return all bridged engines as trait objects.
    pub fn engines(&self) -> Vec<Arc<dyn ConsciousnessEngine>> {
        self.engines.clone()
    }

    /// Ping the Bun server health endpoint.
    ///
    /// Returns `Ok(())` when the server responds with 2xx, or an
    /// `EngineError::BridgeError` on failure.
    pub async fn health_check(&self) -> Result<(), EngineError> {
        let url = format!("{}/health", self.base_url);

        let response = reqwest::get(&url).await.map_err(|e| {
            EngineError::BridgeError(format!("Health check failed for {}: {}", url, e))
        })?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(EngineError::BridgeError(format!(
                "Health check returned {}", response.status()
            )))
        }
    }
}
