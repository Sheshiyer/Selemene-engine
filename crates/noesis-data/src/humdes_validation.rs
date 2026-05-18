//! humdes validation persistence (Tier-2 storage).
//!
//! Writer module for the `humdes_validation_runs` / `humdes_validation_records`
//! tables introduced by migration `031_humdes_validation.sql`.
//!
//! Gated entirely behind the `record-validation` Cargo feature so the default
//! build of `noesis-data` (and any crate that depends on it) carries zero
//! reference to this code. The intended caller is a one-shot binary or a
//! future post-test hook that takes a `PgPool`, never the test harness itself.
//!
//! See `tests/fixtures/humdes/README.md` (Tier-2 section) and issue #856.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

/// Canonical field names that the writer will accept in
/// [`ValidationRecord::field`]. This is a closed set deliberately mirrored
/// in the migration's column comment so the database documents its own
/// contract. New fields should be appended here AND in the comment.
///
/// Order: Wave-0 cardinal fields, then Wave-1 enrichment fields (HV-T02),
/// then forward-declared placeholders that the engine doesn't yet emit.
pub const KNOWN_FIELDS: &[&str] = &[
    // Wave-0 cardinal fields (engine baseline).
    "type",
    "profile",
    "authority",
    "personality_sun",
    "personality_earth",
    "design_sun",
    "design_earth",
    "incarnation_cross",
    // Wave-1 enrichment (HV-T02 #854).
    "definition",
    "strategy",
    "not_self_theme",
    // Forward-declared — not yet populated by the engine but reserved so
    // future runs don't need a schema change.
    "defined_centers",
    "active_channels",
];

/// Returns `true` if `field` is in [`KNOWN_FIELDS`]. Callers are free to
/// persist unknown fields (the DB doesn't enforce the set), but the writer
/// uses this to attach a non-fatal note to the row.
pub fn is_known_field(field: &str) -> bool {
    KNOWN_FIELDS.contains(&field)
}

/// One humdes-validation invocation. Maps 1:1 to a row in
/// `humdes_validation_runs`.
#[derive(Debug, Clone)]
pub struct ValidationRun {
    pub run_at: DateTime<Utc>,
    pub engine_version: String,
    pub selemene_commit: String,
    pub fixtures_count: i32,
    /// Top-line per-field match percentage, e.g. `{"type": 92.1, "profile": 100.0}`.
    /// Stored as JSONB so the schema doesn't need updating when fields are added.
    pub per_field_pct: Value,
    pub notes: Option<String>,
}

/// One (person, field) diff inside a run. Maps 1:1 to a row in
/// `humdes_validation_records`.
#[derive(Debug, Clone)]
pub struct ValidationRecord {
    pub person_id: String,
    pub reading_hash: String,
    pub reading_type: String,
    pub field: String,
    /// humdes ground-truth value. `None` means humdes had no value for this
    /// (person, field) pair — written as SQL `NULL`, distinct from `Some(Value::Null)`.
    pub expected: Option<Value>,
    pub got: Value,
    pub matched: bool,
    pub notes: Option<String>,
}

/// Persist a validation run with its per-record diffs atomically.
///
/// Inserts one row into `humdes_validation_runs` followed by N rows into
/// `humdes_validation_records`, all inside a single transaction so a partial
/// failure leaves no orphan run row. Returns the freshly minted `run_id`.
///
/// The writer attaches an automatic `[unknown-field]` note tag to any record
/// whose `field` is not in [`KNOWN_FIELDS`], appended to the caller-supplied
/// `notes` if any. This makes drift in the field-set visible without
/// rejecting writes.
pub async fn record_validation_run(
    pool: &PgPool,
    run: ValidationRun,
    records: Vec<ValidationRecord>,
) -> Result<Uuid, sqlx::Error> {
    let run_id = Uuid::new_v4();

    let mut tx = pool.begin().await?;

    sqlx::query(
        r#"
        INSERT INTO humdes_validation_runs (
            id, run_at, engine_version, selemene_commit,
            fixtures_count, per_field_pct, notes
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(run_id)
    .bind(run.run_at)
    .bind(&run.engine_version)
    .bind(&run.selemene_commit)
    .bind(run.fixtures_count)
    .bind(&run.per_field_pct)
    .bind(run.notes.as_deref())
    .execute(&mut *tx)
    .await?;

    for record in records {
        let augmented_notes = match (is_known_field(&record.field), record.notes.as_deref()) {
            (true, n) => n.map(|s| s.to_string()),
            (false, Some(existing)) => Some(format!("{existing} [unknown-field]")),
            (false, None) => Some("[unknown-field]".to_string()),
        };

        sqlx::query(
            r#"
            INSERT INTO humdes_validation_records (
                id, run_id, person_id, reading_hash, reading_type,
                field, expected, got, matched, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(run_id)
        .bind(&record.person_id)
        .bind(&record.reading_hash)
        .bind(&record.reading_type)
        .bind(&record.field)
        .bind(record.expected.as_ref())
        .bind(&record.got)
        .bind(record.matched)
        .bind(augmented_notes.as_deref())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(run_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_fields_includes_wave_1_additions() {
        // Cardinal
        assert!(is_known_field("type"));
        assert!(is_known_field("profile"));
        assert!(is_known_field("incarnation_cross"));
        // Wave-1 enrichment (HV-T02)
        assert!(is_known_field("definition"));
        assert!(is_known_field("strategy"));
        assert!(is_known_field("not_self_theme"));
        // Forward-declared
        assert!(is_known_field("defined_centers"));
        assert!(is_known_field("active_channels"));
        // Not in set
        assert!(!is_known_field("totally_made_up"));
        assert!(!is_known_field(""));
    }

    #[test]
    fn known_fields_has_no_duplicates() {
        let mut seen: Vec<&&str> = KNOWN_FIELDS.iter().collect();
        seen.sort();
        let len_before = seen.len();
        seen.dedup();
        assert_eq!(len_before, seen.len(), "KNOWN_FIELDS contains duplicates");
    }
}
