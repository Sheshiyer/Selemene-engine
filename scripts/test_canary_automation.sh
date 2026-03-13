#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

make_executable() {
  chmod +x "$1"
}

write_mock_prometheus() {
  local path="$1"
  cat >"$path" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
metric_name="$1"
mode="${MOCK_PROM_MODE:-healthy}"

case "${mode}:${metric_name}" in
  healthy:error_rate)
    printf '{"status":"success","data":{"result":[{"value":[0,"0.001"]}]}}'
    ;;
  healthy:request_p95_seconds)
    printf '{"status":"success","data":{"result":[{"value":[0,"1.2"]}]}}'
    ;;
  fail:error_rate)
    printf '{"status":"success","data":{"result":[{"value":[0,"0.02"]}]}}'
    ;;
  fail:request_p95_seconds)
    printf '{"status":"success","data":{"result":[{"value":[0,"2.5"]}]}}'
    ;;
  *)
    printf '{"status":"success","data":{"result":[]}}'
    ;;
esac
EOF
  make_executable "$path"
}

write_mock_sentry() {
  local path="$1"
  cat >"$path" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "${MOCK_SENTRY_COUNT:-0}"
EOF
  make_executable "$path"
}

write_mock_health_script() {
  local path="$1"
  cat >"$path" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
stage="${CANARY_STAGE_PERCENT:-0}"
case "$stage" in
  5|25)
    printf '{"overall_status":"pass","canary_healthy":true}'
    ;;
  50|100)
    printf '{"overall_status":"fail","canary_healthy":false}'
    ;;
  *)
    printf '{"overall_status":"pass","canary_healthy":true}'
    ;;
esac
EOF
  make_executable "$path"
}

write_recording_hook() {
  local path="$1"
  local log_file="$2"
  cat >"$path" <<EOF
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "\$*" >>"$log_file"
EOF
  make_executable "$path"
}

assert_jq() {
  local file="$1"
  local filter="$2"
  jq -e "$filter" "$file" >/dev/null
}

main() {
  local mock_prom="$TMP_DIR/mock-prometheus.sh"
  local mock_sentry="$TMP_DIR/mock-sentry.sh"
  local mock_health="$TMP_DIR/mock-health.sh"
  local promote_hook="$TMP_DIR/promote-hook.sh"
  local rollback_hook="$TMP_DIR/rollback-hook.sh"
  local hook_log="$TMP_DIR/hook.log"
  local result_file

  write_mock_prometheus "$mock_prom"
  write_mock_sentry "$mock_sentry"
  write_mock_health_script "$mock_health"
  : >"$hook_log"
  write_recording_hook "$promote_hook" "$hook_log"
  write_recording_hook "$rollback_hook" "$hook_log"

  result_file="$TMP_DIR/health-pass.json"
  PROMETHEUS_QUERY_CMD="$mock_prom" \
  SENTRY_COUNT_CMD="$mock_sentry" \
  MOCK_PROM_MODE="healthy" \
  MOCK_SENTRY_COUNT="0" \
    bash "$ROOT_DIR/scripts/canary-health-score.sh" >"$result_file"
  assert_jq "$result_file" '.overall_status == "pass" and .canary_healthy == true and .metrics.error_rate.status == "pass" and .metrics.request_p95_seconds.status == "pass"'

  result_file="$TMP_DIR/health-fail.json"
  PROMETHEUS_QUERY_CMD="$mock_prom" \
  SENTRY_COUNT_CMD="$mock_sentry" \
  MOCK_PROM_MODE="fail" \
  MOCK_SENTRY_COUNT="2" \
    bash "$ROOT_DIR/scripts/canary-health-score.sh" >"$result_file"
  assert_jq "$result_file" '.overall_status == "fail" and .canary_healthy == false and .metrics.error_rate.status == "fail" and .metrics.request_p95_seconds.status == "fail" and .metrics.sentry_critical_count.status == "fail"'

  : >"$hook_log"
  result_file="$TMP_DIR/promote-dry-run.json"
  CANARY_HEALTH_SCORE_SCRIPT="$mock_health" \
  CANARY_PROMOTE_CMD="$promote_hook" \
  CANARY_ROLLBACK_CMD="$rollback_hook" \
    bash "$ROOT_DIR/scripts/canary-promote.sh" --dry-run >"$result_file"
  assert_jq "$result_file" '.overall_status == "fail" and .rolled_back == true and .final_stage == 25 and .stages[0].decision == "would_promote" and .stages[1].decision == "would_promote" and .stages[2].decision == "would_rollback"'
  [[ ! -s "$hook_log" ]]

  : >"$hook_log"
  result_file="$TMP_DIR/promote-live.json"
  CANARY_HEALTH_SCORE_SCRIPT="$mock_health" \
  CANARY_PROMOTE_CMD="$promote_hook" \
  CANARY_ROLLBACK_CMD="$rollback_hook" \
    bash "$ROOT_DIR/scripts/canary-promote.sh" >"$result_file"
  assert_jq "$result_file" '.overall_status == "fail" and .rolled_back == true and .final_stage == 25 and .stages[2].decision == "rolled_back"'
  grep -qx '5' "$hook_log"
  grep -qx '25' "$hook_log"
  grep -qx '25 50' "$hook_log"

  echo "canary automation tests passed"
}

main "$@"
