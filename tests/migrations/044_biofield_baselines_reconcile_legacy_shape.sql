-- Run with: psql -X -v ON_ERROR_STOP=1 -f tests/migrations/044_biofield_baselines_reconcile_legacy_shape.sql
-- Temporary tables shadow application tables; everything is rolled back.
BEGIN;
CREATE TEMP TABLE biofield_baselines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    name TEXT,
    is_active BOOLEAN NOT NULL DEFAULT false,
    source_session_id UUID,
    source_snapshot_id UUID,
    source_reading_id UUID,
    baseline_scores JSONB NOT NULL DEFAULT '{}',
    baseline_metrics JSONB NOT NULL DEFAULT '{}',
    provenance JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TEMP TABLE biofield_baseline_readings (
    baseline_id UUID NOT NULL,
    reading_id UUID NOT NULL,
    PRIMARY KEY (baseline_id, reading_id)
);
INSERT INTO biofield_baselines (user_id, name, baseline_scores, provenance)
VALUES ('00000000-0000-0000-0000-000000000001', 'Legacy fixture',
        '{"energy":0.5}', '{"synthetic":true}');
DO $$ BEGIN
    PERFORM notes FROM biofield_baselines;
    RAISE EXCEPTION 'Fixture must reproduce the missing notes column';
EXCEPTION WHEN undefined_column THEN NULL;
END $$;

\ir ../../migrations/044_biofield_baselines_reconcile_legacy_shape.sql
INSERT INTO biofield_baselines (user_id, name, notes)
VALUES ('00000000-0000-0000-0000-000000000001', 'API fixture', 'Synthetic notes');
INSERT INTO biofield_baseline_readings
SELECT id, '00000000-0000-0000-0000-000000000002' FROM biofield_baselines WHERE name = 'API fixture';
\ir ../../migrations/044_biofield_baselines_reconcile_legacy_shape.sql
DO $$ BEGIN
    ASSERT (SELECT baseline_scores = '{"energy":0.5}'::jsonb
            AND provenance = '{"synthetic":true}'::jsonb AND notes IS NULL
            FROM biofield_baselines WHERE name = 'Legacy fixture'), 'Legacy values changed';
    ASSERT (SELECT notes = 'Synthetic notes' AND baseline_scores = '{}'::jsonb
            AND NOT is_active FROM biofield_baselines WHERE name = 'API fixture'), 'API insert failed';
    ASSERT (SELECT count(*) = 2 FROM biofield_baselines), 'Rows changed during replay';
    ASSERT (SELECT count(*) = 2 FROM (
        SELECT baseline.id, baseline.user_id, baseline.name, baseline.notes,
               baseline.created_at, baseline.updated_at, COUNT(link.reading_id)::BIGINT AS reading_count
        FROM biofield_baselines baseline LEFT JOIN biofield_baseline_readings link ON link.baseline_id = baseline.id
        WHERE baseline.user_id = '00000000-0000-0000-0000-000000000001'
        GROUP BY baseline.id
    ) summary), 'Repository list query failed';
END $$;

DROP TABLE pg_temp.biofield_baseline_readings;
DROP TABLE pg_temp.biofield_baselines;
CREATE TEMP TABLE biofield_baselines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), user_id UUID NOT NULL,
    name VARCHAR(128) NOT NULL, notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
INSERT INTO biofield_baselines (user_id, name, notes)
VALUES ('00000000-0000-0000-0000-000000000001', 'Canonical fixture', 'Keep existing notes');
\ir ../../migrations/044_biofield_baselines_reconcile_legacy_shape.sql
DO $$ BEGIN
    ASSERT (SELECT notes = 'Keep existing notes' FROM biofield_baselines), 'Canonical notes changed';
END $$;
ROLLBACK;
