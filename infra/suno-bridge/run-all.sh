#!/usr/bin/env bash
# Suno bridge end-to-end orchestrator with graceful fallbacks.
#
# Runs the five-step deploy → smoke → bulk-gen pipeline. Every external call
# has a timeout. Every step has explicit failure handling — if a step can't
# run yet (e.g. cookie not pasted, vercel not authed), it skips that step
# and reports clearly what's blocking.
#
# Usage:
#   bash infra/suno-bridge/run-all.sh                  # full pipeline
#   bash infra/suno-bridge/run-all.sh --dry-run        # check state only, no work
#   bash infra/suno-bridge/run-all.sh --skip-bulk      # stop after smoke
#   bash infra/suno-bridge/run-all.sh --bulk-only      # just kick off bulk-gen
#   bash infra/suno-bridge/run-all.sh --verbose        # show all subcommand output
#
# State file: infra/suno-bridge/.run-state.json (gitignored)
# Log file:   infra/suno-bridge/run-all.log (gitignored, last run)

set -uo pipefail

# ── Configuration ─────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
STATE_FILE="$SCRIPT_DIR/.run-state.json"
LOG_FILE="$SCRIPT_DIR/run-all.log"
START_TIME=$(date +%s)

# Per-step timeouts (seconds). Tweak via env if needed.
TIMEOUT_QUOTA_CHECK=${TIMEOUT_QUOTA_CHECK:-15}
TIMEOUT_VERCEL_DEPLOY=${TIMEOUT_VERCEL_DEPLOY:-300}        # 5 min
TIMEOUT_MIGRATION=${TIMEOUT_MIGRATION:-60}
TIMEOUT_SMOKE_TEST=${TIMEOUT_SMOKE_TEST:-180}              # 3 min
TIMEOUT_BULK_GEN=${TIMEOUT_BULK_GEN:-1800}                 # 30 min

# Flags
DRY_RUN=0; SKIP_BULK=0; BULK_ONLY=0; VERBOSE=0
for arg in "$@"; do
  case "$arg" in
    --dry-run)   DRY_RUN=1 ;;
    --skip-bulk) SKIP_BULK=1 ;;
    --bulk-only) BULK_ONLY=1 ;;
    --verbose)   VERBOSE=1 ;;
    *) echo "Unknown flag: $arg"; exit 2 ;;
  esac
done

# ── Output helpers ────────────────────────────────────────────────────────
# Color codes (skip if NO_COLOR or non-tty)
if [[ -t 1 && -z "${NO_COLOR:-}" ]]; then
  C_RST=$'\033[0m'; C_RED=$'\033[31m'; C_GRN=$'\033[32m'; C_YLW=$'\033[33m'
  C_BLU=$'\033[34m'; C_DIM=$'\033[2m'; C_BLD=$'\033[1m'
else
  C_RST=''; C_RED=''; C_GRN=''; C_YLW=''; C_BLU=''; C_DIM=''; C_BLD=''
fi

log() { echo "[$(date +%H:%M:%S)] $*" | tee -a "$LOG_FILE"; }
ok()   { echo "${C_GRN}✓${C_RST} $*"; log "OK: $*"; }
warn() { echo "${C_YLW}⚠${C_RST} $*"; log "WARN: $*"; }
err()  { echo "${C_RED}✗${C_RST} $*"; log "ERR: $*"; }
info() { echo "${C_BLU}▸${C_RST} ${C_BLD}$*${C_RST}"; log "INFO: $*"; }
skip() { echo "${C_DIM}⏸ $*${C_RST}"; log "SKIP: $*"; }

# ── Pre-flight helpers ────────────────────────────────────────────────────

# Check that a command exists in PATH.
has_cmd() { command -v "$1" >/dev/null 2>&1; }

