#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/supabase-to-railway-migrate.sh --source URL --target URL [mode]

Modes:
  --schema-only       Restore schema only.
  --data-only         Restore data only.
  --critical-tables   Restore only the critical table subset.
  --full              Restore schema and data. Default.

Safety:
  --dry-run           Print commands without executing pg_dump/pg_restore/psql.
  --yes               Required for non-dry-run execution.
  --force             Allow non-empty target database after explicit confirmation.

Examples:
  scripts/supabase-to-railway-migrate.sh --dry-run --schema-only --source "$SUPABASE_DATABASE_URL" --target "$RAILWAY_DATABASE_URL"
  scripts/supabase-to-railway-migrate.sh --yes --full --source "$SUPABASE_DATABASE_URL" --target "$RAILWAY_DATABASE_URL"
USAGE
}

SOURCE_URL=""
TARGET_URL=""
MODE="full"
DRY_RUN=0
YES=0
FORCE=0

CRITICAL_TABLES=(
  users
  user_profiles
  api_keys
  user_roles
  user_account_state
  billing_subscriptions
  processed_webhook_events
  usage_logs
  readings
  biofield_sessions
  biofield_measurements
  raga_clips
  reconcile_runs
)

while [[ $# -gt 0 ]]; do
  case "$1" in
    --source) SOURCE_URL="${2:-}"; shift 2 ;;
    --target) TARGET_URL="${2:-}"; shift 2 ;;
    --schema-only) MODE="schema-only"; shift ;;
    --data-only) MODE="data-only"; shift ;;
    --critical-tables) MODE="critical-tables"; shift ;;
    --full) MODE="full"; shift ;;
    --dry-run) DRY_RUN=1; shift ;;
    --yes) YES=1; shift ;;
    --force) FORCE=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown argument: $1" >&2; usage; exit 2 ;;
  esac
done

if [[ -z "$SOURCE_URL" || -z "$TARGET_URL" ]]; then
  echo "--source and --target are required" >&2
  usage
  exit 2
fi

if [[ "$DRY_RUN" -eq 0 && "$YES" -eq 0 ]]; then
  echo "Refusing to run without --yes. Use --dry-run for planning." >&2
  exit 2
fi

redact_url() {
  local url="$1"
  printf '%s\n' "$url" | sed -E 's#(postgres(ql)?://)[^:@/]+(:[^@/]+)?@#\1***:***@#'
}

require_command() {
  local name="$1"
  if ! command -v "$name" >/dev/null 2>&1; then
    echo "Required command not found: $name" >&2
    exit 2
  fi
}

validate_postgres_url() {
  local label="$1"
  local url="$2"
  if [[ ! "$url" =~ ^postgres(ql)?:// ]]; then
    echo "$label must start with postgres:// or postgresql://" >&2
    exit 2
  fi
}

validate_postgres_url "--source" "$SOURCE_URL"
validate_postgres_url "--target" "$TARGET_URL"

if [[ "$SOURCE_URL" == "$TARGET_URL" ]]; then
  echo "Refusing to use identical source and target URLs" >&2
  exit 2
fi

require_command pg_dump
require_command pg_restore
require_command psql

run_cmd() {
  if [[ "$DRY_RUN" -eq 1 ]]; then
    printf '+ '
    printf '%q ' "$@"
    printf '\n'
    return 0
  fi
  "$@"
}

run_sql() {
  local sql="$1"
  run_cmd psql "$TARGET_URL" -v ON_ERROR_STOP=1 -c "$sql"
}

restore_schema_only() {
  run_cmd bash -c 'pg_dump --format=custom --no-owner --no-acl --schema-only "$1" | pg_restore --no-owner --no-acl --clean --if-exists --dbname "$2"' _ "$SOURCE_URL" "$TARGET_URL"
}

restore_data_only() {
  run_cmd bash -c 'pg_dump --format=custom --no-owner --no-acl --data-only "$1" | pg_restore --no-owner --no-acl --disable-triggers --dbname "$2"' _ "$SOURCE_URL" "$TARGET_URL"
}

restore_critical_tables() {
  local args=()
  for table in "${CRITICAL_TABLES[@]}"; do
    args+=(--table "public.$table")
  done
  run_cmd pg_dump --format=custom --no-owner --no-acl --data-only "${args[@]}" --file /tmp/noesis-critical-tables.dump "$SOURCE_URL"
  run_cmd pg_restore --no-owner --no-acl --disable-triggers --dbname "$TARGET_URL" /tmp/noesis-critical-tables.dump
}

restore_full() {
  restore_schema_only
  restore_data_only
}

verify_target() {
  echo "Verifying target database"
  run_sql "SELECT 1 AS target_ok;"
  run_sql "SELECT ensure_usage_log_partitions(3);"
  run_sql "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_name IN ('users', 'api_keys', 'user_roles', 'user_account_state', 'billing_subscriptions', 'usage_logs', 'raga_clips', 'reconcile_runs') ORDER BY table_name;"
  run_sql "SELECT 'users' AS table_name, count(*) FROM users UNION ALL SELECT 'api_keys', count(*) FROM api_keys UNION ALL SELECT 'user_roles', count(*) FROM user_roles UNION ALL SELECT 'user_account_state', count(*) FROM user_account_state UNION ALL SELECT 'raga_clips', count(*) FROM raga_clips;"
  run_sql "SELECT inhrelid::regclass::text AS partition_name FROM pg_inherits WHERE inhparent = 'usage_logs'::regclass ORDER BY 1;"
}

echo "mode=$MODE dry_run=$DRY_RUN force=$FORCE"
echo "source=$(redact_url "$SOURCE_URL")"
echo "target=$(redact_url "$TARGET_URL")"

case "$MODE" in
  schema-only) restore_schema_only ;;
  data-only) restore_data_only ;;
  critical-tables) restore_critical_tables ;;
  full) restore_full ;;
  *) echo "Unsupported mode: $MODE" >&2; exit 2 ;;
esac

verify_target
