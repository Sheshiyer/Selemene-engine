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
REPROCESS_BODY="$TMP_DIR/reprocess.json"
REPROCESS_DETAIL_BODY="$TMP_DIR/reprocess-detail.json"
BASELINE_CREATE_BODY="$TMP_DIR/baseline-create.json"
BASELINE_LIST_BODY="$TMP_DIR/baseline-list.json"
COMPARISON_BODY="$TMP_DIR/comparison.json"
EXPORT_BODY="$TMP_DIR/export.json"
CLOSE_BODY="$TMP_DIR/close.json"
GENERATED_IMAGE="$TMP_DIR/smoke.png"
ARTIFACT_ROOT="${BIOFIELD_ARTIFACTS_DIR:-$(pwd)/.runtime/biofield-artifacts}"

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
  export BIOFIELD_CAPTURE_IMAGE
fi

if [[ ! -f "$BIOFIELD_CAPTURE_IMAGE" ]]; then
  fail "BIOFIELD_CAPTURE_IMAGE does not exist: $BIOFIELD_CAPTURE_IMAGE"
fi

echo "Running biofield-web BF3 smoke checks"
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

post_json "$API_BASE_URL/api/v1/biofield/readings/$READING_ID/reprocess" "201" '{}' "$REPROCESS_BODY" "$AUTH_HEADER"
REPROCESSED_READING_ID="$(json_field "$REPROCESS_BODY" reading_id)"
SOURCE_READING_ID="$(json_field "$REPROCESS_BODY" source_reading_id)"
if [[ -z "$REPROCESSED_READING_ID" || "$SOURCE_READING_ID" != "$READING_ID" ]]; then
  fail "Reprocess response did not return a new reading linked to the original reading"
fi
echo "✅ Reprocess created reading $REPROCESSED_READING_ID from $READING_ID"

get_json "$API_BASE_URL/api/v1/biofield/readings/$REPROCESSED_READING_ID" "200" "$REPROCESS_DETAIL_BODY" "$AUTH_HEADER"
REPROCESS_DETAIL_ID="$(json_field "$REPROCESS_DETAIL_BODY" reading_id)"
if [[ "$REPROCESS_DETAIL_ID" != "$REPROCESSED_READING_ID" ]]; then
  fail "Reprocessed reading detail did not match the reprocess response"
fi
echo "✅ Reprocessed reading detail resolves"

export READING_ID REPROCESSED_READING_ID
BASELINE_PAYLOAD="$(python3 - <<'PY'
import json
import os
print(json.dumps({
    'name': 'Smoke baseline',
    'notes': 'Generated during BF2 smoke verification',
    'reading_ids': [os.environ['READING_ID'], os.environ['REPROCESSED_READING_ID']],
}))
PY
)"
post_json "$API_BASE_URL/api/v1/biofield/baselines" "201" "$BASELINE_PAYLOAD" "$BASELINE_CREATE_BODY" "$AUTH_HEADER"
BASELINE_ID="$(json_field "$BASELINE_CREATE_BODY" baseline_id)"
BASELINE_COUNT="$(json_field "$BASELINE_CREATE_BODY" reading_count)"
if [[ -z "$BASELINE_ID" || "$BASELINE_COUNT" != "2" ]]; then
  fail "Baseline creation did not return the expected id and reading count"
fi
echo "✅ Baseline created with 2 readings"
export BASELINE_ID

get_json "$API_BASE_URL/api/v1/biofield/baselines" "200" "$BASELINE_LIST_BODY" "$AUTH_HEADER"
LIST_BASELINE_ID="$(json_field "$BASELINE_LIST_BODY" items.0.baseline_id)"
if [[ "$LIST_BASELINE_ID" != "$BASELINE_ID" ]]; then
  fail "Baseline list did not return the created baseline"
fi
echo "✅ Baseline list returns the created baseline"

