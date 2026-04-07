-- Migration: 017_biofield_sessions
-- Description: Add biofield session lifecycle and artifact metadata persistence surfaces.

CREATE TABLE IF NOT EXISTS biofield_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(16) NOT NULL DEFAULT 'active',
    client_device_id VARCHAR(128),
    viewer_version VARCHAR(64),
    notes TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT biofield_sessions_status_check CHECK (
        status IN ('active', 'closed', 'abandoned')
    ),
    CONSTRAINT biofield_sessions_status_closed_at_check CHECK (
        (status = 'active' AND closed_at IS NULL)
        OR (status IN ('closed', 'abandoned') AND closed_at IS NOT NULL)
    ),
    CONSTRAINT biofield_sessions_closed_at_order_check CHECK (
        closed_at IS NULL OR closed_at >= started_at
    )
);

CREATE INDEX IF NOT EXISTS idx_biofield_sessions_user_id
    ON biofield_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_biofield_sessions_user_started_at
    ON biofield_sessions(user_id, started_at DESC);
CREATE INDEX IF NOT EXISTS idx_biofield_sessions_user_status_started_at
    ON biofield_sessions(user_id, status, started_at DESC);

DROP TRIGGER IF EXISTS update_biofield_sessions_updated_at ON biofield_sessions;
CREATE TRIGGER update_biofield_sessions_updated_at
    BEFORE UPDATE ON biofield_sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE IF NOT EXISTS biofield_capture_artifacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES biofield_sessions(id) ON DELETE CASCADE,
    reading_id UUID REFERENCES readings(id) ON DELETE SET NULL,
    artifact_kind VARCHAR(32) NOT NULL,
    storage_path TEXT NOT NULL,
    mime_type VARCHAR(128) NOT NULL,
    byte_size BIGINT NOT NULL,
    capture_metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT biofield_capture_artifacts_kind_check CHECK (
        artifact_kind IN (
            'source-image',
            'segmentation-mask',
            'analysis-overlay',
            'thumbnail'
        )
    ),
    CONSTRAINT biofield_capture_artifacts_storage_path_nonempty CHECK (
        btrim(storage_path) <> ''
    ),
    CONSTRAINT biofield_capture_artifacts_mime_type_nonempty CHECK (
        btrim(mime_type) <> ''
    ),
    CONSTRAINT biofield_capture_artifacts_byte_size_nonnegative CHECK (
        byte_size >= 0
    )
);

CREATE INDEX IF NOT EXISTS idx_biofield_capture_artifacts_session_id
    ON biofield_capture_artifacts(session_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_biofield_capture_artifacts_reading_id
    ON biofield_capture_artifacts(reading_id, created_at DESC)
    WHERE reading_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_biofield_capture_artifacts_session_kind
    ON biofield_capture_artifacts(session_id, artifact_kind, created_at DESC);
CREATE UNIQUE INDEX IF NOT EXISTS uq_biofield_capture_artifacts_storage_path
    ON biofield_capture_artifacts(storage_path);
