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
/// Default max retries for retriable HTTP errors
const DEFAULT_MAX_RETRIES: u32 = 3;
/// Default exponential backoff base in milliseconds
const DEFAULT_BACKOFF_MS: u64 = 200;
/// Default maximum idle connections per host
const DEFAULT_POOL_MAX_IDLE_PER_HOST: usize = 16;
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

    /// Maximum retry attempts for retriable errors (5xx, timeout)
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Base backoff in milliseconds (exponential: base * 2^attempt)
    #[serde(default = "default_backoff_ms")]
    pub backoff_ms: u64,

    /// Maximum number of idle pooled connections per host
    #[serde(default = "default_pool_max_idle_per_host")]
    pub pool_max_idle_per_host: usize,

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

fn default_max_retries() -> u32 {
    DEFAULT_MAX_RETRIES
}

fn default_backoff_ms() -> u64 {
    DEFAULT_BACKOFF_MS
}

fn default_pool_max_idle_per_host() -> usize {
    DEFAULT_POOL_MAX_IDLE_PER_HOST
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
            "Config loaded: api_url={}, timeout={}ms, retries={}, backoff={}ms, pool_idle_per_host={}, cache={}",
            config.api_url,
            config.timeout_ms,
            config.max_retries,
            config.backoff_ms,
            config.pool_max_idle_per_host,
            config.cache_enabled
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
        if other.max_retries != DEFAULT_MAX_RETRIES {
            self.max_retries = other.max_retries;
        }
        if other.backoff_ms != DEFAULT_BACKOFF_MS {
            self.backoff_ms = other.backoff_ms;
        }
        if other.pool_max_idle_per_host != DEFAULT_POOL_MAX_IDLE_PER_HOST {
            self.pool_max_idle_per_host = other.pool_max_idle_per_host;
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

        // NOESIS_MAX_RETRIES
        if let Ok(max_retries) = std::env::var("NOESIS_MAX_RETRIES") {
            if let Ok(v) = max_retries.parse() {
                debug!("Using NOESIS_MAX_RETRIES from environment");
                self.max_retries = v;
            }
        }

        // NOESIS_BACKOFF_MS
        if let Ok(backoff_ms) = std::env::var("NOESIS_BACKOFF_MS") {
            if let Ok(v) = backoff_ms.parse() {
                debug!("Using NOESIS_BACKOFF_MS from environment");
                self.backoff_ms = v;
            }
        }

        // NOESIS_POOL_MAX_IDLE_PER_HOST
        if let Ok(pool_size) = std::env::var("NOESIS_POOL_MAX_IDLE_PER_HOST") {
            if let Ok(v) = pool_size.parse() {
                debug!("Using NOESIS_POOL_MAX_IDLE_PER_HOST from environment");
                self.pool_max_idle_per_host = v;
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
            max_retries: DEFAULT_MAX_RETRIES,
            backoff_ms: DEFAULT_BACKOFF_MS,
            pool_max_idle_per_host: DEFAULT_POOL_MAX_IDLE_PER_HOST,
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
    max_retries: Option<u32>,
    backoff_ms: Option<u64>,
    pool_max_idle_per_host: Option<usize>,
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

    /// Set the maximum number of retries for retriable requests.
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = Some(retries);
        self
    }

    /// Set base backoff in milliseconds for exponential retry backoff.
    pub fn backoff_ms(mut self, ms: u64) -> Self {
        self.backoff_ms = Some(ms);
        self
    }

    /// Set maximum idle pooled connections per host.
    pub fn pool_max_idle_per_host(mut self, pool_size: usize) -> Self {
        self.pool_max_idle_per_host = Some(pool_size);
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
            max_retries: self.max_retries.unwrap_or(DEFAULT_MAX_RETRIES),
            backoff_ms: self.backoff_ms.unwrap_or(DEFAULT_BACKOFF_MS),
            pool_max_idle_per_host: self
                .pool_max_idle_per_host
                .unwrap_or(DEFAULT_POOL_MAX_IDLE_PER_HOST),
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

# Retry behavior for 5xx and timeout errors
max_retries = 3
backoff_ms = 200

# Connection pool controls
pool_max_idle_per_host = 16

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
        assert_eq!(config.max_retries, DEFAULT_MAX_RETRIES);
        assert_eq!(config.backoff_ms, DEFAULT_BACKOFF_MS);
        assert_eq!(
            config.pool_max_idle_per_host,
            DEFAULT_POOL_MAX_IDLE_PER_HOST
        );
    }

    #[test]
    fn test_config_builder() {
        let config = Config::builder()
            .api_url("http://localhost:8080")
            .api_key("test_key")
            .timeout_ms(5000)
            .max_retries(5)
            .backoff_ms(150)
            .pool_max_idle_per_host(24)
            .cache_enabled(false)
            .build();

        assert_eq!(config.api_url, "http://localhost:8080");
        assert_eq!(config.api_key, Some("test_key".into()));
        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.backoff_ms, 150);
        assert_eq!(config.pool_max_idle_per_host, 24);
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
        let loaded = Config::load_from_path(&path).unwrap();
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
