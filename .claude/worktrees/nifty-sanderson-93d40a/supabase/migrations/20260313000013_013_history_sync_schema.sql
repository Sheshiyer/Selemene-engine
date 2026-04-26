-- Migration: 013_history_sync_schema
-- Description: Add history sync tables and idempotent reading write columns.

CREATE TABLE IF NOT EXISTS user_devices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    client_device_id VARCHAR(128) NOT NULL,
    platform VARCHAR(32),
    app_version VARCHAR(32),
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_client_event_id VARCHAR(128),
    last_seen_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT user_devices_user_client_key UNIQUE (user_id, client_device_id)
);

CREATE INDEX IF NOT EXISTS idx_user_devices_user_id ON user_devices(user_id);
CREATE INDEX IF NOT EXISTS idx_user_devices_last_seen_at ON user_devices(last_seen_at DESC);

DROP TRIGGER IF EXISTS update_user_devices_updated_at ON user_devices;
CREATE TRIGGER update_user_devices_updated_at
    BEFORE UPDATE ON user_devices
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE IF NOT EXISTS history_sync_state (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id UUID NOT NULL REFERENCES user_devices(id) ON DELETE CASCADE,
    last_synced_cursor BIGINT NOT NULL DEFAULT 0,
    last_client_event_id VARCHAR(128),
    last_synced_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT history_sync_state_pk PRIMARY KEY (user_id, device_id),
    CONSTRAINT history_sync_state_cursor_nonnegative CHECK (last_synced_cursor >= 0)
);

CREATE INDEX IF NOT EXISTS idx_history_sync_state_user_cursor
    ON history_sync_state(user_id, last_synced_cursor DESC);
CREATE INDEX IF NOT EXISTS idx_history_sync_state_device_id
    ON history_sync_state(device_id);

DROP TRIGGER IF EXISTS update_history_sync_state_updated_at ON history_sync_state;
CREATE TRIGGER update_history_sync_state_updated_at
    BEFORE UPDATE ON history_sync_state
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE SEQUENCE IF NOT EXISTS readings_sync_cursor_seq;

ALTER TABLE readings
ADD COLUMN IF NOT EXISTS source_device_id UUID REFERENCES user_devices(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS client_event_id VARCHAR(128),
ADD COLUMN IF NOT EXISTS sync_cursor BIGINT;

ALTER TABLE readings
ALTER COLUMN sync_cursor SET DEFAULT nextval('readings_sync_cursor_seq');

UPDATE readings
SET sync_cursor = nextval('readings_sync_cursor_seq')
WHERE sync_cursor IS NULL;

SELECT setval(
    'readings_sync_cursor_seq',
    GREATEST(COALESCE((SELECT MAX(sync_cursor) FROM readings), 1), 1),
    true
);

ALTER TABLE readings
ALTER COLUMN sync_cursor SET NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_readings_sync_cursor ON readings(sync_cursor);
CREATE UNIQUE INDEX IF NOT EXISTS idx_readings_user_client_event_id
    ON readings(user_id, client_event_id)
    WHERE client_event_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_readings_user_sync_cursor
    ON readings(user_id, sync_cursor DESC);
