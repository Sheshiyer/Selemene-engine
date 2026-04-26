//! HTTP client for Python sidecar services (MediaPipe, Biofield CV)
//!
//! Provides a reusable client for calling FastAPI services that process images
//! and return analysis results. Supports multipart/form-data uploads, health
//! checks, configurable timeouts, and graceful degradation when services are
//! unavailable.
//!
//! # Usage
//!
//! ```rust,ignore
//! use noesis_bridge::PythonServiceClient;
//!
//! let client = PythonServiceClient::new("mediapipe", "http://localhost:8001");
//! // Check if service is available
//! let available = client.is_available().await;
//! // Send image for analysis
//! let result = client.analyze_image(image_bytes, "image/jpeg", None).await;
//! ```

use std::time::Duration;

use reqwest::multipart;
use serde_json::Value;
use tracing::{debug, info, warn};

use crate::error::BridgeError;

/// Default timeout for Python service requests (longer due to CV processing)
pub const DEFAULT_PYTHON_TIMEOUT_SECS: u64 = 10;

/// HTTP client for a Python sidecar service
pub struct PythonServiceClient {
    service_name: String,
    base_url: String,
    client: reqwest::Client,
    timeout: Duration,
}

impl PythonServiceClient {
    /// Create a new client for a Python sidecar service.
    ///
    /// * `service_name` - Human-readable name (e.g. "mediapipe", "biofield-cv")
    /// * `base_url` - Root URL (e.g. "http://localhost:8001")
    pub fn new(service_name: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self::with_timeout(
            service_name,
            base_url,
            Duration::from_secs(DEFAULT_PYTHON_TIMEOUT_SECS),
        )
    }

