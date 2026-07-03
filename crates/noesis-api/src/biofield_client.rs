use std::time::Duration;

use noesis_bridge::{BridgeError, PythonServiceClient};
use serde_json::Value;

use crate::ApiConfig;

pub struct BiofieldAnalyzeRequest {
    pub image_data: Vec<u8>,
    pub content_type: String,
    pub algorithms: Option<Vec<String>>,
    pub options: Option<Value>,
}

pub struct BiofieldClient {
    inner: PythonServiceClient,
    timeout: Duration,
}

impl BiofieldClient {
    pub fn new(base_url: impl Into<String>, timeout: Duration) -> Self {
        let url = base_url.into();
        Self {
            inner: PythonServiceClient::with_timeout("biofield-cv", url, timeout),
            timeout,
        }
    }

    pub fn from_config(config: &ApiConfig) -> Self {
        Self::new(
            config.python_biofield_url.clone(),
            Duration::from_millis(config.python_biofield_timeout_ms),
        )
    }

    pub fn base_url(&self) -> &str {
        self.inner.base_url()
    }

    pub fn timeout_ms(&self) -> u64 {
        self.timeout.as_millis() as u64
    }

    pub async fn health_check(&self) -> Result<Value, BridgeError> {
        self.inner.health_check().await
    }

    pub async fn analyze_capture(
        &self,
        request: BiofieldAnalyzeRequest,
    ) -> Result<Value, BridgeError> {
        let options_json = request
            .options
            .map(|value| serde_json::to_string(&value))
            .transpose()
            .map_err(|err| BridgeError::HttpError(format!("Failed to serialize options: {err}")))?;

        if let Some(algorithms) = request.algorithms.as_ref() {
            let refs = algorithms.iter().map(String::as_str).collect::<Vec<_>>();
            self.inner
                .analyze_image_with_algorithms(
                    request.image_data,
                    &request.content_type,
                    &refs,
                    options_json.as_deref(),
                )
                .await
        } else {
            self.inner
                .analyze_image(
                    request.image_data,
                    &request.content_type,
                    options_json.as_deref(),
                )
                .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{DEFAULT_PYTHON_BIOFIELD_TIMEOUT_MS, DEFAULT_PYTHON_BIOFIELD_URL},
        ApiConfig,
    };
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    fn test_config() -> ApiConfig {
        ApiConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            jwt_secret: "test-secret-at-least-32-chars-long".to_string(),
            database_url: None,
            redis_url: None,
            allowed_origins: vec![],
            rate_limit_requests: 100,
            rate_limit_window_secs: 60,
            request_timeout_secs: 30,
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
            cf_access_issuer: None,
            cf_access_audience: None,
            cf_dev_bypass_token: None,
            dodo_payments_api_key: None,
            dodo_payments_webhook_key: None,
            dodo_payments_env: None,
            python_biofield_url: DEFAULT_PYTHON_BIOFIELD_URL.to_string(),
            python_biofield_timeout_ms: DEFAULT_PYTHON_BIOFIELD_TIMEOUT_MS,
            gateway_url: None,
            gateway_token: None,
        }
    }

    #[test]
    fn builds_from_config() {
        let mut config = test_config();
        config.python_biofield_url = "http://biofield.internal:8002".to_string();
        config.python_biofield_timeout_ms = 15_000;

        let client = BiofieldClient::from_config(&config);
        assert_eq!(client.base_url(), "http://biofield.internal:8002");
        assert_eq!(client.timeout_ms(), 15_000);
    }

    #[tokio::test]
    async fn health_check_hits_private_sidecar_health_route() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "healthy",
                "service": "biofield_cv_service"
            })))
            .mount(&server)
            .await;

        let client = BiofieldClient::new(server.uri(), Duration::from_millis(1_000));
        let response = client.health_check().await.expect("health should succeed");

        assert_eq!(response["status"], "healthy");
    }

    #[tokio::test]
    async fn analyze_capture_posts_to_analyze_route() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/analyze"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "contract_version": "biofield-cv/v1",
                "analysis_version": "test",
                "metrics": {},
                "quality_assessment": {
                    "sharpness": 0.9,
                    "contrast": 0.8,
                    "noise_level": 0.1,
                    "exposure": 0.5,
                    "sufficient_quality": true
                },
                "algorithms_run": ["fractal_dimension"],
                "processing_time_ms": 12.5
            })))
            .mount(&server)
            .await;

        let client = BiofieldClient::new(server.uri(), Duration::from_millis(1_000));
        let response = client
            .analyze_capture(BiofieldAnalyzeRequest {
                image_data: vec![1, 2, 3],
                content_type: "image/jpeg".to_string(),
                algorithms: Some(vec!["fractal_dimension".to_string()]),
                options: Some(serde_json::json!({ "mode": "capture" })),
            })
            .await
            .expect("analysis should succeed");

        assert_eq!(response["contract_version"], "biofield-cv/v1");
        assert_eq!(response["analysis_version"], "test");
    }
}
