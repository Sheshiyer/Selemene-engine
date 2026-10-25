# Production migration adoption runbook (go/no-go)

Date: 2026-10-26
Scope: bring the production Postgres (Railway project `11eedde4-41e6-4f51-b86b-cf77111cf592`, environment `production` `702b945e-2c66-4d5a-bae1-4c67ea14c3bb`, service `Postgres` `1ba4a991-ca9a-4831-981e-aa3470101ef1`) under `noesis_migration_journal`, and make `POST /api/v1/biofield/sessions` work there.
Status: **PLAN ONLY.** Every query behind this document was read-only. Nothing in production has been modified. No step below may run until the operator signs off on section 8.

## 0. Headline findings

1. **Adopting the journal does not fix the biofield API by itself.** Production `biofield_sessions` is a Supabase-era table that predates root migration 017. Migration 017 was run against it later with `CREATE TABLE IF NOT EXISTS`, which silently kept the old shape and only added 017's indexes and trigger. The Rust repository inserts `notes` and updates `closed_at` with status `'closed'`; the production table has neither column and its status CHECK rejects `'closed'`. No existing migration adds them, so a **new migration `040`** is required (section 5.3).
2. **Correct adoption number is `--adopt-through 036`.** Everything through 036 is present in production (017 counts as "ran, shape diverged"). A plain run after adoption replays exactly `037`, `038`, `039`, all of which are safe on the current data (section 4).
3. **There is no usable backup today.** PITR is disabled, no backup schedule is configured, and the only volume backup ("Pre-Security-Patch Backup", 2026-08-23) expired on 2026-09-22. A fresh `pg_dump` plus an on-demand volume backup are hard prerequisites.
4. The repo's `admin_repository.rs` session listing reads `bs.ended_at AS closed_at`, i.e. it was written against the **production** shape, while `biofield_repository.rs` was written against the **017** shape. Migration 040 must therefore keep `ended_at` and add `closed_at`; otherwise one of the two code paths breaks.

## 1. Production inventory (read-only, 2026-10-26)

| Item | Value |
|---|---|
| Server | PostgreSQL 18.6 (Railway image `ghcr.io/railwayapp-templates/postgres-ssl:18`) |
| Database | `railway`, 527 MB (`pg_database_size` = 552,638,143 bytes) |
| Largest relations | `readings` 458 MB, `usage_logs_2026_07` 28 MB, `progression_logs` 23 MB |
| Live rows | users 35, api_keys 41, readings 300, progression_logs 300, usage_logs 303 across partitions, user_profiles 6; **biofield_sessions 0, raga_clips 0, user_roles 0, billing_subscriptions 0** |
| Oldest data | users from 2026-02-09, readings from 2026-02-15 |
| Journal | `to_regclass('public.noesis_migration_journal')` is NULL |
| Volume | `postgres-volume` `ccdec92c-e2de-4802-87be-01498a638ace`, 5000 MB, region asia-southeast1 |
| PITR | disabled, no bucket wired |
| Volume backups | one manual backup 2026-08-23 (62 MB), expired 2026-09-22; no schedule |
| Backups source | `railway postgres pitr status/backup list/schedule list --service Postgres --environment production` |

Supabase provenance of the database is unambiguous: schemas `extensions`, `graphql`, `graphql_public`, function `pgbouncer.get_auth`, `extensions.pgrst_ddl_watch`, `extensions.grant_pg_cron_access` and friends all exist. `scripts/supabase-to-railway-migrate.sh` (mode `--full`, `pg_dump | pg_restore`) is the repo tool that produced this database, and its `CRITICAL_TABLES` list already names `biofield_sessions` and `biofield_measurements`, confirming the biofield tables existed in Supabase before root migration 017 was written.

Supabase-era objects that no root migration creates and that must be left alone: tables `biofield_artifacts`, `biofield_snapshots`, `biofield_timeline_points`, `decoder_state`; views `biofield_reading_history`, `biofield_session_summary`, `biofield_snapshot_detail`; functions `set_current_timestamp_updated_at`, `update_updated_at`; triggers `set_biofield_*_updated_at`, `users_updated_at`, `user_profiles_updated_at`, `decoder_state_updated_at`. Several of them hold foreign keys into `biofield_sessions` and `readings`, so **nothing may drop or rename `biofield_sessions` columns** without first dropping those views.

Connection pattern for every command in this document (never print the URL):

```bash
railway run --service Postgres --environment production -- sh -c 'psql "$DATABASE_PUBLIC_URL" -X -v ON_ERROR_STOP=1 -f FILE.sql'
```

## 2. Migration-by-migration diagnosis

Evidence source: `information_schema.tables/columns`, `pg_indexes`, `pg_constraint`, `pg_proc`, `pg_trigger`, `pg_inherits` in production. "Applied" means every object the file creates exists in the expected shape. The evidence query column shows the probe that decides the row; `E` prefixes stand for:

