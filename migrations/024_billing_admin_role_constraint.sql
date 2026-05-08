-- Migration 024: Add billing-admin to user_roles CHECK constraint
--
-- The user_roles table (migration 010) restricts roles to:
--   ('viewer', 'support', 'admin', 'platform-admin')
-- billing-admin was introduced in v3.3.0 but is absent from that constraint.
-- This migration adds it so the user_roles table path can be used for
-- billing-admin grants (currently granted via user_profiles.preferences fallback).
--
-- After this migration is applied, operators can use:
--   INSERT INTO user_roles (user_id, role) VALUES ('<id>', 'billing-admin')
-- instead of the user_profiles.preferences workaround.

-- Drop and recreate the constraint with billing-admin included
ALTER TABLE user_roles
  DROP CONSTRAINT IF EXISTS user_roles_role_check;

ALTER TABLE user_roles
  ADD CONSTRAINT user_roles_role_check
  CHECK (role IN ('viewer', 'support', 'admin', 'platform-admin', 'billing-admin'));

-- Also add a composite index for faster role lookups per user
CREATE INDEX IF NOT EXISTS idx_user_roles_user_role
  ON user_roles (user_id, role);
