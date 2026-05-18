//! One-shot binary that invokes the humdes-validation writer with a tiny
//! synthetic payload. Useful for:
//!   1. Smoke-testing migration 031 against a scratch Postgres
//!   2. Documenting the call shape future test harnesses should adopt
//!
//! Run with:
//!   DATABASE_URL=postgres://... \
//!     cargo run --package noesis-data --features record-validation \
//!         --example record_humdes_run
//!
//! Idempotency: each invocation creates a fresh run row with a new UUID, so
//! repeated runs are safe (each just appends another row).

use std::env;

use chrono::Utc;
use noesis_data::{create_pool, humdes_validation};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL").map_err(|_| {
        "DATABASE_URL is required — point at a scratch Postgres with migration 031 applied"
    })?;

    let pool = create_pool(&database_url).await?;

    let run = humdes_validation::ValidationRun {
        run_at: Utc::now(),
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        selemene_commit: env::var("SELEMENE_COMMIT").unwrap_or_else(|_| "unknown".to_string()),
        fixtures_count: 2,
        per_field_pct: json!({
            "type": 100.0,
            "profile": 100.0,
        }),
        notes: Some("record_humdes_run example invocation".to_string()),
    };

    let records = vec![
        humdes_validation::ValidationRecord {
            person_id: "example-person-a".to_string(),
            reading_hash: "deadbeef".to_string(),
            reading_type: "personal".to_string(),
            field: "type".to_string(),
            expected: Some(json!("Generator")),
            got: json!("Generator"),
            matched: true,
            notes: None,
        },
        humdes_validation::ValidationRecord {
            person_id: "example-person-b".to_string(),
            reading_hash: "cafebabe".to_string(),
            reading_type: "personal".to_string(),
            field: "profile".to_string(),
            expected: Some(json!("1/3")),
            got: json!("1/3"),
            matched: true,
            notes: None,
        },
    ];

    let run_id = humdes_validation::record_validation_run(&pool, run, records).await?;
    println!("inserted humdes_validation_runs row id={run_id}");

    Ok(())
}
