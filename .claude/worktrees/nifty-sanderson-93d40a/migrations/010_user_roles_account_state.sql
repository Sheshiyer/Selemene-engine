-- Migration: 010_user_roles_account_state
-- Description: Add canonical user_roles and user_account_state tables with backfill.

CREATE TABLE IF NOT EXISTS user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL CHECK (role IN ('viewer', 'support', 'admin', 'platform-admin')),
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, role)
);

CREATE INDEX IF NOT EXISTS idx_user_roles_role ON user_roles(role);

CREATE TABLE IF NOT EXISTS user_account_state (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    state VARCHAR(50) NOT NULL CHECK (state IN ('active', 'locked')),
    locked_until TIMESTAMPTZ,
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT user_account_state_locked_consistency CHECK (
        (state = 'active' AND locked_until IS NULL)
        OR (state = 'locked' AND locked_until IS NOT NULL)
    )
);

CREATE INDEX IF NOT EXISTS idx_user_account_state_state ON user_account_state(state);
CREATE INDEX IF NOT EXISTS idx_user_account_state_locked_until ON user_account_state(locked_until);

DROP TRIGGER IF EXISTS update_user_account_state_updated_at ON user_account_state;
CREATE TRIGGER update_user_account_state_updated_at
    BEFORE UPDATE ON user_account_state
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

INSERT INTO user_roles (user_id, role)
SELECT up.user_id, role_entry.role
FROM user_profiles up
CROSS JOIN LATERAL (
    SELECT jsonb_array_elements_text(
        CASE
            WHEN jsonb_typeof(COALESCE(up.preferences -> 'admin_roles', '[]'::jsonb)) = 'array'
                THEN COALESCE(up.preferences -> 'admin_roles', '[]'::jsonb)
            ELSE '[]'::jsonb
        END
    ) AS role
) AS role_entry
WHERE role_entry.role IN ('viewer', 'support', 'admin', 'platform-admin')
ON CONFLICT (user_id, role) DO NOTHING;

INSERT INTO user_account_state (user_id, state, locked_until, reason)
SELECT
    u.id,
    CASE
        WHEN u.locked_until IS NOT NULL AND u.locked_until > NOW() THEN 'locked'
        ELSE 'active'
    END AS state,
    CASE
        WHEN u.locked_until IS NOT NULL AND u.locked_until > NOW() THEN u.locked_until
        ELSE NULL
    END AS locked_until,
    CASE
        WHEN u.locked_until IS NOT NULL AND u.locked_until > NOW() THEN 'Backfilled from users.locked_until'
        ELSE NULL
    END AS reason
FROM users u
ON CONFLICT (user_id) DO UPDATE
SET
    state = EXCLUDED.state,
    locked_until = EXCLUDED.locked_until,
    reason = EXCLUDED.reason,
    updated_at = NOW();