# Run a command with a timeout. POSIX `timeout` if available; fallback to background-kill.
# Args: <seconds> <cmd...>
with_timeout() {
  local secs=$1; shift
  if has_cmd timeout; then
    timeout "${secs}s" "$@"
    return $?
  fi
  # macOS fallback (BSD coreutils may not ship `timeout`)
  if has_cmd gtimeout; then
    gtimeout "${secs}s" "$@"
    return $?
  fi
  # Last-resort background kill — silence the SIGTERM noise so output stays clean
  "$@" &
  local pid=$!
  (
    sleep "$secs"
    kill -TERM "$pid" 2>/dev/null
  ) &
  local watcher=$!
  # Disown the watcher so its exit isn't reported as job status
  disown "$watcher" 2>/dev/null || true
  wait "$pid" 2>/dev/null
  local code=$?
  kill -TERM "$watcher" 2>/dev/null
  wait "$watcher" 2>/dev/null
  return "$code"
}

# Run a subcommand silently unless --verbose or it failed.
quiet_run() {
  if (( VERBOSE )); then "$@"; return $?; fi
  local out; out=$("$@" 2>&1)
  local code=$?
  if (( code != 0 )); then echo "$out"; fi
  return "$code"
}

# Read JSON-ish state. If file missing, treat as empty.
state_get() {
  local key=$1
  [[ -f "$STATE_FILE" ]] || { echo ""; return; }
  # Simple grep-based extractor; we control the schema so this is fine.
  grep -oE "\"$key\"\s*:\s*\"[^\"]*\"" "$STATE_FILE" 2>/dev/null \
    | sed -E "s/.*:\s*\"([^\"]*)\"/\1/" | head -1
}

state_set() {
  local key=$1 val=$2
  mkdir -p "$(dirname "$STATE_FILE")"
  if [[ -f "$STATE_FILE" ]] && grep -q "\"$key\"" "$STATE_FILE"; then
    # Replace existing key
    local tmp; tmp=$(mktemp)
    sed -E "s|\"$key\"\s*:\s*\"[^\"]*\"|\"$key\": \"$val\"|" "$STATE_FILE" > "$tmp"
    mv "$tmp" "$STATE_FILE"
  else
    # Append (rebuild for simplicity)
    local kv="\"$key\": \"$val\""
    if [[ -f "$STATE_FILE" ]]; then
      local existing; existing=$(cat "$STATE_FILE" | tr -d '\n' | sed 's/^{//;s/}$//')
      echo "{ $existing, $kv }" > "$STATE_FILE"
    else
      echo "{ $kv }" > "$STATE_FILE"
    fi
  fi
}

# ── Step runner: takes a name + a function; reports outcome to state. ───
step_run() {
  local name=$1 fn=$2
  local prev_status; prev_status=$(state_get "step_${name}")
  if [[ "$prev_status" == "done" ]]; then
    skip "[$name] already completed (per .run-state.json). Re-run with deleted state to redo."
    return 0
  fi
  info "$name — running…"
  if (( DRY_RUN )); then
    skip "[$name] dry-run — would execute now"
    return 0
  fi
  if $fn; then
    ok "[$name] done"
    state_set "step_${name}" "done"
    state_set "step_${name}_at" "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    return 0
  else
    local code=$?
    err "[$name] failed (exit $code) — see next-step hints below"
    state_set "step_${name}" "failed"
    state_set "step_${name}_at" "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    return $code
  fi
}

# ──────────────────────────────────────────────────────────────────────────
# Pre-flight: hard requirements
# ──────────────────────────────────────────────────────────────────────────
echo
info "Pre-flight"

PREFLIGHT_OK=1

check_cmd() {
  local cmd=$1 label=$2
  if has_cmd "$cmd"; then ok "$label installed";
  else err "$label not installed (install via: $3)"; PREFLIGHT_OK=0; fi
}

check_cmd git     "git"     "brew install git"
check_cmd gh      "gh"      "brew install gh"
check_cmd vercel  "vercel"  "npm i -g vercel"
check_cmd wrangler "wrangler" "npm i -g wrangler"
check_cmd pnpm    "pnpm"    "npm i -g pnpm"
check_cmd node    "node"    "see https://nodejs.org"
check_cmd curl    "curl"    "(should be built-in)"

