-- Migration: 005_readings
-- Description: Create readings table for storing engine/workflow calculation results

CREATE TABLE IF NOT EXISTS readings (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    engine_id           VARCHAR(64) NOT NULL,
    workflow_id         VARCHAR(64),
    input_hash          VARCHAR(64) NOT NULL,
    input_data          JSONB NOT NULL,
    result_data         JSONB NOT NULL,
    witness_prompt      TEXT,
    consciousness_level SMALLINT NOT NULL DEFAULT 0,
    calculation_time_ms DOUBLE PRECISION,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_readings_user_id ON readings(user_id);
CREATE INDEX idx_readings_user_engine ON readings(user_id, engine_id);
CREATE INDEX idx_readings_created_at ON readings(created_at DESC);
CREATE INDEX idx_readings_input_hash ON readings(input_hash);