- E1 `SELECT to_regclass('public.<table>')`
- E2 `SELECT column_name FROM information_schema.columns WHERE table_name='<t>'`
- E3 `SELECT indexname FROM pg_indexes WHERE indexname='<idx>'`
- E4 `SELECT conname, pg_get_constraintdef(oid), convalidated FROM pg_constraint WHERE conname='<c>'`
- E5 `SELECT proname FROM pg_proc WHERE pronamespace='public'::regnamespace`
- E6 `SELECT tgname FROM pg_trigger WHERE NOT tgisinternal`
- E7 `SELECT inhrelid::regclass FROM pg_inherits WHERE inhparent='usage_logs'::regclass`

| Migration | Status in production | Evidence |
|---|---|---|
| 001_initial_users | applied | E1 users, user_profiles; E5 update_updated_at_column; E6 update_users_updated_at |
| 002_password_reset | applied | E2 users.reset_token, reset_token_expires_at; E3 idx_users_reset_token |
| 003_user_progression | applied | E1 progression_logs; E2 users.experience_points; E3 idx_progression_logs_user_id |
| 004_api_keys | applied | E1 api_keys, usage_logs (partitioned); E7 usage_logs_2026_01..12; E3 idx_api_keys_key_hash |
| 005_readings | applied | E1 readings; E3 idx_readings_user_engine, idx_readings_input_hash, idx_readings_created_at |
| 006_auth_improvements | applied | E2 users.auth_provider, discord_id; password_hash nullable |
| 007_oauth_accounts | applied then dropped by 032 | E1 oauth_accounts is NULL (expected after 032) |
| 007_progression_logs_default_uuid | applied | E2 progression_logs.id default gen_random_uuid() |
| 008_auth_columns_backfill | applied | E2 users.last_login_at, failed_login_attempts NOT NULL default 0, locked_until |
| 009_api_keys_name_prefix | applied | E2 api_keys.name, key_prefix; E3 idx_api_keys_name |
| 010_user_roles_account_state | applied | E1 user_roles, user_account_state; E4 user_account_state_state_check; E6 update_user_account_state_updated_at |
| 011_api_key_events | applied | E1 api_key_events; E2 api_keys.created_by_user_id, revoked_at, revoked_by_user_id, revoked_reason, rotated_from_key_id |
| 012_usage_partition_maintenance | applied | E5 ensure_usage_log_partitions |
| 013_history_sync_schema | applied | E1 user_devices, history_sync_state; E2 readings.sync_cursor NOT NULL, client_event_id, source_device_id; E3 idx_readings_sync_cursor |
| 014_plan_catalog_billing_subscriptions | applied | E1 plan_catalog (codes free, basic, standard, premium, enterprise), billing_subscriptions, view user_active_plan_resolutions |
| 017_biofield_sessions | **partial (shape diverged)** | E1 biofield_sessions and biofield_capture_artifacts exist; E3 idx_biofield_sessions_user_started_at, idx_biofield_sessions_user_status_started_at and E6 update_biofield_sessions_updated_at exist (so 017 did run); but E2 shows the Supabase column set and E4 shows `biofield_sessions_status_check` = (active, paused, completed, aborted). See section 3. |
| 018_biofield_baselines | applied | E1 biofield_baselines, biofield_baseline_readings; E3 idx_biofield_baselines_user_updated_at; E6 update_biofield_baselines_updated_at |
| 019_biofield_exports | applied | E1 biofield_exports; E3 idx_biofield_exports_storage_path; E4 biofield_exports_format_check |
| 020_dodo_payments_columns | applied | E2 users.dodo_customer_id, plan_catalog.dodo_product_id; E3 uq_users_dodo_customer_id, idx_billing_subscriptions_provider_customer |
| 021_processed_webhook_events | applied | E1 processed_webhook_events |
| 022_engine_usage_monthly | applied | E1 engine_usage_monthly; E4 engine_usage_monthly_count_nonneg |
| 023_reconcile_runs | applied | E1 reconcile_runs; E3 idx_reconcile_runs_started_at |
| 024_billing_admin_role_constraint | applied | E4 user_roles_role_check includes billing-admin; E3 idx_user_roles_user_role |
| 025_usage_logs_2027_partitions | applied | E7 usage_logs_2027_01..12 (plus 2028_01..03 created by the maintenance function) |
| 026_tier_check_constraints | applied | E4 chk_users_tier, chk_api_keys_tier both validated |
| 027_usage_logs_user_created_index | applied | E3 idx_usage_logs_user_created on parent and every partition |
| 028_raga_clips | applied | E1 raga_clips; E5 set_raga_clip_approved_at; E6 trg_raga_clips_set_approved_at; E3 idx_raga_clips_suno_song_id |
| 029_readings_consciousness_check | applied | E4 readings_consciousness_level_check, convalidated = t; data min 0 max 5 |
| 030_api_key_auto_purge | applied | E3 idx_api_keys_revoked_cleanup |
| 031_humdes_validation | applied | E1 humdes_validation_runs, humdes_validation_records; E3 idx_humdes_records_person_field |
| 032_drop_oauth_accounts | applied | E1 oauth_accounts is NULL |
| 033_witness_dyad_executions | applied | E1 witness_dyad_executions with the full column set; E3 idx_wde_tier, idx_wde_llm_powered |
| 034_admin_readings_index | applied | E3 idx_readings_user_engine_created |
| 035_biofield_sessions_device_columns | applied | E2 biofield_sessions.client_device_id varchar(128), viewer_version varchar(64) |
| 036_living_readings_archive | applied | E1 reading_import_runs, reading_sources, corpus_subjects, corpus_subject_aliases, corpus_relationships, corpus_relationship_members, archived_readings, archived_reading_subjects, archived_reading_artifacts, archived_reading_evidence, archived_reading_editorial_states; E6 all five update_*_updated_at triggers |
| 037_living_reading_invitations | **missing** | E1 archived_reading_invitations is NULL |
| 038_billing_mode_control | **missing** | E1 billing_mode_control is NULL |
| 039_raga_clips_style_traditional | **missing** | E4 raga_clips_style_check = (ambient, meditative, cinematic, acid); `traditional` absent |

