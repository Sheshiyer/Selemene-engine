-- Migration 031: humdes validation persistence (Tier-2 storage).
--
-- Stores per-invocation results of the humdes ground-truth validation
-- harness (crates/engine-human-design/tests/humdes_validation_tests.rs)
-- so we can chart per-engine-version drift over time across the 89-person
-- humdes fixture corpus.
--
-- Writer: crates/noesis-data/src/humdes_validation.rs, gated behind the
-- `record-validation` Cargo feature. Tests and production paths do NOT
-- reference this writer unless the feature is enabled by an explicit
-- one-shot binary (see crates/noesis-data/examples/record_humdes_run.rs).
--
-- See: tests/fixtures/humdes/README.md (Tier-2 section) and issue #856.
-- Reverse: see DOWN block at bottom.

-- ── UP ───────────────────────────────────────────────────────────────────

CREATE TABLE humdes_validation_runs (
    id              UUID            PRIMARY KEY,
    run_at          TIMESTAMPTZ     NOT NULL,
    engine_version  TEXT            NOT NULL,       -- e.g. env!("CARGO_PKG_VERSION")
    selemene_commit TEXT            NOT NULL,       -- git sha at run time
    fixtures_count  INT             NOT NULL CHECK (fixtures_count >= 0),
    per_field_pct   JSONB           NOT NULL,       -- {"type": 92.1, "profile": 100.0, ...}
    notes           TEXT
);

CREATE TABLE humdes_validation_records (
    id              UUID            PRIMARY KEY,
    run_id          UUID            NOT NULL
                    REFERENCES humdes_validation_runs(id) ON DELETE CASCADE,
    person_id       TEXT            NOT NULL,       -- humdes person_id (stable hash)
    reading_hash    TEXT            NOT NULL,       -- humdes reading hash
    reading_type    TEXT            NOT NULL,       -- personal|hologenetic|compatibility|business|family
    field           TEXT            NOT NULL,       -- canonical field name (see writer's KNOWN_FIELDS)
    expected        JSONB,                          -- humdes ground-truth value; NULL = no ground truth
    got             JSONB           NOT NULL,       -- engine output value
    matched         BOOL            NOT NULL,
    notes           TEXT
);

-- Hot path: per-run drill-down by field ("which fixtures failed `type` in run X?").
CREATE INDEX idx_humdes_records_run_field
    ON humdes_validation_records (run_id, field);

-- Hot path: per-person trajectory across runs ("when did fixture Y start failing?").
CREATE INDEX idx_humdes_records_person_field
    ON humdes_validation_records (person_id, field);

COMMENT ON TABLE humdes_validation_runs IS
  'One row per humdes_validation_tests invocation. Persisted only when the noesis-data record-validation feature is enabled. See issue #856.';
COMMENT ON TABLE humdes_validation_records IS
  'One row per (run, person, field) — the fine-grained diff between engine output and humdes ground truth. Cascade-deletes with the parent run.';
COMMENT ON COLUMN humdes_validation_runs.per_field_pct IS
  'Top-line per-field match percentage as JSON, e.g. {"type": 92.1, "profile": 100.0}. Convenient for trend dashboards without joining records.';
COMMENT ON COLUMN humdes_validation_records.field IS
  'Canonical field name. Closed set, mirrored by noesis_data::humdes_validation::KNOWN_FIELDS: type, profile, authority, personality_sun, personality_earth, design_sun, design_earth, incarnation_cross, definition, strategy, not_self_theme, defined_centers, active_channels.';
COMMENT ON COLUMN humdes_validation_records.expected IS
  'humdes ground-truth value. NULL is meaningful — it means humdes had no value for this (person, field), distinct from a JSON null.';

-- ── DOWN ─────────────────────────────────────────────────────────────────
-- DROP INDEX IF EXISTS idx_humdes_records_person_field;
-- DROP INDEX IF EXISTS idx_humdes_records_run_field;
-- DROP TABLE IF EXISTS humdes_validation_records;
-- DROP TABLE IF EXISTS humdes_validation_runs;
