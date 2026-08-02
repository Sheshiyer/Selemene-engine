use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct LivingReadingListFilters {
    pub query: Option<String>,
    pub reading_type: Option<LivingReadingTypeFilter>,
    pub owner_user_id: Option<Uuid>,
    pub subject_id: Option<Uuid>,
    pub relationship_id: Option<Uuid>,
    pub source_id: Option<Uuid>,
    pub import_run_id: Option<Uuid>,
    pub editorial_state: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LivingReadingTypeFilter {
    Solo,
    Synastry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivingReadingListPage {
    pub items: Vec<LivingReadingListItem>,
    pub total: i64,
    pub completed_total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivingReadingListItem {
    pub id: Uuid,
    pub owner_user_id: Uuid,
    pub owner_email: String,
    pub stable_reading_id: String,
    pub title: String,
    pub summary: Option<String>,
    pub reading_type: String,
    pub language_tag: String,
    pub producer_kind: String,
    pub producer_ref: Option<String>,
    pub source_id: Uuid,
    pub source_locator: String,
    pub source_sha256: Option<String>,
    pub import_run_id: Uuid,
    pub import_manifest_id: String,
    pub relationship_id: Option<Uuid>,
    pub relationship_label: Option<String>,
    pub relationship_kind: Option<String>,
    pub editorial_state: Option<String>,
    pub editorial_visibility: Option<String>,
    pub publication_availability: String,
    pub subjects: Vec<LivingReadingSubject>,
    pub captured_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivingReadingSubject {
    pub id: Uuid,
    pub subject_key: String,
    pub canonical_name: String,
    pub subject_type: String,
    pub role: String,
    pub confidence: Option<f64>,
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LivingReadingSource {
    pub id: Uuid,
    pub stable_source_id: String,
    pub source_kind: String,
    pub locator: String,
    pub content_sha256: Option<String>,
    pub byte_size: Option<i64>,
    pub media_type: Option<String>,
    pub observed_at: Option<DateTime<Utc>>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LivingReadingImportRun {
    pub id: Uuid,
    pub manifest_id: String,
    pub manifest_schema_version: String,
    pub manifest_sha256: String,
    pub source_root_locator: String,
    pub state: String,
    pub stats: Value,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivingReadingRelationship {
    pub id: Uuid,
    pub relationship_key: String,
    pub relationship_kind: String,
    pub label: Option<String>,
    pub reconciliation_state: String,
    pub members: Vec<LivingReadingRelationshipMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivingReadingRelationshipMember {
    pub subject_id: Uuid,
    pub canonical_name: String,
    pub role: String,
    pub position: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LivingReadingArtifact {
    pub id: Uuid,
    pub artifact_key: String,
    pub artifact_role: String,
    pub storage_provider: String,
    pub object_locator: String,
    pub content_sha256: String,
    pub byte_size: i64,
    pub media_type: Option<String>,
    pub availability_state: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LivingReadingEvidence {
    pub id: Uuid,
    pub source_id: Option<Uuid>,
    pub artifact_id: Option<Uuid>,
    pub evidence_key: String,
    pub evidence_type: String,
    pub claim: String,
    pub excerpt: Option<String>,
    pub review_state: String,
    pub confidence: Option<f64>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LivingReadingEditorialState {
    pub id: Uuid,
    pub state: String,
    pub visibility: String,
    pub changed_by_user_id: Option<Uuid>,
    pub changed_by_email: Option<String>,
    pub change_role: String,
    pub revision: i32,
    pub is_current: bool,
    pub rationale: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivingReadingDetail {
    pub access_reason: String,
    pub reading: LivingReadingListItem,
    pub source: LivingReadingSource,
    pub import_run: LivingReadingImportRun,
    pub relationships: Vec<LivingReadingRelationship>,
    pub artifacts: Vec<LivingReadingArtifact>,
    pub evidence: Vec<LivingReadingEvidence>,
    pub editorial_history: Vec<LivingReadingEditorialState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LivingReadingInvitation {
    pub id: Uuid,
    pub reading_id: Uuid,
    pub created_by_user_id: Option<Uuid>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_by_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewLivingReadingInvitation {
    pub reading_id: Uuid,
    pub token_digest: String,
    pub created_by_user_id: Option<Uuid>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct LivingReadingPublicationArtifactCandidate {
    pub storage_provider: String,
    pub object_locator: String,
    pub content_sha256: String,
    pub byte_size: i64,
    pub media_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivingReadingPublicSubject {
    pub canonical_name: String,
    pub role: String,
}

#[derive(Debug, Clone)]
pub struct LivingReadingInvitationResolutionRecord {
    pub reading_id: Uuid,
    pub stable_reading_id: String,
    pub title: String,
    pub summary: Option<String>,
    pub reading_type: String,
    pub language_tag: String,
    pub editorial_state: Option<String>,
    pub subjects: Vec<LivingReadingPublicSubject>,
    pub relationship_label: Option<String>,
    pub artifact: Option<LivingReadingPublicationArtifactCandidate>,
}

/// Deliberately small recipient contract. Administrative IDs, ownership,
/// provenance, locators, checksums, import metadata, and evidence never enter it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivingReadingInvitationResolution {
    pub reading_id: Uuid,
    pub stable_reading_id: String,
    pub title: String,
    pub summary: Option<String>,
    pub reading_type: String,
    pub language_tag: String,
    pub editorial_state: Option<String>,
    pub subjects: Vec<LivingReadingPublicSubject>,
    pub relationship_label: Option<String>,
    pub artifact: LivingReadingPublishedArtifact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivingReadingPublishedArtifact {
    pub availability: String,
    pub media_type: Option<String>,
    pub content: Option<String>,
}

pub fn invitation_is_usable(
    invitation_reading_id: Uuid,
    requested_reading_id: Uuid,
    expires_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> bool {
    invitation_reading_id == requested_reading_id && revoked_at.is_none() && expires_at > now
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn invitation_rejects_expired_revoked_and_wrong_reading() {
        let now = Utc::now();
        let reading_id = Uuid::new_v4();
        assert!(!invitation_is_usable(
            reading_id,
            reading_id,
            now - Duration::seconds(1),
            None,
            now
        ));
        assert!(!invitation_is_usable(
            reading_id,
            reading_id,
            now + Duration::hours(1),
            Some(now),
            now
        ));
        assert!(!invitation_is_usable(
            reading_id,
            Uuid::new_v4(),
            now + Duration::hours(1),
            None,
            now
        ));
        assert!(invitation_is_usable(
            reading_id,
            reading_id,
            now + Duration::hours(1),
            None,
            now
        ));
    }

    #[test]
    fn recipient_serialization_omits_sensitive_archive_fields() {
        let payload = LivingReadingInvitationResolution {
            reading_id: Uuid::new_v4(),
            stable_reading_id: "reading-1".into(),
            title: "A reading".into(),
            summary: Some("A safe summary".into()),
            reading_type: "natal".into(),
            language_tag: "en".into(),
            editorial_state: Some("published".into()),
            subjects: vec![LivingReadingPublicSubject {
                canonical_name: "Recipient".into(),
                role: "primary".into(),
            }],
            relationship_label: None,
            artifact: LivingReadingPublishedArtifact {
                availability: "publication_unavailable".into(),
                media_type: None,
                content: None,
            },
        };
        let value = serde_json::to_value(payload).expect("serialize public payload");
        let serialized = serde_json::to_string(&value).expect("serialize public value");
        for forbidden in [
            "owner_email",
            "owner_user_id",
            "source_locator",
            "object_locator",
            "content_sha256",
            "import_run_id",
            "metadata",
            "artifact_key",
        ] {
            assert!(value.get(forbidden).is_none(), "{forbidden} leaked");
            assert!(!serialized.contains(forbidden), "{forbidden} leaked");
        }
    }
}