Staging, for comparison, has 38 journal rows (`001` through `039`, two `007` files) and the 017-shaped `biofield_sessions` (`id,user_id,status,client_device_id,viewer_version,notes,started_at,closed_at,created_at,updated_at`).

## 3. Where the production `biofield_sessions` shape came from

Production columns: `id, user_id, status(text), analysis_mode, analysis_region, source_kind, summary_reading_id, latest_snapshot_id, started_at, paused_at, ended_at, duration_seconds, score_recipe_version, metric_recipe_version, metadata, created_at, updated_at, client_device_id, viewer_version`.

Migration 017 defines: `id, user_id, status(varchar 16), client_device_id, viewer_version, notes, started_at, closed_at, created_at, updated_at`, plus checks `biofield_sessions_status_check (active|closed|abandoned)`, `biofield_sessions_status_closed_at_check`, `biofield_sessions_closed_at_order_check`.

Diff:

| Column / constraint | Production | 017 + 035 | Consequence |
|---|---|---|---|
| `notes` | absent | TEXT | `INSERT ... notes` fails: this is the live 500 (`column "notes" of relation "biofield_sessions" does not exist`) |
| `closed_at` | absent (`ended_at` instead) | TIMESTAMPTZ | `close_session` UPDATE fails; `BiofieldSession` struct `SELECT *` mapping fails |
| status CHECK | active, paused, completed, aborted | active, closed, abandoned | `close_session` writing `'closed'` would be rejected even if the column existed |
| `analysis_mode`, `analysis_region`, `source_kind`, `score_recipe_version`, `metric_recipe_version`, `metadata` | NOT NULL with defaults | not present | harmless for inserts (defaults fill them); referenced by Supabase-era views |
| `summary_reading_id`, `latest_snapshot_id`, `paused_at`, `ended_at`, `duration_seconds` | nullable | not present | harmless; `ended_at` is read by `admin_repository.rs` |
| `client_device_id`, `viewer_version` | present | present (017, re-added by 035) | fine |
| trigger | both `set_biofield_sessions_updated_at` (Supabase) and `update_biofield_sessions_updated_at` (017) | 017 only | harmless double-set of `updated_at` |

Repository history: `git log --all -S score_recipe_version` returns only four commits (`b075fbfb` a 2026-04 stash, `b211416f` the orphan `codex/archive-cleanup` root, `745f8549`, `63e900ca`), none of which contains the string in a tracked `.sql` file; `git grep` over every remote ref finds no SQL defining `analysis_region` or `score_recipe_version`. The shape was never authored under `migrations/`. It arrived with the Supabase database itself: the Supabase project carried its own biofield schema (sessions, snapshots, artifacts, timeline points, views, `set_current_timestamp_updated_at`), `scripts/supabase-to-railway-migrate.sh --full` restored that schema and data into Railway, and root migration 017 was later executed on top of it, where `CREATE TABLE IF NOT EXISTS` no-opped the table and only its `CREATE INDEX IF NOT EXISTS` and trigger statements took effect. Migration 035's own description ("Restore client_device_id and viewer_version columns") is the fossil of a previous encounter with this same drift.

The parent session's note that `biofield_readings` is missing is a red herring: no code under `crates/` references a `biofield_readings` table. Capture readings are rows in `readings` (engine `biofield-capture`) linked through `biofield_capture_artifacts.reading_id`, and both of those tables are present with the 017 shape.

## 4. `scripts/apply-migrations.sh` and `--adopt-through`

What the script does, in order, on every invocation:

