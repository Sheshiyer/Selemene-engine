-- Backfill per-engine permission scopes on legacy FULL-ACCESS API keys only.
--
-- Context: engine routes now check "<engine_id>:read" on API-key principals
-- (ENGINE_SCOPE_MODE=audit|enforce). Keys issued by scripts/generate_api_keys.py
-- with the legacy ALL_ENGINE_PERMS bundle predate several runtime engines
-- (raaga, transits, biofield-capture, financial-biosensor, sigil-forge).
--
-- Authorization is per key, never per tier: this script expands ONLY keys that
-- hold the complete legacy full-access bundle (all 17 legacy scopes below),
-- i.e. keys that were clearly issued as "all engines". A key that was issued
-- with a deliberately narrow list (for example basic:access + face-reading:read)
-- is left exactly as it is, whatever its tier. Idempotent; wrapped in a
-- transaction. Review the preview counts, then COMMIT or ROLLBACK.

BEGIN;

-- The legacy "all engines" bundle as generate_api_keys.py issued it.
CREATE TEMP TABLE legacy_full_bundle (perm text) ON COMMIT DROP;
INSERT INTO legacy_full_bundle VALUES
  ('basic:access'),
  ('panchanga:read'),('panchanga:batch'),('numerology:read'),('biorhythm:read'),
  ('human-design:read'),('gene-keys:read'),('vimshottari:read'),('vedic-clock:read'),
  ('biofield:read'),('face-reading:read'),('nadabrahman:read'),('transits:read'),
  ('tarot:read'),('i-ching:read'),('enneagram:read'),('sacred-geometry:read'),
  ('sigil-forge:read');

-- Engine scopes added to the runtime after the bundle was defined.
CREATE TEMP TABLE newer_engine_scopes (perm text) ON COMMIT DROP;
INSERT INTO newer_engine_scopes VALUES
  ('raaga:read'),('biofield-capture:read'),('financial-biosensor:read');

-- Candidates: active keys holding EVERY legacy bundle scope (jsonb ?& = contains all).
CREATE TEMP TABLE candidates ON COMMIT DROP AS
SELECT k.id, k.tier, k.name, k.permissions,
       (SELECT jsonb_agg(DISTINCT e)
          FROM jsonb_array_elements(k.permissions || (SELECT jsonb_agg(to_jsonb(perm)) FROM newer_engine_scopes)) e) AS merged
FROM api_keys k
WHERE k.is_active
  AND k.permissions ?& (SELECT array_agg(perm) FROM legacy_full_bundle)
  AND NOT (k.permissions ? 'engines:*')
  AND NOT (k.permissions ? 'admin:*');

-- Preview: what would change. Narrow keys never appear here.
SELECT tier, count(*) AS full_bundle_keys,
       count(*) FILTER (WHERE merged <> permissions) AS keys_changing
FROM candidates GROUP BY tier ORDER BY tier;

SELECT tier, name, jsonb_array_length(permissions) AS before_count,
       jsonb_array_length(merged) AS after_count
FROM candidates WHERE merged <> permissions ORDER BY tier, name;

UPDATE api_keys k
SET permissions = c.merged
FROM candidates c
WHERE k.id = c.id AND c.merged <> k.permissions;

-- Inspect the preview above, then: COMMIT;  -- or ROLLBACK;
