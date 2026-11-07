//! Short metadata transactions. Upload/hash verification occurs before calling
//! this repository; it never fetches objects or changes a second writable index.
use crate::models::witness_catalog::*;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum WitnessCatalogError {
    #[error("{0}")]
    Invalid(&'static str),
    #[error("WITNESS_ACCESS_DENIED")]
    Denied,
    #[error("{0}")]
    Conflict(&'static str),
    #[error("Witness storage unavailable")]
    Storage(#[from] sqlx::Error),
}
const ENTRY: &str = "SELECT to_char(v.report_date,'YYYY-MM-DD') AS date,v.kind,v.sha256,v.object_key,v.report_schema,v.profile_key,v.packet_sha256,v.contract_sha256,v.approval,v.published_at,h.revision AS head_revision FROM witness_heads h JOIN witness_versions v ON v.id=h.version_id AND v.subject_id=h.subject_id AND v.owner_user_id=h.owner_user_id AND v.kind=h.kind AND v.report_date=h.report_date";

async fn readable(
    tx: &mut Transaction<'_, Postgres>,
    actor: Uuid,
    subject: Uuid,
) -> Result<(Uuid, String), WitnessCatalogError> {
    // Explicit predicate is required even when the connection role bypasses RLS.
    sqlx::query_as::<_,(Uuid,String)>("SELECT s.owner_user_id,s.subject_key FROM corpus_subjects s JOIN subject_grants g ON g.subject_id=s.id AND g.owner_user_id=s.owner_user_id JOIN user_account_state a ON a.user_id=g.grantee_user_id WHERE s.id=$1 AND g.grantee_user_id=$2 AND identity_account()=$2 AND g.revoked_at IS NULL AND a.state='active' AND s.archived_at IS NULL FOR SHARE OF s,g,a")
        .bind(subject).bind(actor).fetch_optional(&mut **tx).await?.ok_or(WitnessCatalogError::Denied)
}
pub async fn catalog(
    tx: &mut Transaction<'_, Postgres>,
    actor: Uuid,
    subject: Uuid,
) -> Result<WitnessCatalog, WitnessCatalogError> {
    readable(tx, actor, subject).await?;
    let entries = sqlx::query_as::<_,WitnessEntry>(&format!("{ENTRY} WHERE h.subject_id=$1 AND witness_subject_read(h.subject_id) ORDER BY h.report_date DESC"))
        .bind(subject).fetch_all(&mut **tx).await?;
    let mut result = WitnessCatalog {
        schema: "noesis.witness-catalog.v1".into(),
        subject_id: subject,
        folio: None,
        daily: Default::default(),
    };
    for entry in entries {
        if entry.kind == "folio" {
            if result.folio.is_none() {
                result.folio = Some(entry);
            }
        } else {
            result.daily.insert(entry.date.clone(), entry);
        }
    }
    Ok(result)
}
#[derive(sqlx::FromRow)]
struct Head {
    version_id: Uuid,
    sha256: String,
    revision: i64,
    previous_sha: Option<String>,
}
#[derive(sqlx::FromRow)]
struct Version {
    id: Uuid,
    profile_key: String,
    kind: String,
    report_date: NaiveDate,
    sha256: String,
    object_key: String,
    report_schema: String,
    packet_sha256: String,
    contract_sha256: String,
    effective_at: DateTime<Utc>,
    approval: String,
}
impl Version {
    fn matches(&self, p: &WitnessPublish) -> bool {
        let a = &p.artifact;
        self.profile_key == a.profile
            && self.kind == a.kind
            && self.report_date.to_string() == a.date
            && self.sha256 == a.sha256
            && self.object_key == a.object_key
            && self.report_schema == a.report_schema
            && self.packet_sha256 == a.packet_sha256
            && self.contract_sha256 == a.contract_sha256
            && self.effective_at.format("%Y-%m-%dT%H:%M:%SZ").to_string() == a.effective_at
            && self.approval == p.approval
    }
}
fn same_head(expected: &Option<WitnessExpectedHead>, head: Option<&Head>) -> bool {
    match (expected, head) {
        (None, None) => true,
        (Some(e), Some(h)) => e.sha256 == h.sha256 && e.revision == h.revision,
        _ => false,
    }
}
pub async fn publish(
    tx: &mut Transaction<'_, Postgres>,
    actor: Uuid,
    subject: Uuid,
    p: &WitnessPublish,
) -> Result<WitnessPublication, WitnessCatalogError> {
    // Validate before taking a lock, then validate the profile key against DB truth.
    let date = p
        .validate(&p.artifact.profile)
        .map_err(WitnessCatalogError::Invalid)?;
    let expected = p.expected().map_err(WitnessCatalogError::Invalid)?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(format!(
            "witness:{subject}:{}:{}",
            p.artifact.kind, p.artifact.date
        ))
        .execute(&mut **tx)
        .await?;
    let (owner, profile_key) = readable(tx, actor, subject).await?;
    p.validate(&profile_key)
        .map_err(WitnessCatalogError::Invalid)?;
    if actor != owner {
        let granted:Option<Uuid>=sqlx::query_scalar("SELECT grantee_user_id FROM witness_publish_grants WHERE subject_id=$1 AND owner_user_id=$2 AND grantee_user_id=$3 AND revoked_at IS NULL FOR SHARE")
            .bind(subject).bind(owner).bind(actor).fetch_optional(&mut **tx).await?;
        if granted.is_none() {
            return Err(WitnessCatalogError::Denied);
        }
    }
    let head=sqlx::query_as::<_,Head>("SELECT h.version_id,v.sha256,h.revision,prior.sha256 AS previous_sha FROM witness_heads h JOIN witness_versions v ON v.id=h.version_id LEFT JOIN witness_versions prior ON prior.id=h.previous_version_id WHERE h.subject_id=$1 AND h.kind=$2 AND h.report_date=$3 FOR UPDATE OF h")
        .bind(subject).bind(&p.artifact.kind).bind(date).fetch_optional(&mut **tx).await?;
    let version=sqlx::query_as::<_,Version>("SELECT id,profile_key,kind,report_date,sha256,object_key,report_schema,packet_sha256,contract_sha256,effective_at,approval FROM witness_versions WHERE subject_id=$1 AND kind=$2 AND report_date=$3 AND sha256=$4")
        .bind(subject).bind(&p.artifact.kind).bind(date).bind(&p.artifact.sha256).fetch_optional(&mut **tx).await?;
    if version.as_ref().is_some_and(|v| !v.matches(p)) {
        return Err(WitnessCatalogError::Conflict(
            "IMMUTABLE_DESCRIPTOR_CONFLICT",
        ));
    }
    let idempotent = head.as_ref().is_some_and(|h| h.sha256 == p.artifact.sha256);
    if idempotent {
        let h = head.as_ref().unwrap();
        // Retries may name this head or its exact predecessor. Older ABA heads
        // cannot silently succeed after a rollback because revisions increase.
        let retry = match (&expected, &h.previous_sha) {
            (None, None) => h.revision == 1,
            (Some(e), Some(previous)) => e.sha256 == *previous && e.revision == h.revision - 1,
            _ => false,
        };
        if !same_head(&expected, head.as_ref()) && !retry {
            return Err(WitnessCatalogError::Conflict("WITNESS_HEAD_CONFLICT"));
        }
    } else if !same_head(&expected, head.as_ref()) {
        return Err(WitnessCatalogError::Conflict("WITNESS_HEAD_CONFLICT"));
    }
    let previous_head = head.as_ref().map(|h| WitnessExpectedHead {
        sha256: h.sha256.clone(),
        revision: h.revision,
    });
    if !idempotent {
        let version_id = if let Some(v) = version {
            v.id
        } else {
            sqlx::query_scalar::<_,Uuid>("INSERT INTO witness_versions(subject_id,owner_user_id,profile_key,kind,report_date,sha256,object_key,report_schema,packet_sha256,contract_sha256,effective_at,approval,published_by) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) RETURNING id")
                .bind(subject).bind(owner).bind(&p.artifact.profile).bind(&p.artifact.kind).bind(date).bind(&p.artifact.sha256).bind(&p.artifact.object_key)
                .bind(&p.artifact.report_schema).bind(&p.artifact.packet_sha256).bind(&p.artifact.contract_sha256)
                .bind(date.and_hms_opt(0,30,0).unwrap().and_utc()).bind(&p.approval).bind(actor).fetch_one(&mut **tx).await.map_err(|error| {
                    if error.as_database_error().and_then(|e| e.constraint()) == Some("witness_versions_object_key_key") {
                        WitnessCatalogError::Conflict("WITNESS_OBJECT_ALREADY_SCOPED")
                    } else { WitnessCatalogError::Storage(error) }
                })?
        };
        let revision = head.as_ref().map_or(1, |h| h.revision + 1);
        let previous_id = head.as_ref().map(|h| h.version_id);
        sqlx::query("INSERT INTO witness_heads(subject_id,owner_user_id,kind,report_date,version_id,previous_version_id,revision,updated_by) VALUES($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT(subject_id,kind,report_date) DO UPDATE SET version_id=excluded.version_id,previous_version_id=excluded.previous_version_id,revision=excluded.revision,updated_by=excluded.updated_by,updated_at=now()")
            .bind(subject).bind(owner).bind(&p.artifact.kind).bind(date).bind(version_id).bind(previous_id).bind(revision).bind(actor).execute(&mut **tx).await?;
        sqlx::query("INSERT INTO witness_head_events(subject_id,owner_user_id,kind,report_date,version_id,previous_version_id,revision,actor_user_id) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
            .bind(subject).bind(owner).bind(&p.artifact.kind).bind(date).bind(version_id).bind(previous_id).bind(revision).bind(actor).execute(&mut **tx).await?;
    }
    let entry = sqlx::query_as::<_, WitnessEntry>(&format!(
        "{ENTRY} WHERE h.subject_id=$1 AND h.kind=$2 AND h.report_date=$3"
    ))
    .bind(subject)
    .bind(&p.artifact.kind)
    .bind(date)
    .fetch_one(&mut **tx)
    .await?;
    Ok(WitnessPublication {
        schema: "noesis.witness-publication.v1",
        subject_id: subject,
        entry,
        idempotent,
        previous_head,
    })
}
