#!/usr/bin/env bash
set -euo pipefail

BIOFIELD_WEB_URL="${BIOFIELD_WEB_URL:-}"
API_BASE_URL="${API_BASE_URL:-}"
PYTHON_BIOFIELD_URL="${PYTHON_BIOFIELD_URL:-}"

fail() {
  echo "❌ $1" >&2
  exit 1
}

check_status() {
  local url="$1"
  local expected="$2"
  local method="${3:-GET}"
  local status
  status="$(curl -sS -X "$method" -o /dev/null -w "%{http_code}" "$url")"
  if [[ "$status" != "$expected" ]]; then
    fail "Expected $method $url to return $expected, got $status"
  fi
  echo "✅ $method $url -> $status"
}

if [[ -z "$BIOFIELD_WEB_URL" || -z "$API_BASE_URL" || -z "$PYTHON_BIOFIELD_URL" ]]; then
  cat <<'USAGE'
Usage:
  BIOFIELD_WEB_URL=http://127.0.0.1:3002 \
  API_BASE_URL=http://127.0.0.1:8080 \
  PYTHON_BIOFIELD_URL=http://127.0.0.1:8002 \
  bash scripts/smoke_biofield_web.sh
USAGE
  fail "BIOFIELD_WEB_URL, API_BASE_URL, and PYTHON_BIOFIELD_URL are required"
fi

echo "Running biofield-web smoke checks"
echo "BIOFIELD_WEB_URL=$BIOFIELD_WEB_URL"
echo "API_BASE_URL=$API_BASE_URL"
echo "PYTHON_BIOFIELD_URL=$PYTHON_BIOFIELD_URL"

check_status "$BIOFIELD_WEB_URL/login" "200"
check_status "$BIOFIELD_WEB_URL/viewer" "200"
check_status "$BIOFIELD_WEB_URL/history" "200"
check_status "$API_BASE_URL/health/live" "200"
check_status "$PYTHON_BIOFIELD_URL/health" "200"

echo "🎉 Biofield-web smoke checks passed"
