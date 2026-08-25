#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
Usage: apply-migrations.sh [--migrations-dir DIR] [--adopt-through NNN] [--] [psql arguments...]

--adopt-through is an explicit one-time operation for a pre-journal, nonempty
database whose historical migrations were already applied. It validates the
repository ledger, then records exact filename/checksum pairs through NNN
without replaying their SQL.
EOF
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
migrations_dir="migrations"
adopt_through=""
while (($# > 0)); do
  case "$1" in
    --migrations-dir)
      if (($# < 2)); then usage; exit 2; fi
      migrations_dir="$2"
      shift 2
      ;;
    --adopt-through)
      if (($# < 2)); then usage; exit 2; fi
      adopt_through="$2"
      shift 2
      ;;
    --)
      shift
      break
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      break
      ;;
  esac
done

if [[ ! -d "$migrations_dir" ]]; then
  echo "Migration directory does not exist: $migrations_dir" >&2
  exit 2
fi
if [[ -n "$adopt_through" && ! "$adopt_through" =~ ^[0-9]{3}$ ]]; then
  echo "--adopt-through must be a three-digit migration version" >&2
  exit 2
fi

export LC_ALL=C
shopt -s nullglob
migrations=("$migrations_dir"/*.sql)
if ((${#migrations[@]} == 0)); then
  echo "No SQL migrations found in: $migrations_dir" >&2
  exit 2
fi

# Validate the immutable ledger before opening any database connection. This
# makes the append protocol explicit: a pending 038+ file must first be added
# to history.sha256 with its checksum and continuous legal version.
history_file="$migrations_dir/history.sha256"
python3 "$script_dir/validate_migrations.py" \
  --migrations-dir "$migrations_dir" \
  --history-file "$history_file" >/dev/null

psql_base=(psql -X -v ON_ERROR_STOP=1 "$@")
journal_lock_key="684022132617857143"

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

psql_input() {
  local operation=$1
  shift
  "${psql_base[@]}" -v "gate_operation=$operation" "$@"
}

probe="$({
  cat <<'SQL'
SELECT
  CASE
    WHEN to_regclass('public.noesis_migration_journal') IS NOT NULL THEN 't'
    ELSE 'f'
  END
  || '|'
  || CASE
    WHEN EXISTS (
      SELECT 1
      FROM pg_class AS relation
      JOIN pg_namespace AS namespace ON namespace.oid = relation.relnamespace
      WHERE namespace.nspname NOT IN ('pg_catalog', 'information_schema')
        AND namespace.nspname !~ '^pg_toast'
        AND relation.relkind IN ('r', 'p', 'v', 'm', 'S', 'f')
        AND NOT (
          namespace.nspname = 'public'
          AND relation.relname = 'noesis_migration_journal'
        )
    ) THEN 't'
    ELSE 'f'
  END;
SQL
} | psql_input probe --tuples-only --no-align)"

normalize_boolean() {
  case "$1" in
    t|T|true|TRUE|True) printf 't\n' ;;
    f|F|false|FALSE|False) printf 'f\n' ;;
    *) return 1 ;;
  esac
}

if ! journal_exists="$(normalize_boolean "${probe%%|*}")" \
  || ! schema_nonempty="$(normalize_boolean "${probe#*|}")"; then
  echo "Could not determine migration journal state: $probe" >&2
  exit 1
fi
if [[ "$journal_exists" != "t" && "$journal_exists" != "f" ]]; then
  echo "Could not determine migration journal state: $probe" >&2
  exit 1
fi

write_journal_ddl() {
  cat <<'SQL'
CREATE TABLE IF NOT EXISTS public.noesis_migration_journal (
  filename text PRIMARY KEY,
  checksum text NOT NULL CHECK (checksum ~ '^[0-9a-f]{64}$'),
  state text NOT NULL CHECK (state IN ('applying', 'applied')),
  applied_at timestamptz,
  CHECK ((state = 'applying' AND applied_at IS NULL) OR (state = 'applied' AND applied_at IS NOT NULL))
);
COMMENT ON TABLE public.noesis_migration_journal IS
  'Repository-owned exact filename/checksum migration journal; applying rows require operator reconciliation.';
SQL
}

initialize_journal() {
  write_journal_ddl | psql_input init
}

if [[ "$journal_exists" == "f" && "$schema_nonempty" == "t" && -z "$adopt_through" ]]; then
  cat >&2 <<'EOF'
Refusing to replay migrations over a nonempty schema with no migration journal.
Audit the database, then explicitly adopt verified history with --adopt-through NNN.
EOF
  exit 1
fi
if [[ "$journal_exists" == "t" && -n "$adopt_through" ]]; then
  echo "--adopt-through is only valid when the migration journal does not yet exist" >&2
  exit 1
fi
if [[ "$journal_exists" == "f" && "$schema_nonempty" == "f" && -n "$adopt_through" ]]; then
  echo "Refusing history adoption on an empty database; run normally to apply migrations" >&2
  exit 1
fi

if [[ "$journal_exists" == "t" ]]; then
  journal_count="$({
    cat <<'SQL'
SELECT count(*)
FROM public.noesis_migration_journal;
SQL
  } | psql_input journal_count --tuples-only --no-align)"
  if [[ ! "$journal_count" =~ ^[0-9]+$ ]]; then
    echo "Could not determine migration journal row count: $journal_count" >&2
    exit 1
  fi
  if [[ "$journal_count" == "0" && "$schema_nonempty" == "t" ]]; then
    cat >&2 <<'EOF'
Refusing to replay migrations over a nonempty schema with an empty migration journal.
Audit the database and empty journal, then remove the journal and use --adopt-through NNN under an operator-approved procedure.
EOF
    exit 1
  fi
fi

if [[ "$journal_exists" == "f" && -z "$adopt_through" ]]; then
  initialize_journal
fi

if [[ -n "$adopt_through" ]]; then
  echo "Validated canonical migration history before adoption through $adopt_through."
  adopted_names=()
  adopted_checksums=()
  for migration in "${migrations[@]}"; do
    migration_name="$(basename "$migration")"
    migration_version="${migration_name%%_*}"
    if ((10#$migration_version > 10#$adopt_through)); then
      continue
    fi
    adopted_names+=("$migration_name")
    adopted_checksums+=("$(sha256_file "$migration")")
  done
  {
    printf 'BEGIN;\nSELECT pg_advisory_xact_lock(%s);\n' "$journal_lock_key"
    write_journal_ddl
    for index in "${!adopted_names[@]}"; do
      printf "\\set migration_filename '%s'\n" "${adopted_names[$index]}"
      printf "\\set migration_checksum '%s'\n" "${adopted_checksums[$index]}"
      cat <<'SQL'
INSERT INTO public.noesis_migration_journal (filename, checksum, state, applied_at)
VALUES (:'migration_filename', :'migration_checksum', 'applied', clock_timestamp());
SQL
    done
    printf 'COMMIT;\n'
  } | psql_input adopt
  for migration_name in "${adopted_names[@]}"; do
    echo "Adopted already-applied migration: $migration_name"
  done
fi

dirty_rows="$({
  cat <<'SQL'
SELECT filename || '|' || checksum
FROM public.noesis_migration_journal
WHERE state = 'applying'
ORDER BY filename;
SQL
} | psql_input dirty --tuples-only --no-align)"
if [[ -n "$dirty_rows" ]]; then
  cat >&2 <<EOF
Refusing to continue: dirty migration journal row(s) remain in state 'applying':
$dirty_rows
Reconcile the database manually, then update or remove the dirty row under an audited operator procedure.
EOF
  exit 1
fi

applied_count=0
skipped_count=0
for migration in "${migrations[@]}"; do
  migration_name="$(basename "$migration")"
  migration_checksum="$(sha256_file "$migration")"
  existing="$({
    cat <<'SQL'
SELECT checksum || '|' || state
FROM public.noesis_migration_journal
WHERE filename = :'migration_filename';
SQL
  } | psql_input lookup \
    -v "migration_filename=$migration_name" \
    --tuples-only --no-align)"

  if [[ -n "$existing" ]]; then
    existing_checksum="${existing%%|*}"
    existing_state="${existing#*|}"
    if [[ "$existing_state" != "applied" ]]; then
      echo "Refusing migration $migration_name in unexpected journal state: $existing_state" >&2
      exit 1
    fi
    if [[ "$existing_checksum" != "$migration_checksum" ]]; then
      echo "Migration checksum mismatch for $migration_name: journal=$existing_checksum disk=$migration_checksum" >&2
      exit 1
    fi
    echo "Skipping already-applied migration: $migration_name"
    ((skipped_count += 1))
    continue
  fi

  if sed 's/--.*$//' "$migration" | tr '\n' ' ' | grep -Eiq \
      'CREATE[[:space:]]+(UNIQUE[[:space:]]+)?INDEX[[:space:]]+CONCURRENTLY'; then
    migration_mode="nontransactional"
    echo "Applying nontransactional migration with dirty-state journal protection: $migration_name"
    {
      cat <<SQL
SELECT pg_advisory_lock($journal_lock_key);
INSERT INTO public.noesis_migration_journal (filename, checksum, state, applied_at)
VALUES (:'migration_filename', :'migration_checksum', 'applying', NULL);
SQL
      cat "$migration"
      cat <<SQL
UPDATE public.noesis_migration_journal
SET state = 'applied', applied_at = clock_timestamp()
WHERE filename = :'migration_filename' AND checksum = :'migration_checksum' AND state = 'applying';
SELECT pg_advisory_unlock($journal_lock_key);
SQL
    } | psql_input apply \
      -v "migration_filename=$migration_name" \
      -v "migration_checksum=$migration_checksum" \
      -v "migration_mode=$migration_mode"
  else
    migration_mode="transactional"
    echo "Applying migration: $migration_name"
    {
      cat <<SQL
BEGIN;
SELECT pg_advisory_xact_lock($journal_lock_key);
SQL
      cat "$migration"
      cat <<'SQL'
INSERT INTO public.noesis_migration_journal (filename, checksum, state, applied_at)
VALUES (:'migration_filename', :'migration_checksum', 'applied', clock_timestamp());
COMMIT;
SQL
    } | psql_input apply \
      -v "migration_filename=$migration_name" \
      -v "migration_checksum=$migration_checksum" \
      -v "migration_mode=$migration_mode"
  fi
  ((applied_count += 1))
done

echo "Migration run complete: applied=$applied_count skipped=$skipped_count."