# Auth checks
if has_cmd gh && gh auth status >/dev/null 2>&1; then ok "gh authenticated"; else warn "gh not authenticated — run: gh auth login"; fi
if has_cmd vercel && with_timeout 30 vercel whoami >/dev/null 2>&1; then ok "vercel authenticated"; else warn "vercel not authenticated (or slow auth check) — run: vercel login"; fi
# Wrangler quirk: `wrangler whoami` returns exit 0 even when not authed, just
# prints "Not logged in." to stderr. Inspect output text instead of exit code.
if has_cmd wrangler; then
  wrangler_out=$(with_timeout 30 wrangler whoami 2>&1)
  if echo "$wrangler_out" | grep -qE 'Account ID|You are logged in'; then
    ok "wrangler authenticated"
  else
    warn "wrangler not authenticated — run: wrangler login"
  fi
fi

# Cookie check
if [[ -f "$SCRIPT_DIR/.env" ]] && grep -qE '^SUNO_COOKIE=.+' "$SCRIPT_DIR/.env"; then
  COOKIE_LEN=$(grep -E '^SUNO_COOKIE=' "$SCRIPT_DIR/.env" | cut -d= -f2- | tr -d '"' | wc -c | xargs)
  ok ".env has SUNO_COOKIE (${COOKIE_LEN} chars)"
else
  warn ".env missing or SUNO_COOKIE empty — see infra/suno-bridge/README.md step 2"
  if (( ! BULK_ONLY )); then PREFLIGHT_OK=0; fi
fi

# DATABASE_URL
if [[ -n "${DATABASE_URL:-}" ]]; then ok "DATABASE_URL set (${DATABASE_URL%%@*}@***)";
elif [[ -f "$REPO_ROOT/.env" ]] && grep -q "DATABASE_URL=" "$REPO_ROOT/.env"; then ok "DATABASE_URL in repo .env";
else warn "DATABASE_URL not set — migration step will be skipped"; fi

if (( ! PREFLIGHT_OK && ! DRY_RUN )); then
  echo
  err "Pre-flight failed. Resolve the warnings above and re-run."
  echo "  - Cookie capture: open suno.com → DevTools → Network → copy cookie:"
  echo "  - Paste into:    $SCRIPT_DIR/.env"
  echo "  - Auth fixes:    gh auth login · vercel login · wrangler login"
  exit 1
fi

# ──────────────────────────────────────────────────────────────────────────
# Step 1 — Deploy Suno bridge (clones gcui-art/suno-api + vercel deploy)
# ──────────────────────────────────────────────────────────────────────────
step_deploy_bridge() {
  if [[ ! -x "$SCRIPT_DIR/deploy.sh" ]]; then
    err "deploy.sh missing or not executable"
    return 1
  fi
  log "Running deploy.sh with ${TIMEOUT_VERCEL_DEPLOY}s timeout…"
  if with_timeout "$TIMEOUT_VERCEL_DEPLOY" bash "$SCRIPT_DIR/deploy.sh" 2>&1 | tee -a "$LOG_FILE"; then
    # Extract deployed URL — deploy.sh prints `Bridge URL: https://...`
    local url; url=$(grep -oE 'https://[a-zA-Z0-9.-]+\.vercel\.app' "$LOG_FILE" | tail -1)
    if [[ -n "$url" ]]; then
      state_set "bridge_url" "$url"
      ok "bridge_url: $url"
    fi
    return 0
  fi
  return $?
}

