//! Stable shared account and subject IDs; revision-qualified server truth.
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub preferred_email: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectAccess {
    pub id: Uuid,
    pub key: String,
    pub display_name: String,
    pub can_read: bool,
    pub can_edit: bool,
    pub can_calculate: bool,
    pub profile_revision: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipAccess {
    pub id: Uuid,
    pub key: String,
    pub label: Option<String>,
    pub member_subject_ids: Vec<Uuid>,
    pub can_read: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountContext {
    pub schema: String,
    pub account: Account,
    pub default_subject_id: Option<Uuid>,
    pub subjects: Vec<SubjectAccess>,
    pub relationships: Vec<RelationshipAccess>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectProfile {
    pub schema: String,
    pub subject_id: Uuid,
    pub revision: i64,
    pub profile: serde_json::Value,
}
