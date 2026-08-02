-- Expiring, revocable invitations for exactly one archived reading.
-- The bearer secret is returned once by the API; PostgreSQL stores only its SHA-256 digest.

CREATE TABLE archived_reading_invitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reading_id UUID NOT NULL REFERENCES archived_readings(id) ON DELETE CASCADE,
    token_digest VARCHAR(64) NOT NULL UNIQUE,
    created_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    revoked_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT archived_reading_invitations_digest_format
        CHECK (token_digest ~ '^[0-9a-f]{64}$'),
    CONSTRAINT archived_reading_invitations_expiry_after_creation
        CHECK (expires_at > created_at),
    CONSTRAINT archived_reading_invitations_revocation_consistency
        CHECK (
            (revoked_at IS NULL AND revoked_by_user_id IS NULL)
            OR revoked_at IS NOT NULL
        )
);

CREATE INDEX idx_archived_reading_invitations_reading_created
    ON archived_reading_invitations(reading_id, created_at DESC);
CREATE INDEX idx_archived_reading_invitations_active_expiry
    ON archived_reading_invitations(expires_at)
    WHERE revoked_at IS NULL;

COMMENT ON COLUMN archived_reading_invitations.token_digest IS
    'SHA-256 digest of the 256-bit bearer token. Plaintext tokens are never persisted.';

-- <down>
-- DROP TABLE IF EXISTS archived_reading_invitations;
-- </down>
