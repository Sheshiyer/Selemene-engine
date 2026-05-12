-- Migration 030: Index for efficient auto-purge of revoked API keys
--
-- The daily background task runs:
--   DELETE FROM api_keys WHERE is_active = false AND revoked_at < NOW() - '30 days'::interval
--
-- Without an index this is a full table scan. This partial index covers only
-- revoked rows, keeping it small even as the active key set grows.

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_api_keys_revoked_cleanup
    ON api_keys (revoked_at)
    WHERE is_active = false;

COMMENT ON INDEX idx_api_keys_revoked_cleanup IS
    'Supports daily auto-purge of revoked API keys older than 30 days';
