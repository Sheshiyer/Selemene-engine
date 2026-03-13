#!/usr/bin/env bash
set -euo pipefail

API_BASE_URL="${1:-${API_BASE_URL:-}}"
SMOKE_TEST_JWT="${SMOKE_TEST_JWT:-}"
SMOKE_TEST_API_KEY="${SMOKE_TEST_API_KEY:-}"
SMOKE_REPORT_FILE="${SMOKE_REPORT_FILE:-}"

usage() {
  cat <<'USAGE'
Usage:
  SMOKE_TEST_JWT=<jwt> bash scripts/smoke-test-runner.sh https://api.example.com

Optional environment:
  API_BASE_URL        Fallback target URL when omitted as argv[1]
  SMOKE_TEST_JWT      Bearer token for protected endpoints
  SMOKE_TEST_API_KEY  Alternative auth mechanism for protected endpoints
  SMOKE_REPORT_FILE   Optional path to also write the JSON report
USAGE
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

append_result() {
  local check="$1"
  local status="$2"
  local http_status="${3:-null}"
  local latency_ms="${4:-null}"
  local details_json="${5:-}"
  local tmp details_file
  if [[ -z "$details_json" ]]; then
    details_json='{}'
  fi
  tmp="$(mktemp)"
  details_file="$(mktemp)"
  printf '%s' "$details_json" >"$details_file"

  if jq -e . "$details_file" >/dev/null 2>&1; then
    jq \
      --arg check "$check" \
      --arg status "$status" \
      --arg http_status "$http_status" \
      --arg latency_ms "$latency_ms" \
      --slurpfile details "$details_file" \
      '. + [{
        check: $check,
        status: $status,
        http_status: (if $http_status == "null" then null else ($http_status | tonumber? // $http_status) end),
        latency_ms: (if $latency_ms == "null" then null else ($latency_ms | tonumber? // $latency_ms) end),
        details: $details[0]
      }]' \
      "$RESULTS_FILE" >"$tmp"
  else
    jq \
      --arg check "$check" \
      --arg status "$status" \
      --arg http_status "$http_status" \
      --arg latency_ms "$latency_ms" \
      --arg raw_details "$details_json" \
      '. + [{
        check: $check,
        status: $status,
        http_status: (if $http_status == "null" then null else ($http_status | tonumber? // $http_status) end),
        latency_ms: (if $latency_ms == "null" then null else ($latency_ms | tonumber? // $latency_ms) end),
        details: {raw: $raw_details}
      }]' \
      "$RESULTS_FILE" >"$tmp"
  fi

  rm -f "$details_file"
  mv "$tmp" "$RESULTS_FILE"
}

latency_ms_from_seconds() {
  if [[ -z "${1:-}" ]]; then
    printf 'null\n'
    return
  fi
  awk "BEGIN { printf \"%d\", ($1 * 1000) }"
}

make_request() {
  local method="$1"
  local url="$2"
  local auth_mode="$3"
  local body="${4:-}"
  local response_file shell_meta
  response_file="$(mktemp)"

  local -a curl_args
  curl_args=(-sS -o "$response_file" -w "%{http_code}|%{time_total}|%{content_type}" -X "$method")

  if [[ "$auth_mode" == "auth" ]]; then
    if [[ -n "$SMOKE_TEST_JWT" ]]; then
      curl_args+=(-H "Authorization: Bearer ${SMOKE_TEST_JWT}")
    elif [[ -n "$SMOKE_TEST_API_KEY" ]]; then
      curl_args+=(-H "X-API-Key: ${SMOKE_TEST_API_KEY}")
    else
      echo "AUTH_REQUIRED|||$response_file"
      return 0
    fi
  fi

  if [[ -n "$body" ]]; then
    curl_args+=(-H "Content-Type: application/json" --data "$body")
  fi

  if ! shell_meta="$(curl "${curl_args[@]}" "$url")"; then
    echo "CURL_FAILED|||$response_file"
    return 0
  fi

  echo "${shell_meta}|${response_file}"
}

check_health_live() {
  local response body_file http_status latency_s latency_ms details
  response="$(make_request GET "${API_BASE_URL}/health/live" none)"
  IFS='|' read -r http_status latency_s _content_type body_file <<<"$response"

  if [[ "$http_status" == "CURL_FAILED" ]]; then
    append_result "health_live" "fail" null null '{"error":"request failed"}'
    FAILURES=$((FAILURES + 1))
    return
  fi

  latency_ms="$(latency_ms_from_seconds "$latency_s")"

  if [[ "$http_status" == "200" ]] && jq -e '.status == "ok"' "$body_file" >/dev/null 2>&1; then
    details="$(jq -c '{status, engines_loaded, workflows_loaded}' "$body_file")"
    append_result "health_live" "pass" "$http_status" "$latency_ms" "$details"
  else
    details="$(jq -c '.' "$body_file" 2>/dev/null || jq -nc --arg error "unexpected response" '{error: $error}')"
    append_result "health_live" "fail" "${http_status:-null}" "${latency_ms:-null}" "$details"
    FAILURES=$((FAILURES + 1))
  fi
}

check_health_ready() {
  local response body_file http_status latency_s latency_ms details
  response="$(make_request GET "${API_BASE_URL}/health/ready" none)"
  IFS='|' read -r http_status latency_s _content_type body_file <<<"$response"

  if [[ "$http_status" == "CURL_FAILED" ]]; then
    append_result "health_ready" "fail" null null '{"error":"request failed"}'
    FAILURES=$((FAILURES + 1))
    return
  fi

  latency_ms="$(latency_ms_from_seconds "$latency_s")"

  if [[ "$http_status" == "200" ]] && jq -e '.overall_status == "ready" and (.redis|type=="string") and (.postgres|type=="string") and (.bridge_status|type=="string")' "$body_file" >/dev/null 2>&1; then
    details="$(jq -c '{redis, postgres, bridge_status, overall_status}' "$body_file")"
    append_result "health_ready" "pass" "$http_status" "$latency_ms" "$details"
  else
    details="$(jq -c '.' "$body_file" 2>/dev/null || jq -nc --arg error "unexpected response" '{error: $error}')"
    append_result "health_ready" "fail" "${http_status:-null}" "${latency_ms:-null}" "$details"
    FAILURES=$((FAILURES + 1))
  fi
}

check_engines_list() {
  local response body_file http_status latency_s latency_ms details
  response="$(make_request GET "${API_BASE_URL}/api/v1/engines" auth)"
  IFS='|' read -r http_status latency_s _content_type body_file <<<"$response"

  if [[ "$http_status" == "AUTH_REQUIRED" ]]; then
    append_result "engines_list" "fail" null null '{"error":"SMOKE_TEST_JWT or SMOKE_TEST_API_KEY is required"}'
    FAILURES=$((FAILURES + 1))
    return
  elif [[ "$http_status" == "CURL_FAILED" ]]; then
    append_result "engines_list" "fail" null null '{"error":"request failed"}'
    FAILURES=$((FAILURES + 1))
    return
  fi

  latency_ms="$(latency_ms_from_seconds "$latency_s")"
  if jq -e '
      (.engines | length) >= 16 and
      (.engines | index("tarot")) and
      (.engines | index("i-ching")) and
      (.engines | index("enneagram")) and
      (.engines | index("sacred-geometry")) and
      (.engines | index("sigil-forge"))
    ' "$body_file" >/dev/null 2>&1; then
    details="$(jq -c '{engine_count: (.engines | length), sample: (.engines[0:5])}' "$body_file")"
    append_result "engines_list" "pass" "$http_status" "$latency_ms" "$details"
  else
    details="$(jq -c '.' "$body_file" 2>/dev/null || jq -nc --arg error "unexpected response" '{error: $error}')"
    append_result "engines_list" "fail" "$http_status" "$latency_ms" "$details"
    FAILURES=$((FAILURES + 1))
  fi
}

check_panchanga_calc() {
  local payload response body_file http_status latency_s latency_ms details
  payload="$(jq -nc '{
    birth_data: {
      date: "1991-08-13",
      time: "13:31",
      latitude: 12.9340,
      longitude: 77.6214,
      timezone: "Asia/Kolkata"
    },
    options: {}
  }')"
  response="$(make_request POST "${API_BASE_URL}/api/v1/engines/panchanga/calculate" auth "$payload")"
  IFS='|' read -r http_status latency_s _content_type body_file <<<"$response"

  if [[ "$http_status" == "AUTH_REQUIRED" ]]; then
    append_result "panchanga_calc" "fail" null null '{"error":"SMOKE_TEST_JWT or SMOKE_TEST_API_KEY is required"}'
    FAILURES=$((FAILURES + 1))
    return
  elif [[ "$http_status" == "CURL_FAILED" ]]; then
    append_result "panchanga_calc" "fail" null null '{"error":"request failed"}'
    FAILURES=$((FAILURES + 1))
    return
  fi

  latency_ms="$(latency_ms_from_seconds "$latency_s")"
  if [[ "$http_status" == "200" ]] && jq -e '.result.tithi_name and .result.nakshatra_name and .result.yoga_name' "$body_file" >/dev/null 2>&1; then
    details="$(jq -c '{tithi: .result.tithi_name, nakshatra: .result.nakshatra_name, yoga: .result.yoga_name}' "$body_file")"
    append_result "panchanga_calc" "pass" "$http_status" "$latency_ms" "$details"
  else
    details="$(jq -c '.' "$body_file" 2>/dev/null || jq -nc --arg error "unexpected response" '{error: $error}')"
    append_result "panchanga_calc" "fail" "$http_status" "$latency_ms" "$details"
    FAILURES=$((FAILURES + 1))
  fi
}

check_workflow_exec() {
  local payload response body_file http_status latency_s latency_ms details
  payload="$(jq -nc '{
    birth_data: {
      name: "Smoke Test User",
      date: "1991-08-13",
      time: "13:31",
      latitude: 12.9340,
      longitude: 77.6214,
      timezone: "Asia/Kolkata"
    },
    options: {}
  }')"
  response="$(make_request POST "${API_BASE_URL}/api/v1/workflows/birth-blueprint/execute" auth "$payload")"
  IFS='|' read -r http_status latency_s _content_type body_file <<<"$response"

  if [[ "$http_status" == "AUTH_REQUIRED" ]]; then
    append_result "workflow_exec" "fail" null null '{"error":"SMOKE_TEST_JWT or SMOKE_TEST_API_KEY is required"}'
    FAILURES=$((FAILURES + 1))
    return
  elif [[ "$http_status" == "CURL_FAILED" ]]; then
    append_result "workflow_exec" "fail" null null '{"error":"request failed"}'
    FAILURES=$((FAILURES + 1))
    return
  fi

  latency_ms="$(latency_ms_from_seconds "$latency_s")"
  if [[ "$http_status" == "200" ]] && jq -e '(.engine_results | type == "object") and ((.engine_results | keys | length) >= 3)' "$body_file" >/dev/null 2>&1; then
    details="$(jq -c '{workflow_id, engines_executed: (.engine_results | keys | length)}' "$body_file")"
    append_result "workflow_exec" "pass" "$http_status" "$latency_ms" "$details"
  else
    details="$(jq -c '.' "$body_file" 2>/dev/null || jq -nc --arg error "unexpected response" '{error: $error}')"
    append_result "workflow_exec" "fail" "$http_status" "$latency_ms" "$details"
    FAILURES=$((FAILURES + 1))
  fi
}

check_metrics_endpoint() {
  local response body_file http_status latency_s latency_ms details
  response="$(make_request GET "${API_BASE_URL}/metrics" none)"
  IFS='|' read -r http_status latency_s content_type body_file <<<"$response"

  if [[ "$http_status" == "CURL_FAILED" ]]; then
    append_result "metrics_endpoint" "fail" null null '{"error":"request failed"}'
    FAILURES=$((FAILURES + 1))
    return
  fi

  latency_ms="$(latency_ms_from_seconds "$latency_s")"

  if [[ "$http_status" == "200" ]] && [[ "$content_type" == text/plain* ]] && grep -q "noesis_calculations_total" "$body_file"; then
    details="$(jq -nc --arg content_type "$content_type" '{content_type: $content_type, metric: "noesis_calculations_total"}')"
    append_result "metrics_endpoint" "pass" "$http_status" "$latency_ms" "$details"
  else
    details="$(jq -nc --arg content_type "${content_type:-}" '{content_type: $content_type, error: "metrics missing or request failed"}')"
    append_result "metrics_endpoint" "fail" "${http_status:-null}" "${latency_ms:-null}" "$details"
    FAILURES=$((FAILURES + 1))
  fi
}

check_ts_bridge() {
  local payload response body_file http_status latency_s latency_ms details
  payload="$(jq -nc '{
    options: {
      question: "What is the current bridge state?",
      spread: "single_card"
    }
  }')"
  response="$(make_request POST "${API_BASE_URL}/api/v1/engines/tarot/calculate" auth "$payload")"
  IFS='|' read -r http_status latency_s _content_type body_file <<<"$response"

  if [[ "$http_status" == "AUTH_REQUIRED" ]]; then
    append_result "ts_bridge" "fail" null null '{"error":"SMOKE_TEST_JWT or SMOKE_TEST_API_KEY is required"}'
    FAILURES=$((FAILURES + 1))
    return
  elif [[ "$http_status" == "CURL_FAILED" ]]; then
    append_result "ts_bridge" "warn" null null '{"error":"request failed","degraded":true}'
    return
  fi

  latency_ms="$(latency_ms_from_seconds "$latency_s")"
  if [[ "$http_status" == "200" ]] && jq -e '.engine_id == "tarot" and .envelope_version == "1" and (.result | type == "object") and (.metadata.backend | type == "string")' "$body_file" >/dev/null 2>&1; then
    details="$(jq -c '{engine_id, envelope_version, backend: .metadata.backend}' "$body_file")"
    append_result "ts_bridge" "pass" "$http_status" "$latency_ms" "$details"
  elif [[ "$http_status" =~ ^5[0-9][0-9]$ ]] || jq -e '.error_code == "BRIDGE_ERROR"' "$body_file" >/dev/null 2>&1; then
    details="$(jq -c '. // {"error":"bridge degraded","degraded":true}' "$body_file" 2>/dev/null || jq -nc --arg error "bridge degraded" '{error: $error, degraded: true}')"
    append_result "ts_bridge" "warn" "$http_status" "$latency_ms" "$details"
  else
    details="$(jq -c '.' "$body_file" 2>/dev/null || jq -nc --arg error "unexpected response" '{error: $error}')"
    append_result "ts_bridge" "fail" "$http_status" "$latency_ms" "$details"
    FAILURES=$((FAILURES + 1))
  fi
}

finalize_report() {
  local tmp summary
  tmp="$(mktemp)"
  summary="$(jq -n \
    --arg target_url "$API_BASE_URL" \
    --arg generated_at "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
    --slurpfile results "$RESULTS_FILE" \
    '{
      target_url: $target_url,
      generated_at: $generated_at,
      overall_status: (if ($results[0] | map(select(.status == "fail")) | length) == 0 then "pass" else "fail" end),
      total_checks: ($results[0] | length),
      failed_checks: ($results[0] | map(select(.status == "fail")) | length),
      results: $results[0]
    }')"
  printf '%s\n' "$summary" | tee "$tmp"
  if [[ -n "$SMOKE_REPORT_FILE" ]]; then
    cp "$tmp" "$SMOKE_REPORT_FILE"
  fi
}

main() {
  if [[ -z "$API_BASE_URL" ]]; then
    usage
    exit 1
  fi

  require_cmd curl
  require_cmd jq

  RESULTS_FILE="$(mktemp)"
  printf '[]' >"$RESULTS_FILE"
  FAILURES=0

  check_health_live
  check_health_ready
  check_engines_list
  check_panchanga_calc
  check_workflow_exec
  check_metrics_endpoint
  check_ts_bridge
  finalize_report

  if [[ "$FAILURES" -gt 0 ]]; then
    exit 1
  fi
}

main "$@"
