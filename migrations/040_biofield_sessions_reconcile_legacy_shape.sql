-- Migration: 040_biofield_sessions_reconcile_legacy_shape
-- Description: Production biofield_sessions predates 017 (Supabase-era shape:
-- analysis_mode, analysis_region, source_kind, ended_at, score_recipe_version,
-- ...). 017 ran over it with CREATE TABLE IF NOT EXISTS and only added indexes
-- and a trigger, so the 017 columns notes/closed_at and the 017 status values
-- are missing there. Add them additively. Never drop or rename legacy columns:
-- Supabase-era views (biofield_session_summary, biofield_snapshot_detail,
-- biofield_reading_history) and foreign keys depend on them, and
-- admin_repository.rs reads ended_at. Idempotent on a 017-shaped database.
-- See docs/plans/2026-10-26-prod-migration-adoption-runbook.md section 5.

ALTER TABLE biofield_sessions
    ADD COLUMN IF NOT EXISTS notes TEXT,
    ADD COLUMN IF NOT EXISTS closed_at TIMESTAMPTZ;

ALTER TABLE biofield_sessions DROP CONSTRAINT IF EXISTS biofield_sessions_status_check;
ALTER TABLE biofield_sessions ADD CONSTRAINT biofield_sessions_status_check CHECK (
    status IN ('active', 'closed', 'abandoned', 'paused', 'completed', 'aborted')
);

ALTER TABLE biofield_sessions DROP CONSTRAINT IF EXISTS biofield_sessions_status_closed_at_check;
ALTER TABLE biofield_sessions ADD CONSTRAINT biofield_sessions_status_closed_at_check CHECK (
    (status = 'active' AND closed_at IS NULL)
    OR (status IN ('closed', 'abandoned') AND closed_at IS NOT NULL)
    OR status IN ('paused', 'completed', 'aborted')
);

ALTER TABLE biofield_sessions DROP CONSTRAINT IF EXISTS biofield_sessions_closed_at_order_check;
ALTER TABLE biofield_sessions ADD CONSTRAINT biofield_sessions_closed_at_order_check CHECK (
    closed_at IS NULL OR closed_at >= started_at
);
