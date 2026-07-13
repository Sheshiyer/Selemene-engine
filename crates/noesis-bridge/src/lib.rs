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
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use crate::ts_client::TsEngineResponse;

pub mod error;
pub mod python_client;
pub mod ts_client;
pub use error::BridgeError;
pub use python_client::PythonServiceClient;
pub use ts_client::{TsHealthResponse, WitnessPrompt};

pub use noesis_core::{
    CalculationMetadata, ConsciousnessEngine, EngineError, EngineInput, EngineOutput,
    ValidationResult,
};

/// Default URL for the TypeScript engines server.
pub const DEFAULT_TS_SERVER_URL: &str = "http://localhost:3001";

/// Default timeout for HTTP requests in seconds (overridable via `TS_BRIDGE_TIMEOUT` env var).
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Read the configured bridge timeout from the environment.
///
/// Reads `TS_BRIDGE_TIMEOUT` (seconds, integer). Falls back to [`DEFAULT_TIMEOUT_SECS`] if
/// the variable is unset or unparseable.
pub fn configured_timeout() -> Duration {
    std::env::var("TS_BRIDGE_TIMEOUT")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_secs(DEFAULT_TIMEOUT_SECS))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarEngineHealth {
    pub engine_id: String,
    pub healthy: bool,
    pub detail: String,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarReadinessStatus {
    pub status: String,
    pub engines: Vec<SidecarEngineHealth>,
    pub failed_engines: Vec<String>,
}

// ---------------------------------------------------------------------------
// Circuit breaker
// ---------------------------------------------------------------------------

/// Failure threshold before the circuit opens (trips).
const CB_FAILURE_THRESHOLD: u32 = 5;
/// Seconds the circuit stays open before allowing a half-open probe.
const CB_RESET_SECS: u64 = 30;
/// Env var to override the failure threshold.
const CB_THRESHOLD_ENV: &str = "TS_BRIDGE_CB_THRESHOLD";
/// Env var to override the reset window in seconds.
const CB_RESET_ENV: &str = "TS_BRIDGE_CB_RESET_SECS";

#[derive(Debug, Clone, Serialize)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreakerState {
    pub fn as_str(&self) -> &'static str {
        match self {
            CircuitBreakerState::Closed => "closed",
            CircuitBreakerState::Open => "open",
            CircuitBreakerState::HalfOpen => "half_open",
        }
    }
}

/// Simple three-state circuit breaker: Closed → Open → Half-Open → Closed.
///
/// All state is stored in atomics so the breaker is cheaply shared via `Arc`.
pub struct BridgeCircuitBreaker {
    /// Consecutive failure count.
    failure_count: std::sync::atomic::AtomicU32,
    /// Unix timestamp (seconds) of the last failure.
    last_failure_ts: std::sync::atomic::AtomicU64,
    /// Trip threshold: failures before opening.
    open_threshold: u32,
    /// Seconds the open circuit waits before entering half-open.
    reset_secs: u64,
}

impl BridgeCircuitBreaker {
    fn new() -> Self {
        let open_threshold = std::env::var(CB_THRESHOLD_ENV)
            .ok()
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(CB_FAILURE_THRESHOLD);
        let reset_secs = std::env::var(CB_RESET_ENV)
            .ok()
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(CB_RESET_SECS);
        Self {
            failure_count: std::sync::atomic::AtomicU32::new(0),
            last_failure_ts: std::sync::atomic::AtomicU64::new(0),
            open_threshold,
            reset_secs,
        }
    }

    fn now_secs() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Returns `true` if a request should be allowed through.
    fn allow(&self) -> bool {
        use std::sync::atomic::Ordering::Relaxed;
        let failures = self.failure_count.load(Relaxed);
        if failures < self.open_threshold {
            return true; // Closed
        }
        // Open: check if reset window has elapsed (half-open probe)
        let last = self.last_failure_ts.load(Relaxed);
        Self::now_secs().saturating_sub(last) >= self.reset_secs
    }

