//! Local Profile Management — Birth data and preferences persistence
//!
//! Stores user profile data locally at `~/.noesis/profile.json`.

use crate::{client::UpdateUserRequest, Error, NoesisClient, Result};
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
    /// Consciousness level (0-5): Dormant → Glimpsing → Practicing → Integrated → Embodied
    #[serde(default)]
    pub consciousness_level: u8,
    /// Engine-specific preferences
    #[serde(default)]
    pub preferences: HashMap<String, serde_json::Value>,
    /// When this profile was last synchronized with server-side profile
    #[serde(default)]
    pub last_synced_at: Option<DateTime<Utc>>,
    /// When the profile was created
    pub created_at: DateTime<Utc>,
    /// When the profile was last updated
    pub updated_at: DateTime<Utc>,
}

/// Result summary from LocalProfile sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub pushed_changes: bool,
    pub pulled_server_state: bool,
    pub synced_at: DateTime<Utc>,
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
            consciousness_level: 0,
            preferences: HashMap::new(),
            last_synced_at: None,
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

        let content = fs::read_to_string(&path)
            .map_err(|e| Error::Profile(format!("Failed to read profile at {:?}: {}", path, e)))?;

        serde_json::from_str(&content)
            .map_err(|e| Error::Profile(format!("Failed to parse profile: {}", e)))
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
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| Error::Profile(format!("Failed to serialize profile: {}", e)))?;

        fs::write(&path, content)
            .map_err(|e| Error::Profile(format!("Failed to write profile: {}", e)))?;

        info!("Saved profile to {:?}", path);
        Ok(())
    }

    /// Delete the profile from disk.
    pub fn delete() -> Result<()> {
        let path = Self::path()?;
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| Error::Profile(format!("Failed to delete profile: {}", e)))?;
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
        self.birth_data
            .validate()
            .map_err(|e| Error::Profile(format!("Invalid birth data: {}", e)))?;

        if self.name.trim().is_empty() {
            return Err(Error::Profile("Name cannot be empty".into()));
        }

        Ok(())
    }

    /// Sync local profile with server profile using deterministic conflict resolution.
    ///
    /// Conflict policy:
    /// - `consciousness_level`: **server wins**
    /// - `birth_data`: **local wins**
    ///
    /// The method computes a diff against server state and only PATCHes fields that differ.
    pub async fn sync(&mut self, client: &NoesisClient) -> Result<SyncResult> {
        let server_profile = client.get_me().await?;
        let patch = self.build_sync_patch(&server_profile);

        let mut pushed_changes = false;
        if !patch.is_empty() {
            client.update_me(&patch).await?;
            pushed_changes = true;
        }

        // Deterministic merge policy
        self.consciousness_level =
            server_profile.consciousness_level.clamp(0, u8::MAX as i32) as u8;

        let synced_at = Utc::now();
        self.last_synced_at = Some(synced_at);

        Ok(SyncResult {
            pushed_changes,
            pulled_server_state: true,
            synced_at,
        })
    }

    fn build_sync_patch(&self, server_profile: &crate::client::UserProfile) -> UpdateUserRequest {
        let mut patch = UpdateUserRequest::default();

        // Local profile name wins over server full_name.
        if !self.name.trim().is_empty() && self.name != server_profile.full_name {
            patch.full_name = Some(self.name.clone());
        }

        // Birth data is local-wins.
        if Some(self.birth_data.date.clone()) != server_profile.birth_date {
            patch.birth_date = Some(self.birth_data.date.clone());
        }
        if self.birth_data.time != server_profile.birth_time {
            patch.birth_time = self.birth_data.time.clone();
        }

        let server_lat = server_profile.birth_location.as_ref().map(|loc| loc.lat);
        let server_lng = server_profile.birth_location.as_ref().map(|loc| loc.lng);
        let server_loc_name = server_profile
            .birth_location
            .as_ref()
            .and_then(|loc| loc.name.clone());

        if Some(self.birth_data.latitude) != server_lat {
            patch.birth_location_lat = Some(self.birth_data.latitude);
        }
        if Some(self.birth_data.longitude) != server_lng {
            patch.birth_location_lng = Some(self.birth_data.longitude);
        }
        if self.birth_data.name != server_loc_name {
            patch.birth_location_name = self.birth_data.name.clone();
        }

        if Some(self.birth_data.timezone.clone()) != server_profile.timezone {
            patch.timezone = Some(self.birth_data.timezone.clone());
        }

        let local_preferences =
            serde_json::to_value(&self.preferences).unwrap_or(serde_json::json!({}));
        if local_preferences != server_profile.preferences {
            patch.preferences = Some(local_preferences);
        }

        patch
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
        let name = self
            .name
            .ok_or_else(|| Error::Profile("Name is required".into()))?;
        let date = self
            .date
            .ok_or_else(|| Error::Profile("Date is required".into()))?;
        let latitude = self
            .latitude
            .ok_or_else(|| Error::Profile("Latitude is required".into()))?;
        let longitude = self
            .longitude
            .ok_or_else(|| Error::Profile("Longitude is required".into()))?;
        let timezone = self
            .timezone
            .ok_or_else(|| Error::Profile("Timezone is required".into()))?;

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
        let result = ProfileBuilder::new().name("Incomplete").build();

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sync_pushes_local_birth_data_and_pulls_consciousness_level() {
        use wiremock::matchers::{header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/users/me"))
            .and(header("x-api-key", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "u1",
                "email": "test@example.com",
                "full_name": "Server Name",
                "tier": "free",
                "consciousness_level": 4,
                "experience_points": 120,
                "birth_date": "1991-01-01",
                "birth_time": "12:00:00",
                "birth_location": {"lat": 10.0, "lng": 20.0, "name": "Server City"},
                "timezone": "UTC",
                "preferences": {}
            })))
            .mount(&server)
            .await;

        Mock::given(method("PATCH"))
            .and(path("/api/v1/users/me"))
            .and(header("x-api-key", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "message": "Profile updated successfully"
            })))
            .mount(&server)
            .await;

        let client = NoesisClient::builder()
            .base_url(server.uri())
            .api_key("test-key")
            .build()
            .unwrap();

        let mut profile = LocalProfile::new(
            "Local Name",
            BirthData {
                name: Some("Local City".into()),
                date: "1990-05-15".into(),
                time: Some("14:30".into()),
                latitude: 12.9716,
                longitude: 77.5946,
                timezone: "Asia/Kolkata".into(),
            },
        );

        let sync_result = profile.sync(&client).await.unwrap();

        assert!(sync_result.pushed_changes);
        assert_eq!(profile.consciousness_level, 4);
        assert!(profile.last_synced_at.is_some());
    }

    #[tokio::test]
    async fn test_sync_no_patch_when_profiles_match() {
        use wiremock::matchers::{header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/users/me"))
            .and(header("x-api-key", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "u1",
                "email": "test@example.com",
                "full_name": "Local Name",
                "tier": "free",
                "consciousness_level": 2,
                "experience_points": 120,
                "birth_date": "1990-05-15",
                "birth_time": "14:30",
                "birth_location": {"lat": 12.9716, "lng": 77.5946, "name": "Local City"},
                "timezone": "Asia/Kolkata",
                "preferences": {}
            })))
            .mount(&server)
            .await;

        let client = NoesisClient::builder()
            .base_url(server.uri())
            .api_key("test-key")
            .build()
            .unwrap();

        let mut profile = LocalProfile::new(
            "Local Name",
            BirthData {
                name: Some("Local City".into()),
                date: "1990-05-15".into(),
                time: Some("14:30".into()),
                latitude: 12.9716,
                longitude: 77.5946,
                timezone: "Asia/Kolkata".into(),
            },
        );

        let sync_result = profile.sync(&client).await.unwrap();

        assert!(!sync_result.pushed_changes);
        assert_eq!(profile.consciousness_level, 2);
    }

    #[tokio::test]
    async fn test_sync_offline_resilience_keeps_local_state() {
        let client = NoesisClient::builder()
            .base_url("http://127.0.0.1:1")
            .api_key("test-key")
            .max_retries(0)
            .build()
            .unwrap();

        let mut profile = test_profile();
        let before = profile.clone();

        let result = profile.sync(&client).await;
        assert!(result.is_err());
        assert_eq!(profile.consciousness_level, before.consciousness_level);
        assert_eq!(profile.last_synced_at, before.last_synced_at);
    }
}