# ──────────────────────────────────────────────────────────────────────────
# Step 2 — Write SUNO_BRIDGE_URL into apps/noesis-web/.env.local
# ──────────────────────────────────────────────────────────────────────────
step_wire_env() {
  local url; url=$(state_get "bridge_url")
  if [[ -z "$url" ]]; then
    warn "no bridge_url in state — did step 1 succeed? Trying to read from vercel directly…"
    url=$(with_timeout 10 vercel ls 2>/dev/null | grep -oE 'https://[a-zA-Z0-9.-]+selemene-suno[a-zA-Z0-9.-]*\.vercel\.app' | head -1)
  fi
  if [[ -z "$url" ]]; then
    err "could not determine SUNO_BRIDGE_URL — set it manually in apps/noesis-web/.env.local"
    return 1
  fi
  local env_local="$REPO_ROOT/apps/noesis-web/.env.local"
  if [[ -f "$env_local" ]] && grep -q "^SUNO_BRIDGE_URL=" "$env_local"; then
    # Replace existing
    local tmp; tmp=$(mktemp)
    sed -E "s|^SUNO_BRIDGE_URL=.*|SUNO_BRIDGE_URL=$url|" "$env_local" > "$tmp"
    mv "$tmp" "$env_local"
    ok "updated SUNO_BRIDGE_URL in $env_local"
  else
    echo "SUNO_BRIDGE_URL=$url" >> "$env_local"
    ok "appended SUNO_BRIDGE_URL to $env_local"
  fi

  # Quick health-check that the bridge actually responds
  log "Health-checking bridge: GET /api/get_limit (${TIMEOUT_QUOTA_CHECK}s timeout)…"
  if with_timeout "$TIMEOUT_QUOTA_CHECK" curl -sf "${url}/api/get_limit" >/dev/null; then
    ok "bridge /api/get_limit responds 200"
    return 0
  else
    err "bridge unreachable or returned non-2xx. Cookie may be invalid — see SUNO_AUTH_RUNBOOK.md"
    return 1
  fi
}

# ──────────────────────────────────────────────────────────────────────────
# Step 3 — Apply migration
# ──────────────────────────────────────────────────────────────────────────
step_migrate() {
  if [[ -z "${DATABASE_URL:-}" ]] && ! grep -q "DATABASE_URL=" "$REPO_ROOT/.env" 2>/dev/null; then
    warn "DATABASE_URL not set — skipping migration step. Apply manually with:"
    echo "    sqlx migrate run --source ./migrations --database-url \"\$DATABASE_URL\""
    return 0  # not a hard failure
  fi
  if ! has_cmd sqlx; then
    warn "sqlx CLI not installed (cargo install sqlx-cli) — skipping. Migration SQL at migrations/028_raga_clips.sql"
    return 0
  fi
  log "Applying migrations (${TIMEOUT_MIGRATION}s timeout)…"
  if with_timeout "$TIMEOUT_MIGRATION" \
      bash -c "cd '$REPO_ROOT' && sqlx migrate run --source ./migrations ${DATABASE_URL:+--database-url \"$DATABASE_URL\"}" 2>&1 | tee -a "$LOG_FILE"; then
    return 0
  else
    err "migration failed — check DATABASE_URL credentials + table conflicts"
    return 1
  fi
}

# ──────────────────────────────────────────────────────────────────────────
# Step 4 — Smoke test (single raga, uses ~10 credits)
# ──────────────────────────────────────────────────────────────────────────
step_smoke() {
  local web_dir="$REPO_ROOT/apps/noesis-web"
  if [[ ! -f "$web_dir/scripts/suno-smoke.ts" ]]; then
    err "suno-smoke.ts not found at $web_dir/scripts/"
    return 1
  fi
  # Install tsx if needed
  if [[ ! -f "$web_dir/node_modules/.bin/tsx" ]]; then
    info "installing tsx (one-time)…"
    quiet_run bash -c "cd '$web_dir' && pnpm install -D tsx" || { err "tsx install failed"; return 1; }
  fi
  log "Running smoke test for melakarta #15 (Mayamalavagaula)…"
  if with_timeout "$TIMEOUT_SMOKE_TEST" \
      bash -c "cd '$web_dir' && pnpm tsx scripts/suno-smoke.ts 15" 2>&1 | tee -a "$LOG_FILE"; then
    local cdn; cdn=$(grep -oE 'https://[a-zA-Z0-9.-]+r2\.dev/clips/[^ ]+\.mp3' "$LOG_FILE" | tail -1)
    if [[ -n "$cdn" ]]; then
      state_set "smoke_cdn_url" "$cdn"
      ok "smoke clip uploaded: $cdn"
    fi
    return 0
  else
    err "smoke test failed (timeout or error). Common causes:"
    echo "    - SUNO_BRIDGE_URL invalid → check ${web_dir}/.env.local"
    echo "    - Suno cookie expired → see SUNO_AUTH_RUNBOOK.md"
    echo "    - Wrangler not authed → run: wrangler login"
    return 1
  fi
}

