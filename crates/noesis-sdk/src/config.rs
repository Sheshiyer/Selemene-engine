//! Configuration Module — Runtime settings for the SDK
//!
//! Loads configuration from:
//! 1. Environment variables (highest priority)
//! 2. Config file (~/.noesis/config.toml)
//! 3. Built-in defaults

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info};

/// Default API URL
const DEFAULT_API_URL: &str = "https://selemene.tryambakam.space";
/// Default request timeout in milliseconds
const DEFAULT_TIMEOUT_MS: u64 = 30_000;
/// Default cache TTL in seconds
const DEFAULT_CACHE_TTL_SECS: u64 = 300;
/// Config file name
const CONFIG_FILE: &str = "config.toml";
/// Noesis directory name
const NOESIS_DIR: &str = ".noesis";

/// SDK configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// API base URL
    #[serde(default = "default_api_url")]
    pub api_url: String,

    /// API key (can also be set via NOESIS_API_KEY env var)
    #[serde(default)]
    pub api_key: Option<String>,

    /// Request timeout in milliseconds
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,

    /// Cache TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_secs: u64,

    /// Whether to enable response caching
    #[serde(default = "default_cache_enabled")]
    pub cache_enabled: bool,

    /// Engine-specific settings
    #[serde(default)]
    pub engines: HashMap<String, EngineConfig>,
}

fn default_api_url() -> String {
    DEFAULT_API_URL.into()
}

fn default_timeout_ms() -> u64 {
    DEFAULT_TIMEOUT_MS
}

fn default_cache_ttl() -> u64 {
    DEFAULT_CACHE_TTL_SECS
}

fn default_cache_enabled() -> bool {
    true
}

impl Config {
    /// Load configuration with the following priority:
    /// 1. Environment variables
    /// 2. Config file (~/.noesis/config.toml)
    /// 3. Defaults
    pub fn load() -> Result<Self> {
        let mut config = Self::default();

        // Try to load from config file
        if let Ok(file_config) = Self::load_from_file() {
            config = config.merge(file_config);
            debug!("Loaded config from file");
        }

        // Override with environment variables
        config = config.with_env_overrides();

        info!(
            "Config loaded: api_url={}, timeout={}ms, cache={}",
            config.api_url, config.timeout_ms, config.cache_enabled
        );

        Ok(config)
    }

    /// Load configuration from the default config file.
    pub fn load_from_file() -> Result<Self> {
        let path = Self::path()?;
        Self::load_from_path(&path)
    }

    /// Load configuration from a specific path.
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Err(Error::Config(format!("Config file not found: {:?}", path)));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("Failed to read config: {}", e)))?;

        toml::from_str(&content)
            .map_err(|e| Error::Config(format!("Failed to parse config: {}", e)))
    }

    /// Get the default config directory (~/.noesis).
    pub fn dir() -> Result<PathBuf> {
        dirs::home_dir()
            .map(|home| home.join(NOESIS_DIR))
            .ok_or_else(|| Error::Config("Could not determine home directory".into()))
    }

    /// Get the default config file path.
    pub fn path() -> Result<PathBuf> {
        Ok(Self::dir()?.join(CONFIG_FILE))
    }

    /// Save configuration to the default config file.
    pub fn save(&self) -> Result<()> {
        let dir = Self::dir()?;
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .map_err(|e| Error::Config(format!("Failed to create config dir: {}", e)))?;
        }

        let path = Self::path()?;
        let content = toml::to_string_pretty(self)
            .map_err(|e| Error::Config(format!("Failed to serialize config: {}", e)))?;

        fs::write(&path, content)
            .map_err(|e| Error::Config(format!("Failed to write config: {}", e)))?;

        info!("Saved config to {:?}", path);
        Ok(())
    }

    /// Merge another config into this one (other takes priority).
    fn merge(mut self, other: Self) -> Self {
        if other.api_url != DEFAULT_API_URL {
            self.api_url = other.api_url;
        }
        if other.api_key.is_some() {
            self.api_key = other.api_key;
        }
        if other.timeout_ms != DEFAULT_TIMEOUT_MS {
            self.timeout_ms = other.timeout_ms;
        }
        if other.cache_ttl_secs != DEFAULT_CACHE_TTL_SECS {
            self.cache_ttl_secs = other.cache_ttl_secs;
        }
        self.cache_enabled = other.cache_enabled;
        self.engines.extend(other.engines);
        self
    }

    /// Apply environment variable overrides.
    fn with_env_overrides(mut self) -> Self {
        // NOESIS_API_URL
        if let Ok(url) = std::env::var("NOESIS_API_URL") {
            debug!("Using NOESIS_API_URL from environment");
            self.api_url = url;
        }

        // NOESIS_API_KEY
        if let Ok(key) = std::env::var("NOESIS_API_KEY") {
            debug!("Using NOESIS_API_KEY from environment");
            self.api_key = Some(key);
        }

        // NOESIS_TIMEOUT_MS
        if let Ok(timeout) = std::env::var("NOESIS_TIMEOUT_MS") {
            if let Ok(ms) = timeout.parse() {
                debug!("Using NOESIS_TIMEOUT_MS from environment");
                self.timeout_ms = ms;
            }
        }

        // NOESIS_CACHE_TTL
        if let Ok(ttl) = std::env::var("NOESIS_CACHE_TTL") {
            if let Ok(secs) = ttl.parse() {
                debug!("Using NOESIS_CACHE_TTL from environment");
                self.cache_ttl_secs = secs;
            }
        }

        // NOESIS_CACHE_ENABLED
        if let Ok(enabled) = std::env::var("NOESIS_CACHE_ENABLED") {
            self.cache_enabled = enabled.to_lowercase() == "true" || enabled == "1";
        }

        self
    }

    /// Get engine-specific config, falling back to defaults.
    pub fn engine_config(&self, engine_id: &str) -> EngineConfig {
        self.engines.get(engine_id).cloned().unwrap_or_default()
    }

    /// Create a builder for programmatic configuration.
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: DEFAULT_API_URL.into(),
            api_key: None,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            cache_ttl_secs: DEFAULT_CACHE_TTL_SECS,
            cache_enabled: true,
            engines: HashMap::new(),
        }
    }
}

