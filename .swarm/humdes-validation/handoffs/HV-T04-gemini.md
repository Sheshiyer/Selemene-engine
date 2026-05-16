# HV-T04 — Tier-2 storage: persist validation runs for engine-version drift tracking

> Self-contained handoff packet. Read this only, then begin work.

## Issue
[#856 — Tier-2 storage: persist humdes-validation runs to noesis-data](https://github.com/Sheshiyer/Selemene-engine/issues/856)

## Identity
- **Agent:** Gemini (validation/QA role; schema design + careful gating is the dominant skill)
- **Branch:** `swarm/humdes-validation/p1-w2/storage/856-gemini`
- **Worktree:** `.worktrees/856-gemini`
- **Wave:** Phase 1, Wave 2, Swarm B
- **Soft-blocked on:** HV-T02 (final field set affects which `field` strings the writer accepts)

## Goal
Make each `humdes_validation_tests` invocation **persistable** to Postgres so we can chart per-engine-version drift over time. Schema, migration, writer module, feature gate, and one trend-query snippet in the README.

## Allowed edit surface
| File | Reason |
|---|---|
| `crates/noesis-data/migrations/<timestamp>_humdes_validation.{up,down}.sql` | NEW migration (use existing migration convention discovered in step 1) |
| `crates/noesis-data/src/humdes_validation.rs` | NEW writer module |
| `crates/noesis-data/src/lib.rs` | Re-export the writer module |
| `crates/noesis-data/Cargo.toml` | Add `[features] record-validation = []` |
| `tests/fixtures/humdes/README.md` | Update Tier-2 section with the trend query + brief usage |
| (optional) `crates/noesis-data/examples/record_humdes_run.rs` | One-shot binary that invokes the writer once for proof |

## Forbidden surface
- Any engine source. Storage layer only.
- The HD validation test file (`humdes_validation_tests.rs`) — don't bolt the writer into the test directly; expose a callable function that a separate binary (or future hook) invokes.
- `tests/fixtures/humdes/_index.json` and fixtures (HV-T02's lock zone).

## Required reads (do these in order)
1. [`crates/noesis-data/`](../../../crates/noesis-data/) — survey directory layout to find the existing migration convention. Note whether `sqlx::migrate!`, `refinery`, `barrel`, raw SQL, or a custom scheme is in use.
2. [`crates/noesis-data/src/lib.rs`](../../../crates/noesis-data/src/lib.rs) — public module map
3. [`crates/noesis-data/Cargo.toml`](../../../crates/noesis-data/Cargo.toml) — existing features + DB driver
4. Issue [#856](https://github.com/Sheshiyer/Selemene-engine/issues/856) — has the schema you should adapt to whatever convention is in use

## Schema (from #856 — adapt to existing migration style)
```sql
CREATE TABLE humdes_validation_runs (
    id              UUID PRIMARY KEY,
    run_at          TIMESTAMPTZ NOT NULL,
    engine_version  TEXT NOT NULL,
    selemene_commit TEXT NOT NULL,
    fixtures_count  INT  NOT NULL,
    per_field_pct   JSONB NOT NULL,
    notes           TEXT
);

CREATE TABLE humdes_validation_records (
    id              UUID PRIMARY KEY,
    run_id          UUID NOT NULL REFERENCES humdes_validation_runs(id) ON DELETE CASCADE,
    person_id       TEXT NOT NULL,
    reading_hash    TEXT NOT NULL,
    reading_type    TEXT NOT NULL,
    field           TEXT NOT NULL,
    expected        JSONB,
    got             JSONB NOT NULL,
    matched         BOOL NOT NULL,
    notes           TEXT
);

CREATE INDEX idx_humdes_records_run_field ON humdes_validation_records(run_id, field);
CREATE INDEX idx_humdes_records_person_field ON humdes_validation_records(person_id, field);
```

## Writer API to expose
```rust
#[cfg(feature = "record-validation")]
pub mod humdes_validation {
    use serde_json::Value;
    use sqlx::PgPool;
    use uuid::Uuid;
    use chrono::{DateTime, Utc};

    #[derive(Debug, Clone)]
    pub struct ValidationRun {
        pub run_at: DateTime<Utc>,
        pub engine_version: String,
        pub selemene_commit: String,
        pub fixtures_count: i32,
        pub per_field_pct: Value,    // {"type": 92.1, ...}
        pub notes: Option<String>,
    }

    #[derive(Debug, Clone)]
    pub struct ValidationRecord {
        pub person_id: String,
        pub reading_hash: String,
        pub reading_type: String,
        pub field: String,
        pub expected: Option<Value>,
        pub got: Value,
        pub matched: bool,
        pub notes: Option<String>,
    }

    pub async fn record_validation_run(
        pool: &PgPool,
        run: ValidationRun,
        records: Vec<ValidationRecord>,
    ) -> Result<Uuid, sqlx::Error> {
        // Transaction: insert run, insert all records, return run_id
        unimplemented!()
    }
}
```

## Suggested first commands
```bash
cd /Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/Selemene-engine
git worktree add .worktrees/856-gemini swarm/humdes-validation/p1-w2/storage/856-gemini
cd .worktrees/856-gemini

# Step 1: discover migration convention
ls -la crates/noesis-data/
find crates/noesis-data -name "*.sql" | head
find crates/noesis-data -name "migrations" -type d
grep -r "sqlx::migrate\|refinery\|barrel" crates/noesis-data/ | head

# Step 2: discover Postgres driver / pool style
grep -E "PgPool|sqlx::Pool|Postgres" crates/noesis-data/src/ -r | head

# Step 3: confirm UUID + chrono availability
grep -E "^(uuid|chrono|sqlx) =" crates/noesis-data/Cargo.toml
```

## Verification before opening PR
```bash
# Without the feature (default test): nothing in noesis-data must reference record_validation_run
cargo build --package noesis-data
cargo test --package noesis-data

# With the feature: writer compiles + types resolve
cargo build --package noesis-data --features record-validation

# If you can spin up a scratch postgres, prove the migration applies
# (optional but recommended):
# psql -c 'CREATE DATABASE scratch_humdes;'
# DATABASE_URL=postgres://...scratch_humdes <run migration tool>

# Run example writer (only if you added the binary)
cargo run --package noesis-data --features record-validation \
    --example record_humdes_run
```

## PR template
```
Title: [HV-T04 #856] Tier-2 storage: humdes validation persistence

Closes #856

## Summary
- New tables: `humdes_validation_runs`, `humdes_validation_records`
- New module `noesis_data::humdes_validation` behind `record-validation` feature
- Single `record_validation_run(pool, run, records) -> Uuid` entry point
- README updated with the recommended trend query

## Schema
[paste final SQL]

## Trend query (added to README)
```sql
SELECT
    engine_version,
    (per_field_pct->>'type')::float  AS type_pct,
    (per_field_pct->>'profile')::float AS profile_pct,
    fixtures_count,
    run_at
FROM humdes_validation_runs
ORDER BY run_at DESC LIMIT 20;
```

## Verification
- `cargo build --package noesis-data` (default) — passes
- `cargo build --package noesis-data --features record-validation` — passes
- Migration applies cleanly on scratch DB
```

## Escalation triggers
- If existing migration convention is bespoke (not sqlx-migrate / refinery / barrel): match it exactly; if unclear, ask human lead.
- If the noesis-data crate has multiple Postgres pool types or feature-gated drivers, pick the one that maps to the existing convention rather than introducing a new one.
- If the writer pattern would force changes to a public type used by API services: STOP, file a contract issue, scope down.
