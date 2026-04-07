use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

pub const BIOFIELD_SESSION_STATUS_ACTIVE: &str = "active";
pub const BIOFIELD_SESSION_STATUS_CLOSED: &str = "closed";
pub const BIOFIELD_SESSION_STATUS_ABANDONED: &str = "abandoned";

pub const BIOFIELD_CAPTURE_ARTIFACT_SOURCE_IMAGE: &str = "source-image";
pub const BIOFIELD_CAPTURE_ARTIFACT_SEGMENTATION_MASK: &str = "segmentation-mask";
pub const BIOFIELD_CAPTURE_ARTIFACT_ANALYSIS_OVERLAY: &str = "analysis-overlay";
pub const BIOFIELD_CAPTURE_ARTIFACT_THUMBNAIL: &str = "thumbnail";

pub const BIOFIELD_SESSION_STATUSES: [&str; 3] = [
    BIOFIELD_SESSION_STATUS_ACTIVE,
    BIOFIELD_SESSION_STATUS_CLOSED,
    BIOFIELD_SESSION_STATUS_ABANDONED,
];

pub const BIOFIELD_CAPTURE_ARTIFACT_KINDS: [&str; 4] = [
    BIOFIELD_CAPTURE_ARTIFACT_SOURCE_IMAGE,
    BIOFIELD_CAPTURE_ARTIFACT_SEGMENTATION_MASK,
    BIOFIELD_CAPTURE_ARTIFACT_ANALYSIS_OVERLAY,
    BIOFIELD_CAPTURE_ARTIFACT_THUMBNAIL,
];

pub fn is_valid_biofield_session_status(status: &str) -> bool {
    BIOFIELD_SESSION_STATUSES.contains(&status)
}

pub fn is_valid_biofield_capture_artifact_kind(kind: &str) -> bool {
    BIOFIELD_CAPTURE_ARTIFACT_KINDS.contains(&kind)
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BiofieldSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub client_device_id: Option<String>,
    pub viewer_version: Option<String>,
    pub notes: Option<String>,
    pub started_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewBiofieldSession {
    pub user_id: Uuid,
    pub client_device_id: Option<String>,
    pub viewer_version: Option<String>,
    pub notes: Option<String>,
}

impl NewBiofieldSession {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            client_device_id: None,
            viewer_version: None,
            notes: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BiofieldCaptureArtifact {
    pub id: Uuid,
    pub session_id: Uuid,
    pub reading_id: Option<Uuid>,
    pub artifact_kind: String,
    pub storage_path: String,
    pub mime_type: String,
    pub byte_size: i64,
    pub capture_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewBiofieldCaptureArtifact {
    pub session_id: Uuid,
    pub reading_id: Option<Uuid>,
    pub artifact_kind: String,
    pub storage_path: String,
    pub mime_type: String,
    pub byte_size: i64,
    pub capture_metadata: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biofield_models_session_status_values_are_stable() {
        assert_eq!(
            BIOFIELD_SESSION_STATUSES,
            ["active", "closed", "abandoned"]
        );
        assert!(is_valid_biofield_session_status("active"));
        assert!(is_valid_biofield_session_status("closed"));
        assert!(is_valid_biofield_session_status("abandoned"));
        assert!(!is_valid_biofield_session_status("requested"));
    }

    #[test]
    fn biofield_models_artifact_kind_values_are_stable() {
        assert_eq!(
            BIOFIELD_CAPTURE_ARTIFACT_KINDS,
            [
                "source-image",
                "segmentation-mask",
                "analysis-overlay",
                "thumbnail",
            ]
        );
        assert!(is_valid_biofield_capture_artifact_kind("source-image"));
        assert!(is_valid_biofield_capture_artifact_kind("segmentation-mask"));
        assert!(is_valid_biofield_capture_artifact_kind("analysis-overlay"));
        assert!(is_valid_biofield_capture_artifact_kind("thumbnail"));
        assert!(!is_valid_biofield_capture_artifact_kind("video"));
    }

    #[test]
    fn biofield_models_new_session_defaults_are_safe() {
        let session = NewBiofieldSession::new(Uuid::nil());

        assert_eq!(session.user_id, Uuid::nil());
        assert!(session.client_device_id.is_none());
        assert!(session.viewer_version.is_none());
        assert!(session.notes.is_none());
    }
}
