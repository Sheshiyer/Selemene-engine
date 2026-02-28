-- Migration: 008_auth_columns_backfill
-- Description: Ensure auth login tracking columns exist and are normalized.

ALTER TABLE users
ADD COLUMN IF NOT EXISTS last_login_at TIMESTAMPTZ;

ALTER TABLE users
ADD COLUMN IF NOT EXISTS failed_login_attempts INTEGER;

UPDATE users
SET failed_login_attempts = 0
WHERE failed_login_attempts IS NULL;

ALTER TABLE users
ALTER COLUMN failed_login_attempts SET DEFAULT 0;

ALTER TABLE users
ALTER COLUMN failed_login_attempts SET NOT NULL;

ALTER TABLE users
ADD COLUMN IF NOT EXISTS locked_until TIMESTAMPTZ;
