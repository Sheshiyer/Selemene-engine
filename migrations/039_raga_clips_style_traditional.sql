-- Migration 039: allow style 'traditional' on raga_clips.
--
-- The Suno scripts (ts-engines/scripts/suno-bulk-gen.ts, suno-smoke.ts) and the
-- GET /api/v1/raga/:num/clip handler use style 'traditional', which the CHECK
-- from 028_raga_clips.sql rejects. Widen the constraint; nothing else changes.
-- Reverse: see DOWN block at bottom.

-- ── UP ───────────────────────────────────────────────────────────────────

ALTER TABLE raga_clips DROP CONSTRAINT IF EXISTS raga_clips_style_check;
ALTER TABLE raga_clips ADD CONSTRAINT raga_clips_style_check
    CHECK (style IN ('ambient', 'meditative', 'cinematic', 'acid', 'traditional'));

-- ── DOWN ─────────────────────────────────────────────────────────────────
-- ALTER TABLE raga_clips DROP CONSTRAINT IF EXISTS raga_clips_style_check;
-- ALTER TABLE raga_clips ADD CONSTRAINT raga_clips_style_check
--     CHECK (style IN ('ambient', 'meditative', 'cinematic', 'acid'));
