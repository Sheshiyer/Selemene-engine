//! Noesis Bridge -- HTTP adapter for TypeScript consciousness engines
//!
//! Wraps TypeScript engines (running as Bun HTTP servers) behind the
//! `ConsciousnessEngine` trait so the orchestrator can treat all engines uniformly.
//!
//! # Usage
//!
//! ```rust,no_run
//! use noesis_bridge::BridgeEngine;
//!
//! // Create a tarot engine with default settings
//! let tarot = BridgeEngine::tarot();
//!
//! // Or with custom URL
//! let tarot = BridgeEngine::tarot_with_url("http://custom:3001");
//! ```

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

pub mod error;
pub use error::BridgeError;

pub use noesis_core::{
    ConsciousnessEngine, EngineError, EngineInput, EngineOutput, ValidationResult,
};

/// Default URL for the TypeScript engines server.
pub const DEFAULT_TS_SERVER_URL: &str = "http://localhost:3001";

/// Default timeout for HTTP requests in seconds.
pub const DEFAULT_TIMEOUT_SECS: u64 = 5;

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
    timeout: Duration,
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
        Self::with_timeout(
            engine_id,
            engine_name,
            required_phase,
            base_url,
            Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        )
    }

    /// Create a new bridge with a custom timeout.
    pub fn with_timeout(
        engine_id: impl Into<String>,
        engine_name: impl Into<String>,
        required_phase: u8,
        base_url: impl Into<String>,
        timeout: Duration,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .connect_timeout(Duration::from_secs(2))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            engine_id: engine_id.into(),
            engine_name: engine_name.into(),
            required_phase,
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            client,
            timeout,
        }
    }

    // -------------------------------------------------------------------------
    // Factory methods for TypeScript engines
    // -------------------------------------------------------------------------

    /// Create a Tarot engine bridge with default URL.
    pub fn tarot() -> Self {
        Self::tarot_with_url(DEFAULT_TS_SERVER_URL)
    }

    /// Create a Tarot engine bridge with custom URL.
    pub fn tarot_with_url(base_url: impl Into<String>) -> Self {
        Self::new("tarot", "Tarot", 0, base_url)
    }

    /// Create an I Ching engine bridge with default URL.
    pub fn i_ching() -> Self {
        Self::i_ching_with_url(DEFAULT_TS_SERVER_URL)
    }

    /// Create an I Ching engine bridge with custom URL.
    pub fn i_ching_with_url(base_url: impl Into<String>) -> Self {
        Self::new("i-ching", "I Ching", 1, base_url)
    }

    /// Create an Enneagram engine bridge with default URL.
    pub fn enneagram() -> Self {
        Self::enneagram_with_url(DEFAULT_TS_SERVER_URL)
    }

    /// Create an Enneagram engine bridge with custom URL.
    pub fn enneagram_with_url(base_url: impl Into<String>) -> Self {
        Self::new("enneagram", "Enneagram", 1, base_url)
    }

    /// Create a Sacred Geometry engine bridge with default URL.
    pub fn sacred_geometry() -> Self {
        Self::sacred_geometry_with_url(DEFAULT_TS_SERVER_URL)
    }

    /// Create a Sacred Geometry engine bridge with custom URL.
    pub fn sacred_geometry_with_url(base_url: impl Into<String>) -> Self {
        Self::new("sacred-geometry", "Sacred Geometry", 2, base_url)
    }

    /// Create a Sigil Forge engine bridge with default URL.
    pub fn sigil_forge() -> Self {
        Self::sigil_forge_with_url(DEFAULT_TS_SERVER_URL)
    }

    /// Create a Sigil Forge engine bridge with custom URL.
    pub fn sigil_forge_with_url(base_url: impl Into<String>) -> Self {
        Self::new("sigil-forge", "Sigil Forge", 3, base_url)
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

        debug!(
            engine = %self.engine_id,
            %url,
            timeout_secs = self.timeout.as_secs(),
            "bridge calculate request"
        );

        let response = self
            .client
            .post(&url)
            .json(&input)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    warn!(engine = %self.engine_id, "Bridge request timed out");
                    EngineError::BridgeError(format!(
                        "Request to {} timed out after {}s",
                        self.engine_id,
                        self.timeout.as_secs()
                    ))
                } else if e.is_connect() {
                    warn!(engine = %self.engine_id, %url, "Bridge connection refused");
                    EngineError::BridgeError(format!(
                        "Connection to {} refused (is the TS server running at {}?)",
                        self.engine_id,
                        self.base_url
                    ))
                } else {
                    warn!(engine = %self.engine_id, error = %e, "Bridge HTTP error");
                    EngineError::BridgeError(format!(
                        "HTTP request to {} failed: {}",
                        url, e
                    ))
                }
            })?;

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

        info!(engine = %self.engine_id, "Bridge calculate succeeded");
        
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
            .map_err(|e| {
                if e.is_timeout() {
                    EngineError::BridgeError(format!(
                        "Validate request to {} timed out after {}s",
                        self.engine_id,
                        self.timeout.as_secs()
                    ))
                } else if e.is_connect() {
                    EngineError::BridgeError(format!(
                        "Connection to {} refused (is the TS server running at {}?)",
                        self.engine_id,
                        self.base_url
                    ))
                } else {
                    EngineError::BridgeError(format!(
                        "HTTP request to {} failed: {}",
                        url, e
                    ))
                }
            })?;

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
            Arc::new(BridgeEngine::tarot_with_url(&base_url)),
            Arc::new(BridgeEngine::i_ching_with_url(&base_url)),
            Arc::new(BridgeEngine::enneagram_with_url(&base_url)),
            Arc::new(BridgeEngine::sacred_geometry_with_url(&base_url)),
            Arc::new(BridgeEngine::sigil_forge_with_url(&base_url)),
        ];

        info!(
            base_url = %base_url,
            engine_count = engines.len(),
            "BridgeManager initialized"
        );

        Self { base_url, engines }
    }

    /// Create a new manager using the `TS_ENGINES_URL` environment variable,
    /// or falling back to the default URL if not set.
    pub fn from_env() -> Self {
        let url = std::env::var("TS_ENGINES_URL")
            .unwrap_or_else(|_| DEFAULT_TS_SERVER_URL.to_string());
        info!(url = %url, "BridgeManager loading from environment");
        Self::new(url)
    }

    /// Return all bridged engines as trait objects.
    pub fn engines(&self) -> Vec<Arc<dyn ConsciousnessEngine>> {
        self.engines.clone()
    }

    /// Get the base URL this manager is configured to use.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Ping the Bun server health endpoint.
    ///
    /// Returns `Ok(())` when the server responds with 2xx, or an
    /// `EngineError::BridgeError` on failure.
    pub async fn health_check(&self) -> Result<(), EngineError> {
        let url = format!("{}/health", self.base_url);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .map_err(|e| EngineError::BridgeError(format!("Failed to create client: {}", e)))?;

        let response = client.get(&url).send().await.map_err(|e| {
            if e.is_connect() {
                EngineError::BridgeError(format!(
                    "TS server not reachable at {} (connection refused)",
                    self.base_url
                ))
            } else if e.is_timeout() {
                EngineError::BridgeError(format!(
                    "TS server health check timed out at {}",
                    self.base_url
                ))
            } else {
                EngineError::BridgeError(format!("Health check failed for {}: {}", url, e))
            }
        })?;

        if response.status().is_success() {
            info!(url = %self.base_url, "TS server health check passed");
            Ok(())
        } else {
            Err(EngineError::BridgeError(format!(
                "Health check returned {}", response.status()
            )))
        }
    }

    /// Check if the TS server is available (non-blocking, returns false on error).
    pub async fn is_available(&self) -> bool {
        self.health_check().await.is_ok()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use chrono::Utc;

    fn test_input() -> EngineInput {
        EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: noesis_core::Precision::Standard,
            options: HashMap::new(),
        }
    }

    #[test]
    fn bridge_engine_factory_tarot() {
        let engine = BridgeEngine::tarot();
        assert_eq!(engine.engine_id(), "tarot");
        assert_eq!(engine.engine_name(), "Tarot");
        assert_eq!(engine.required_phase(), 0);
    }

    #[test]
    fn bridge_engine_factory_i_ching() {
        let engine = BridgeEngine::i_ching();
        assert_eq!(engine.engine_id(), "i-ching");
        assert_eq!(engine.engine_name(), "I Ching");
        assert_eq!(engine.required_phase(), 1);
    }

    #[test]
    fn bridge_engine_factory_enneagram() {
        let engine = BridgeEngine::enneagram();
        assert_eq!(engine.engine_id(), "enneagram");
        assert_eq!(engine.engine_name(), "Enneagram");
        assert_eq!(engine.required_phase(), 1);
    }

    #[test]
    fn bridge_engine_factory_sacred_geometry() {
        let engine = BridgeEngine::sacred_geometry();
        assert_eq!(engine.engine_id(), "sacred-geometry");
        assert_eq!(engine.engine_name(), "Sacred Geometry");
        assert_eq!(engine.required_phase(), 2);
    }

    #[test]
    fn bridge_engine_factory_sigil_forge() {
        let engine = BridgeEngine::sigil_forge();
        assert_eq!(engine.engine_id(), "sigil-forge");
        assert_eq!(engine.engine_name(), "Sigil Forge");
        assert_eq!(engine.required_phase(), 3);
    }

    #[test]
    fn bridge_engine_with_custom_url() {
        let engine = BridgeEngine::tarot_with_url("http://custom:4000");
        assert_eq!(engine.engine_id(), "tarot");
        assert_eq!(engine.base_url, "http://custom:4000");
    }

    #[test]
    fn bridge_engine_trims_trailing_slash() {
        let engine = BridgeEngine::new("test", "Test", 0, "http://localhost:3001/");
        assert_eq!(engine.base_url, "http://localhost:3001");
    }

    #[test]
    fn bridge_engine_cache_key_deterministic() {
        let engine = BridgeEngine::tarot();
        let input = test_input();
        let key1 = engine.cache_key(&input);
        let key2 = engine.cache_key(&input);
        assert_eq!(key1, key2);
        assert!(key1.starts_with("tarot:"));
    }

    #[test]
    fn bridge_engine_default_timeout() {
        let engine = BridgeEngine::tarot();
        assert_eq!(engine.timeout, Duration::from_secs(DEFAULT_TIMEOUT_SECS));
    }

    #[test]
    fn bridge_engine_custom_timeout() {
        let engine = BridgeEngine::with_timeout(
            "test",
            "Test",
            0,
            "http://localhost:3001",
            Duration::from_secs(10),
        );
        assert_eq!(engine.timeout, Duration::from_secs(10));
    }

    #[test]
    fn bridge_manager_creates_all_engines() {
        let manager = BridgeManager::new("http://localhost:3001");
        let engines = manager.engines();
        assert_eq!(engines.len(), 5);

        let ids: Vec<&str> = engines.iter().map(|e| e.engine_id()).collect();
        assert!(ids.contains(&"tarot"));
        assert!(ids.contains(&"i-ching"));
        assert!(ids.contains(&"enneagram"));
        assert!(ids.contains(&"sacred-geometry"));
        assert!(ids.contains(&"sigil-forge"));
    }

    #[test]
    fn bridge_manager_base_url() {
        let manager = BridgeManager::new("http://custom:4000");
        assert_eq!(manager.base_url(), "http://custom:4000");
    }

    #[tokio::test]
    async fn bridge_engine_calculate_connection_refused() {
        // Use a port that's almost certainly not running anything
        let engine = BridgeEngine::new("test", "Test", 0, "http://localhost:59999");
        let input = test_input();
        
        let result = engine.calculate(input).await;
        assert!(result.is_err());
        
        let err = result.unwrap_err();
        match err {
            EngineError::BridgeError(msg) => {
                assert!(msg.contains("Connection") || msg.contains("refused") || msg.contains("failed"));
            }
            _ => panic!("Expected BridgeError, got {:?}", err),
        }
    }

    #[tokio::test]
    async fn bridge_manager_health_check_fails_gracefully() {
        let manager = BridgeManager::new("http://localhost:59999");
        let result = manager.health_check().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn bridge_manager_is_available_false_when_not_running() {
        let manager = BridgeManager::new("http://localhost:59999");
        assert!(!manager.is_available().await);
    }
}
