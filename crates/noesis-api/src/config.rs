//! Environment-based configuration for Noesis API
//!
//! Loads configuration from environment variables with sensible defaults
//! for development and production environments.

use noesis_core::EngineError;
use std::env;

pub const DEFAULT_PYTHON_BIOFIELD_URL: &str = "http://localhost:8002";
pub const DEFAULT_PYTHON_BIOFIELD_TIMEOUT_MS: u64 = 10_000;

/// API server configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Server host address (default: "0.0.0.0")
    pub host: String,

    /// Server port (default: 8080)
    pub port: u16,

    /// JWT secret for token signing (required, no default in production)
    pub jwt_secret: String,

    /// Database URL for PostgreSQL (optional — server starts in degraded mode without it)
    pub database_url: Option<String>,

    /// Redis connection URL for L2 cache (optional, None disables Redis)
    pub redis_url: Option<String>,

    /// Allowed CORS origins (comma-separated list)
    pub allowed_origins: Vec<String>,

    /// Rate limit: max requests per window (default: 100)
    pub rate_limit_requests: u32,

    /// Rate limit: window duration in seconds (default: 60)
    pub rate_limit_window_secs: u64,

    /// Request timeout in seconds (default: 30)
    pub request_timeout_secs: u64,

    /// Log level (default: "info")
    pub log_level: String,

    /// Log format: "pretty" or "json" (default: "pretty" for dev, "json" for prod)
    pub log_format: String,

    /// Discord OAuth2 client ID (optional — Discord login disabled when unset)
    pub discord_client_id: Option<String>,

    /// Discord OAuth2 client secret (optional — Discord login disabled when unset)
    pub discord_client_secret: Option<String>,

    /// Discord OAuth2 redirect URI (optional — Discord login disabled when unset)
    pub discord_redirect_uri: Option<String>,

    /// Dodo Payments API key (optional — billing integration disabled when unset)
    pub dodo_payments_api_key: Option<String>,

    /// Dodo Payments webhook signing key (optional — webhook verification disabled when unset)
    pub dodo_payments_webhook_key: Option<String>,

    /// Dodo Payments environment (`test` or `live`)
    pub dodo_payments_env: Option<String>,

    /// Biofield CV Python sidecar URL
    pub python_biofield_url: String,

    /// Biofield CV Python sidecar request timeout in milliseconds
    pub python_biofield_timeout_ms: u64,

    /// OpenClaw Gateway URL (ws:// or wss://) for OpenClaw integration
    pub gateway_url: Option<String>,

    /// OpenClaw Gateway token for authentication
    pub gateway_token: Option<String>,
}

