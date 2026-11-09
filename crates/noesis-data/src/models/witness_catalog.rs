//! Versioned immutable-artifact descriptors; never report body bytes.
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WitnessArtifact {
    pub profile: String,
    pub kind: String,
    pub date: String,
    pub sha256: String,
    pub object_key: String,
    pub report_schema: String,
    pub packet_sha256: String,
    pub contract_sha256: String,
    pub effective_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WitnessExpectedHead {
    pub sha256: String,
    pub revision: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WitnessPublish {
    // Required even for initial publication; explicit null means no accepted head.
    pub expected_head: serde_json::Value,
    pub artifact: WitnessArtifact,
    pub approval: String,
}
impl WitnessPublish {
    pub fn expected(&self) -> Result<Option<WitnessExpectedHead>, &'static str> {
        if self.expected_head.is_null() {
            return Ok(None);
        }
        let h: WitnessExpectedHead = serde_json::from_value(self.expected_head.clone())
            .map_err(|_| "INVALID_EXPECTED_HEAD")?;
        if !digest(&h.sha256) || h.revision < 1 {
            return Err("INVALID_EXPECTED_HEAD");
        }
        Ok(Some(h))
    }
    pub fn validate(&self, subject_id: Uuid, subject_key: &str) -> Result<NaiveDate, &'static str> {
        self.expected()?;
        let a = &self.artifact;
        if a.profile != subject_key
            || a.profile.is_empty()
            || a.profile.len() > 128
            || !a.profile.bytes().enumerate().all(|(i, b)| {
                b.is_ascii_lowercase() || b.is_ascii_digit() || (i > 0 && (b == b'_' || b == b'-'))
            })
        {
            return Err("WITNESS_SUBJECT_SCOPE_MISMATCH");
        }
        if !matches!(a.kind.as_str(), "daily" | "folio")
            || !matches!(self.approval.as_str(), "owner" | "automatic")
        {
            return Err("INVALID_WITNESS_KIND_OR_APPROVAL");
        }
        if !matches!(
            a.report_schema.as_str(),
            "noesis.witness-report.v1" | "noesis.witness-report.v2"
        ) {
            return Err("INVALID_WITNESS_SCHEMA");
        }
        if ![&a.sha256, &a.packet_sha256, &a.contract_sha256]
            .iter()
            .all(|h| digest(h))
        {
            return Err("INVALID_WITNESS_DIGEST");
        }
        let date =
            NaiveDate::parse_from_str(&a.date, "%Y-%m-%d").map_err(|_| "INVALID_WITNESS_DATE")?;
        if date.format("%Y-%m-%d").to_string() != a.date
            || a.date.len() != 10
            || a.date.as_str() < "1900-01-01"
        {
            return Err("INVALID_WITNESS_DATE");
        }
        if a.effective_at != format!("{}T00:30:00Z", a.date) {
            return Err("INVALID_WITNESS_ANCHOR");
        }
        if a.object_key
            != format!(
                "witness-v2/{}/{}/{}/{}/{}.json",
                subject_id, a.profile, a.kind, a.date, a.sha256
            )
        {
            return Err("INVALID_WITNESS_OBJECT_SCOPE");
        }
        Ok(date)
    }
}
pub fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WitnessEntry {
    pub date: String,
    pub kind: String,
    pub sha256: String,
    pub object_key: String,
    pub report_schema: String,
    pub profile_key: String,
    pub packet_sha256: String,
    pub contract_sha256: String,
    pub approval: String,
    pub published_at: DateTime<Utc>,
    pub head_revision: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct WitnessCatalog {
    pub schema: String,
    pub subject_id: Uuid,
    pub folio: Option<WitnessEntry>,
    pub daily: BTreeMap<String, WitnessEntry>,
}
#[derive(Debug, Serialize)]
pub struct WitnessPublication {
    pub schema: &'static str,
    pub subject_id: Uuid,
    pub entry: WitnessEntry,
    pub idempotent: bool,
    pub previous_head: Option<WitnessExpectedHead>,
}