# ──────────────────────────────────────────────────────────────────────────
# Step 5 — Bulk gen (all 72 ragas, ~30 min, ~144 credits)
# ──────────────────────────────────────────────────────────────────────────
step_bulk() {
  local web_dir="$REPO_ROOT/apps/noesis-web"
  log "Starting bulk gen for ambient style (${TIMEOUT_BULK_GEN}s timeout)…"
  if with_timeout "$TIMEOUT_BULK_GEN" \
      bash -c "cd '$web_dir' && pnpm tsx scripts/suno-bulk-gen.ts ambient" 2>&1 | tee -a "$LOG_FILE"; then
    local done_count; done_count=$(grep -cE '✓.*ambient' "$LOG_FILE" 2>/dev/null || echo 0)
    state_set "bulk_done_count" "$done_count"
    ok "bulk gen complete (~$done_count successful uploads)"
    return 0
  else
    warn "bulk gen interrupted or timed out — re-run is resumable (.suno-checkpoint.json)"
    return 1
  fi
}

# ──────────────────────────────────────────────────────────────────────────
# Orchestration
# ──────────────────────────────────────────────────────────────────────────
: > "$LOG_FILE"   # reset log
echo
info "Orchestrating Suno bridge → smoke → bulk-gen pipeline"
[[ $DRY_RUN -eq 1 ]] && warn "DRY-RUN mode — no work will be done"
[[ $BULK_ONLY -eq 1 ]] && warn "BULK-ONLY mode — skipping steps 1-4"

if (( ! BULK_ONLY )); then
  step_run "1_deploy_bridge"  step_deploy_bridge  || true
  step_run "2_wire_env"       step_wire_env       || true
  step_run "3_migrate"        step_migrate        || true
  step_run "4_smoke_test"     step_smoke          || true
fi

if (( ! SKIP_BULK )); then
  # Only run bulk if smoke passed (or BULK_ONLY explicit)
  if (( BULK_ONLY )) || [[ "$(state_get step_4_smoke_test)" == "done" ]]; then
    step_run "5_bulk_gen" step_bulk || true
  else
    skip "[5_bulk_gen] smoke test didn't pass — refusing to spend 144 credits blindly"
  fi
else
  skip "[5_bulk_gen] --skip-bulk flag set"
fi

# ── Summary ───────────────────────────────────────────────────────────────
echo
echo "${C_BLD}═══ Summary ═══${C_RST}"
for step in 1_deploy_bridge 2_wire_env 3_migrate 4_smoke_test 5_bulk_gen; do
  status=$(state_get "step_${step}")
  case "$status" in
    done)   echo "  ${C_GRN}✓${C_RST} $step" ;;
    failed) echo "  ${C_RED}✗${C_RST} $step" ;;
    "")     echo "  ${C_DIM}⏸ $step (not run)${C_RST}" ;;
    *)      echo "  ${C_YLW}?${C_RST} $step ($status)" ;;
  esac
done
echo
ELAPSED=$(($(date +%s) - START_TIME))
echo "Elapsed: ${ELAPSED}s · State: $STATE_FILE · Log: $LOG_FILE"

# Determine exit code: success only if all attempted steps succeeded
all_ok=1
for step in 1_deploy_bridge 2_wire_env 3_migrate 4_smoke_test 5_bulk_gen; do
  status=$(state_get "step_${step}")
  if [[ "$status" == "failed" ]]; then all_ok=0; fi
done
if (( all_ok )); then
  echo "${C_GRN}${C_BLD}✓ All attempted steps succeeded.${C_RST}"
  echo "Production UI will auto-upgrade as approved clips populate raga_clips."
  exit 0
else
  echo "${C_YLW}⚠ Some steps failed — see log for details. State preserved; re-run to retry.${C_RST}"
  exit 3
fi
