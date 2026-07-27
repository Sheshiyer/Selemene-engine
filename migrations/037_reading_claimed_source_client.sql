-- First-party client attribution for engine and workflow readings.
--
-- This value is intentionally named "claimed": public API callers assert it,
-- so it is useful for operations and rollout visibility but never authority.
-- NULL means the reading predates this migration or omitted client context.
ALTER TABLE readings
    ADD COLUMN IF NOT EXISTS claimed_source_client VARCHAR(64);

CREATE INDEX IF NOT EXISTS idx_readings_claimed_source_created_at
    ON readings (claimed_source_client, created_at DESC)
    WHERE claimed_source_client IS NOT NULL;
