-- Migration: 043_asset_jobs
-- Description: Background jobs for model-written readings (/api/v1/assets/jobs).
--
-- A written L0 reading takes several minutes: twelve sequential model passes.
-- That is longer than the edge proxy allows for one request, so the API starts
-- a job, returns its id, and the client polls. Each row tracks one job's
-- progress and, once done, the full response. Jobs are owned by the user who
-- started them; there is no foreign key to users because development-bypass
-- identities have no users row.

CREATE TABLE IF NOT EXISTS asset_jobs (
    id               UUID PRIMARY KEY,
    user_id          TEXT NOT NULL,
    mode             TEXT NOT NULL,
    status           TEXT NOT NULL CHECK (status IN ('running', 'succeeded', 'failed')),
    completed_passes INTEGER NOT NULL DEFAULT 0 CHECK (completed_passes >= 0),
    total_passes     INTEGER NOT NULL CHECK (total_passes >= 0),
    current_pass     TEXT,
    result           JSONB,
    error            TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_asset_jobs_user_status
    ON asset_jobs(user_id, status);