1. Validates the ledger with `scripts/validate_migrations.py`: every `.sql` must be listed in `migrations/history.sha256` with a matching checksum, the historical block through 037 must hash to the canonical baseline, and versions above 037 must be contiguous (`038`, `039`, next legal `040`). A new 040 file without a ledger line fails here, before any connection is opened.
2. Probes the database: `journal_exists` and `schema_nonempty`.
3. Refuses outright when `journal_exists = f`, `schema_nonempty = t` and no `--adopt-through` is given. This is the production state today, which is why a plain run cannot be used first.
4. Refuses `--adopt-through` when the journal already exists, or when the schema is empty.
5. With `--adopt-through NNN` on a pre-journal, non-empty database it runs **one transaction** under `pg_advisory_xact_lock(684022132617857143)` that creates `noesis_migration_journal` and inserts one `applied` row (filename, on-disk SHA-256, `clock_timestamp()`) for every file whose three-digit prefix is `<= NNN`. **No migration SQL is executed during adoption.** Both `007_*` files are adopted because both have prefix 007.
6. It then continues in the same invocation as a plain run: checks for dirty `applying` rows, and for every file not in the journal applies it. Files whose prefix is `> NNN` are therefore applied immediately after adoption, in the same command.
7. Plain-run semantics: a journaled file whose on-disk checksum differs aborts the whole run (`Migration checksum mismatch`). Files containing `CREATE INDEX CONCURRENTLY` run non-transactionally with a dirty-state row; all others run inside `BEGIN ... COMMIT` together with their journal insert.

Correct NNN for production: **036**. Rationale: every file through 036 has its objects in place (section 2); 037, 038 and 039 are genuinely absent; adopting through 036 and letting the script apply the rest is exactly the designed path. Adopting through 039 would lie about three missing objects and the next deploy that touches `billing_mode_control`, `archived_reading_invitations` or `raga_clips.style = 'traditional'` would fail at runtime instead of at migration time. Adopting through 016 or lower and replaying 017+ is **not** an option: 026 and 029 would re-run `UPDATE users/api_keys` and constraint adds (they would error on duplicate constraint names and abort), 032 would attempt a drop, 036 uses bare `CREATE TABLE` and would fail on existing tables.

What a plain run replays after `--adopt-through 036`, and whether each is safe on production data:

| File | Statements | Classification | Safe on production? |
|---|---|---|---|
| 037_living_reading_invitations | `CREATE TABLE archived_reading_invitations` (bare), two `CREATE INDEX` | additive; not idempotent but the table is absent | yes |
| 038_billing_mode_control | `CREATE TABLE IF NOT EXISTS billing_mode_control`, `COMMENT`, `DROP TRIGGER IF EXISTS` + `CREATE TRIGGER` | additive, idempotent | yes |
| 039_raga_clips_style_traditional | `ALTER TABLE raga_clips DROP CONSTRAINT IF EXISTS raga_clips_style_check; ADD CONSTRAINT ...` widened | constraint swap; `raga_clips` has 0 rows | yes (no rows can violate) |

None of the three touches `users`, `api_keys`, `readings`, `usage_logs`, `billing_subscriptions` or `progression_logs`. No file in the replay set is destructive. Expected transactional mode for all three (none uses `CONCURRENTLY`).

Checksum note: adoption records the on-disk SHA-256 of the working tree that runs it. The run **must** happen from a checkout of `origin/main` at or after `5da8f8d2` (the tree that contains 038 and 039 and the ledger lines for them). Running from the `codex/selemene-capability-consumer-surface` checkout would fail validation because that branch has no 038/039 files while the ledger it carries ends at 037, and any later run from main would then see the journal missing those rows.

## 5. The biofield fix: migration 040

### 5.1 Why adoption alone is insufficient

After `--adopt-through 036` and the replay of 037 to 039, `biofield_sessions` is unchanged. `POST /api/v1/biofield/sessions` still executes `INSERT INTO biofield_sessions (user_id, status, client_device_id, viewer_version, notes)` and still fails on `notes`. `close_session` needs `closed_at` and status `'closed'`. The readiness probe (`/health/ready`) checks only connectivity, so it will keep reporting `postgres: ok` throughout.

### 5.2 Design constraints

- Must be safe on **both** shapes, because staging already has the 017 shape and will replay 040 too. Use `ADD COLUMN IF NOT EXISTS` and `DROP CONSTRAINT IF EXISTS` before every `ADD CONSTRAINT`.
- Must **not** drop or rename any production column: `biofield_session_summary`, `biofield_snapshot_detail` and `biofield_reading_history` views, and the `biofield_snapshots`/`biofield_artifacts`/`biofield_timeline_points`/`biofield_baselines` foreign keys depend on the table. Keep `ended_at` because `admin_repository.rs` selects `bs.ended_at AS closed_at`.
- `biofield_sessions` has 0 rows in production, so replacing the status CHECK cannot violate existing data. The new CHECK should be the 017 set plus the legacy values, so Supabase-era views and any future legacy row remain valid: `active, closed, abandoned, paused, completed, aborted`.
- The 017 `status_closed_at_check` ties `closed` and `abandoned` to a non-null `closed_at`; apply it as written. Do not add a check that references `ended_at`.

