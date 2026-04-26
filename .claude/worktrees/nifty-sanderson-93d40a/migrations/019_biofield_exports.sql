CREATE TABLE IF NOT EXISTS biofield_exports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reading_id UUID NOT NULL REFERENCES readings(id) ON DELETE CASCADE,
    baseline_id UUID NULL REFERENCES biofield_baselines(id) ON DELETE SET NULL,
    export_format TEXT NOT NULL,
    file_name TEXT NOT NULL,
    storage_path TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    byte_size BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT biofield_exports_format_check CHECK (export_format IN ('json')),
    CONSTRAINT biofield_exports_file_name_non_empty CHECK (char_length(trim(file_name)) > 0),
    CONSTRAINT biofield_exports_storage_path_non_empty CHECK (char_length(trim(storage_path)) > 0),
    CONSTRAINT biofield_exports_mime_type_non_empty CHECK (char_length(trim(mime_type)) > 0),
    CONSTRAINT biofield_exports_byte_size_non_negative CHECK (byte_size >= 0)
);

CREATE INDEX IF NOT EXISTS idx_biofield_exports_user_created_at
    ON biofield_exports (user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_biofield_exports_reading_created_at
    ON biofield_exports (reading_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_biofield_exports_baseline_created_at
    ON biofield_exports (baseline_id, created_at DESC)
    WHERE baseline_id IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_biofield_exports_storage_path
    ON biofield_exports (storage_path);
