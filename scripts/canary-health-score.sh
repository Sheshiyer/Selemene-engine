#!/usr/bin/env bash
set -euo pipefail

CANARY_WINDOW="${CANARY_WINDOW:-5m}"
CANARY_ERROR_RATE_THRESHOLD="${CANARY_ERROR_RATE_THRESHOLD:-0.01}"
CANARY_P95_THRESHOLD_SECONDS="${CANARY_P95_THRESHOLD_SECONDS:-2}"
CANARY_SENTRY_CRITICAL_THRESHOLD="${CANARY_SENTRY_CRITICAL_THRESHOLD:-0}"

PROMETHEUS_BASE_URL="${PROMETHEUS_BASE_URL:-}"
PROMETHEUS_QUERY_ENDPOINT="${PROMETHEUS_QUERY_ENDPOINT:-/api/v1/query}"
PROMETHEUS_AUTH_HEADER="${PROMETHEUS_AUTH_HEADER:-}"
PROMETHEUS_QUERY_CMD="${PROMETHEUS_QUERY_CMD:-}"

SENTRY_COUNT_CMD="${SENTRY_COUNT_CMD:-}"
SENTRY_CRITICAL_COUNT="${SENTRY_CRITICAL_COUNT:-}"
SENTRY_API_URL="${SENTRY_API_URL:-}"
SENTRY_AUTH_TOKEN="${SENTRY_AUTH_TOKEN:-}"

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/canary-health-score.sh

Environment:
  CANARY_WINDOW                      Prometheus lookback window (default: 5m)
  CANARY_ERROR_RATE_THRESHOLD        Max allowed error rate (default: 0.01)
  CANARY_P95_THRESHOLD_SECONDS       Max allowed p95 latency seconds (default: 2)
  CANARY_SENTRY_CRITICAL_THRESHOLD   Max allowed critical Sentry errors (default: 0)

  PROMETHEUS_BASE_URL                Base URL for Prometheus (required unless PROMETHEUS_QUERY_CMD is set)
  PROMETHEUS_QUERY_ENDPOINT          Query endpoint path (default: /api/v1/query)
  PROMETHEUS_AUTH_HEADER             Optional header value, e.g. 'Authorization: Bearer ...'
  PROMETHEUS_QUERY_CMD               Optional test hook executable: receives <metric_name> <promql>

  SENTRY_COUNT_CMD                   Optional test hook executable returning an integer count
  SENTRY_CRITICAL_COUNT              Optional explicit integer override for Sentry critical count
  SENTRY_API_URL                     Optional Sentry URL returning a count-like JSON payload
  SENTRY_AUTH_TOKEN                  Optional bearer token for Sentry API calls
USAGE
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

compare_gt() {
  awk -v left="$1" -v right="$2" 'BEGIN { exit !(left > right) }'
}

prometheus_scalar() {
  jq -r '
    if .status == "success" then
      if (.data.result | type) == "array" and (.data.result | length) > 0 then
        .data.result[0].value[1]
      elif (.data.result | type) == "string" then
        .data.result
      else
        empty
      end
    else
      empty
    end
  ' 2>/dev/null
}

query_prometheus() {
  local metric_name="$1"
  local promql="$2"

  if [[ -n "$PROMETHEUS_QUERY_CMD" ]]; then
    "$PROMETHEUS_QUERY_CMD" "$metric_name" "$promql"
    return 0
  fi

  if [[ -z "$PROMETHEUS_BASE_URL" ]]; then
    return 1
  fi

  local -a curl_args
  curl_args=(-sS -G "${PROMETHEUS_BASE_URL%/}${PROMETHEUS_QUERY_ENDPOINT}")

  if [[ -n "$PROMETHEUS_AUTH_HEADER" ]]; then
    curl_args+=(-H "$PROMETHEUS_AUTH_HEADER")
  fi

  curl_args+=(--data-urlencode "query=$promql")
  curl "${curl_args[@]}"
}

fetch_prometheus_metric() {
  local metric_name="$1"
  local promql="$2"
  local tmp_file observed
  tmp_file="$(mktemp)"

  if ! query_prometheus "$metric_name" "$promql" >"$tmp_file"; then
    rm -f "$tmp_file"
    return 1
  fi

  observed="$(prometheus_scalar <"$tmp_file" || true)"
  if [[ -z "$observed" ]]; then
    rm -f "$tmp_file"
    return 1
  fi

  printf '%s|%s\n' "$observed" "$(jq -c . <"$tmp_file")"
  rm -f "$tmp_file"
}

fetch_sentry_count() {
  if [[ -n "$SENTRY_COUNT_CMD" ]]; then
    "$SENTRY_COUNT_CMD"
    return 0
  fi

  if [[ -n "$SENTRY_CRITICAL_COUNT" ]]; then
    printf '%s\n' "$SENTRY_CRITICAL_COUNT"
    return 0
  fi

  if [[ -n "$SENTRY_API_URL" ]]; then
    local -a curl_args
    curl_args=(-sS "$SENTRY_API_URL")
    if [[ -n "$SENTRY_AUTH_TOKEN" ]]; then
      curl_args+=(-H "Authorization: Bearer ${SENTRY_AUTH_TOKEN}")
    fi

    curl "${curl_args[@]}" | jq -r '
      if type == "object" and .count != null then .count
      elif type == "object" and .total != null then .total
      elif type == "array" then length
      else empty
      end
    '
    return 0
  fi

  return 1
}

