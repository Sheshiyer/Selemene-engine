-- One daily-field reading per user, part and IST date. Concurrent refreshes and the
-- scheduler insert with ON CONFLICT DO NOTHING against this index, so a part is never
-- stored twice. Any duplicates left by an earlier build keep their oldest row.
DELETE FROM readings AS dup
USING readings AS keep
WHERE dup.workflow_id = 'daily-field'
  AND keep.workflow_id = 'daily-field'
  AND dup.user_id = keep.user_id
  AND dup.engine_id = keep.engine_id
  AND dup.input_data->>'daily_date' = keep.input_data->>'daily_date'
  AND (keep.created_at, keep.id) < (dup.created_at, dup.id);

CREATE UNIQUE INDEX IF NOT EXISTS idx_readings_daily_field_unique
    ON readings (user_id, engine_id, (input_data->>'daily_date'))
    WHERE workflow_id = 'daily-field';
