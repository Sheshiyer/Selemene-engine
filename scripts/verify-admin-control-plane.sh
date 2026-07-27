#!/usr/bin/env bash
set -euo pipefail

deployment="${1:?usage: verify-admin-control-plane.sh <vercel-deployment-url> [scope]}"
scope="${2:-sheshiyers-projects}"

headers="$(
  vercel curl / \
    --deployment "$deployment" \
    --scope "$scope" \
    -- \
    --silent \
    --show-error \
    --head
)"

status="$(
  printf '%s\n' "$headers" |
    awk '/^HTTP\// { code = $2 } END { print code }'
)"
location="$(
  printf '%s\n' "$headers" |
    awk 'BEGIN { IGNORECASE = 1 } /^location:/ { sub(/\r$/, "", $2); print $2 }'
)"

if [[ "$status" != "307" || "$location" != "/admin/login" ]]; then
  printf 'admin control-plane probe failed: status=%s location=%s\n' \
    "${status:-missing}" "${location:-missing}" >&2
  exit 1
fi

printf 'admin control-plane probe passed: status=%s location=%s\n' \
  "$status" "$location"
