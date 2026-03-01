#!/usr/bin/env bash
set -euo pipefail

ADMIN_WEB_URL="${ADMIN_WEB_URL:-}"
API_BASE_URL="${API_BASE_URL:-}"

fail() {
  echo "❌ $1" >&2
  exit 1
}

check_status() {
  local url="$1"
  local expected="$2"
  local status
  status="$(curl -sS -o /dev/null -w "%{http_code}" "$url")"
  if [[ "$status" != "$expected" ]]; then
    fail "Expected $url to return $expected, got $status"
  fi
  echo "✅ $url -> $status"
}

if [[ -z "$ADMIN_WEB_URL" || -z "$API_BASE_URL" ]]; then
  cat <<'USAGE'
Usage:
  ADMIN_WEB_URL=https://<admin-web-domain> \
  API_BASE_URL=https://<api-domain> \
  bash scripts/smoke_admin_web.sh
USAGE
  fail "ADMIN_WEB_URL and API_BASE_URL are required"
fi

echo "Running admin-web smoke checks"
echo "ADMIN_WEB_URL=$ADMIN_WEB_URL"
echo "API_BASE_URL=$API_BASE_URL"

# Admin frontend should be reachable on /admin/login
check_status "$ADMIN_WEB_URL/admin/login" "200"

# Backend health should be live
check_status "$API_BASE_URL/health/live" "200"

# Admin session endpoint should reject unauthenticated access (proves route exists + auth middleware)
check_status "$API_BASE_URL/api/v1/admin/session" "401"

# API keys list should reject unauthenticated access (proves route exists)
check_status "$API_BASE_URL/api/v1/admin/api-keys" "401"

echo "🎉 Admin-web smoke checks passed"
