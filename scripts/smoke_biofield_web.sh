#!/usr/bin/env bash
set -euo pipefail

BIOFIELD_WEB_URL="${BIOFIELD_WEB_URL:-}"
API_BASE_URL="${API_BASE_URL:-}"
PYTHON_BIOFIELD_URL="${PYTHON_BIOFIELD_URL:-}"
BIOFIELD_EMAIL="${BIOFIELD_EMAIL:-biofield-smoke-$(date +%s)-$$@example.com}"
BIOFIELD_PASSWORD="${BIOFIELD_PASSWORD:-SmokePass123}"
BIOFIELD_FULL_NAME="${BIOFIELD_FULL_NAME:-Biofield Smoke User}"
BIOFIELD_CAPTURE_IMAGE="${BIOFIELD_CAPTURE_IMAGE:-}"
export BIOFIELD_EMAIL BIOFIELD_PASSWORD BIOFIELD_FULL_NAME BIOFIELD_CAPTURE_IMAGE

TMP_DIR="$(mktemp -d)"
REGISTER_BODY="$TMP_DIR/register.json"
LOGIN_BODY="$TMP_DIR/login.json"
SESSION_BODY="$TMP_DIR/session.json"
CAPTURE_BODY="$TMP_DIR/capture.json"
HISTORY_BODY="$TMP_DIR/history.json"
DETAIL_BODY="$TMP_DIR/detail.json"
CLOSE_BODY="$TMP_DIR/close.json"
GENERATED_IMAGE="$TMP_DIR/smoke.png"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

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

json_field() {
  local file="$1"
  local path="$2"
  python3 - "$file" "$path" <<'PY'
import json
import sys

file_path, dotted = sys.argv[1], sys.argv[2]
with open(file_path, 'r', encoding='utf-8') as handle:
    payload = json.load(handle)
value = payload
for chunk in dotted.split('.'):
    if isinstance(value, list):
        value = value[int(chunk)]
    else:
        value = value[chunk]
if isinstance(value, bool):
    print('true' if value else 'false')
elif value is None:
    print('null')
elif isinstance(value, (dict, list)):
    print(json.dumps(value))
else:
    print(value)
PY
}

post_json() {
  local url="$1"
  local expected="$2"
  local payload="$3"
  local output="$4"
  local auth_header="${5:-}"
  local status

  if [[ -n "$auth_header" ]]; then
    status="$(curl -sS -o "$output" -w "%{http_code}" -X POST "$url" -H "Content-Type: application/json" -H "$auth_header" --data "$payload")"
  else
    status="$(curl -sS -o "$output" -w "%{http_code}" -X POST "$url" -H "Content-Type: application/json" --data "$payload")"
  fi

  if [[ "$status" != "$expected" ]]; then
    echo "Response body from $url:" >&2
    cat "$output" >&2
    fail "Expected POST $url to return $expected, got $status"
  fi

  echo "✅ POST $url -> $status"
}

get_json() {
  local url="$1"
  local expected="$2"
  local output="$3"
  local auth_header="$4"
  local status
  status="$(curl -sS -o "$output" -w "%{http_code}" -X GET "$url" -H "$auth_header")"

  if [[ "$status" != "$expected" ]]; then
    echo "Response body from $url:" >&2
    cat "$output" >&2
    fail "Expected GET $url to return $expected, got $status"
  fi

  echo "✅ GET $url -> $status"
}

write_default_capture() {
  python3 - "$GENERATED_IMAGE" <<'PY'
import base64
import sys

png_bytes = base64.b64decode(
    'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO5Hn6kAAAAASUVORK5CYII='
)
with open(sys.argv[1], 'wb') as handle:
    handle.write(png_bytes)
    handle.write(b'\0' * 150_000)
PY
}

if [[ -z "$BIOFIELD_WEB_URL" || -z "$API_BASE_URL" || -z "$PYTHON_BIOFIELD_URL" ]]; then
  cat <<'USAGE'
Usage:
  BIOFIELD_WEB_URL=http://127.0.0.1:3002 \
  API_BASE_URL=http://127.0.0.1:8080 \
  PYTHON_BIOFIELD_URL=http://127.0.0.1:8002 \
  bash scripts/smoke_biofield_web.sh

Optional:
  BIOFIELD_EMAIL
  BIOFIELD_PASSWORD
  BIOFIELD_FULL_NAME
  BIOFIELD_CAPTURE_IMAGE
USAGE
  fail "BIOFIELD_WEB_URL, API_BASE_URL, and PYTHON_BIOFIELD_URL are required"
fi

if [[ -z "$BIOFIELD_CAPTURE_IMAGE" ]]; then
  write_default_capture
  BIOFIELD_CAPTURE_IMAGE="$GENERATED_IMAGE"
fi

if [[ ! -f "$BIOFIELD_CAPTURE_IMAGE" ]]; then
  fail "BIOFIELD_CAPTURE_IMAGE does not exist: $BIOFIELD_CAPTURE_IMAGE"
fi

echo "Running biofield-web Phase 1 smoke checks"
echo "BIOFIELD_WEB_URL=$BIOFIELD_WEB_URL"
echo "API_BASE_URL=$API_BASE_URL"
echo "PYTHON_BIOFIELD_URL=$PYTHON_BIOFIELD_URL"
echo "BIOFIELD_EMAIL=$BIOFIELD_EMAIL"
echo "BIOFIELD_CAPTURE_IMAGE=$BIOFIELD_CAPTURE_IMAGE"

