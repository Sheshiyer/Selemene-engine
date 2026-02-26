-- Migration: 006_auth_improvements
-- Description: Add login tracking and account lockout columns to users table

ALTER TABLE users
ADD COLUMN last_login_at TIMESTAMPTZ,
ADD COLUMN failed_login_attempts INTEGER NOT NULL DEFAULT 0,
ADD COLUMN locked_until TIMESTAMPTZ;