    fn record_success(&self) {
        use std::sync::atomic::Ordering::Relaxed;
        self.failure_count.store(0, Relaxed);
    }

    fn record_failure(&self) {
        use std::sync::atomic::Ordering::Relaxed;
        self.failure_count.fetch_add(1, Relaxed);
        self.last_failure_ts.store(Self::now_secs(), Relaxed);
    }

    #[allow(dead_code)]
    fn is_open(&self) -> bool {
        use std::sync::atomic::Ordering::Relaxed;
        let failures = self.failure_count.load(Relaxed);
        if failures < self.open_threshold {
            return false;
        }
        let last = self.last_failure_ts.load(Relaxed);
        Self::now_secs().saturating_sub(last) < self.reset_secs
    }

    /// Returns the current circuit breaker state (Closed, Open, or HalfOpen).
    pub fn state(&self) -> CircuitBreakerState {
        use std::sync::atomic::Ordering::Relaxed;
        let failures = self.failure_count.load(Relaxed);
        if failures < self.open_threshold {
            return CircuitBreakerState::Closed;
        }
        let last = self.last_failure_ts.load(Relaxed);
        if Self::now_secs().saturating_sub(last) >= self.reset_secs {
            CircuitBreakerState::HalfOpen
        } else {
            CircuitBreakerState::Open
        }
    }

    /// Returns the consecutive failure count.
    pub fn failure_count(&self) -> u32 {
        use std::sync::atomic::Ordering::Relaxed;
        self.failure_count.load(Relaxed)
    }

    /// Returns the timestamp of the last failure, if any.
    pub fn last_failure_at(&self) -> Option<DateTime<Utc>> {
        use std::sync::atomic::Ordering::Relaxed;
        let ts = self.last_failure_ts.load(Relaxed);
        if ts == 0 {
            None
        } else {
            chrono::DateTime::from_timestamp(ts as i64, 0)
        }
    }
}

// ---------------------------------------------------------------------------
// BridgeEngine
// ---------------------------------------------------------------------------

