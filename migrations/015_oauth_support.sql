-- Migration: 015_oauth_support
-- Description: Support OAuth login (Discord) and nullable password_hash for OAuth-only users

-- Make password_hash nullable for OAuth-only users (who don't have a password)
ALTER TABLE users ALTER COLUMN password_hash DROP NOT NULL;

-- Track which auth provider created the account
ALTER TABLE users ADD COLUMN IF NOT EXISTS auth_provider VARCHAR(50) DEFAULT 'email';

-- Store Discord user ID for OAuth lookup
ALTER TABLE users ADD COLUMN IF NOT EXISTS discord_id VARCHAR(100);

-- Unique partial index: only one row per Discord ID (NULLs are excluded)
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_discord_id
    ON users(discord_id)
    WHERE discord_id IS NOT NULL;
