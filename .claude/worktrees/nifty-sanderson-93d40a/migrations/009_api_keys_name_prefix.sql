-- Migration 009: Add human-friendly name and visible prefix to api_keys
-- Both columns nullable for backward compatibility with existing keys.

ALTER TABLE api_keys ADD COLUMN name VARCHAR(100);
ALTER TABLE api_keys ADD COLUMN key_prefix VARCHAR(12);

-- Partial index for search by name (only non-null rows indexed)
CREATE INDEX IF NOT EXISTS idx_api_keys_name ON api_keys(name) WHERE name IS NOT NULL;
