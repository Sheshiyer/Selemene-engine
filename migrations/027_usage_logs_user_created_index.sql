-- Migration: 027_usage_logs_user_created_index
-- Description: Add composite index on usage_logs(user_id, created_at) for
--              fast per-user analytics queries used by admin billing dashboard (G-21).
--              Note: CONCURRENTLY is not supported on partitioned tables; use regular CREATE.

CREATE INDEX IF NOT EXISTS idx_usage_logs_user_created
  ON usage_logs (user_id, created_at DESC);
