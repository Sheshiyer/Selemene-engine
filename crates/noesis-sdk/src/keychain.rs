//! Secure Credential Storage — Keychain integration for API keys
//!
//! Provides platform-native secure storage:
//! - macOS: Keychain Services
//! - Linux: Secret Service (GNOME Keyring, KDE Wallet)
//! - Windows: Credential Manager
//!
//! Falls back to plaintext file storage when keychain is unavailable.

use crate::{Error, Result};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Service name for keychain entries
const SERVICE_NAME: &str = "noesis-engine";
/// Username for the API key entry
const API_KEY_USERNAME: &str = "api_key";
/// Fallback file for plaintext storage
const FALLBACK_FILE: &str = ".noesis_credentials";

/// Trait for credential storage backends.
pub trait CredentialStore: Send + Sync {
    /// Store a credential securely.
    fn store(&self, key: &str, value: &str) -> Result<()>;

    /// Retrieve a credential.
    fn retrieve(&self, key: &str) -> Result<Option<String>>;

    /// Delete a credential.
    fn delete(&self, key: &str) -> Result<()>;

    /// Check if a credential exists.
    fn exists(&self, key: &str) -> bool {
        self.retrieve(key).map(|v| v.is_some()).unwrap_or(false)
    }
}

/// Keychain-based credential store (macOS/Linux/Windows).
///
/// Uses the `keyring` crate to access platform-native secure storage.
#[cfg(feature = "keychain")]
pub struct KeychainStore {
    service: String,
}

#[cfg(feature = "keychain")]
impl KeychainStore {
    /// Create a new keychain store with the default service name.
    pub fn new() -> Self {
        Self {
            service: SERVICE_NAME.into(),
        }
    }

    /// Create a keychain store with a custom service name.
    pub fn with_service(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }

    /// Store the API key.
    pub fn store_api_key(&self, api_key: &str) -> Result<()> {
        self.store(API_KEY_USERNAME, api_key)
    }

    /// Retrieve the API key.
    pub fn get_api_key(&self) -> Result<Option<String>> {
        self.retrieve(API_KEY_USERNAME)
    }

    /// Delete the API key.
    pub fn delete_api_key(&self) -> Result<()> {
        self.delete(API_KEY_USERNAME)
    }
}

#[cfg(feature = "keychain")]
impl Default for KeychainStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "keychain")]
impl CredentialStore for KeychainStore {
    fn store(&self, key: &str, value: &str) -> Result<()> {
        let entry = keyring::Entry::new(&self.service, key)
            .map_err(|e| Error::Credential(format!("Failed to create keychain entry: {}", e)))?;

        entry
            .set_password(value)
            .map_err(|e| Error::Credential(format!("Failed to store credential: {}", e)))?;

        info!("Stored credential '{}' in keychain", key);
        Ok(())
    }

    fn retrieve(&self, key: &str) -> Result<Option<String>> {
        let entry = keyring::Entry::new(&self.service, key)
            .map_err(|e| Error::Credential(format!("Failed to create keychain entry: {}", e)))?;

        match entry.get_password() {
            Ok(password) => {
                debug!("Retrieved credential '{}' from keychain", key);
                Ok(Some(password))
            }
            Err(keyring::Error::NoEntry) => {
                debug!("Credential '{}' not found in keychain", key);
                Ok(None)
            }
            Err(e) => Err(Error::Credential(format!(
                "Failed to retrieve credential: {}",
                e
            ))),
        }
    }

    fn delete(&self, key: &str) -> Result<()> {
        let entry = keyring::Entry::new(&self.service, key)
            .map_err(|e| Error::Credential(format!("Failed to create keychain entry: {}", e)))?;

        match entry.delete_credential() {
            Ok(()) => {
                info!("Deleted credential '{}' from keychain", key);
                Ok(())
            }
            Err(keyring::Error::NoEntry) => {
                debug!("Credential '{}' not found, nothing to delete", key);
                Ok(())
            }
            Err(e) => Err(Error::Credential(format!(
                "Failed to delete credential: {}",
                e
            ))),
        }
    }
}

/// Plaintext file-based credential store (fallback).
///
/// **Warning:** This stores credentials in plaintext and should only be used
/// when keychain is unavailable (e.g., headless Linux servers).
pub struct PlaintextStore {
    path: PathBuf,
}

impl PlaintextStore {
    /// Create a new plaintext store at the default location.
    pub fn new() -> Result<Self> {
        let path = dirs::home_dir()
            .ok_or_else(|| Error::Credential("Could not determine home directory".into()))?
            .join(FALLBACK_FILE);

        Ok(Self { path })
    }

