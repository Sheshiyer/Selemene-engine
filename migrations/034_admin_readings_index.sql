CREATE INDEX IF NOT EXISTS idx_readings_user_engine_created
    ON readings(user_id, engine_id, created_at DESC);