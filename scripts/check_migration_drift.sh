#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROOT_MIG_DIR="$ROOT_DIR/migrations"
SUPABASE_MIG_DIR="$ROOT_DIR/supabase/migrations"

if [[ ! -d "$ROOT_MIG_DIR" ]]; then
  echo "error: missing root migrations directory: $ROOT_MIG_DIR" >&2
  exit 1
fi

if [[ ! -d "$SUPABASE_MIG_DIR" ]]; then
  echo "error: missing supabase migrations directory: $SUPABASE_MIG_DIR" >&2
  exit 1
fi

mapfile -t ROOT_FILES < <(find "$ROOT_MIG_DIR" -maxdepth 1 -type f -name '[0-9][0-9][0-9]_*.sql' -exec basename {} \; | sort)
mapfile -t SUPABASE_FILES < <(find "$SUPABASE_MIG_DIR" -maxdepth 1 -type f -name '*.sql' -exec basename {} \; | sort)

if [[ ${#ROOT_FILES[@]} -eq 0 ]]; then
  echo "error: no root migrations found" >&2
  exit 1
fi

missing=()
multiple=()
mismatch=()
extra=()

# Root -> Supabase parity check.
for root_file in "${ROOT_FILES[@]}"; do
  mapfile -t matches < <(find "$SUPABASE_MIG_DIR" -maxdepth 1 -type f -name "*_${root_file}" -exec basename {} \; | sort)

  if [[ ${#matches[@]} -eq 0 ]]; then
    missing+=("$root_file")
    continue
  fi

  if [[ ${#matches[@]} -gt 1 ]]; then
    multiple+=("$root_file => ${matches[*]}")
    continue
  fi

  supabase_file="${matches[0]}"
  root_hash="$(shasum -a 256 "$ROOT_MIG_DIR/$root_file" | awk '{print $1}')"
  supabase_hash="$(shasum -a 256 "$SUPABASE_MIG_DIR/$supabase_file" | awk '{print $1}')"

  if [[ "$root_hash" != "$supabase_hash" ]]; then
    mismatch+=("$root_file != $supabase_file")
  fi
done

# Supabase files that do not map to root file names are drift.
for supabase_file in "${SUPABASE_FILES[@]}"; do
  if [[ "$supabase_file" != *_*.sql ]]; then
    extra+=("$supabase_file")
    continue
  fi

  suffix="${supabase_file#*_}"
  if [[ ! -f "$ROOT_MIG_DIR/$suffix" ]]; then
    extra+=("$supabase_file")
  fi
done

has_error=0

if [[ ${#missing[@]} -gt 0 ]]; then
  has_error=1
  echo "Missing canonical mirrors in supabase/migrations:" >&2
  printf '  - %s\n' "${missing[@]}" >&2
fi

if [[ ${#multiple[@]} -gt 0 ]]; then
  has_error=1
  echo "Multiple supabase mirrors found for one root migration:" >&2
  printf '  - %s\n' "${multiple[@]}" >&2
fi

if [[ ${#mismatch[@]} -gt 0 ]]; then
  has_error=1
  echo "Content mismatches between root and supabase mirrors:" >&2
  printf '  - %s\n' "${mismatch[@]}" >&2
fi

if [[ ${#extra[@]} -gt 0 ]]; then
  has_error=1
  echo "Unexpected supabase migrations not mapped to root canonical set:" >&2
  printf '  - %s\n' "${extra[@]}" >&2
fi

if [[ $has_error -ne 0 ]]; then
  exit 1
fi

echo "Migration drift check passed: root and supabase migration sets are aligned."