get_json "$API_BASE_URL/api/v1/biofield/readings/$READING_ID?baseline_id=$BASELINE_ID" "200" "$COMPARISON_BODY" "$AUTH_HEADER"
COMPARISON_BASELINE_ID="$(json_field "$COMPARISON_BODY" comparison.baseline.baseline_id)"
COMPARISON_VERSION="$(json_field "$COMPARISON_BODY" comparison.comparison_version)"
FIRST_COMPARISON_KEY="$(json_field "$COMPARISON_BODY" comparison.deltas.0.key)"
if [[ "$COMPARISON_BASELINE_ID" != "$BASELINE_ID" || -z "$COMPARISON_VERSION" || -z "$FIRST_COMPARISON_KEY" ]]; then
  fail "Comparison detail did not return the expected baseline comparison payload"
fi
echo "✅ Comparison detail returns baseline deltas ($COMPARISON_VERSION)"

EXPORT_PAYLOAD="$(python3 - <<'PY'
import json
import os
print(json.dumps({
    'reading_id': os.environ['READING_ID'],
    'baseline_id': os.environ['BASELINE_ID'],
    'format': 'json',
}))
PY
)"
post_json "$API_BASE_URL/api/v1/biofield/exports" "201" "$EXPORT_PAYLOAD" "$EXPORT_BODY" "$AUTH_HEADER"
EXPORT_ID="$(json_field "$EXPORT_BODY" export_id)"
EXPORT_FORMAT="$(json_field "$EXPORT_BODY" format)"
EXPORT_STORAGE_PATH="$(json_field "$EXPORT_BODY" storage_path)"
EXPORT_CONTRACT_VERSION="$(json_field "$EXPORT_BODY" bundle.contract_version)"
if [[ -z "$EXPORT_ID" || "$EXPORT_FORMAT" != "json" || "$EXPORT_CONTRACT_VERSION" != "biofield-export/v1" ]]; then
  fail "Export response did not return the expected persisted bundle metadata"
fi
if [[ ! -f "$ARTIFACT_ROOT/$EXPORT_STORAGE_PATH" ]]; then
  fail "Expected persisted export bundle at $ARTIFACT_ROOT/$EXPORT_STORAGE_PATH"
fi
echo "✅ Export bundle persisted at $ARTIFACT_ROOT/$EXPORT_STORAGE_PATH"

close_payload='{"reason":"smoke-complete"}'
CLOSE_STATUS="$(curl -sS -o "$CLOSE_BODY" -w "%{http_code}" -X POST "$API_BASE_URL/api/v1/biofield/sessions/$SESSION_ID/close" -H "Content-Type: application/json" -H "$AUTH_HEADER" --data "$close_payload")"
if [[ "$CLOSE_STATUS" == "429" ]]; then
  RESET_AT="$(json_field "$CLOSE_BODY" details.reset_at)"
  SLEEP_SECONDS="$(RESET_AT="$RESET_AT" python3 - <<'PY'
import os
import time
reset_at = int(os.environ["RESET_AT"])
print(max(1, reset_at - int(time.time()) + 1))
PY
)"
  echo "⏳ Close request hit rate limit; sleeping ${SLEEP_SECONDS}s for reset"
  sleep "$SLEEP_SECONDS"
  post_json "$API_BASE_URL/api/v1/biofield/sessions/$SESSION_ID/close" "200" "$close_payload" "$CLOSE_BODY" "$AUTH_HEADER"
else
  if [[ "$CLOSE_STATUS" != "200" ]]; then
    echo "Response body from $API_BASE_URL/api/v1/biofield/sessions/$SESSION_ID/close:" >&2
    cat "$CLOSE_BODY" >&2
    fail "Expected POST $API_BASE_URL/api/v1/biofield/sessions/$SESSION_ID/close to return 200, got $CLOSE_STATUS"
  fi
  echo "✅ POST $API_BASE_URL/api/v1/biofield/sessions/$SESSION_ID/close -> $CLOSE_STATUS"
fi
CLOSED_STATUS="$(json_field "$CLOSE_BODY" status)"
if [[ "$CLOSED_STATUS" != "closed" ]]; then
  fail "Session close did not return closed status"
fi

echo "🎉 Biofield BF3 smoke checks passed"
