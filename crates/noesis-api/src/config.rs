//! Environment-based configuration for Noesis API
//!
//! Loads configuration from environment variables with sensible defaults
//! for development and production environments.

use std::env;

/// API server configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Server host address (default: "0.0.0.0")
    pub host: String,
    
    /// Server port (default: 8080)
    pub port: u16,
    
    /// JWT secret for token signing (required, no default in production)
    pub jwt_secret: String,
    
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
}

impl ApiConfig {
    /// Load configuration from environment variables with defaults
    ///
    /// # Environment Variables
    /// - `HOST`: Server host address (default: "0.0.0.0")
    /// - `PORT`: Server port (default: 8080)
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
    /// Configured `ApiConfig` instance
    ///
    /// # Panics
    /// Panics if `JWT_SECRET` is not set and app is running in production mode
    /// (determined by `RUST_ENV=production` or absence of dev default)
    pub fn from_env() -> Self {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
        
        // JWT secret handling: require in production, allow default in dev
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
            let is_production = env::var("RUST_ENV")
                .map(|e| e == "production")
                .unwrap_or(false);
            
            if is_production {
                panic!("JWT_SECRET must be set in production environment");
            }
            
            tracing::warn!("JWT_SECRET not set, using development default (DO NOT USE IN PRODUCTION)");
            "noesis-dev-secret-change-in-production".to_string()
        });
        
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
        
        let log_level = env::var("RUST_LOG")
            .unwrap_or_else(|_| "info,noesis_api=debug".to_string());
        
        let log_format = env::var("LOG_FORMAT")
            .unwrap_or_else(|_| "pretty".to_string());
        
        Self {
            host,
            port,
            jwt_secret,
            redis_url,
            allowed_origins,
            rate_limit_requests,
            rate_limit_window_secs,
            request_timeout_secs,
            log_level,
            log_format,
        }
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
        
        // Validate JWT secret length (minimum 32 characters recommended)
        if self.jwt_secret.len() < 32 {
            tracing::warn!(
                "JWT_SECRET is only {} characters long, recommend at least 32 characters for security",
                self.jwt_secret.len()
            );
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
    
    #[test]
    fn test_bind_address() {
        let config = ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
            jwt_secret: "test-secret-at-least-32-chars-long".to_string(),
            redis_url: None,
            allowed_origins: vec![],
            rate_limit_requests: 100,
            rate_limit_window_secs: 60,
            request_timeout_secs: 30,
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
        };
        
        assert_eq!(config.bind_address(), "127.0.0.1:3000");
    }
    
    #[test]
    fn test_validate_invalid_window() {
        let config = ApiConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            jwt_secret: "test-secret-at-least-32-chars-long".to_string(),
            redis_url: None,
            allowed_origins: vec![],
            rate_limit_requests: 100,
            rate_limit_window_secs: 0, // Invalid!
            request_timeout_secs: 30,
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
        };
        
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_validate_invalid_timeout() {
        let config = ApiConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            jwt_secret: "test-secret-at-least-32-chars-long".to_string(),
            redis_url: None,
            allowed_origins: vec![],
            rate_limit_requests: 100,
            rate_limit_window_secs: 60,
            request_timeout_secs: 0, // Invalid!
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
        };
        
        assert!(config.validate().is_err());
    }
}
