-- Migration: 003_user_progression
-- Description: Add experience_points column to users table and create progression_logs

ALTER TABLE users
ADD COLUMN experience_points INTEGER NOT NULL DEFAULT 0;

CREATE TABLE progression_logs (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    action_type VARCHAR(50) NOT NULL, -- e.g. "calculation", "daily_login", "reflection"
    xp_amount INTEGER NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_progression_logs_user_id ON progression_logs(user_id);
