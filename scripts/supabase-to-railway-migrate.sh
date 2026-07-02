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

echo "mode=$MODE dry_run=$DRY_RUN force=$FORCE"
echo "source=$(redact_url "$SOURCE_URL")"
echo "target=$(redact_url "$TARGET_URL")"
