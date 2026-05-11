-- Migration: 026_tier_check_constraints
-- Description: Normalize existing invalid tier values then add CHECK constraints
--              on users.tier and api_keys.tier (G-20).
--              Valid tiers: free, basic, premium, enterprise.
--
-- Step 1: Normalize dirty data from before the constraint existed
--   'standard' was an old alias for 'basic'
--   'Free' (capitalized) → 'free'
--   any other unknown tier → 'free' (safe default)

UPDATE users SET tier = 'free'  WHERE tier = 'Free';
UPDATE users SET tier = 'basic' WHERE tier = 'standard';
UPDATE users SET tier = 'free'  WHERE tier NOT IN ('free', 'basic', 'premium', 'enterprise');

UPDATE api_keys SET tier = 'basic' WHERE tier = 'standard';
UPDATE api_keys SET tier = 'free'  WHERE tier NOT IN ('free', 'basic', 'premium', 'enterprise');

-- Step 2: Add CHECK constraints (will succeed now that data is clean)
ALTER TABLE users
  ADD CONSTRAINT chk_users_tier
  CHECK (tier IN ('free', 'basic', 'premium', 'enterprise'));

ALTER TABLE api_keys
  ADD CONSTRAINT chk_api_keys_tier
  CHECK (tier IN ('free', 'basic', 'premium', 'enterprise'));
