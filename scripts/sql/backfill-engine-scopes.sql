-- Backfill per-engine permission scopes on existing API keys.
--
-- Context: engine routes now check "<engine_id>:read" on API-key principals
-- (ENGINE_SCOPE_MODE=audit|enforce). Keys issued by scripts/generate_api_keys.py
-- predate several runtime engines, so premium/enterprise keys lack scopes for
-- engines they are entitled to. Free and basic tiers are intentionally left as
-- issued. Idempotent; safe to re-run. Review the SELECT before the UPDATE.
--
-- Usage (read-only preview):
--   psql "$DATABASE_URL" -c "\i scripts/sql/backfill-engine-scopes.sql" -v dry_run=1
-- The UPDATE is guarded by a transaction; COMMIT only after reviewing counts.

BEGIN;

WITH full_set AS (
  SELECT jsonb_agg(to_jsonb(p)) AS perms FROM unnest(ARRAY[
    'basic:access',
    'panchanga:read','panchanga:batch','numerology:read','biorhythm:read',
    'human-design:read','gene-keys:read','vimshottari:read','vedic-clock:read',
    'biofield:read','biofield-capture:read','face-reading:read','nadabrahman:read',
    'transits:read','financial-biosensor:read',
    'tarot:read','i-ching:read','enneagram:read','sacred-geometry:read',
    'sigil-forge:read','raaga:read'
  ]) AS p
),
candidates AS (
  SELECT k.id, k.tier, k.permissions,
         (SELECT jsonb_agg(DISTINCT e) FROM jsonb_array_elements(k.permissions || (SELECT perms FROM full_set)) e) AS merged
  FROM api_keys k
  WHERE k.is_active
    AND k.tier IN ('premium','enterprise')
    AND NOT (k.permissions ? 'engines:*')
    AND NOT (k.permissions ? 'admin:*')
)
SELECT tier, count(*) AS keys_to_update,
       count(*) FILTER (WHERE merged <> permissions) AS keys_actually_changing
FROM candidates GROUP BY tier;

UPDATE api_keys k
SET permissions = c.merged
FROM (
  SELECT k2.id,
         (SELECT jsonb_agg(DISTINCT e) FROM jsonb_array_elements(k2.permissions || (
            SELECT jsonb_agg(to_jsonb(p)) FROM unnest(ARRAY[
              'basic:access',
              'panchanga:read','panchanga:batch','numerology:read','biorhythm:read',
              'human-design:read','gene-keys:read','vimshottari:read','vedic-clock:read',
              'biofield:read','biofield-capture:read','face-reading:read','nadabrahman:read',
              'transits:read','financial-biosensor:read',
              'tarot:read','i-ching:read','enneagram:read','sacred-geometry:read',
              'sigil-forge:read','raaga:read'
            ]) AS p)) e) AS merged
  FROM api_keys k2
  WHERE k2.is_active AND k2.tier IN ('premium','enterprise')
    AND NOT (k2.permissions ? 'engines:*') AND NOT (k2.permissions ? 'admin:*')
) c
WHERE k.id = c.id AND c.merged <> k.permissions;

-- Inspect, then COMMIT; or ROLLBACK.