### 5.3 Proposed file (to be added in a separate code PR, not in this runbook PR)

`migrations/040_biofield_sessions_reconcile_legacy_shape.sql`:

```sql
-- Migration: 040_biofield_sessions_reconcile_legacy_shape
-- Description: Production biofield_sessions predates 017 (Supabase-era shape).
-- Add the 017 columns and constraints additively; never drop legacy columns
-- (views and FKs depend on them). Idempotent on a 017-shaped database.

ALTER TABLE biofield_sessions
    ADD COLUMN IF NOT EXISTS notes TEXT,
    ADD COLUMN IF NOT EXISTS closed_at TIMESTAMPTZ;

ALTER TABLE biofield_sessions DROP CONSTRAINT IF EXISTS biofield_sessions_status_check;
ALTER TABLE biofield_sessions ADD CONSTRAINT biofield_sessions_status_check CHECK (
    status IN ('active', 'closed', 'abandoned', 'paused', 'completed', 'aborted')
);

ALTER TABLE biofield_sessions DROP CONSTRAINT IF EXISTS biofield_sessions_status_closed_at_check;
ALTER TABLE biofield_sessions ADD CONSTRAINT biofield_sessions_status_closed_at_check CHECK (
    (status = 'active' AND closed_at IS NULL)
    OR (status IN ('closed', 'abandoned') AND closed_at IS NOT NULL)
    OR status IN ('paused', 'completed', 'aborted')
);

ALTER TABLE biofield_sessions DROP CONSTRAINT IF EXISTS biofield_sessions_closed_at_order_check;
ALTER TABLE biofield_sessions ADD CONSTRAINT biofield_sessions_closed_at_order_check CHECK (
    closed_at IS NULL OR closed_at >= started_at
);
```

Plus one line appended to `migrations/history.sha256` with the file's SHA-256 (the validator enforces this; `python3 scripts/validate_migrations.py` must print `next legal version is 041`). Staging must replay 040 first and the biofield probe set (section 7) must pass there before production.

`status` is `text` in production and `varchar(16)` in 017; the CHECK works on both and no type change is needed.

### 5.4 Follow-ups surfaced, not part of this runbook

- `admin_repository.rs` reads `bs.ended_at`, which does not exist on a 017-shaped database. The admin biofield session listing is therefore expected to fail in staging today. Decide whether admin should read `closed_at` (and 040 can then stop caring about `ended_at` in the long term).
- Migration 017 and 036 use bare `CREATE TABLE` in places; the ledger freezes them, so this is documentation, not a change request.

## 6. Backup and rollback

All commands run from a macOS shell with Homebrew PostgreSQL 18.6 client tools (`pg_dump 18.6` matches the server major; verified). Keep the dump **outside the repository**.

### 6.1 Logical backup (custom format)

```bash
BACKUP_DIR="$HOME/Backups/selemene-prod"
mkdir -p "$BACKUP_DIR"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
railway run --service Postgres --environment production -- sh -c \
  'pg_dump "$DATABASE_PUBLIC_URL" --format=custom --no-owner --no-privileges --verbose --file="'"$BACKUP_DIR"'/selemene-prod-'"$STAMP"'.dump"' \
  2> "$BACKUP_DIR/selemene-prod-$STAMP.pg_dump.log"
pg_restore --list "$BACKUP_DIR/selemene-prod-$STAMP.dump" | wc -l
shasum -a 256 "$BACKUP_DIR/selemene-prod-$STAMP.dump" | tee "$BACKUP_DIR/selemene-prod-$STAMP.sha256"
```

Size estimate: `pg_database_size` is 527 MB of which `readings` (JSONB) is 458 MB; custom-format dumps compress JSONB well, expect roughly 100 to 200 MB on disk and a few minutes over the public TCP proxy from this region. The listing step proves the archive is readable; the stop point requires a non-zero count and a log that ends without `error`.

Also take a Railway volume backup before any write, so the two mechanisms cover each other:

```bash
railway postgres pitr backup create --service Postgres --environment production --name pre-journal-adoption-2026-10-26
railway postgres pitr backup list --service Postgres --environment production
```

Volume backups are incremental copy-on-write snapshots of the whole data directory (Railway docs: Back Up and Restore Postgres). Restore from the Backups tab or `railway postgres pitr backup restore <ID>` stages a volume swap and **redeploys the Postgres service** on the restored data, which is a full-database rollback with brief downtime. PITR (continuous WAL archive) is not enabled on this service; enabling it is a separate operator decision and is not required for this change.

### 6.2 Restore commands

Preferred rollback for this change is a scoped SQL revert (section 6.3), because the migrations involved are small and additive. Full restore is the last resort.

Restore to a scratch database first (never straight onto production):

