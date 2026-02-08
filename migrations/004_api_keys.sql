-- Migration: 004_api_keys
-- Description: Create api_keys table and usage_logs partitioned table for API key persistence and usage tracking

-- ============================================================
-- API Keys table
-- Stores hashed API keys with user association, tier, and rate limits.
-- key_hash is SHA-256 hex digest (64 chars) of the plaintext key.
-- ============================================================
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_hash VARCHAR(64) UNIQUE NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tier VARCHAR(50) NOT NULL,
    permissions JSONB NOT NULL DEFAULT '[]',
    consciousness_level INTEGER NOT NULL DEFAULT 0,
    rate_limit INTEGER NOT NULL DEFAULT 60,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    last_used TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true
);

-- Fast lookup by key_hash (primary auth path)
CREATE UNIQUE INDEX IF NOT EXISTS idx_api_keys_key_hash ON api_keys(key_hash);

-- Find all keys for a given user
CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys(user_id);

-- ============================================================
-- Usage Logs table (partitioned by month on created_at)
-- Tracks per-request usage for billing, analytics, and rate enforcement.
-- Partitioned for efficient time-range queries and easy data retention.
-- ============================================================
CREATE TABLE IF NOT EXISTS usage_logs (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    engine_id VARCHAR(100),
    workflow_id VARCHAR(100),
    status VARCHAR(50) NOT NULL,
    duration_ms INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

-- Index on user_id for per-user usage queries
CREATE INDEX IF NOT EXISTS idx_usage_logs_user_id ON usage_logs(user_id);

-- Index on created_at for time-range queries (each partition gets this)
CREATE INDEX IF NOT EXISTS idx_usage_logs_created_at ON usage_logs(created_at);

-- ============================================================
-- Create initial partitions (2026 monthly)
-- New partitions should be created as part of operational maintenance.
-- ============================================================
CREATE TABLE usage_logs_2026_01 PARTITION OF usage_logs
    FOR VALUES FROM ('2026-01-01') TO ('2026-02-01');

CREATE TABLE usage_logs_2026_02 PARTITION OF usage_logs
    FOR VALUES FROM ('2026-02-01') TO ('2026-03-01');

CREATE TABLE usage_logs_2026_03 PARTITION OF usage_logs
    FOR VALUES FROM ('2026-03-01') TO ('2026-04-01');

CREATE TABLE usage_logs_2026_04 PARTITION OF usage_logs
    FOR VALUES FROM ('2026-04-01') TO ('2026-05-01');

CREATE TABLE usage_logs_2026_05 PARTITION OF usage_logs
    FOR VALUES FROM ('2026-05-01') TO ('2026-06-01');

CREATE TABLE usage_logs_2026_06 PARTITION OF usage_logs
    FOR VALUES FROM ('2026-06-01') TO ('2026-07-01');

CREATE TABLE usage_logs_2026_07 PARTITION OF usage_logs
    FOR VALUES FROM ('2026-07-01') TO ('2026-08-01');

CREATE TABLE usage_logs_2026_08 PARTITION OF usage_logs
    FOR VALUES FROM ('2026-08-01') TO ('2026-09-01');

CREATE TABLE usage_logs_2026_09 PARTITION OF usage_logs
    FOR VALUES FROM ('2026-09-01') TO ('2026-10-01');

CREATE TABLE usage_logs_2026_10 PARTITION OF usage_logs
    FOR VALUES FROM ('2026-10-01') TO ('2026-11-01');

CREATE TABLE usage_logs_2026_11 PARTITION OF usage_logs
    FOR VALUES FROM ('2026-11-01') TO ('2026-12-01');

CREATE TABLE usage_logs_2026_12 PARTITION OF usage_logs
    FOR VALUES FROM ('2026-12-01') TO ('2027-01-01');