impl ApiConfig {
    /// Load configuration from environment variables with defaults
    ///
    /// # Environment Variables
    /// - `HOST`: Server host address (default: "0.0.0.0")
    /// - `SERVER_HOST`: Legacy alias for `HOST`
    /// - `PORT`: Server port (default: 8080)
    /// - `SERVER_PORT`: Legacy alias for `PORT`
    /// - `JWT_SECRET`: JWT secret (required in production, has dev default)
    /// - `REDIS_URL`: Redis connection URL (optional)
    /// - `ALLOWED_ORIGINS`: Comma-separated CORS origins (default: localhost:3000,5173)
    /// - `RATE_LIMIT_REQUESTS`: Max requests per window (default: 100)
    /// - `RATE_LIMIT_WINDOW_SECS`: Rate limit window in seconds (default: 60)
    /// - `REQUEST_TIMEOUT_SECS`: Request timeout in seconds (default: 30)
    /// - `RUST_LOG`: Log level (default: "info,noesis_api=debug")
    /// - `LOG_FORMAT`: Log format "pretty" or "json" (default: "pretty")
    ///
    /// # Returns
    /// `Ok(ApiConfig)` on success, `Err(EngineError::ConfigError)` when:
    /// - `JWT_SECRET` is missing and `RUST_ENV=production`
    /// - `DATABASE_URL` is set but does not start with `postgresql://` or `postgres://`
    pub fn from_env() -> Result<Self, EngineError> {
        let is_production = env::var("RUST_ENV")
            .map(|e| e == "production")
            .unwrap_or(false);

        let host = env::var("HOST")
            .or_else(|_| env::var("SERVER_HOST"))
            .unwrap_or_else(|_| "0.0.0.0".to_string());

        let port = env::var("PORT")
            .or_else(|_| env::var("SERVER_PORT"))
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        // JWT secret handling: require in production, allow default in dev
        let jwt_secret = match env::var("JWT_SECRET") {
            Ok(secret) => secret,
            Err(_) if is_production => {
                return Err(EngineError::ConfigError(
                    "JWT_SECRET is required when RUST_ENV=production".to_string(),
                ));
            }
            Err(_) => {
                tracing::warn!(
                    "JWT_SECRET not set, using development default (DO NOT USE IN PRODUCTION)"
                );
                "noesis-dev-secret-change-in-production".to_string()
            }
        };

        // DATABASE_URL handling: optional — server starts in degraded mode without it
        let database_url = match env::var("DATABASE_URL") {
            Ok(url) => {
                // Validate format early (fail fast on obviously wrong URLs)
                if !url.starts_with("postgresql://") && !url.starts_with("postgres://") {
                    return Err(EngineError::ConfigError(format!(
                        "DATABASE_URL must start with postgresql:// or postgres://, got: {}...",
                        &url[..url.len().min(20)]
                    )));
                }
                Some(url)
            }
            Err(_) if is_production => {
                tracing::warn!(
                    "DATABASE_URL not set in production — auth endpoints will be unavailable"
                );
                None
            }
            Err(_) => {
                tracing::warn!("DATABASE_URL not set — auth endpoints will be unavailable");
                None
            }
        };

        let redis_url = env::var("REDIS_URL").ok();

        let allowed_origins = env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000,http://localhost:5173".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let rate_limit_requests = env::var("RATE_LIMIT_REQUESTS")
            .ok()
            .and_then(|r| r.parse().ok())
            .unwrap_or(100);

        let rate_limit_window_secs = env::var("RATE_LIMIT_WINDOW_SECS")
            .ok()
            .and_then(|w| w.parse().ok())
            .unwrap_or(60);

        let request_timeout_secs = env::var("REQUEST_TIMEOUT_SECS")
            .ok()
            .and_then(|t| t.parse().ok())
            .unwrap_or(30);

        let log_level =
            env::var("RUST_LOG").unwrap_or_else(|_| "info,noesis_api=debug".to_string());

        let log_format = env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string());

        let discord_client_id = env::var("DISCORD_CLIENT_ID").ok();
        let discord_client_secret = env::var("DISCORD_CLIENT_SECRET").ok();
        let discord_redirect_uri = env::var("DISCORD_REDIRECT_URI").ok();
        let dodo_payments_api_key = env::var("DODO_PAYMENTS_API_KEY")
            .or_else(|_| env::var("DODO_API_KEY"))
            .ok();
        let dodo_payments_webhook_key = env::var("DODO_PAYMENTS_WEBHOOK_KEY")
            .or_else(|_| env::var("DODO_WEBHOOK_KEY"))
            .ok();
        let dodo_payments_env = env::var("DODO_PAYMENTS_ENV").ok();

        let python_biofield_url = env::var("PYTHON_BIOFIELD_URL")
            .unwrap_or_else(|_| DEFAULT_PYTHON_BIOFIELD_URL.to_string());

        let python_biofield_timeout_ms = match env::var("PYTHON_BIOFIELD_TIMEOUT_MS") {
            Ok(value) => value.parse().map_err(|_| {
                EngineError::ConfigError(format!(
                    "PYTHON_BIOFIELD_TIMEOUT_MS must be a positive integer, got '{}'",
                    value
                ))
            })?,
            Err(_) => DEFAULT_PYTHON_BIOFIELD_TIMEOUT_MS,
        };