    /// Create a plaintext store at a custom path.
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    fn read_all(&self) -> Result<std::collections::HashMap<String, String>> {
        if !self.path.exists() {
            return Ok(std::collections::HashMap::new());
        }

        let content = fs::read_to_string(&self.path)
            .map_err(|e| Error::Credential(format!("Failed to read credentials file: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| Error::Credential(format!("Failed to parse credentials file: {}", e)))
    }

    fn write_all(&self, creds: &std::collections::HashMap<String, String>) -> Result<()> {
        let content = serde_json::to_string_pretty(creds)
            .map_err(|e| Error::Credential(format!("Failed to serialize credentials: {}", e)))?;

        fs::write(&self.path, content)
            .map_err(|e| Error::Credential(format!("Failed to write credentials file: {}", e)))?;

        // Set restrictive permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&self.path, perms)
                .map_err(|e| Error::Credential(format!("Failed to set file permissions: {}", e)))?;
        }

        Ok(())
    }

    /// Store the API key.
    pub fn store_api_key(&self, api_key: &str) -> Result<()> {
        warn!("Storing API key in plaintext file (keychain unavailable)");
        self.store(API_KEY_USERNAME, api_key)
    }

    /// Retrieve the API key.
    pub fn get_api_key(&self) -> Result<Option<String>> {
        self.retrieve(API_KEY_USERNAME)
    }
}

impl Default for PlaintextStore {
    fn default() -> Self {
        Self::new().expect("Failed to create plaintext store")
    }
}

impl CredentialStore for PlaintextStore {
    fn store(&self, key: &str, value: &str) -> Result<()> {
        let mut creds = self.read_all()?;
        creds.insert(key.into(), value.into());
        self.write_all(&creds)?;
        info!("Stored credential '{}' in plaintext file", key);
        Ok(())
    }

    fn retrieve(&self, key: &str) -> Result<Option<String>> {
        let creds = self.read_all()?;
        Ok(creds.get(key).cloned())
    }

    fn delete(&self, key: &str) -> Result<()> {
        let mut creds = self.read_all()?;
        creds.remove(key);
        self.write_all(&creds)?;
        info!("Deleted credential '{}' from plaintext file", key);
        Ok(())
    }
}

/// Auto-selecting credential store.
///
/// Tries keychain first, falls back to plaintext if unavailable.
pub fn auto_store() -> Box<dyn CredentialStore> {
    #[cfg(feature = "keychain")]
    {
        // Try keychain first
        let store = KeychainStore::new();
        // Test if keychain is available by attempting to access it
        if store.retrieve("__test_availability__").is_ok() {
            debug!("Using keychain for credential storage");
            return Box::new(store);
        }
        warn!("Keychain unavailable, falling back to plaintext storage");
    }

    // Fallback to plaintext
    Box::new(PlaintextStore::default())
}

/// Helper function to store API key using auto-selected backend.
pub fn store_api_key(api_key: &str) -> Result<()> {
    auto_store().store(API_KEY_USERNAME, api_key)
}

/// Helper function to retrieve API key using auto-selected backend.
pub fn get_api_key() -> Result<Option<String>> {
    auto_store().retrieve(API_KEY_USERNAME)
}

/// Helper function to delete API key using auto-selected backend.
pub fn delete_api_key() -> Result<()> {
    auto_store().delete(API_KEY_USERNAME)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_plaintext_store() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_creds");
        let store = PlaintextStore::with_path(&path);

        // Store
        store.store("test_key", "test_value").unwrap();
        assert!(path.exists());

        // Retrieve
        let value = store.retrieve("test_key").unwrap();
        assert_eq!(value, Some("test_value".into()));

        // Delete
        store.delete("test_key").unwrap();
        let value = store.retrieve("test_key").unwrap();
        assert!(value.is_none());
    }

    #[test]
    fn test_plaintext_api_key() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_creds");
        let store = PlaintextStore::with_path(&path);

        store.store_api_key("nk_test_123").unwrap();
        let key = store.get_api_key().unwrap();
        assert_eq!(key, Some("nk_test_123".into()));
    }

    #[test]
    fn test_credential_exists() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_creds");
        let store = PlaintextStore::with_path(&path);

        assert!(!store.exists("nonexistent"));

        store.store("exists", "value").unwrap();
        assert!(store.exists("exists"));
    }

    #[cfg(feature = "keychain")]
    #[test]
    #[ignore] // Requires actual keychain access
    fn test_keychain_store() {
        let store = KeychainStore::with_service("noesis-sdk-test");

        // Clean up first
        let _ = store.delete("test_key");

        // Store
        store.store("test_key", "test_value").unwrap();

        // Retrieve
        let value = store.retrieve("test_key").unwrap();
        assert_eq!(value, Some("test_value".into()));

        // Delete
        store.delete("test_key").unwrap();
        let value = store.retrieve("test_key").unwrap();
        assert!(value.is_none());
    }
}