/// Engine-specific configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Override timeout for this engine
    pub timeout_ms: Option<u64>,
    /// Override cache TTL for this engine
    pub cache_ttl_secs: Option<u64>,
    /// Engine-specific options
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
}

/// Builder for Config.
#[derive(Default)]
pub struct ConfigBuilder {
    api_url: Option<String>,
    api_key: Option<String>,
    timeout_ms: Option<u64>,
    cache_ttl_secs: Option<u64>,
    cache_enabled: Option<bool>,
    engines: HashMap<String, EngineConfig>,
}

impl ConfigBuilder {
    /// Set the API URL.
    pub fn api_url(mut self, url: impl Into<String>) -> Self {
        self.api_url = Some(url.into());
        self
    }

    /// Set the API key.
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set the request timeout.
    pub fn timeout_ms(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }

    /// Set the cache TTL.
    pub fn cache_ttl_secs(mut self, secs: u64) -> Self {
        self.cache_ttl_secs = Some(secs);
        self
    }

    /// Enable or disable caching.
    pub fn cache_enabled(mut self, enabled: bool) -> Self {
        self.cache_enabled = Some(enabled);
        self
    }

    /// Add engine-specific config.
    pub fn engine(mut self, engine_id: impl Into<String>, config: EngineConfig) -> Self {
        self.engines.insert(engine_id.into(), config);
        self
    }

    /// Build the configuration.
    pub fn build(self) -> Config {
        Config {
            api_url: self.api_url.unwrap_or_else(default_api_url),
            api_key: self.api_key,
            timeout_ms: self.timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS),
            cache_ttl_secs: self.cache_ttl_secs.unwrap_or(DEFAULT_CACHE_TTL_SECS),
            cache_enabled: self.cache_enabled.unwrap_or(true),
            engines: self.engines,
        }
    }
}

/// Generate a sample config file content.
pub fn sample_config() -> String {
    r#"# Noesis SDK Configuration
# See https://selemene.tryambakam.space/docs for details

# API endpoint (default: production server)
api_url = "https://selemene.tryambakam.space"

# API key (can also be set via NOESIS_API_KEY environment variable)
# api_key = "nk_your_key_here"

# Request timeout in milliseconds
timeout_ms = 30000

# Response cache TTL in seconds
cache_ttl_secs = 300

# Enable response caching
cache_enabled = true

# Engine-specific settings
[engines.panchanga]
cache_ttl_secs = 3600  # Cache panchanga for 1 hour

[engines.biorhythm]
cache_ttl_secs = 86400  # Cache biorhythm for 24 hours
"#
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.api_url, DEFAULT_API_URL);
        assert_eq!(config.timeout_ms, DEFAULT_TIMEOUT_MS);
        assert!(config.cache_enabled);
        assert!(config.api_key.is_none());
    }

    #[test]
    fn test_config_builder() {
        let config = Config::builder()
            .api_url("http://localhost:8080")
            .api_key("test_key")
            .timeout_ms(5000)
            .cache_enabled(false)
            .build();

        assert_eq!(config.api_url, "http://localhost:8080");
        assert_eq!(config.api_key, Some("test_key".into()));
        assert_eq!(config.timeout_ms, 5000);
        assert!(!config.cache_enabled);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml).unwrap();

        assert_eq!(parsed.api_url, config.api_url);
        assert_eq!(parsed.timeout_ms, config.timeout_ms);
    }

    #[test]
    fn test_config_save_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let config = Config::builder()
            .api_url("http://test.local")
            .api_key("test_key_123")
            .build();

        // Save
        let content = toml::to_string_pretty(&config).unwrap();
        fs::write(&path, content).unwrap();

        // Load
        let loaded = Config::load_from_path(&path.into()).unwrap();
        assert_eq!(loaded.api_url, "http://test.local");
        assert_eq!(loaded.api_key, Some("test_key_123".into()));
    }

    #[test]
    fn test_engine_config() {
        let mut config = Config::default();
        config.engines.insert(
            "panchanga".into(),
            EngineConfig {
                timeout_ms: Some(5000),
                cache_ttl_secs: Some(3600),
                options: HashMap::new(),
            },
        );

        let panchanga_config = config.engine_config("panchanga");
        assert_eq!(panchanga_config.timeout_ms, Some(5000));
        assert_eq!(panchanga_config.cache_ttl_secs, Some(3600));

        let default_config = config.engine_config("unknown");
        assert!(default_config.timeout_ms.is_none());
    }

    #[test]
    fn test_sample_config() {
        let sample = sample_config();
        assert!(sample.contains("api_url"));
        assert!(sample.contains("timeout_ms"));

        // Ensure it parses
        let _parsed: Config = toml::from_str(&sample).unwrap();
    }
}