        // OpenClaw Gateway configuration
        let gateway_url = env::var("GATEWAY_URL").ok();
        let gateway_token = env::var("GATEWAY_TOKEN").ok();

        Ok(Self {
            host,
            port,
            jwt_secret,
            database_url,
            redis_url,
            allowed_origins,
            rate_limit_requests,
            rate_limit_window_secs,
            request_timeout_secs,
            log_level,
            log_format,
            discord_client_id,
            discord_client_secret,
            discord_redirect_uri,
            dodo_payments_api_key,
            dodo_payments_webhook_key,
            dodo_payments_env,
            python_biofield_url,
            python_biofield_timeout_ms,
            gateway_url,
            gateway_token,
        })
    }

    /// Validate the configuration
    ///
    /// Checks for common configuration errors and logs warnings for
    /// potentially problematic settings.
    ///
    /// # Returns
    /// `Ok(())` if configuration is valid, `Err` with error message otherwise
    pub fn validate(&self) -> Result<(), String> {
        // Validate JWT secret is not the default in production-like settings
        if self.jwt_secret == "noesis-dev-secret-change-in-production" {
            let is_production = env::var("RUST_ENV")
                .map(|e| e == "production")
                .unwrap_or(false);

            if is_production {
                return Err("JWT_SECRET must not use default value in production".to_string());
            }
        }

        // Validate JWT secret length (minimum 32 characters required in production)
        if self.jwt_secret.len() < 32 {
            let is_production = env::var("RUST_ENV")
                .map(|e| e == "production")
                .unwrap_or(false);

            if is_production {
                return Err(format!(
                    "JWT_SECRET must be at least 32 characters in production (currently {})",
                    self.jwt_secret.len()
                ));
            }

            tracing::warn!(
                "JWT_SECRET is only {} characters long, recommend at least 32 characters for security",
                self.jwt_secret.len()
            );
        }

        // Validate DATABASE_URL format (if set)
        if let Some(ref db_url) = self.database_url {
            if !db_url.starts_with("postgresql://") && !db_url.starts_with("postgres://") {
                return Err(format!(
                    "DATABASE_URL must start with 'postgresql://' or 'postgres://', got: {}...",
                    &db_url[..db_url.len().min(20)]
                ));
            }
        }

        // Validate port range
        if self.port < 1024 {
            tracing::warn!(
                "Port {} is in privileged range (<1024), may require root/admin privileges",
                self.port
            );
        }

        // Validate rate limit settings
        if self.rate_limit_requests == 0 {
            tracing::warn!("Rate limit requests set to 0, effectively blocking all requests");
        }

        if self.rate_limit_window_secs == 0 {
            return Err("Rate limit window cannot be 0 seconds".to_string());
        }

        // Validate timeout
        if self.request_timeout_secs == 0 {
            return Err("Request timeout cannot be 0 seconds".to_string());
        }

        // Validate log format
        if self.log_format != "pretty" && self.log_format != "json" {
            tracing::warn!(
                "Unknown LOG_FORMAT '{}', using 'pretty' format instead",
                self.log_format
            );
        }

        if let Some(ref dodo_env) = self.dodo_payments_env {
            if dodo_env != "test" && dodo_env != "live" {
                return Err(format!(
                    "DODO_PAYMENTS_ENV must be 'test' or 'live', got '{}'",
                    dodo_env
                ));
            }
        }

        if self.dodo_payments_api_key.is_some() && self.dodo_payments_env.is_none() {
            tracing::warn!(
                "DODO_PAYMENTS_API_KEY is set without DODO_PAYMENTS_ENV; billing should explicitly declare test or live mode"
            );
        }

        if self.dodo_payments_api_key.is_some() && self.dodo_payments_webhook_key.is_none() {
            tracing::warn!(
                "DODO_PAYMENTS_API_KEY is set without webhook signing key; webhook verification will be unavailable"
            );
        }

        if !self.python_biofield_url.starts_with("http://")
            && !self.python_biofield_url.starts_with("https://")
        {
            return Err(format!(
                "PYTHON_BIOFIELD_URL must start with 'http://' or 'https://', got: {}",
                self.python_biofield_url
            ));
        }

        if self.python_biofield_timeout_ms == 0 {
            return Err("PYTHON_BIOFIELD_TIMEOUT_MS must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Get the server bind address as a string
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvGuard {
        saved: Vec<(&'static str, Option<String>)>,
    }

    impl EnvGuard {
        fn set(vars: &[(&'static str, &str)]) -> Self {
            let mut saved = Vec::with_capacity(vars.len());
            for (key, value) in vars {
                saved.push((*key, std::env::var(key).ok()));
                std::env::set_var(key, value);
            }
            Self { saved }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, prior) in self.saved.drain(..).rev() {
                if let Some(value) = prior {
                    std::env::set_var(key, value);
                } else {
                    std::env::remove_var(key);
                }
            }
        }
    }

    #[test]
    fn test_bind_address() {
        let config = ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
            jwt_secret: "test-secret-at-least-32-chars-long".to_string(),
            database_url: Some("postgres://localhost/test".to_string()),
            redis_url: None,
            allowed_origins: vec![],
            rate_limit_requests: 100,
            rate_limit_window_secs: 60,
            request_timeout_secs: 30,
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
            discord_client_id: None,
            discord_client_secret: None,
            discord_redirect_uri: None,
            dodo_payments_api_key: None,
            dodo_payments_webhook_key: None,
            dodo_payments_env: None,
            python_biofield_url: DEFAULT_PYTHON_BIOFIELD_URL.to_string(),
            python_biofield_timeout_ms: DEFAULT_PYTHON_BIOFIELD_TIMEOUT_MS,
            gateway_url: None,
            gateway_token: None,
        };

        assert_eq!(config.bind_address(), "127.0.0.1:3000");
    }

    #[test]
    fn test_validate_invalid_window() {
        let config = ApiConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            jwt_secret: "test-secret-at-least-32-chars-long".to_string(),
            database_url: Some("postgres://localhost/test".to_string()),
            redis_url: None,
            allowed_origins: vec![],
            rate_limit_requests: 100,
            rate_limit_window_secs: 0, // Invalid!
            request_timeout_secs: 30,
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
            discord_client_id: None,
            discord_client_secret: None,
            discord_redirect_uri: None,
            dodo_payments_api_key: None,
            dodo_payments_webhook_key: None,
            dodo_payments_env: None,
            python_biofield_url: DEFAULT_PYTHON_BIOFIELD_URL.to_string(),
            python_biofield_timeout_ms: DEFAULT_PYTHON_BIOFIELD_TIMEOUT_MS,
            gateway_url: None,
            gateway_token: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_database_url() {
        let config = ApiConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            jwt_secret: "test-secret-at-least-32-chars-long".to_string(),
            database_url: Some("mysql://localhost/test".to_string()),
            redis_url: None,
            allowed_origins: vec![],
            rate_limit_requests: 100,
            rate_limit_window_secs: 60,
            request_timeout_secs: 30,
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
            discord_client_id: None,
            discord_client_secret: None,
            discord_redirect_uri: None,
            dodo_payments_api_key: None,
            dodo_payments_webhook_key: None,
            dodo_payments_env: None,
            python_biofield_url: DEFAULT_PYTHON_BIOFIELD_URL.to_string(),
            python_biofield_timeout_ms: DEFAULT_PYTHON_BIOFIELD_TIMEOUT_MS,
            gateway_url: None,
            gateway_token: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_valid_database_url_variants() {
        for url in &["postgres://localhost/test", "postgresql://localhost/test"] {
            let config = ApiConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                jwt_secret: "test-secret-at-least-32-chars-long".to_string(),
                database_url: Some(url.to_string()),
                redis_url: None,
                allowed_origins: vec![],
                rate_limit_requests: 100,
                rate_limit_window_secs: 60,
                request_timeout_secs: 30,
                log_level: "info".to_string(),
                log_format: "pretty".to_string(),
                discord_client_id: None,
                discord_client_secret: None,
                discord_redirect_uri: None,
                dodo_payments_api_key: None,
                dodo_payments_webhook_key: None,
                dodo_payments_env: None,
                python_biofield_url: DEFAULT_PYTHON_BIOFIELD_URL.to_string(),
                python_biofield_timeout_ms: DEFAULT_PYTHON_BIOFIELD_TIMEOUT_MS,
                gateway_url: None,
                gateway_token: None,
            };

            assert!(
                config.validate().is_ok(),
                "should accept DATABASE_URL: {}",
                url
            );
        }
    }

    #[test]
    fn test_validate_invalid_timeout() {
        let config = ApiConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            jwt_secret: "test-secret-at-least-32-chars-long".to_string(),
            database_url: Some("postgres://localhost/test".to_string()),
            redis_url: None,
            allowed_origins: vec![],
            rate_limit_requests: 100,
            rate_limit_window_secs: 60,
            request_timeout_secs: 0, // Invalid!
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
            discord_client_id: None,
            discord_client_secret: None,
            discord_redirect_uri: None,
            dodo_payments_api_key: None,
            dodo_payments_webhook_key: None,
            dodo_payments_env: None,
            python_biofield_url: DEFAULT_PYTHON_BIOFIELD_URL.to_string(),
            python_biofield_timeout_ms: DEFAULT_PYTHON_BIOFIELD_TIMEOUT_MS,
            gateway_url: None,
            gateway_token: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_from_env_uses_server_host_alias() {
        let _lock = env_lock().lock().unwrap();
        let _guard = EnvGuard::set(&[
            ("RUST_ENV", "development"),
            ("SERVER_HOST", "127.0.0.2"),
            ("JWT_SECRET", "test-secret-at-least-32-chars-long"),
        ]);
        std::env::remove_var("HOST");
        std::env::remove_var("PORT");
        std::env::remove_var("SERVER_PORT");

        let config = ApiConfig::from_env().expect("config should load");
        assert_eq!(config.host, "127.0.0.2");
    }

    #[test]
    fn test_from_env_uses_server_port_alias() {
        let _lock = env_lock().lock().unwrap();
        let _guard = EnvGuard::set(&[
            ("RUST_ENV", "development"),
            ("SERVER_PORT", "9090"),
            ("JWT_SECRET", "test-secret-at-least-32-chars-long"),
        ]);
        std::env::remove_var("HOST");
        std::env::remove_var("SERVER_HOST");
        std::env::remove_var("PORT");

        let config = ApiConfig::from_env().expect("config should load");
        assert_eq!(config.port, 9090);
    }

    #[test]
    fn test_from_env_reads_dodo_payments_settings() {
        let _lock = env_lock().lock().unwrap();
        let _guard = EnvGuard::set(&[
            ("RUST_ENV", "development"),
            ("JWT_SECRET", "test-secret-at-least-32-chars-long"),
            ("DODO_PAYMENTS_API_KEY", "dodo_test_key"),
            ("DODO_PAYMENTS_WEBHOOK_KEY", "dodo_webhook_secret"),
            ("DODO_PAYMENTS_ENV", "test"),
        ]);

        let config = ApiConfig::from_env().expect("config should load");
        assert_eq!(
            config.dodo_payments_api_key.as_deref(),
            Some("dodo_test_key")
        );
        assert_eq!(
            config.dodo_payments_webhook_key.as_deref(),
            Some("dodo_webhook_secret")
        );
        assert_eq!(config.dodo_payments_env.as_deref(), Some("test"));
    }

    #[test]
    fn test_from_env_reads_python_biofield_settings() {
        let _lock = env_lock().lock().unwrap();
        let _guard = EnvGuard::set(&[
            ("RUST_ENV", "development"),
            ("JWT_SECRET", "test-secret-at-least-32-chars-long"),
            ("PYTHON_BIOFIELD_URL", "http://biofield.internal:8002"),
            ("PYTHON_BIOFIELD_TIMEOUT_MS", "15000"),
        ]);

        let config = ApiConfig::from_env().expect("config should load");
        assert_eq!(config.python_biofield_url, "http://biofield.internal:8002");
        assert_eq!(config.python_biofield_timeout_ms, 15_000);
    }

    #[test]
    fn test_from_env_defaults_python_biofield_settings() {
        let _lock = env_lock().lock().unwrap();
        let _guard = EnvGuard::set(&[
            ("RUST_ENV", "development"),
            ("JWT_SECRET", "test-secret-at-least-32-chars-long"),
        ]);
        std::env::remove_var("PYTHON_BIOFIELD_URL");
        std::env::remove_var("PYTHON_BIOFIELD_TIMEOUT_MS");

        let config = ApiConfig::from_env().expect("config should load");
        assert_eq!(config.python_biofield_url, DEFAULT_PYTHON_BIOFIELD_URL);
        assert_eq!(
            config.python_biofield_timeout_ms,
            DEFAULT_PYTHON_BIOFIELD_TIMEOUT_MS
        );
    }

    #[test]
    fn test_validate_rejects_invalid_python_biofield_url() {
        let config = ApiConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            jwt_secret: "test-secret-at-least-32-chars-long".to_string(),
            database_url: Some("postgres://localhost/test".to_string()),
            redis_url: None,
            allowed_origins: vec![],
            rate_limit_requests: 100,
            rate_limit_window_secs: 60,
            request_timeout_secs: 30,
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
            discord_client_id: None,
            discord_client_secret: None,
            discord_redirect_uri: None,
            dodo_payments_api_key: None,
            dodo_payments_webhook_key: None,
            dodo_payments_env: None,
            python_biofield_url: "biofield.internal:8002".to_string(),
            python_biofield_timeout_ms: DEFAULT_PYTHON_BIOFIELD_TIMEOUT_MS,
            gateway_url: None,
            gateway_token: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_rejects_zero_python_biofield_timeout() {
        let config = ApiConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            jwt_secret: "test-secret-at-least-32-chars-long".to_string(),
            database_url: Some("postgres://localhost/test".to_string()),
            redis_url: None,
            allowed_origins: vec![],
            rate_limit_requests: 100,
            rate_limit_window_secs: 60,
            request_timeout_secs: 30,
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
            discord_client_id: None,
            discord_client_secret: None,
            discord_redirect_uri: None,
            dodo_payments_api_key: None,
            dodo_payments_webhook_key: None,
            dodo_payments_env: None,
            python_biofield_url: DEFAULT_PYTHON_BIOFIELD_URL.to_string(),
            python_biofield_timeout_ms: 0,
            gateway_url: None,
            gateway_token: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_rejects_invalid_dodo_payments_env() {
        let config = ApiConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            jwt_secret: "test-secret-at-least-32-chars-long".to_string(),
            database_url: Some("postgres://localhost/test".to_string()),
            redis_url: None,
            allowed_origins: vec![],
            rate_limit_requests: 100,
            rate_limit_window_secs: 60,
            request_timeout_secs: 30,
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
            discord_client_id: None,
            discord_client_secret: None,
            discord_redirect_uri: None,
            dodo_payments_api_key: Some("dodo_key".to_string()),
            dodo_payments_webhook_key: Some("dodo_webhook".to_string()),
            dodo_payments_env: Some("staging".to_string()),
            python_biofield_url: DEFAULT_PYTHON_BIOFIELD_URL.to_string(),
            python_biofield_timeout_ms: DEFAULT_PYTHON_BIOFIELD_TIMEOUT_MS,
            gateway_url: None,
            gateway_token: None,
        };

        assert!(config.validate().is_err());
    }
}
