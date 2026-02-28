-- Migration: 010_api_keys_expiration_tracking
-- Description: Add revocation metadata and expiration tracking for auto-delete functionality

-- Add revocation metadata columns
ALTER TABLE api_keys 
    ADD COLUMN IF NOT EXISTS revoked_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS revoked_reason VARCHAR(50),
    ADD COLUMN IF NOT EXISTS expiration_warning_sent_at TIMESTAMPTZ;

-- Add comment explaining the columns
COMMENT ON COLUMN api_keys.revoked_at IS 'Timestamp when the key was revoked (soft delete)';
COMMENT ON COLUMN api_keys.revoked_reason IS 'Reason for revocation: expired, manual, security, abuse';
COMMENT ON COLUMN api_keys.expiration_warning_sent_at IS 'When expiration warning email was sent';

-- Create partial index for finding active keys with expiration
-- This is crucial for the auto-delete job performance
CREATE INDEX IF NOT EXISTS idx_api_keys_active_expires 
    ON api_keys(expires_at) 
    WHERE is_active = true AND expires_at IS NOT NULL;

-- Create index for revoked keys audit queries
CREATE INDEX IF NOT EXISTS idx_api_keys_revoked 
    ON api_keys(revoked_at) 
    WHERE revoked_at IS NOT NULL;

-- Function to auto-revoke expired API keys
-- This can be called by pg_cron, a background job, or external cron
CREATE OR REPLACE FUNCTION revoke_expired_api_keys()
RETURNS TABLE(
    revoked_count INTEGER,
    revoked_key_ids UUID[]
) AS $$
DECLARE
    count INTEGER;
    key_ids UUID[];
BEGIN
    -- Collect IDs of keys that will be revoked (for return value)
    SELECT ARRAY_AGG(id) INTO key_ids
    FROM api_keys 
    WHERE is_active = true 
      AND expires_at IS NOT NULL 
      AND expires_at < NOW();

    -- Perform the revocation
    UPDATE api_keys 
    SET is_active = false,
        revoked_at = NOW(),
        revoked_reason = 'expired'
    WHERE is_active = true 
      AND expires_at IS NOT NULL 
      AND expires_at < NOW();

    GET DIAGNOSTICS count = ROW_COUNT;
    
    RETURN QUERY SELECT count, COALESCE(key_ids, ARRAY[]::UUID[]);
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION revoke_expired_api_keys() IS 'Revokes all API keys that have passed their expiration date. Returns count and IDs of revoked keys.';

-- Function to send expiration warnings (placeholder for email integration)
CREATE OR REPLACE FUNCTION mark_expiration_warnings_sent(
    hours_before_expiration INTEGER DEFAULT 24
)
RETURNS TABLE(
    warned_count INTEGER,
    warned_key_ids UUID[]
) AS $$
DECLARE
    count INTEGER;
    key_ids UUID[];
    warning_threshold TIMESTAMPTZ;
BEGIN
    warning_threshold := NOW() + (hours_before_expiration || ' hours')::INTERVAL;

    -- Collect IDs of keys needing warning
    SELECT ARRAY_AGG(id) INTO key_ids
    FROM api_keys 
    WHERE is_active = true 
      AND expires_at IS NOT NULL 
      AND expires_at < warning_threshold
      AND expires_at > NOW()
      AND expiration_warning_sent_at IS NULL;

    -- Mark warnings as sent
    UPDATE api_keys 
    SET expiration_warning_sent_at = NOW()
    WHERE is_active = true 
      AND expires_at IS NOT NULL 
      AND expires_at < warning_threshold
      AND expires_at > NOW()
      AND expiration_warning_sent_at IS NULL;

    GET DIAGNOSTICS count = ROW_COUNT;
    
    RETURN QUERY SELECT count, COALESCE(key_ids, ARRAY[]::UUID[]);
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION mark_expiration_warnings_sent(INTEGER) IS 'Marks API keys as having received expiration warnings. Call this after sending emails.';

-- View for monitoring key expiration status
CREATE OR REPLACE VIEW api_key_expiration_status AS
SELECT 
    k.id,
    k.name,
    k.user_id,
    u.email as user_email,
    k.tier,
    k.is_active,
    k.created_at,
    k.expires_at,
    k.revoked_at,
    k.revoked_reason,
    CASE 
        WHEN NOT k.is_active THEN 'revoked'
        WHEN k.expires_at IS NULL THEN 'never'
        WHEN k.expires_at < NOW() THEN 'expired'
        WHEN k.expires_at < NOW() + INTERVAL '24 hours' THEN 'expiring_soon'
        ELSE 'active'
    END as expiration_status,
    CASE 
        WHEN k.expires_at IS NULL THEN NULL
        ELSE EXTRACT(EPOCH FROM (k.expires_at - NOW())) / 3600
    END as hours_until_expiration
FROM api_keys k
JOIN users u ON u.id = k.user_id;

COMMENT ON VIEW api_key_expiration_status IS 'Convenient view for monitoring API key expiration status';

-- Grant appropriate permissions (adjust based on your setup)
-- GRANT SELECT ON api_key_expiration_status TO selemene_readonly;
