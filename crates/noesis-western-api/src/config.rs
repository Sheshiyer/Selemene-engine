use std::env;
use crate::error::{Result, WesternApiError};

#[derive(Debug, Clone)]
pub struct Config {
    pub api_key: String,
    pub base_url: String,
    pub timeout_seconds: u64,
}

impl Config {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://json.freeastrologyapi.com".to_string(), // Default base URL
            timeout_seconds: 10,
        }
    }

    pub fn from_env() -> Result<Self> {
        let api_key = env::var("FREE_ASTROLOGY_API_KEY")
            .map_err(|_| WesternApiError::ConfigError("FREE_ASTROLOGY_API_KEY not set".to_string()))?;
            
        let base_url = env::var("FREE_ASTROLOGY_API_BASE_URL")
            .unwrap_or_else(|_| "https://json.freeastrologyapi.com".to_string());
            
        Ok(Self {
            api_key,
            base_url,
            timeout_seconds: 10,
        })
    }
}
