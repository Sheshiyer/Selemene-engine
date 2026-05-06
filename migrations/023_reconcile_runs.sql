-- Migration: 023_reconcile_runs
-- Description: Persists one row per execution of `dodo_reconcile` (cron + manual).
--              The admin dashboard reads `latest` to surface the most recent drift
--              report. Without this, "show drift" can only be solved by re-running
--              the bin synchronously which doesn't fit the request lifecycle.
--
-- Pruning: rows older than 30 days can be deleted by maintenance. Sentry + Grafana
-- carry the alerting story; this table is for human-readable inspection only.

CREATE TABLE IF NOT EXISTS reconcile_runs (
    id              BIGSERIAL PRIMARY KEY,
    started_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at     TIMESTAMPTZ,
    force_cancel    BOOLEAN     NOT NULL DEFAULT FALSE,
    -- Full JSON output of the reconcile bin: counts per drift class, sample IDs,
    -- error if any. Never contains secrets.
    drift_json      JSONB       NOT NULL DEFAULT '{}'::jsonb,
    error           TEXT
);

CREATE INDEX IF NOT EXISTS idx_reconcile_runs_started_at
    ON reconcile_runs(started_at DESC);
