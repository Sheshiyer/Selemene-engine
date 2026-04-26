#!/usr/bin/env bash
set -euo pipefail

if ! command -v psql >/dev/null 2>&1; then
  echo "ALERT: psql is required to check usage_logs partitions" >&2
  exit 1
fi

if [[ -z "${DATABASE_URL:-}" ]]; then
  echo "ALERT: DATABASE_URL is required to check usage_logs partitions" >&2
  exit 1
fi

MONTHS_AHEAD="${MONTHS_AHEAD:-3}"

echo "Checking usage_logs partitions ${MONTHS_AHEAD} month(s) ahead..."

if ! psql "${DATABASE_URL}" -v ON_ERROR_STOP=1 -P pager=off -c \
  "SELECT partition_name, partition_start, partition_end, created FROM ensure_usage_log_partitions(${MONTHS_AHEAD}::INTEGER);"; then
  echo "ALERT: usage_logs partition maintenance failed" >&2
  exit 1
fi

echo "usage_logs partition maintenance check succeeded."
