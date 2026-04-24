-- Migration: 018_biofield_baselines
-- Description: Add biofield baseline persistence surfaces.

CREATE TABLE IF NOT EXISTS biofield_baselines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(128) NOT NULL,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT biofield_baselines_name_nonempty CHECK (
        btrim(name) <> ''
    )
);

CREATE INDEX IF NOT EXISTS idx_biofield_baselines_user_id
    ON biofield_baselines(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_biofield_baselines_user_updated_at
    ON biofield_baselines(user_id, updated_at DESC);

DROP TRIGGER IF EXISTS update_biofield_baselines_updated_at ON biofield_baselines;
CREATE TRIGGER update_biofield_baselines_updated_at
    BEFORE UPDATE ON biofield_baselines
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE IF NOT EXISTS biofield_baseline_readings (
    baseline_id UUID NOT NULL REFERENCES biofield_baselines(id) ON DELETE CASCADE,
    reading_id UUID NOT NULL REFERENCES readings(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (baseline_id, reading_id),
    CONSTRAINT biofield_baseline_readings_sort_order_nonnegative CHECK (
        sort_order >= 0
    )
);

CREATE INDEX IF NOT EXISTS idx_biofield_baseline_readings_baseline_id
    ON biofield_baseline_readings(baseline_id, sort_order ASC, created_at ASC);
CREATE INDEX IF NOT EXISTS idx_biofield_baseline_readings_reading_id
    ON biofield_baseline_readings(reading_id, created_at DESC);
