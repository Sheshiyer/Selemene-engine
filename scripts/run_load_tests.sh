#!/bin/bash
# scripts/run_load_tests.sh
# Orchestrates load testing for Noesis API
#
# Usage:
#   ./scripts/run_load_tests.sh              # Run all scenarios
#   ./scripts/run_load_tests.sh smoke        # Smoke test only
#   ./scripts/run_load_tests.sh scenario1    # Run specific scenario
#   ./scripts/run_load_tests.sh quick        # Quick versions (reduced duration)
#
# Prerequisites:
#   - k6 installed (brew install k6)
#   - Rust toolchain (cargo)

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOAD_TEST_DIR="${PROJECT_ROOT}/tests/load"
RESULTS_DIR="${LOAD_TEST_DIR}/results"
SERVER_PID=""
API_URL="${API_URL:-http://localhost:8080}"
SERVER_PORT="${PORT:-8080}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info()  { echo -e "${BLUE}[INFO]${NC} $*"; }
log_ok()    { echo -e "${GREEN}[OK]${NC} $*"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }

cleanup() {
    if [ -n "$SERVER_PID" ] && kill -0 "$SERVER_PID" 2>/dev/null; then
        log_info "Stopping server (PID: $SERVER_PID)..."
        kill "$SERVER_PID" 2>/dev/null || true
        wait "$SERVER_PID" 2>/dev/null || true
        log_ok "Server stopped."
    fi
}
trap cleanup EXIT

# Create results directory
mkdir -p "$RESULTS_DIR"

# Step 1: Generate JWT token
generate_token() {
    log_info "Generating JWT test token..."

    # Generate token using the credentials binary
    JWT_TOKEN=$(cd "$PROJECT_ROOT" && cargo run --bin generate_test_credentials 2>/dev/null \
        | grep "^Level 5:" | awk '{print $3}')

    if [ -z "$JWT_TOKEN" ]; then
        log_error "Failed to generate JWT token"
        exit 1
    fi

    export JWT_TOKEN
    log_ok "JWT token generated (consciousness level 5, premium tier)"
}

# Step 2: Start server (if not already running)
start_server() {
    # Check if server is already running
    if curl -sf "${API_URL}/health" > /dev/null 2>&1; then
        log_ok "Server already running at ${API_URL}"
        return 0
    fi

    log_info "Building release binary..."
    cd "$PROJECT_ROOT"
    cargo build --release --bin noesis-server 2>&1 | tail -3

    log_info "Starting server on port ${SERVER_PORT}..."
    PORT="$SERVER_PORT" RUST_LOG="warn" LOG_FORMAT="json" \
        ./target/release/noesis-server &
    SERVER_PID=$!

    # Wait for server to be ready
    log_info "Waiting for server to be ready..."
    for i in $(seq 1 30); do
        if curl -sf "${API_URL}/health" > /dev/null 2>&1; then
            log_ok "Server is ready (PID: $SERVER_PID)"
            return 0
        fi
        sleep 1
    done

    log_error "Server failed to start within 30 seconds"
    exit 1
}

# Run a k6 scenario
run_scenario() {
    local name="$1"
    local file="$2"
    local extra_args="${3:-}"

    log_info "Running: ${name}..."
    echo "---"

    k6 run \
        --env "API_URL=${API_URL}" \
        --env "JWT_TOKEN=${JWT_TOKEN}" \
        ${extra_args} \
        "${LOAD_TEST_DIR}/${file}" 2>&1

    local exit_code=$?
    echo "---"

    if [ $exit_code -eq 0 ]; then
        log_ok "${name}: PASSED"
    else
        log_warn "${name}: COMPLETED with threshold violations (exit code: ${exit_code})"
    fi

    return $exit_code
}

# Smoke test
run_smoke() {
    run_scenario "Smoke Test" "smoke-test.js"
}

# Quick versions (reduced duration for CI/dev)
run_quick() {
    log_info "Running quick load tests (reduced durations)..."

    local overall_exit=0

    # Scenario 1: Steady load - 10 VUs for 30s
    run_scenario "Quick Steady Load (10 VUs, 30s)" "scenario1-steady.js" \
        "--vus 10 --duration 30s" || overall_exit=1

    # Scenario 2: Spike - reduced stages
    run_scenario "Quick Spike Test" "scenario2-spike.js" \
        "--vus 20 --duration 30s" || overall_exit=1

    # Scenario 3: Workflows - 5 VUs for 30s
    run_scenario "Quick Workflow Load (5 VUs, 30s)" "scenario3-workflows.js" \
        "--vus 5 --duration 30s" || overall_exit=1

    # Scenario 5: Rate limit - 20s
    run_scenario "Quick Rate Limit Test" "scenario5-ratelimit.js" \
        "--vus 5 --duration 20s" || overall_exit=1

    return $overall_exit
}

# Full test suite
run_all() {
    log_info "Running full load test suite..."
    echo ""

    local overall_exit=0
    local start_time=$(date +%s)

    run_scenario "Scenario 1: Steady Load (100 VUs, 5m)" "scenario1-steady.js" || overall_exit=1
    echo ""

    run_scenario "Scenario 2: Spike Test (10-200-10 VUs)" "scenario2-spike.js" || overall_exit=1
    echo ""

    run_scenario "Scenario 3: Workflow Load (50 VUs, 3m)" "scenario3-workflows.js" || overall_exit=1
    echo ""

    run_scenario "Scenario 4: Cache Performance" "scenario4-cache.js" || overall_exit=1
    echo ""

    run_scenario "Scenario 5: Rate Limiting" "scenario5-ratelimit.js" || overall_exit=1
    echo ""

    local end_time=$(date +%s)
    local total_time=$(( end_time - start_time ))

    echo ""
    echo "================================================"
    echo " Load Test Suite Complete"
    echo " Total time: ${total_time}s"
    echo " Results: ${RESULTS_DIR}/"
    if [ $overall_exit -eq 0 ]; then
        log_ok "All scenarios PASSED"
    else
        log_warn "Some scenarios had threshold violations"
    fi
    echo "================================================"

    return $overall_exit
}

# Main
main() {
    echo ""
    echo "================================================"
    echo " Noesis API Load Test Suite"
    echo " Date: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
    echo " k6 version: $(k6 version 2>&1 | head -1)"
    echo "================================================"
    echo ""

    generate_token
    start_server

    local mode="${1:-all}"

    case "$mode" in
        smoke)
            run_smoke
            ;;
        quick)
            run_quick
            ;;
        scenario1|steady)
            run_scenario "Scenario 1: Steady Load" "scenario1-steady.js"
            ;;
        scenario2|spike)
            run_scenario "Scenario 2: Spike Test" "scenario2-spike.js"
            ;;
        scenario3|workflows)
            run_scenario "Scenario 3: Workflow Load" "scenario3-workflows.js"
            ;;
        scenario4|cache)
            run_scenario "Scenario 4: Cache Performance" "scenario4-cache.js"
            ;;
        scenario5|ratelimit)
            run_scenario "Scenario 5: Rate Limiting" "scenario5-ratelimit.js"
            ;;
        all)
            run_all
            ;;
        *)
            echo "Usage: $0 {smoke|quick|scenario1|scenario2|scenario3|scenario4|scenario5|all}"
            exit 1
            ;;
    esac
}

main "$@"