/// HTTP adapter that proxies trait calls to a TypeScript engine running on Bun.
///
/// Each instance targets a single engine endpoint on the Bun server.
/// Transport failures are normalized through `BridgeError` before crossing the
/// trait boundary into `EngineError`.
pub struct BridgeEngine {
    engine_id: String,
    engine_name: String,
    required_phase: u8,
    base_url: String,
    client: reqwest::Client,
    timeout: Duration,
    circuit: BridgeCircuitBreaker,
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
        .expect("Failed to build BridgeEngine HTTP client — TLS initialization failed")
    }

    /// Create a new bridge using the runtime-configured timeout (reads `TS_BRIDGE_TIMEOUT` env).
    pub fn with_env_timeout(
        engine_id: impl Into<String>,
        engine_name: impl Into<String>,
        required_phase: u8,
        base_url: impl Into<String>,
    ) -> Result<Self, EngineError> {
        Self::with_timeout(
            engine_id,
            engine_name,
            required_phase,
            base_url,
            configured_timeout(),
        )
    }
    pub fn with_timeout(
        engine_id: impl Into<String>,
        engine_name: impl Into<String>,
        required_phase: u8,
        base_url: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self, EngineError> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .connect_timeout(Duration::from_secs(2))
            .build()
            .map_err(|e| EngineError::BridgeError(format!("Failed to build HTTP client: {e}")))?;

        Ok(Self {
            engine_id: engine_id.into(),
            engine_name: engine_name.into(),
            required_phase,
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            client,
            timeout,
            circuit: BridgeCircuitBreaker::new(),
        })
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
        Self::new("i-ching", "I Ching", 0, base_url)
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
        Self::new("sacred-geometry", "Sacred Geometry", 0, base_url)
    }

    /// Create a Sigil Forge engine bridge with default URL.
    pub fn sigil_forge() -> Self {
        Self::sigil_forge_with_url(DEFAULT_TS_SERVER_URL)
    }

    /// Create a Sigil Forge engine bridge with custom URL.
    pub fn sigil_forge_with_url(base_url: impl Into<String>) -> Self {
        Self::new("sigil-forge", "Sigil Forge", 1, base_url)
    }

    /// Create a Raaga (Carnatic melakarta) engine bridge with default URL.
    pub fn raaga() -> Self {
        Self::raaga_with_url(DEFAULT_TS_SERVER_URL)
    }

    /// Create a Raaga engine bridge with custom URL.
    pub fn raaga_with_url(base_url: impl Into<String>) -> Self {
        Self::new("raaga", "Raaga", 0, base_url)
    }

    /// Return a reference to the circuit breaker for admin observability.
    pub fn circuit_breaker(&self) -> &BridgeCircuitBreaker {
        &self.circuit
    }

    fn extract_question(options: &std::collections::HashMap<String, Value>) -> Option<String> {
        const QUESTION_ALIASES: [&str; 4] = ["question", "intention", "intent", "intent_text"];

        QUESTION_ALIASES.iter().find_map(|key| {
            options.get(*key).and_then(Value::as_str).and_then(|raw| {
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            })
        })
    }

    fn to_ts_request(&self, input: &EngineInput) -> crate::ts_client::TsEngineRequest {
        crate::ts_client::TsEngineRequest {
            consciousness_level: self.required_phase,
            parameters: input.options.clone(),
            seed: None,
            question: Self::extract_question(&input.options),
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
        // Circuit-breaker fast-fail: prevents cascading 503s when the TS sidecar is down.
        if !self.circuit.allow() {
            warn!(
                engine = %self.engine_id,
                "Circuit open — failing fast (TS sidecar unreachable)"
            );
            return Err(EngineError::BridgeError(format!(
                "{}: circuit open — too many consecutive failures, retry in {}s",
                self.engine_id, self.circuit.reset_secs
            )));
        }

        if self.engine_id == "sigil-forge" {
            let has_intention = Self::extract_question(&input.options).is_some();
            if !has_intention {
                return Err(EngineError::ValidationError(
                    "sigil-forge requires options.intention (or question/intent/intent_text)"
                        .to_string(),
                ));
            }
        }

        let url = format!("{}/engines/{}/calculate", self.base_url, self.engine_id);

        debug!(
            engine = %self.engine_id,
            %url,
            timeout_secs = self.timeout.as_secs(),
            "bridge calculate request"
        );

        // Convert EngineInput to TsEngineRequest format
        let ts_request = self.to_ts_request(&input);

        let response = self
            .client
            .post(&url)
            .json(&ts_request)
            .send()
            .await
            .map_err(|e| {
                let bridge_error = if e.is_timeout() {
                    warn!(engine = %self.engine_id, "Bridge request timed out");
                    BridgeError::Timeout {
                        timeout_secs: self.timeout.as_secs(),
                    }
                } else if e.is_connect() {
                    warn!(engine = %self.engine_id, %url, "Bridge connection refused");
                    BridgeError::ConnectionRefused {
                        url: self.base_url.clone(),
                    }
                } else {
                    warn!(engine = %self.engine_id, error = %e, "Bridge HTTP error");
                    BridgeError::HttpError(format!("{} ({})", e, url))
                };
                self.circuit.record_failure();
                bridge_error.into_calculate_engine_error(&self.engine_id)
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
            self.circuit.record_failure();
            return Err(BridgeError::EngineResponse {
                status: status.as_u16(),
                body,
            }
            .into_calculate_engine_error(&self.engine_id));
        }

        let ts_response = response.json::<TsEngineResponse>().await.map_err(|e| {
            BridgeError::DeserializationError(format!("TsEngineResponse: {}", e))
                .into_calculate_engine_error(&self.engine_id)
        })?;

        // Request succeeded — reset the circuit.
        self.circuit.record_success();

        info!(
            engine = %self.engine_id,
            processing_time_ms = ts_response.processing_time_ms,
            "Bridge calculate succeeded"
        );

        // Convert TsEngineResponse to EngineOutput
        use chrono::Utc;

        Ok(EngineOutput {
            engine_id: ts_response.engine_id,
            result: ts_response.result,
            witness_prompt: ts_response
                .witness_prompts
                .first()
                .map(|p| p.prompt.clone())
                .unwrap_or_else(|| "What does this reveal to you?".to_string()),
            consciousness_level: self.required_phase,
            metadata: CalculationMetadata {
                calculation_time_ms: ts_response.processing_time_ms as f64,
                backend: "typescript".to_string(),
                precision_achieved: "exact".to_string(),
                cached: false,
                timestamp: Utc::now(),
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        })
    }

    async fn validate(&self, output: &EngineOutput) -> Result<ValidationResult, EngineError> {
        let url = format!("{}/engines/{}/validate", self.base_url, self.engine_id);

        debug!(engine = %self.engine_id, %url, "bridge validate request");

        let response = self
            .client
            .post(&url)
            .json(output)
            .send()
            .await
            .map_err(|e| {
                let bridge_error = if e.is_timeout() {
                    BridgeError::Timeout {
                        timeout_secs: self.timeout.as_secs(),
                    }
                } else if e.is_connect() {
                    BridgeError::ConnectionRefused {
                        url: self.base_url.clone(),
                    }
                } else {
                    BridgeError::HttpError(format!("{} ({})", e, url))
                };
                bridge_error.into_validate_engine_error(&self.engine_id)
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

            return Err(BridgeError::EngineResponse {
                status: status.as_u16(),
                body,
            }
            .into_validate_engine_error(&self.engine_id));
        }

        response.json::<ValidationResult>().await.map_err(|e| {
            BridgeError::DeserializationError(format!("ValidationResult: {}", e))
                .into_validate_engine_error(&self.engine_id)
        })
    }

    fn cache_key(&self, input: &EngineInput) -> String {
        let input_json = serde_json::to_string(input).unwrap_or_default();
        let raw = format!("{}:{}", self.engine_id, input_json);
        let hash = format!("{:x}", Sha256::digest(raw.as_bytes()));
        format!("{}:{}", self.engine_id, hash)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ---------------------------------------------------------------------------
// BridgeManager
// ---------------------------------------------------------------------------

/// Manages the set of TypeScript engines accessible through the HTTP bridge.
///
/// Provides factory construction of `BridgeEngine` instances for the six
/// TypeScript-based consciousness engines and a health-check endpoint.
pub struct BridgeManager {
    base_url: String,
    engines: Vec<Arc<dyn ConsciousnessEngine>>,
}

impl BridgeManager {
    /// Create a new manager pointing at the given Bun server root URL.
    ///
    /// Instantiates `BridgeEngine` wrappers for all six TypeScript engines.
    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url: String = base_url.into();

        let engines: Vec<Arc<dyn ConsciousnessEngine>> = vec![
            Arc::new(BridgeEngine::tarot_with_url(&base_url)),
            Arc::new(BridgeEngine::i_ching_with_url(&base_url)),
            Arc::new(BridgeEngine::enneagram_with_url(&base_url)),
            Arc::new(BridgeEngine::sacred_geometry_with_url(&base_url)),
            Arc::new(BridgeEngine::sigil_forge_with_url(&base_url)),
            Arc::new(BridgeEngine::raaga_with_url(&base_url)),
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
        let url =
            std::env::var("TS_ENGINES_URL").unwrap_or_else(|_| DEFAULT_TS_SERVER_URL.to_string());
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
            .map_err(|e| {
                BridgeError::HttpError(format!("Failed to create client: {}", e))
                    .into_manager_engine_error()
            })?;

        let response = client.get(&url).send().await.map_err(|e| {
            let bridge_error = if e.is_connect() {
                BridgeError::ConnectionRefused {
                    url: self.base_url.clone(),
                }
            } else if e.is_timeout() {
                BridgeError::Timeout { timeout_secs: 2 }
            } else {
                BridgeError::HttpError(format!("Health check failed for {}: {}", url, e))
            };
            bridge_error.into_manager_engine_error()
        })?;

        if response.status().is_success() {
            info!(url = %self.base_url, "TS server health check passed");
            Ok(())
        } else {
            Err(BridgeError::ServerUnavailable(format!(
                "Health check returned {}",
                response.status()
            ))
            .into_manager_engine_error())
        }
    }

    /// Fetch detailed sidecar readiness including per-engine status.
    pub async fn readiness_status(&self) -> Result<SidecarReadinessStatus, EngineError> {
        let url = format!("{}/health/ready", self.base_url);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .map_err(|e| {
                BridgeError::HttpError(format!("Failed to create client: {}", e))
                    .into_manager_engine_error()
            })?;

        let response = client.get(&url).send().await.map_err(|e| {
            let bridge_error = if e.is_connect() {
                BridgeError::ConnectionRefused {
                    url: self.base_url.clone(),
                }
            } else if e.is_timeout() {
                BridgeError::Timeout { timeout_secs: 2 }
            } else {
                BridgeError::HttpError(format!("Readiness check failed for {}: {}", url, e))
            };
            bridge_error.into_manager_engine_error()
        })?;

        let status = response.status();
        if status.as_u16() != 200 && status.as_u16() != 503 {
            return Err(BridgeError::ServerUnavailable(format!(
                "Readiness check returned unexpected status {}",
                status
            ))
            .into_manager_engine_error());
        }

        response
            .json::<SidecarReadinessStatus>()
            .await
            .map_err(|e| {
                BridgeError::DeserializationError(format!("Readiness response from {}: {}", url, e))
                    .into_manager_engine_error()
            })
    }

    /// Check if the TS server is available (non-blocking, returns false on error).
    pub async fn is_available(&self) -> bool {
        self.health_check().await.is_ok()
    }

    /// Return circuit breaker states for all bridge engines, keyed by engine ID.
    pub fn circuit_breaker_states(&self) -> Vec<(String, BridgeCircuitBreakerSnapshot)> {
        self.engines
            .iter()
            .map(|engine| {
                let snapshot = engine
                    .as_ref()
                    .as_any()
                    .downcast_ref::<BridgeEngine>()
                    .map(|be| {
                        let cb = be.circuit_breaker();
                        BridgeCircuitBreakerSnapshot {
                            state: cb.state().as_str().to_string(),
                            failure_count: cb.failure_count(),
                            last_failure_at: cb
                                .last_failure_at()
                                .map(|dt| dt.to_rfc3339()),
                        }
                    })
                    .unwrap_or(BridgeCircuitBreakerSnapshot {
                        state: "unknown".to_string(),
                        failure_count: 0,
                        last_failure_at: None,
                    });
                (engine.engine_id().to_string(), snapshot)
            })
            .collect()
    }
}

/// Snapshot of a bridge engine's circuit breaker for admin observability.
#[derive(Debug, Clone, Serialize)]
pub struct BridgeCircuitBreakerSnapshot {
    pub state: String,
    pub failure_count: u32,
    pub last_failure_at: Option<String>,
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;
    use std::collections::HashMap;

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
        assert_eq!(engine.required_phase(), 0);
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
        assert_eq!(engine.required_phase(), 0);
    }

    #[test]
    fn bridge_engine_factory_sigil_forge() {
        let engine = BridgeEngine::sigil_forge();
        assert_eq!(engine.engine_id(), "sigil-forge");
        assert_eq!(engine.engine_name(), "Sigil Forge");
        assert_eq!(engine.required_phase(), 1);
    }

    #[test]
    fn bridge_engine_factory_raaga() {
        let engine = BridgeEngine::raaga();
        assert_eq!(engine.engine_id(), "raaga");
        assert_eq!(engine.engine_name(), "Raaga");
        assert_eq!(engine.required_phase(), 0);
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
        )
        .expect("test: failed to build engine");
        assert_eq!(engine.timeout, Duration::from_secs(10));
    }

    #[test]
    fn bridge_engine_to_ts_request_extracts_question_aliases() {
        let engine = BridgeEngine::sigil_forge();
        let mut input = test_input();

        input
            .options
            .insert("question".to_string(), json!("What needs form?"));
        let req = engine.to_ts_request(&input);
        assert_eq!(req.question.as_deref(), Some("What needs form?"));

        input.options.clear();
        input
            .options
            .insert("intention".to_string(), json!("I am focused"));
        let req = engine.to_ts_request(&input);
        assert_eq!(req.question.as_deref(), Some("I am focused"));

        input.options.clear();
        input.options.insert("intent".to_string(), json!("Clarity"));
        let req = engine.to_ts_request(&input);
        assert_eq!(req.question.as_deref(), Some("Clarity"));

        input.options.clear();
        input
            .options
            .insert("intent_text".to_string(), json!("Steady attention"));
        let req = engine.to_ts_request(&input);
        assert_eq!(req.question.as_deref(), Some("Steady attention"));
    }

    #[test]
    fn bridge_manager_creates_all_engines() {
        let manager = BridgeManager::new("http://localhost:3001");
        let engines = manager.engines();
        assert_eq!(engines.len(), 6);

        let ids: Vec<&str> = engines.iter().map(|e| e.engine_id()).collect();
        assert!(ids.contains(&"tarot"));
        assert!(ids.contains(&"i-ching"));
        assert!(ids.contains(&"enneagram"));
        assert!(ids.contains(&"sacred-geometry"));
        assert!(ids.contains(&"sigil-forge"));
        assert!(ids.contains(&"raaga"));
    }

    #[test]
    fn bridge_manager_base_url() {
        let manager = BridgeManager::new("http://custom:4000");
        assert_eq!(manager.base_url(), "http://custom:4000");
    }

    #[tokio::test]
    async fn circuit_breaker_trips_after_threshold() {
        use std::time::Duration;
        let cb = BridgeCircuitBreaker {
            failure_count: std::sync::atomic::AtomicU32::new(0),
            last_failure_ts: std::sync::atomic::AtomicU64::new(0),
            open_threshold: 3,
            reset_secs: 60,
        };
        assert!(cb.allow()); // Closed
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        assert!(!cb.allow()); // Open after 3 failures
                              // Success resets it
        cb.record_success();
        assert!(cb.allow()); // Closed again
    }

    #[tokio::test]
    async fn circuit_breaker_half_open_after_reset_window() {
        let cb = BridgeCircuitBreaker {
            failure_count: std::sync::atomic::AtomicU32::new(3),
            last_failure_ts: std::sync::atomic::AtomicU64::new(0), // very old
            open_threshold: 3,
            reset_secs: 1,
        };
        // last_failure was at epoch 0, reset_secs=1 → window elapsed → half-open probe allowed
        assert!(cb.allow());
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
                assert!(
                    msg.contains("Connection") || msg.contains("refused") || msg.contains("failed")
                );
            }
            _ => panic!("Expected BridgeError, got {:?}", err),
        }
    }

    #[tokio::test]
    async fn bridge_engine_sigil_requires_intention() {
        let engine = BridgeEngine::sigil_forge_with_url("http://localhost:59999");
        let input = test_input();

        let result = engine.calculate(input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            EngineError::ValidationError(msg) => {
                assert!(msg.contains("sigil-forge requires options.intention"));
            }
            other => panic!("Expected ValidationError, got {:?}", other),
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
