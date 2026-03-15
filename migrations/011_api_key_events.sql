-- Migration: 011_api_key_events
-- Description: Add API key lifecycle metadata columns and immutable event audit rows.

ALTER TABLE api_keys
ADD COLUMN IF NOT EXISTS created_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS revoked_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS revoked_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS revoked_reason TEXT,
ADD COLUMN IF NOT EXISTS rotated_from_key_id UUID REFERENCES api_keys(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_api_keys_created_by_user_id ON api_keys(created_by_user_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_revoked_by_user_id ON api_keys(revoked_by_user_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_rotated_from_key_id ON api_keys(rotated_from_key_id);

CREATE TABLE IF NOT EXISTS api_key_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_id UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
    actor_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    event_type VARCHAR(50) NOT NULL CHECK (event_type IN ('created', 'revoked', 'rotated')),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_api_key_events_key_id ON api_key_events(key_id);
CREATE INDEX IF NOT EXISTS idx_api_key_events_actor_user_id ON api_key_events(actor_user_id);
CREATE INDEX IF NOT EXISTS idx_api_key_events_event_type_created_at
    ON api_key_events(event_type, created_at DESC);

INSERT INTO api_key_events (key_id, actor_user_id, event_type, metadata, created_at)
SELECT
    k.id,
    k.created_by_user_id,
    'created',
    jsonb_strip_nulls(
        jsonb_build_object(
            'name', k.name,
            'key_prefix', k.key_prefix
        )
    ),
    k.created_at
FROM api_keys k
WHERE NOT EXISTS (
    SELECT 1
    FROM api_key_events e
    WHERE e.key_id = k.id
      AND e.event_type = 'created'
);
