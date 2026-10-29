#!/usr/bin/env bash
# Run a local Selemene stack for end-to-end tests (e.g. Sankalpa's Electron
# suite) inside a Linux cloud container or dev machine.
#
#   bash scripts/cloud/local-stack.sh start   # Postgres + ts-engines + API
#   bash scripts/cloud/local-stack.sh status
#   bash scripts/cloud/local-stack.sh stop
#
# The API runs with RUST_ENV=development, so a request carrying
# `x-noesis-dev-auth: $CF_DEV_BYPASS_TOKEN` counts as a browser
# (Cloudflare Access) identity. Never point this at shared data.
#
# Optional: SIDECARS=1 also starts the MediaPipe (8001) and biofield CV
# (8002) Python sidecars in a Python 3.11 venv.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/../.."
repo="$PWD"
run_dir="${LOCAL_STACK_DIR:-$repo/target/local-stack}"
mkdir -p "$run_dir"

export LOCAL_STACK_DB="${LOCAL_STACK_DB:-noesis_e2e}"
export API_PORT="${API_PORT:-8080}"
export TS_PORT="${TS_PORT:-3001}"
export CF_DEV_BYPASS_TOKEN="${CF_DEV_BYPASS_TOKEN:-local-e2e-dev-bypass-token}"

pid_file() { echo "$run_dir/$1.pid"; }

running() {
  local f; f="$(pid_file "$1")"
  [[ -f "$f" ]] && kill -0 "$(cat "$f")" 2>/dev/null
}

wait_for() {
  local name=$1 url=$2 tries=${3:-120}
  for _ in $(seq 1 "$tries"); do
    if curl -fsS -o /dev/null "$url" 2>/dev/null; then echo "ready: $name ($url)"; return 0; fi
    sleep 1
  done
  echo "error: $name did not become ready at $url; see $run_dir/$name.log" >&2
  return 1
}

start_bg() {
  local name=$1; shift
  if running "$name"; then echo "already running: $name"; return 0; fi
  nohup "$@" >"$run_dir/$name.log" 2>&1 &
  echo $! >"$(pid_file "$name")"
}

start() {
  TEST_DB_NAME="$LOCAL_STACK_DB" bash scripts/cloud/test-db.sh

  (cd ts-engines && [[ -d node_modules ]] || bun install --frozen-lockfile)
  PORT="$TS_PORT" start_bg ts-engines bash -c "cd '$repo/ts-engines' && exec bun run src/index.ts"
  wait_for ts-engines "http://127.0.0.1:$TS_PORT/health"

  if [[ "${SIDECARS:-0}" == "1" ]]; then
    local venv="$run_dir/venv"
    [[ -x "$venv/bin/python" ]] || python3.11 -m venv "$venv"
    "$venv/bin/pip" install -q -e "python-services[dev]"
    # Same entry points as python-services/Dockerfile.sidecar.
    start_bg mediapipe bash -c "cd '$repo/python-services' && exec '$venv/bin/python' -m uvicorn mediapipe_service.main:app --host 127.0.0.1 --port 8001"
    start_bg biofield-cv bash -c "cd '$repo/python-services' && exec '$venv/bin/python' -m uvicorn biofield_cv_service.main:app --host 127.0.0.1 --port 8002"
    wait_for mediapipe "http://127.0.0.1:8001/health" 180
    wait_for biofield-cv "http://127.0.0.1:8002/health" 180
  fi

  cargo build -p noesis-api --bin noesis-server
  start_bg api env \
    RUST_ENV=development \
    HOST=127.0.0.1 PORT="$API_PORT" \
    JWT_SECRET="local-e2e-jwt-secret-at-least-32-characters" \
    DATABASE_URL="postgres://postgres@localhost/$LOCAL_STACK_DB" \
    TS_ENGINES_URL="http://127.0.0.1:$TS_PORT" \
    CF_DEV_BYPASS_TOKEN="$CF_DEV_BYPASS_TOKEN" \
    ENGINE_SCOPE_MODE=enforce BILLING_MODE=free \
    PYTHON_BIOFIELD_URL=http://127.0.0.1:8002 SELEMENE_FACE_CV_URL=http://127.0.0.1:8001 \
    SWISS_EPHEMERIS_PATH="$repo/data/ephemeris" \
    "$repo/target/debug/noesis-server"
  wait_for api "http://127.0.0.1:$API_PORT/health" 180

  cat <<EOF
Selemene local stack is up:
  API         http://127.0.0.1:$API_PORT
  ts-engines  http://127.0.0.1:$TS_PORT
  database    postgres://postgres@localhost/$LOCAL_STACK_DB
  dev bypass  x-noesis-dev-auth: $CF_DEV_BYPASS_TOKEN
  logs        $run_dir/*.log
EOF
}

stop() {
  for name in api biofield-cv mediapipe ts-engines; do
    if running "$name"; then kill "$(cat "$(pid_file "$name")")" && echo "stopped: $name"; fi
    rm -f "$(pid_file "$name")"
  done
}

status() {
  for name in ts-engines mediapipe biofield-cv api; do
    if running "$name"; then echo "running: $name"; else echo "stopped: $name"; fi
  done
}

case "${1:-start}" in
  start) start ;;
  stop) stop ;;
  status) status ;;
  *) echo "usage: $0 start|stop|status" >&2; exit 2 ;;
esac
