-- Migration 040: drop the raga_clips table (Suno integration retired).
--
-- raga_clips (028) held Suno-rendered melakarta clips; the Suno bridge, its
-- scripts and the /raga/{num}/clip + /internal/raga/clip routes are removed.
-- Hosted raaga clips are now rendered in-process by ts-engines and stored on
-- Cloudflare R2 (RAAGA_CLIP_MODE=r2), which needs no database table.
--
-- Operator precondition: export the existing rows to a CSV kept OUTSIDE the
-- repo before applying in any environment, e.g.
--   psql "$DATABASE_URL" -c "\copy raga_clips TO 'raga_clips-<env>-<date>.csv' CSV HEADER"
-- Reverse: see DOWN block at bottom (schema only; rows come back from the CSV).

-- ── UP ───────────────────────────────────────────────────────────────────

DROP TRIGGER IF EXISTS trg_raga_clips_set_approved_at ON raga_clips;
DROP FUNCTION IF EXISTS set_raga_clip_approved_at();
DROP INDEX IF EXISTS idx_raga_clips_suno_song_id;
DROP INDEX IF EXISTS idx_raga_clips_status_created;
DROP INDEX IF EXISTS idx_raga_clips_melakarta_status;
DROP TABLE IF EXISTS raga_clips;

-- ── DOWN ─────────────────────────────────────────────────────────────────
-- Re-apply 028_raga_clips.sql then 039_raga_clips_style_traditional.sql, and
-- restore rows with: \copy raga_clips FROM 'raga_clips-<env>-<date>.csv' CSV HEADER