```bash
# Local verification restore
createdb selemene_prod_verify
pg_restore --no-owner --no-privileges --jobs=4 --dbname=selemene_prod_verify "$BACKUP_DIR/selemene-prod-$STAMP.dump"
psql selemene_prod_verify -Atc "select count(*) from users; select count(*) from readings;"
```

Full production restore (destructive, operator-only, requires the API to be scaled to zero first):

```bash
railway run --service Postgres --environment production -- sh -c \
  'pg_restore --clean --if-exists --no-owner --no-privileges --jobs=4 --dbname="$DATABASE_PUBLIC_URL" "'"$BACKUP_DIR"'/selemene-prod-'"$STAMP"'.dump"'
```

`--clean --if-exists` drops and recreates every object in the archive; objects created after the dump (for example the journal) are not touched by it unless they are in the archive, so follow a full restore with a manual `DROP TABLE IF EXISTS noesis_migration_journal, billing_mode_control, archived_reading_invitations` if the aim is a byte-for-byte return to the pre-change state.

### 6.3 Scoped SQL revert (preferred)

Everything this runbook writes is reversible without a restore:

```sql
BEGIN;
DROP TABLE IF EXISTS archived_reading_invitations;      -- 037
DROP TABLE IF EXISTS billing_mode_control;              -- 038
ALTER TABLE raga_clips DROP CONSTRAINT IF EXISTS raga_clips_style_check;
ALTER TABLE raga_clips ADD CONSTRAINT raga_clips_style_check
    CHECK (style IN ('ambient', 'meditative', 'cinematic', 'acid'));  -- 039
-- 040 (only if applied): columns were absent before, 0 rows, so dropping is lossless
ALTER TABLE biofield_sessions DROP CONSTRAINT IF EXISTS biofield_sessions_status_closed_at_check;
ALTER TABLE biofield_sessions DROP CONSTRAINT IF EXISTS biofield_sessions_closed_at_order_check;
ALTER TABLE biofield_sessions DROP CONSTRAINT IF EXISTS biofield_sessions_status_check;
ALTER TABLE biofield_sessions ADD CONSTRAINT biofield_sessions_status_check
    CHECK (status = ANY (ARRAY['active','paused','completed','aborted']));
ALTER TABLE biofield_sessions DROP COLUMN IF EXISTS notes, DROP COLUMN IF EXISTS closed_at;
DROP TABLE IF EXISTS noesis_migration_journal;
COMMIT;
```

Dropping the journal returns the database to the exact pre-journal state the script recognises, so adoption can be re-attempted.

## 7. Verification plan after apply

### 7.1 Platform

```bash
curl -s https://selemene.tryambakam.space/health/ready | jq '{postgres, redis, orchestrator}'
curl -s https://selemene.tryambakam.space/health/dependencies | jq '{status, sidecars}'   # expect biofield-cv and mediapipe-face-mesh "ready" (true on 2026-10-26)
railway run --service Postgres --environment production -- sh -c 'psql "$DATABASE_PUBLIC_URL" -X -Atc "select count(*), max(filename) from noesis_migration_journal; select count(*) from noesis_migration_journal where state <> '"'"'applied'"'"';"'
```

Expected: `postgres: ok`; journal row count 38 once 037 to 039 are in (35 adopted files through 036, counting both `007` files, plus 3 applied) and 39 once 040 is in; `max(filename)` equal to the last file on disk; zero non-`applied` rows.

A plain re-run of `scripts/apply-migrations.sh` from the same checkout must end with `applied=0 skipped=<N>`.

### 7.2 Mint a scoped production key (the lane s1/s4 pattern)

The existing receipts (`docs/plans/somatic-ralph-state.json` on `origin/ralph/s0-plan`, lanes s1 and s4) minted short-lived keys directly in Postgres via `railway run`, used them for the probe, then revoked them. Auth header is `X-API-Key`; the stored hash is the SHA-256 hex of the plaintext `nk_...` key; `is_active`, `expires_at` and the tier CHECK (`free|basic|premium|enterprise`) apply. The `/biofield/*` routes sit behind `auth_middleware` only and check no per-engine permission, so `["basic:access"]` (the lane s1 pattern) is enough.

```bash
RAW_KEY="nk_$(openssl rand -hex 24)"
export OPERATOR_EMAIL="<operator account email>"
export KEY_HASH="$(printf '%s' "$RAW_KEY" | shasum -a 256 | awk '{print $1}')"
# SQL goes through stdin (not -c) so psql interpolates :'h' and :'e'.
cat <<'SQL' | railway run --service Postgres --environment production -- \
  sh -c 'psql "$DATABASE_PUBLIC_URL" -X -v ON_ERROR_STOP=1 -v h="$KEY_HASH" -v e="$OPERATOR_EMAIL"'
INSERT INTO api_keys (key_hash, user_id, tier, permissions, consciousness_level, rate_limit, is_active, name, key_prefix, expires_at)
SELECT :'h', id, 'premium', '["basic:access"]'::jsonb, 1, 120, true, 'ops-prod-migration-probe', left(:'h', 12), now() + interval '2 hours'
FROM users WHERE email = :'e';
SELECT name, is_active, expires_at FROM api_keys WHERE key_hash = :'h';
SQL
```

