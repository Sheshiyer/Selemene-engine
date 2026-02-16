//! Local Profile Management — Birth data and preferences persistence
//!
//! Stores user profile data locally at `~/.noesis/profile.json`.

use crate::{Error, Result};
use chrono::{DateTime, Utc};
use noesis_core::{BirthData, Coordinates, EngineInput, Precision};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info};

/// Default profile directory name
const NOESIS_DIR: &str = ".noesis";
/// Profile filename
const PROFILE_FILE: &str = "profile.json";

/// Local user profile containing birth data and preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalProfile {
    /// Schema version for future migrations
    pub version: u32,
    /// User's display name
    pub name: String,
    /// Birth data for chart-based calculations
    pub birth_data: BirthData,
    /// Default calculation precision
    #[serde(default)]
    pub default_precision: Precision,
    /// Engine-specific preferences
    #[serde(default)]
    pub preferences: HashMap<String, serde_json::Value>,
    /// When the profile was created
    pub created_at: DateTime<Utc>,
    /// When the profile was last updated
    pub updated_at: DateTime<Utc>,
}

impl LocalProfile {
    /// Create a new profile with the given birth data.
    pub fn new(name: impl Into<String>, birth_data: BirthData) -> Self {
        let now = Utc::now();
        Self {
            version: 1,
            name: name.into(),
            birth_data,
            default_precision: Precision::Standard,
            preferences: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Get the default profile directory path (~/.noesis).
    pub fn dir() -> Result<PathBuf> {
        dirs::home_dir()
            .map(|home| home.join(NOESIS_DIR))
            .ok_or_else(|| Error::Profile("Could not determine home directory".into()))
    }

    /// Get the profile file path (~/.noesis/profile.json).
    pub fn path() -> Result<PathBuf> {
        Ok(Self::dir()?.join(PROFILE_FILE))
    }

    /// Check if a profile exists.
    pub fn exists() -> bool {
        Self::path().map(|p| p.exists()).unwrap_or(false)
    }

    /// Load the profile from disk.
    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        debug!("Loading profile from {:?}", path);

        let content = fs::read_to_string(&path).map_err(|e| {
            Error::Profile(format!("Failed to read profile at {:?}: {}", path, e))
        })?;

        serde_json::from_str(&content).map_err(|e| {
            Error::Profile(format!("Failed to parse profile: {}", e))
        })
    }

    /// Load profile if it exists, otherwise create a new empty one.
    pub fn load_or_default() -> Result<Option<Self>> {
        if Self::exists() {
            Ok(Some(Self::load()?))
        } else {
            Ok(None)
        }
    }

    /// Save the profile to disk.
    pub fn save(&mut self) -> Result<()> {
        self.updated_at = Utc::now();

        let dir = Self::dir()?;
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| {
                Error::Profile(format!("Failed to create profile directory: {}", e))
            })?;
            info!("Created profile directory at {:?}", dir);
        }

        let path = Self::path()?;
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            Error::Profile(format!("Failed to serialize profile: {}", e))
        })?;

        fs::write(&path, content).map_err(|e| {
            Error::Profile(format!("Failed to write profile: {}", e))
        })?;

        info!("Saved profile to {:?}", path);
        Ok(())
    }

    /// Delete the profile from disk.
    pub fn delete() -> Result<()> {
        let path = Self::path()?;
        if path.exists() {
            fs::remove_file(&path).map_err(|e| {
                Error::Profile(format!("Failed to delete profile: {}", e))
            })?;
            info!("Deleted profile at {:?}", path);
        }
        Ok(())
    }

    /// Convert this profile to an EngineInput for API calls.
    pub fn to_engine_input(&self) -> EngineInput {
        EngineInput {
            birth_data: Some(self.birth_data.clone()),
            current_time: Utc::now(),
            location: Some(Coordinates {
                latitude: self.birth_data.latitude,
                longitude: self.birth_data.longitude,
                altitude: None,
            }),
            precision: self.default_precision,
            options: HashMap::new(),
        }
    }

    /// Convert to EngineInput with custom options.
    pub fn to_engine_input_with_options(
        &self,
        options: HashMap<String, serde_json::Value>,
    ) -> EngineInput {
        let mut input = self.to_engine_input();
        input.options = options;
        input
    }

    /// Update the birth data and save.
    pub fn update_birth_data(&mut self, birth_data: BirthData) -> Result<()> {
        self.birth_data = birth_data;
        self.save()
    }

    /// Update a preference and save.
    pub fn set_preference(
        &mut self,
        key: impl Into<String>,
        value: serde_json::Value,
    ) -> Result<()> {
        self.preferences.insert(key.into(), value);
        self.save()
    }

    /// Get a preference value.
    pub fn get_preference(&self, key: &str) -> Option<&serde_json::Value> {
        self.preferences.get(key)
    }

    /// Validate the profile data.
    pub fn validate(&self) -> Result<()> {
        self.birth_data.validate().map_err(|e| {
            Error::Profile(format!("Invalid birth data: {}", e))
        })?;

        if self.name.trim().is_empty() {
            return Err(Error::Profile("Name cannot be empty".into()));
        }

        Ok(())
    }
}

