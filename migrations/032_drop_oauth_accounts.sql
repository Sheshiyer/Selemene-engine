-- Migration 032: Drop oauth_accounts after Cloudflare Access cutover.
--
-- This is intentionally destructive. Apply only after:
-- 1. Discord OAuth routes are removed from noesis-api.
-- 2. Admin web no longer calls Discord OAuth helpers.
-- 3. Production traffic is protected by Cloudflare Access.
--
-- No DOWN migration is provided because the locked product decision is
-- surgical retirement with no rollback path.

DROP TABLE IF EXISTS oauth_accounts;
