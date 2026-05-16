-- Migration 028: raga_clips table for the Suno-rendered melakarta library.
--
-- Stores one row per (melakarta_num, style) clip generated via Suno
-- (gcui-art/suno-api wrapper) and uploaded to Cloudflare R2 for delivery.
--
-- See: raagaegnin/SUNO_INTEGRATION_PLAN.md task S-010.
-- Reverse: see DOWN block at bottom — apply with `sqlx migrate revert`.

-- ── UP ───────────────────────────────────────────────────────────────────

CREATE TABLE raga_clips (
    id              SERIAL          PRIMARY KEY,
    melakarta_num   SMALLINT        NOT NULL CHECK (melakarta_num BETWEEN 1 AND 72),
    style           VARCHAR(32)     NOT NULL DEFAULT 'ambient'
                    CHECK (style IN ('ambient', 'meditative', 'cinematic', 'acid')),
    -- REAL matches the f32 sqlx binding in upsert_raga_clip handler
    duration_sec    REAL            CHECK (duration_sec IS NULL OR (duration_sec > 0 AND duration_sec <= 300)),
    suno_song_id    TEXT            NOT NULL,
    suno_prompt     TEXT,
    r2_key          TEXT            UNIQUE,
    cdn_url         TEXT            NOT NULL,
    status          VARCHAR(16)     NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending', 'generated', 'approved', 'rejected', 'regenerate')),
    audition_notes  TEXT,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    approved_at     TIMESTAMPTZ,

    -- One canonical clip per (raga, style). Re-generations replace via
    -- upsert (ON CONFLICT DO UPDATE); old rows go through `rejected` first.
    UNIQUE (melakarta_num, style)
);

-- Hot path: serve "give me the approved ambient clip for raga 15" → indexed lookup.
CREATE INDEX idx_raga_clips_melakarta_status
    ON raga_clips (melakarta_num, status);

-- Allow admin audition to filter by status quickly.
CREATE INDEX idx_raga_clips_status_created
    ON raga_clips (status, created_at DESC);

-- Allow lookup-by-suno-id for cleanup / dedup operations.
CREATE INDEX idx_raga_clips_suno_song_id
    ON raga_clips (suno_song_id);

-- Auto-set approved_at when status transitions to 'approved'.
CREATE OR REPLACE FUNCTION set_raga_clip_approved_at()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.status = 'approved' AND (OLD.status IS DISTINCT FROM 'approved') THEN
        NEW.approved_at = NOW();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_raga_clips_set_approved_at
    BEFORE UPDATE ON raga_clips
    FOR EACH ROW
    EXECUTE FUNCTION set_raga_clip_approved_at();

COMMENT ON TABLE raga_clips IS
  'Suno-rendered audio clips for the 72 melakartas. One row per (melakarta_num, style). See raagaegnin/SUNO_INTEGRATION_PLAN.md.';
COMMENT ON COLUMN raga_clips.suno_song_id IS
  'The id returned by Suno''s /api/custom_generate endpoint. Used to fetch via /api/get during polling.';
COMMENT ON COLUMN raga_clips.r2_key IS
  'Cloudflare R2 object key. Convention: clips/{style}/{NN}-{suno_song_id}.mp3';
COMMENT ON COLUMN raga_clips.cdn_url IS
  'Public CDN URL for the clip. Cached 1y immutable at edge.';
COMMENT ON COLUMN raga_clips.status IS
  'pending → generated (after Suno+R2 upload) → approved/rejected/regenerate (after audition).';

-- ── DOWN ─────────────────────────────────────────────────────────────────
-- DROP TRIGGER IF EXISTS trg_raga_clips_set_approved_at ON raga_clips;
-- DROP FUNCTION IF EXISTS set_raga_clip_approved_at();
-- DROP INDEX IF EXISTS idx_raga_clips_suno_song_id;
-- DROP INDEX IF EXISTS idx_raga_clips_status_created;
-- DROP INDEX IF EXISTS idx_raga_clips_melakarta_status;
-- DROP TABLE IF EXISTS raga_clips;
