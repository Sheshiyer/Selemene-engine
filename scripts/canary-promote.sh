#!/usr/bin/env bash
set -euo pipefail

DRY_RUN=false
CURRENT_STAGE="${CURRENT_STAGE:-0}"
TARGET_STAGE="${TARGET_STAGE:-100}"
STAGE_HOLD_SECONDS="${STAGE_HOLD_SECONDS:-0}"

CANARY_HEALTH_SCORE_SCRIPT="${CANARY_HEALTH_SCORE_SCRIPT:-scripts/canary-health-score.sh}"
CANARY_PROMOTE_CMD="${CANARY_PROMOTE_CMD:-}"
CANARY_ROLLBACK_CMD="${CANARY_ROLLBACK_CMD:-}"

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/canary-promote.sh [--dry-run] [--current-stage N] [--target-stage N]

Environment:
  CURRENT_STAGE               Current live traffic percentage (default: 0)
  TARGET_STAGE                Final target traffic percentage (default: 100)
  STAGE_HOLD_SECONDS          Optional hold between stages in non-dry-run mode (default: 0)
  CANARY_HEALTH_SCORE_SCRIPT  Health score script path (default: scripts/canary-health-score.sh)
  CANARY_PROMOTE_CMD          Optional executable called as: <cmd> <stage_percent>
  CANARY_ROLLBACK_CMD         Optional executable called as: <cmd> <previous_stage> <failed_stage>
USAGE
}

run_hook() {
  local hook="$1"
  shift

  if [[ -z "$hook" ]]; then
    return 0
  fi

  "$hook" "$@"
}

append_stage() {
  local stage_json="$1"
  local tmp
  tmp="$(mktemp)"
  jq --argjson stage "$stage_json" '. + [$stage]' "$STAGES_FILE" >"$tmp"
  mv "$tmp" "$STAGES_FILE"
}

main() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --dry-run)
        DRY_RUN=true
        shift
        ;;
      --current-stage)
        CURRENT_STAGE="$2"
        shift 2
        ;;
      --target-stage)
        TARGET_STAGE="$2"
        shift 2
        ;;
      --help)
        usage
        exit 0
        ;;
      *)
        echo "Unknown argument: $1" >&2
        usage >&2
        exit 1
        ;;
    esac
  done

  if ! command -v jq >/dev/null 2>&1; then
    echo "Missing required command: jq" >&2
    exit 1
  fi

  local stages=(5 25 50 100)
  STAGES_FILE="$(mktemp)"
  trap 'rm -f "$STAGES_FILE"' EXIT
  printf '[]' >"$STAGES_FILE"

  local previous_stage="$CURRENT_STAGE"
  local final_stage="$CURRENT_STAGE"
  local rolled_back=false
  local overall_status="pass"

  for stage in "${stages[@]}"; do
    if (( stage <= CURRENT_STAGE || stage > TARGET_STAGE )); then
      continue
    fi

    local health_json decision
    health_json="$(CANARY_STAGE_PERCENT="$stage" "$CANARY_HEALTH_SCORE_SCRIPT")"
    local canary_healthy overall_health
    canary_healthy="$(jq -r '.canary_healthy' <<<"$health_json")"
    overall_health="$(jq -r '.overall_status' <<<"$health_json")"

    if [[ "$canary_healthy" == "true" ]]; then
      if [[ "$DRY_RUN" == "true" ]]; then
        decision="would_promote"
      else
        run_hook "$CANARY_PROMOTE_CMD" "$stage"
        decision="promoted"
      fi
      final_stage="$stage"

      append_stage "$(jq -nc \
        --argjson stage "$stage" \
        --arg decision "$decision" \
        --argjson health "$health_json" \
        '{stage_percent: $stage, decision: $decision, health: $health}')"

      if [[ "$DRY_RUN" != "true" && "$STAGE_HOLD_SECONDS" -gt 0 && "$stage" -lt "$TARGET_STAGE" ]]; then
        sleep "$STAGE_HOLD_SECONDS"
      fi
      previous_stage="$stage"
      continue
    fi

    rolled_back=true
    overall_status="fail"
    if [[ "$DRY_RUN" == "true" ]]; then
      decision="would_rollback"
    else
      run_hook "$CANARY_ROLLBACK_CMD" "$previous_stage" "$stage"
      decision="rolled_back"
    fi

    append_stage "$(jq -nc \
      --argjson stage "$stage" \
      --arg decision "$decision" \
      --argjson health "$health_json" \
      '{stage_percent: $stage, decision: $decision, health: $health}')"
    break
  done

  if [[ "$rolled_back" == "false" && "$final_stage" -lt "$TARGET_STAGE" ]]; then
    overall_status="warn"
  elif [[ "$rolled_back" == "false" ]]; then
    overall_status="pass"
  fi

  jq -n \
    --arg schema_version "1" \
    --argjson dry_run "$DRY_RUN" \
    --argjson current_stage "$CURRENT_STAGE" \
    --argjson target_stage "$TARGET_STAGE" \
    --argjson final_stage "$final_stage" \
    --argjson rolled_back "$rolled_back" \
    --arg overall_status "$overall_status" \
    --slurpfile stages "$STAGES_FILE" \
    '{
      schema_version: $schema_version,
      dry_run: $dry_run,
      current_stage: $current_stage,
      target_stage: $target_stage,
      final_stage: $final_stage,
      rolled_back: $rolled_back,
      overall_status: $overall_status,
      stages: $stages[0]
    }'
}

main "$@"