`railway run` passes the shell environment through, so `-v h="$KEY_HASH"` resolves inside the child shell). Keep `RAW_KEY` only in the shell; do not write it to disk or to any receipt. `OPERATOR_EMAIL` is the operator's own account (the probe session and reading will be owned by it and can be deleted afterwards).

### 7.3 Biofield probe set (the set that passed in staging on 2026-10-26)

```bash
BASE=https://selemene.tryambakam.space/api/v1
# 1. create session -> expect 201 with id, status "active"
SESSION_ID=$(curl -s -X POST "$BASE/biofield/sessions" -H "X-API-Key: $RAW_KEY" -H 'content-type: application/json' \
  -d '{"client_device_id":"ops-probe","viewer_version":"runbook-2026-10-26"}' | tee /dev/stderr | jq -r .id)
# 2. capture with consent -> expect 201, reading id, quality_assessment.sufficient_quality true, source-image artifact
curl -s -X POST "$BASE/biofield/sessions/$SESSION_ID/captures" -H "X-API-Key: $RAW_KEY" \
  -F image=@tests/fixtures/images/biofield-portrait-256.jpg \
  -F 'capture_metadata={"consent":{"granted":true,"scopes":["biofield-capture"]}};type=application/json' | tee /tmp/capture.json | jq '{reading_id, provenance, quality_assessment}'
READING_ID=$(jq -r '.reading_id // .id' /tmp/capture.json)
# 3. capture without consent -> expect 422 CONSENT_REQUIRED, details.required_scope "biofield-capture"
curl -s -o /dev/null -w '%{http_code}\n' -X POST "$BASE/biofield/sessions/$SESSION_ID/captures" -H "X-API-Key: $RAW_KEY" \
  -F image=@tests/fixtures/images/biofield-portrait-256.jpg
# 4. capture with wrong scope -> expect 422 CONSENT_REQUIRED
curl -s -X POST "$BASE/biofield/sessions/$SESSION_ID/captures" -H "X-API-Key: $RAW_KEY" \
  -F image=@tests/fixtures/images/biofield-portrait-256.jpg \
  -F 'capture_metadata={"consent":{"granted":true,"scopes":["face-image"]}};type=application/json' | jq '{error_code, details}'
# 5. read the reading back -> expect 200 with metrics
curl -s "$BASE/biofield/readings/$READING_ID" -H "X-API-Key: $RAW_KEY" | jq '{id, engine_id, metrics: (.result.metrics // .metrics) | keys}'
# 6. list readings -> expect the reading present
curl -s "$BASE/biofield/readings" -H "X-API-Key: $RAW_KEY" | jq 'length // .total'
# 7. close session -> expect 200, status "closed", closed_at set (this exercises the 040 columns)
curl -s -X POST "$BASE/biofield/sessions/$SESSION_ID/close" -H "X-API-Key: $RAW_KEY" | jq '{status, closed_at}'
# 8. engine path with the captured reading -> expect provenance.backend "biofield-capture", fallback_used false
curl -s -X POST "$BASE/engines/biofield-capture/calculate" -H "X-API-Key: $RAW_KEY" -H 'content-type: application/json' \
  -d "{\"options\":{\"biofield_capture\":{\"reading_id\":\"$READING_ID\"}},\"consent\":{\"granted\":true,\"scopes\":[\"biofield-capture\"]}}" | jq '.result.provenance // .provenance'
```

Fixture: `tests/fixtures/images/biofield-portrait-256.jpg` exists on `origin/main` (not on the `codex/selemene-capability-consumer-surface` branch); run from the main checkout. Staging on 2026-10-26 returned for steps 1, 2, 3, 4, 5: `201`, `201` with `sufficient_quality true` and a `source-image` artifact, `422 CONSENT_REQUIRED` twice, `200` with metrics. Production must match. If step 2 returns a sidecar error, check `PYTHON_BIOFIELD_URL` and `/health/dependencies` before suspecting the database.

### 7.4 Revoke and clean up

```bash
cat <<'SQL' | railway run --service Postgres --environment production -- \
  sh -c 'psql "$DATABASE_PUBLIC_URL" -X -v ON_ERROR_STOP=1 -v h="$KEY_HASH"'
UPDATE api_keys
SET is_active = false, revoked_at = now(), revoked_reason = 'runbook probe complete'
WHERE key_hash = :'h';
SELECT is_active FROM api_keys WHERE key_hash = :'h';
SQL
curl -s -o /dev/null -w '%{http_code}\n' "$BASE/biofield/readings" -H "X-API-Key: $RAW_KEY"   # expect 401
unset RAW_KEY KEY_HASH
```