impl Default for LocalProfile {
    fn default() -> Self {
        Self::new(
            "Unknown",
            BirthData {
                name: None,
                date: "1990-01-01".into(),
                time: None,
                latitude: 0.0,
                longitude: 0.0,
                timezone: "UTC".into(),
            },
        )
    }
}

/// Builder for creating LocalProfile with validation.
pub struct ProfileBuilder {
    name: Option<String>,
    date: Option<String>,
    time: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    timezone: Option<String>,
}

impl ProfileBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            date: None,
            time: None,
            latitude: None,
            longitude: None,
            timezone: None,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn date(mut self, date: impl Into<String>) -> Self {
        self.date = Some(date.into());
        self
    }

    pub fn time(mut self, time: impl Into<String>) -> Self {
        self.time = Some(time.into());
        self
    }

    pub fn latitude(mut self, lat: f64) -> Self {
        self.latitude = Some(lat);
        self
    }

    pub fn longitude(mut self, lng: f64) -> Self {
        self.longitude = Some(lng);
        self
    }

    pub fn timezone(mut self, tz: impl Into<String>) -> Self {
        self.timezone = Some(tz.into());
        self
    }

    pub fn build(self) -> Result<LocalProfile> {
        let name = self.name.ok_or_else(|| Error::Profile("Name is required".into()))?;
        let date = self.date.ok_or_else(|| Error::Profile("Date is required".into()))?;
        let latitude = self.latitude.ok_or_else(|| Error::Profile("Latitude is required".into()))?;
        let longitude = self.longitude.ok_or_else(|| Error::Profile("Longitude is required".into()))?;
        let timezone = self.timezone.ok_or_else(|| Error::Profile("Timezone is required".into()))?;

        let birth_data = BirthData {
            name: Some(name.clone()),
            date,
            time: self.time,
            latitude,
            longitude,
            timezone,
        };

        birth_data.validate().map_err(|e| Error::Profile(e))?;

        Ok(LocalProfile::new(name, birth_data))
    }
}

impl Default for ProfileBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_profile() -> LocalProfile {
        LocalProfile::new(
            "Test User",
            BirthData {
                name: Some("Test User".into()),
                date: "1990-05-15".into(),
                time: Some("14:30".into()),
                latitude: 12.9716,
                longitude: 77.5946,
                timezone: "Asia/Kolkata".into(),
            },
        )
    }

    #[test]
    fn test_profile_creation() {
        let profile = test_profile();
        assert_eq!(profile.name, "Test User");
        assert_eq!(profile.birth_data.date, "1990-05-15");
        assert_eq!(profile.version, 1);
    }

    #[test]
    fn test_profile_to_engine_input() {
        let profile = test_profile();
        let input = profile.to_engine_input();

        assert!(input.birth_data.is_some());
        let bd = input.birth_data.unwrap();
        assert_eq!(bd.date, "1990-05-15");
        assert_eq!(bd.latitude, 12.9716);
    }

    #[test]
    fn test_profile_serialization() {
        let profile = test_profile();
        let json = serde_json::to_string(&profile).unwrap();
        let parsed: LocalProfile = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, profile.name);
        assert_eq!(parsed.birth_data.date, profile.birth_data.date);
    }

    #[test]
    fn test_profile_validation() {
        let valid = test_profile();
        assert!(valid.validate().is_ok());

        let mut invalid = test_profile();
        invalid.birth_data.latitude = 999.0;
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_profile_builder() {
        let profile = ProfileBuilder::new()
            .name("Builder Test")
            .date("1995-03-20")
            .time("10:00")
            .latitude(40.7128)
            .longitude(-74.0060)
            .timezone("America/New_York")
            .build()
            .unwrap();

        assert_eq!(profile.name, "Builder Test");
        assert_eq!(profile.birth_data.latitude, 40.7128);
    }

    #[test]
    fn test_profile_builder_missing_field() {
        let result = ProfileBuilder::new()
            .name("Incomplete")
            .build();

        assert!(result.is_err());
    }
}