    /// Create a new client with a custom timeout.
    pub fn with_timeout(
        service_name: impl Into<String>,
        base_url: impl Into<String>,
        timeout: Duration,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .connect_timeout(Duration::from_secs(3))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            service_name: service_name.into(),
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            client,
            timeout,
        }
    }

    /// Create a MediaPipe Face Mesh service client.
    pub fn mediapipe(base_url: impl Into<String>) -> Self {
        Self::new("mediapipe", base_url)
    }

    /// Create a Biofield CV service client.
    pub fn biofield_cv(base_url: impl Into<String>) -> Self {
        Self::new("biofield-cv", base_url)
    }

    /// Get the service name.
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    /// Get the base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Check if the Python service is healthy.
    pub async fn health_check(&self) -> Result<Value, BridgeError> {
        let url = format!("{}/health", self.base_url);

        debug!(service = %self.service_name, %url, "Python service health check");

        let response = self.client.get(&url).send().await.map_err(|e| {
            if e.is_connect() {
                BridgeError::ConnectionRefused {
                    url: self.base_url.clone(),
                }
            } else if e.is_timeout() {
                BridgeError::Timeout {
                    timeout_secs: self.timeout.as_secs(),
                }
            } else {
                BridgeError::HttpError(format!(
                    "Health check for {} failed: {}",
                    self.service_name, e
                ))
            }
        })?;

        if !response.status().is_success() {
            return Err(BridgeError::EngineResponse {
                status: response.status().as_u16(),
                body: response.text().await.unwrap_or_default(),
            });
        }

        response.json::<Value>().await.map_err(|e| {
            BridgeError::DeserializationError(format!(
                "Failed to parse health response from {}: {}",
                self.service_name, e
            ))
        })
    }

    /// Check if the service is available (non-blocking, returns false on error).
    pub async fn is_available(&self) -> bool {
        self.health_check().await.is_ok()
    }

    /// Send an image for analysis.
    ///
    /// * `image_data` - Raw image bytes (JPEG or PNG)
    /// * `content_type` - MIME type (e.g. "image/jpeg")
    /// * `options` - Optional JSON string with additional analysis options
    ///
    /// Returns the raw JSON response from the Python service.
    pub async fn analyze_image(
        &self,
        image_data: Vec<u8>,
        content_type: &str,
        options: Option<&str>,
    ) -> Result<Value, BridgeError> {
        let url = format!("{}/analyze", self.base_url);

        debug!(
            service = %self.service_name,
            %url,
            image_size = image_data.len(),
            "Python service analyze request"
        );

        let file_part = multipart::Part::bytes(image_data)
            .file_name("image")
            .mime_str(content_type)
            .map_err(|e| BridgeError::HttpError(format!("Failed to create multipart: {}", e)))?;

        let mut form = multipart::Form::new().part("image", file_part);

        if let Some(opts) = options {
            form = form.text("options", opts.to_owned());
        }

        let response = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    warn!(service = %self.service_name, "Analyze request timed out");
                    BridgeError::Timeout {
                        timeout_secs: self.timeout.as_secs(),
                    }
                } else if e.is_connect() {
                    warn!(service = %self.service_name, %url, "Connection refused");
                    BridgeError::ConnectionRefused {
                        url: self.base_url.clone(),
                    }
                } else {
                    warn!(service = %self.service_name, error = %e, "HTTP error");
                    BridgeError::HttpError(format!("Analyze request to {} failed: {}", url, e))
                }
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            warn!(
                service = %self.service_name,
                %status,
                %body,
                "Analyze returned non-2xx"
            );
            return Err(BridgeError::EngineResponse { status, body });
        }

        info!(service = %self.service_name, "Analyze succeeded");

        response.json::<Value>().await.map_err(|e| {
            BridgeError::DeserializationError(format!(
                "Failed to deserialize response from {}: {}",
                self.service_name, e
            ))
        })
    }

    /// Send an image for analysis with algorithm selection (Biofield CV specific).
    ///
    /// * `image_data` - Raw image bytes
    /// * `content_type` - MIME type
    /// * `algorithms` - JSON array of algorithm names to run
    /// * `options` - Optional JSON string with processing options
    pub async fn analyze_image_with_algorithms(
        &self,
        image_data: Vec<u8>,
        content_type: &str,
        algorithms: &[&str],
        options: Option<&str>,
    ) -> Result<Value, BridgeError> {
        let url = format!("{}/analyze", self.base_url);

        let file_part = multipart::Part::bytes(image_data)
            .file_name("image")
            .mime_str(content_type)
            .map_err(|e| BridgeError::HttpError(format!("Failed to create multipart: {}", e)))?;

        let mut form = multipart::Form::new().part("image", file_part);

        let algo_json = serde_json::to_string(algorithms).map_err(|e| {
            BridgeError::HttpError(format!("Failed to serialize algorithms: {}", e))
        })?;
        form = form.text("algorithms", algo_json);

        if let Some(opts) = options {
            form = form.text("options", opts.to_owned());
        }

        let response = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    BridgeError::Timeout {
                        timeout_secs: self.timeout.as_secs(),
                    }
                } else if e.is_connect() {
                    BridgeError::ConnectionRefused {
                        url: self.base_url.clone(),
                    }
                } else {
                    BridgeError::HttpError(format!("Analyze request failed: {}", e))
                }
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(BridgeError::EngineResponse { status, body });
        }

        response.json::<Value>().await.map_err(|e| {
            BridgeError::DeserializationError(format!(
                "Failed to deserialize response from {}: {}",
                self.service_name, e
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = PythonServiceClient::new("test", "http://localhost:8001");
        assert_eq!(client.service_name(), "test");
        assert_eq!(client.base_url(), "http://localhost:8001");
    }

    #[test]
    fn test_client_trims_trailing_slash() {
        let client = PythonServiceClient::new("test", "http://localhost:8001/");
        assert_eq!(client.base_url(), "http://localhost:8001");
    }

    #[test]
    fn test_mediapipe_factory() {
        let client = PythonServiceClient::mediapipe("http://localhost:8001");
        assert_eq!(client.service_name(), "mediapipe");
    }

    #[test]
    fn test_biofield_cv_factory() {
        let client = PythonServiceClient::biofield_cv("http://localhost:8002");
        assert_eq!(client.service_name(), "biofield-cv");
    }

    #[test]
    fn test_custom_timeout() {
        let client = PythonServiceClient::with_timeout(
            "test",
            "http://localhost:8001",
            Duration::from_secs(30),
        );
        assert_eq!(client.timeout, Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_health_check_connection_refused() {
        let client = PythonServiceClient::new("test", "http://localhost:59998");
        let result = client.health_check().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_is_available_false_when_not_running() {
        let client = PythonServiceClient::new("test", "http://localhost:59998");
        assert!(!client.is_available().await);
    }

    #[tokio::test]
    async fn test_analyze_image_connection_refused() {
        let client = PythonServiceClient::new("test", "http://localhost:59998");
        let result = client
            .analyze_image(vec![0xFF, 0xD8], "image/jpeg", None)
            .await;
        assert!(result.is_err());
    }
}