Probe rows left behind: one `biofield_sessions` row, one `biofield_capture_artifacts` row, one `readings` row (engine `biofield-capture`) and the stored source image. Delete them only if the operator wants a clean table; they are owned by the operator account.

## 8. Order of operations with stop points

Preconditions: operator approval of this document; a checkout of `origin/main` at or after `5da8f8d2`; a merged code PR that adds `040_biofield_sessions_reconcile_legacy_shape.sql` plus its ledger line, replayed and probed green in staging. Do production during a low-traffic window; the API stays up throughout (all statements are short DDL on empty or small tables).

| Step | Action | Stop point (do not continue unless) |
|---|---|---|
| 0 | Re-run the read-only diagnosis queries from section 2 against production | results identical to this document: no journal, 037 to 039 absent, `biofield_sessions` row count still 0 |
| 1 | `railway postgres pitr backup create ... --name pre-journal-adoption-<date>` | backup listed with a size |
| 2 | `pg_dump` custom format to `$BACKUP_DIR` (section 6.1), `pg_restore --list`, checksum | listing non-empty, log clean, sha256 recorded |
| 3 | Optional: local `pg_restore` of the dump into `selemene_prod_verify`, count users and readings | counts 35 and 300 (or current) |
| 4 | From the main checkout: `python3 scripts/validate_migrations.py` | prints `next legal version is 041` (040 ledgered) |
| 5 | Dry read: `bash scripts/apply-migrations.sh --adopt-through 036 -- "$DATABASE_PUBLIC_URL"` inside `railway run --service Postgres --environment production` | **first real write.** Output shows `Adopted already-applied migration:` for 001 through 036 (35 lines, two 007s), then `Applying migration: 037...`, `038...`, `039...`, `040...`, ending `applied=4 skipped=0`. Any error: the adoption transaction and each migration transaction roll back independently; capture the output and go to section 6.3 |
| 6 | Journal check (section 7.1) and a second plain `apply-migrations.sh` run | `applied=0 skipped=39`, zero non-applied rows |
| 7 | `/health/ready`, `/health/dependencies` | postgres ok, biofield sidecar ready |
| 8 | Mint probe key (7.2), run probe set (7.3) | steps 1, 2, 5, 7, 8 return 2xx with the documented fields; 3 and 4 return 422 |
| 9 | Revoke key (7.4) | subsequent call is 401 |
| 10 | Record receipts: journal row count, probe status codes, backup id and dump checksum, in `docs/plans/somatic-ralph-state.json` lane s2 and in the memory note | done |

If step 5 fails after the adoption transaction committed but before 037 applied, the journal exists with 35 rows and the script's plain mode will simply resume from 037 on the next run; no revert needed. If 040 fails (for example because the staging rehearsal was skipped), 037 to 039 remain applied and journaled, which is a consistent state; fix 040 forward rather than reverting.

## 9. Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Adoption records checksums from the wrong tree | low if run from main | next run aborts on checksum mismatch; no data impact | step 4 validator; run from `origin/main` only |
| 040 drops or alters a column a Supabase-era view depends on | none as drafted | view breaks, API unaffected | 040 is additive only; reviewer checks for `DROP COLUMN` / `RENAME` |
| Status CHECK replacement rejects existing rows | none | ALTER fails, transaction rolls back | `biofield_sessions` has 0 rows (re-check at step 0) |
| `raga_clips_style_check` swap rejects existing rows | none | ALTER fails, rolls back | `raga_clips` has 0 rows |
| Public TCP proxy drops the `pg_dump` connection | low | incomplete dump | `pg_restore --list` verifies; retry |
| No PITR, so a mistake between backup and apply is not recoverable to the second | real, pre-existing | would lose writes made during the window | low-traffic window; the window between step 2 and step 5 is minutes; enable PITR as a separate improvement |
| Double `updated_at` triggers on `biofield_sessions` | certain, harmless | both set the same value | leave; document |
| Admin biofield listing uses `ended_at` | certain | admin path works in production only because `ended_at` survives; staging admin path is already broken | 040 keeps `ended_at`; separate code fix |
| Operator runs a plain `apply-migrations.sh` first | n/a | script refuses (nonempty schema, no journal); no writes | by design |

Migrations whose replay would alter or drop existing production data: **none in the replay set** (037, 038, 039, 040). The historical files that would be destructive if they were ever replayed, and are therefore the reason adoption must cover them, are `008` (UPDATE users), `010` (INSERT user_roles / user_account_state backfill), `011` (INSERT api_key_events backfill), `013` (UPDATE readings sync_cursor), `014` (INSERT plan_catalog / billing_subscriptions backfill), `026` (UPDATE users/api_keys tier), `032` (DROP TABLE oauth_accounts). Adopting through 036 records all of them as applied and guarantees they never run again.

Decision required from the operator: approve `--adopt-through 036`, approve the 040 design in section 5.3, and approve the production write window. Until all three are given, production stays untouched.
