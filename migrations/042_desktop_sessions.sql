-- Migration: 042_desktop_sessions
-- Description: Browser-based desktop sign-in (PKCE loopback flow).
--
-- A desktop client (Sankalpa) opens /api/v1/desktop/authorize in the system
-- browser, which sits behind Cloudflare Access. That request issues a
-- single-use authorization code bound to the PKCE challenge and loopback
-- redirect. The client exchanges it at /api/v1/desktop/token for a short-lived
-- Selemene JWT plus a rotating refresh token.
--
-- Only SHA-256 hashes of codes and refresh tokens are stored. Rows snapshot the
-- user's tier and consciousness level at sign-in so refreshes can mint tokens
-- without widening permissions. There is deliberately no foreign key to users:
-- development-bypass identities (local end-to-end runs) have no users row.

CREATE TABLE IF NOT EXISTS desktop_auth_codes (
    code_hash           TEXT PRIMARY KEY,
    user_id             UUID NOT NULL,
    code_challenge      TEXT NOT NULL,
    redirect_uri        TEXT NOT NULL,
    tier                TEXT NOT NULL,
    consciousness_level SMALLINT NOT NULL CHECK (consciousness_level BETWEEN 0 AND 5),
    expires_at          TIMESTAMPTZ NOT NULL,
    consumed_at         TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_desktop_auth_codes_expires_at
    ON desktop_auth_codes(expires_at);

CREATE TABLE IF NOT EXISTS desktop_refresh_tokens (
    token_hash          TEXT PRIMARY KEY,
    family_id           UUID NOT NULL,
    user_id             UUID NOT NULL,
    tier                TEXT NOT NULL,
    consciousness_level SMALLINT NOT NULL CHECK (consciousness_level BETWEEN 0 AND 5),
    expires_at          TIMESTAMPTZ NOT NULL,
    rotated_at          TIMESTAMPTZ,
    revoked_at          TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_desktop_refresh_tokens_family
    ON desktop_refresh_tokens(family_id);
CREATE INDEX IF NOT EXISTS idx_desktop_refresh_tokens_user
    ON desktop_refresh_tokens(user_id);