check_status "$BIOFIELD_WEB_URL/login" "200"
check_status "$BIOFIELD_WEB_URL/viewer" "200"
check_status "$BIOFIELD_WEB_URL/history" "200"
check_status "$API_BASE_URL/health/live" "200"
check_status "$PYTHON_BIOFIELD_URL/health" "200"

register_payload="$(python3 - <<'PY'
import json
import os
print(json.dumps({
    'email': os.environ['BIOFIELD_EMAIL'],
    'password': os.environ['BIOFIELD_PASSWORD'],
    'full_name': os.environ['BIOFIELD_FULL_NAME'],
}))
PY
)"

register_status="$(curl -sS -o "$REGISTER_BODY" -w "%{http_code}" -X POST "$API_BASE_URL/api/v1/auth/register" -H "Content-Type: application/json" --data "$register_payload")"
if [[ "$register_status" != "201" && "$register_status" != "401" ]]; then
  echo "Response body from register:" >&2
  cat "$REGISTER_BODY" >&2
  fail "Expected register to return 201 or 401, got $register_status"
fi
echo "✅ POST $API_BASE_URL/api/v1/auth/register -> $register_status"

login_payload="$(python3 - <<'PY'
import json
import os
print(json.dumps({
    'email': os.environ['BIOFIELD_EMAIL'],
    'password': os.environ['BIOFIELD_PASSWORD'],
}))
PY
)"
post_json "$API_BASE_URL/api/v1/auth/login" "200" "$login_payload" "$LOGIN_BODY"
TOKEN="$(json_field "$LOGIN_BODY" token)"
if [[ -z "$TOKEN" ]]; then
  fail "Login response did not contain a token"
fi
AUTH_HEADER="Authorization: Bearer $TOKEN"

session_payload='{"client_device_id":"biofield-smoke","viewer_version":"biofield-web/smoke","context":{"platform":"smoke-script"}}'
post_json "$API_BASE_URL/api/v1/biofield/sessions" "201" "$session_payload" "$SESSION_BODY" "$AUTH_HEADER"
SESSION_ID="$(json_field "$SESSION_BODY" id)"
if [[ -z "$SESSION_ID" ]]; then
  fail "Session response did not contain an id"
fi

echo "Uploading capture through Noesis → Python sidecar"
CAPTURE_STATUS="$(curl -sS -o "$CAPTURE_BODY" -w "%{http_code}" -X POST "$API_BASE_URL/api/v1/biofield/sessions/$SESSION_ID/captures" -H "$AUTH_HEADER" -F "image=@$BIOFIELD_CAPTURE_IMAGE;type=image/png" -F 'options={"mode":"capture"}' -F 'capture_metadata={"platform":"smoke-script","source":"generated"}')"
if [[ "$CAPTURE_STATUS" != "201" ]]; then
  echo "Response body from capture upload:" >&2
  cat "$CAPTURE_BODY" >&2
  fail "Expected capture upload to return 201, got $CAPTURE_STATUS"
fi
echo "✅ POST $API_BASE_URL/api/v1/biofield/sessions/$SESSION_ID/captures -> $CAPTURE_STATUS"

READING_ID="$(json_field "$CAPTURE_BODY" reading_id)"
ANALYSIS_VERSION="$(json_field "$CAPTURE_BODY" analysis_version)"
QUALITY_OK="$(json_field "$CAPTURE_BODY" quality_assessment.sufficient_quality)"
if [[ -z "$READING_ID" || "$QUALITY_OK" != "true" ]]; then
  fail "Capture response did not contain a successful reading_id / quality_assessment"
fi
echo "✅ Capture persisted reading $READING_ID ($ANALYSIS_VERSION)"

get_json "$API_BASE_URL/api/v1/biofield/readings" "200" "$HISTORY_BODY" "$AUTH_HEADER"
HISTORY_COUNT="$(json_field "$HISTORY_BODY" items | python3 -c 'import json,sys; print(len(json.loads(sys.stdin.read())))')"
if [[ "$HISTORY_COUNT" -lt 1 ]]; then
  fail "History response did not contain any items"
fi
FIRST_HISTORY_ID="$(json_field "$HISTORY_BODY" items.0.reading_id)"
if [[ "$FIRST_HISTORY_ID" != "$READING_ID" ]]; then
  fail "History response top reading_id $FIRST_HISTORY_ID did not match capture reading_id $READING_ID"
fi

echo "✅ History route returns the new reading"

get_json "$API_BASE_URL/api/v1/biofield/readings/$READING_ID" "200" "$DETAIL_BODY" "$AUTH_HEADER"
DETAIL_READING_ID="$(json_field "$DETAIL_BODY" reading_id)"
DETAIL_SESSION_ID="$(json_field "$DETAIL_BODY" session_id)"
DETAIL_QUALITY_OK="$(json_field "$DETAIL_BODY" quality.sufficient_quality)"
if [[ "$DETAIL_READING_ID" != "$READING_ID" || "$DETAIL_SESSION_ID" != "$SESSION_ID" || "$DETAIL_QUALITY_OK" != "true" ]]; then
  fail "Reading detail response did not match the created session/reading"
fi

echo "✅ Reading detail route returns the new reading"

close_payload='{"reason":"smoke-complete"}'
post_json "$API_BASE_URL/api/v1/biofield/sessions/$SESSION_ID/close" "200" "$close_payload" "$CLOSE_BODY" "$AUTH_HEADER"

CLOSED_STATUS="$(json_field "$CLOSE_BODY" status)"
if [[ "$CLOSED_STATUS" != "closed" ]]; then
  fail "Session close did not return closed status"
fi

echo "🎉 Biofield Phase 1 smoke checks passed"