build_metric_json() {
  local metric_name="$1"
  local status="$2"
  local observed="$3"
  local threshold="$4"
  local query="$5"
  local details="$6"

  jq -nc \
    --arg metric_name "$metric_name" \
    --arg status "$status" \
    --arg observed "$observed" \
    --arg threshold "$threshold" \
    --arg query "$query" \
    --arg details "$details" \
    '{
      metric: $metric_name,
      status: $status,
      observed: (if $observed == "" then null else ($observed | tonumber? // $observed) end),
      threshold: (if $threshold == "" then null else ($threshold | tonumber? // $threshold) end),
      query: (if $query == "" then null else $query end),
      details: (if $details == "" then null else $details end)
    }'
}

main() {
  if [[ "${1:-}" == "--help" ]]; then
    usage
    exit 0
  fi

  require_cmd jq
  if [[ -z "$PROMETHEUS_QUERY_CMD" ]]; then
    require_cmd curl
  fi

  local error_rate_query="rate(noesis_requests_total{status=\"error\"}[${CANARY_WINDOW}]) / rate(noesis_requests_total[${CANARY_WINDOW}])"
  local p95_query="histogram_quantile(0.95, rate(noesis_request_duration_seconds_bucket[${CANARY_WINDOW}]))"

  local error_rate_status="fail"
  local error_rate_observed=""
  local error_rate_details=""
  local error_rate_raw=""
  if error_rate_raw="$(fetch_prometheus_metric "error_rate" "$error_rate_query" 2>/dev/null)"; then
    error_rate_observed="${error_rate_raw%%|*}"
    if compare_gt "$error_rate_observed" "$CANARY_ERROR_RATE_THRESHOLD"; then
      error_rate_status="fail"
      error_rate_details="error rate exceeds threshold"
    else
      error_rate_status="pass"
      error_rate_details="error rate within threshold"
    fi
  else
    error_rate_status="fail"
    error_rate_details="Prometheus error rate query failed"
  fi

  local p95_status="fail"
  local p95_observed=""
  local p95_details=""
  local p95_raw=""
  if p95_raw="$(fetch_prometheus_metric "request_p95_seconds" "$p95_query" 2>/dev/null)"; then
    p95_observed="${p95_raw%%|*}"
    if compare_gt "$p95_observed" "$CANARY_P95_THRESHOLD_SECONDS"; then
      p95_status="fail"
      p95_details="request p95 exceeds threshold"
    else
      p95_status="pass"
      p95_details="request p95 within threshold"
    fi
  else
    p95_status="fail"
    p95_details="Prometheus latency query failed"
  fi

  local sentry_status="warn"
  local sentry_observed=""
  local sentry_details="Sentry not configured"
  if sentry_observed="$(fetch_sentry_count 2>/dev/null)"; then
    sentry_details="critical Sentry count available"
    if compare_gt "$sentry_observed" "$CANARY_SENTRY_CRITICAL_THRESHOLD"; then
      sentry_status="fail"
      sentry_details="critical Sentry count exceeds threshold"
    else
      sentry_status="pass"
      sentry_details="critical Sentry count within threshold"
    fi
  fi

  local overall_status="pass"
  local canary_healthy="true"
  if [[ "$error_rate_status" == "fail" || "$p95_status" == "fail" || "$sentry_status" == "fail" ]]; then
    overall_status="fail"
    canary_healthy="false"
  elif [[ "$error_rate_status" == "warn" || "$p95_status" == "warn" || "$sentry_status" == "warn" ]]; then
    overall_status="warn"
    canary_healthy="false"
  fi

  local evaluated_at
  evaluated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

  jq -n \
    --arg schema_version "1" \
    --arg evaluated_at "$evaluated_at" \
    --arg window "$CANARY_WINDOW" \
    --arg overall_status "$overall_status" \
    --argjson canary_healthy "$canary_healthy" \
    --argjson error_metric "$(build_metric_json "error_rate" "$error_rate_status" "$error_rate_observed" "$CANARY_ERROR_RATE_THRESHOLD" "$error_rate_query" "$error_rate_details")" \
    --argjson p95_metric "$(build_metric_json "request_p95_seconds" "$p95_status" "$p95_observed" "$CANARY_P95_THRESHOLD_SECONDS" "$p95_query" "$p95_details")" \
    --argjson sentry_metric "$(build_metric_json "sentry_critical_count" "$sentry_status" "$sentry_observed" "$CANARY_SENTRY_CRITICAL_THRESHOLD" "" "$sentry_details")" \
    '{
      schema_version: $schema_version,
      evaluated_at: $evaluated_at,
      window: $window,
      overall_status: $overall_status,
      canary_healthy: $canary_healthy,
      metrics: {
        error_rate: $error_metric,
        request_p95_seconds: $p95_metric,
        sentry_critical_count: $sentry_metric
      }
    }'
}

main "$@"
