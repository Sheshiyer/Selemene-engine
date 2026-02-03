//! Configuration for FreeAstrologyAPI.com integration

use std::env;
use crate::error::{Result, VedicApiError};

/// Configuration for the Vedic API client
#[derive(Debug, Clone)]
pub struct Config {
    /// API key for FreeAstrologyAPI.com
    pub api_key: String,
    
    /// Base URL for the API
    pub base_url: String,
    
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    
    /// Number of retry attempts
    pub retry_count: u32,
    
    /// Cache TTL in seconds for birth data (infinite = 0)
    pub cache_ttl_birth_data: u64,
    
    /// Cache TTL in seconds for daily data
    pub cache_ttl_daily: u64,
    
    /// Provider type: "api" or "native"
    pub provider: ProviderType,
    
    /// Enable fallback to native calculations
    pub fallback_enabled: bool,
}

/// Provider type for Vedic calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderType {
    /// Use FreeAstrologyAPI.com
    Api,
    /// Use native calculations
    Native,
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::Api => write!(f, "api"),
            ProviderType::Native => write!(f, "native"),
        }
    }
}

impl std::str::FromStr for ProviderType {
    type Err = String;
    
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "api" => Ok(ProviderType::Api),
            "native" => Ok(ProviderType::Native),
            _ => Err(format!("Unknown provider type: {}", s)),
        }
    }
}

impl Config {
    /// Create a new configuration with explicit values
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: crate::API_BASE_URL.to_string(),
            timeout_seconds: 30,
            retry_count: 3,
            cache_ttl_birth_data: 0, // infinite
            cache_ttl_daily: 86400,  // 24 hours
            provider: ProviderType::Api,
            fallback_enabled: true,
        }
    }
    
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let api_key = env::var("FREE_ASTROLOGY_API_KEY")
            .map_err(|_| VedicApiError::Configuration {
                field: "FREE_ASTROLOGY_API_KEY".to_string(),
                message: "API key not found in environment. Get your free key at https://freeastrologyapi.com".to_string(),
            })?;
        
        let base_url = env::var("FREE_ASTROLOGY_API_BASE_URL")
            .unwrap_or_else(|_| crate::API_BASE_URL.to_string());
        
        let timeout_seconds = env::var("FREE_ASTROLOGY_API_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);
        
        let retry_count = env::var("FREE_ASTROLOGY_API_RETRY_COUNT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3);
        
        let cache_ttl_birth_data = env::var("FREE_ASTROLOGY_CACHE_BIRTH_TTL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        
        let cache_ttl_daily = env::var("FREE_ASTROLOGY_CACHE_DAILY_TTL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(86400);
        
        let provider = env::var("VEDIC_ENGINE_PROVIDER")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(ProviderType::Api);
        
        let fallback_enabled = env::var("VEDIC_ENGINE_FALLBACK_ENABLED")
            .ok()
            .map(|s| s.parse().unwrap_or(true))
            .unwrap_or(true);
        
        Ok(Self {
            api_key,
            base_url,
            timeout_seconds,
            retry_count,
            cache_ttl_birth_data,
            cache_ttl_daily,
            provider,
            fallback_enabled,
        })
    }
    
    /// Check if the API provider is enabled
    pub fn is_api_enabled(&self) -> bool {
        matches!(self.provider, ProviderType::Api)
    }
    
    /// Set the base URL (useful for testing)
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }
    
    /// Set the timeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }
    
    /// Set retry count
    pub fn with_retry_count(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }
    
    /// Mask the API key for logging
    pub fn masked_api_key(&self) -> String {
        if self.api_key.len() <= 8 {
            "***".to_string()
        } else {
            format!("{}...{}", 
                &self.api_key[..4], 
                &self.api_key[self.api_key.len()-4..]
            )
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: crate::API_BASE_URL.to_string(),
            timeout_seconds: 30,
            retry_count: 3,
            cache_ttl_birth_data: 0,
            cache_ttl_daily: 86400,
            provider: ProviderType::Api,
            fallback_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let config = Config::new("test_key");
        assert_eq!(config.api_key, "test_key");
        assert!(config.is_api_enabled());
    }

    #[test]
    fn test_masked_api_key() {
        let config = Config::new("sjpRMWCOn340T8JHI8yeL7ucH1741GYT7eMFBMWO");
        let masked = config.masked_api_key();
        assert!(masked.starts_with("sjpR"));
        assert!(masked.ends_with("MWO"));
        assert!(masked.contains("..."));
    }

    #[test]
    fn test_provider_type_from_str() {
        assert_eq!("api".parse::<ProviderType>().unwrap(), ProviderType::Api);
        assert_eq!("native".parse::<ProviderType>().unwrap(), ProviderType::Native);
        assert!("invalid".parse::<ProviderType>().is_err());
    }
}
