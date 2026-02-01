#!/bin/bash
# tests/chaos/run-chaos-tests.sh
# W2-S8-04: Run chaos engineering tests
#
# Tests graceful degradation under various failure scenarios

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/../.."

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}======================================${NC}"
echo -e "${BLUE}Selemene Engine Chaos Tests${NC}"
echo -e "${BLUE}======================================${NC}"

# Function to test scenario
test_scenario() {
    local name="$1"
    local description="$2"
    local test_cmd="$3"
    
    echo -e "\n${YELLOW}=== Scenario: $name ===${NC}"
    echo -e "${YELLOW}$description${NC}\n"
    
    if eval "$test_cmd"; then
        echo -e "${GREEN}✓ $name passed${NC}"
        return 0
    else
        echo -e "${RED}✗ $name failed${NC}"
        return 1
    fi
}

# Scenario 1: Redis Unavailable
scenario_redis_unavailable() {
    echo "Testing API behavior when Redis is unavailable..."
    
    # Save original REDIS_URL if set
    ORIGINAL_REDIS_URL="${REDIS_URL:-}"
    
    # Point to non-existent Redis
    export REDIS_URL="redis://localhost:63790"  # Wrong port
    
    # Run tests that should still work without cache
    cargo test --manifest-path "$PROJECT_ROOT/Cargo.toml" \
        -p noesis-api --test chaos_tests \
        redis_failure -- --nocapture 2>&1 | head -50
    
    local result=$?
    
    # Restore original
    if [ -n "$ORIGINAL_REDIS_URL" ]; then
        export REDIS_URL="$ORIGINAL_REDIS_URL"
    else
        unset REDIS_URL
    fi
    
    return $result
}

# Scenario 2: TS Engine Unavailable
scenario_ts_unavailable() {
    echo "Testing API behavior when TypeScript engines are unavailable..."
    
    # Save original TS_ENGINE_URL if set
    ORIGINAL_TS_URL="${TS_ENGINE_URL:-}"
    
    # Point to non-existent TS server
    export TS_ENGINE_URL="http://localhost:63001"  # Wrong port
    
    # Run tests for TS engine handling
    cargo test --manifest-path "$PROJECT_ROOT/Cargo.toml" \
        -p noesis-api --test chaos_tests \
        ts_engine_failure -- --nocapture 2>&1 | head -50
    
    local result=$?
    
    # Restore original
    if [ -n "$ORIGINAL_TS_URL" ]; then
        export TS_ENGINE_URL="$ORIGINAL_TS_URL"
    else
        unset TS_ENGINE_URL
    fi
    
    return $result
}

# Scenario 3: Ephemeris Files Missing
scenario_ephemeris_missing() {
    echo "Testing API behavior when ephemeris files are missing..."
    
    # Save original path
    ORIGINAL_EPHE_PATH="${SWISS_EPHEMERIS_PATH:-}"
    
    # Point to non-existent path
    export SWISS_EPHEMERIS_PATH="/nonexistent/path/ephemeris"
    
    # Run ephemeris fallback tests
    cargo test --manifest-path "$PROJECT_ROOT/Cargo.toml" \
        -p noesis-api --test chaos_tests \
        ephemeris_failure -- --nocapture 2>&1 | head -50
    
    local result=$?
    
    # Restore original
    if [ -n "$ORIGINAL_EPHE_PATH" ]; then
        export SWISS_EPHEMERIS_PATH="$ORIGINAL_EPHE_PATH"
    else
        unset SWISS_EPHEMERIS_PATH
    fi
    
    return $result
}

# Scenario 4: High Load Pressure
scenario_high_load() {
    echo "Testing API behavior under high concurrent load..."
    
    cargo test --manifest-path "$PROJECT_ROOT/Cargo.toml" \
        -p noesis-api --test chaos_tests \
        resource_pressure -- --nocapture 2>&1 | head -50
    
    return $?
}

# Scenario 5: Edge Cases
scenario_edge_cases() {
    echo "Testing API behavior with edge case inputs..."
    
    cargo test --manifest-path "$PROJECT_ROOT/Cargo.toml" \
        -p noesis-api --test chaos_tests \
        edge_cases -- --nocapture 2>&1 | head -50
    
    return $?
}

# Run all scenarios
run_all_scenarios() {
    local passed=0
    local failed=0
    
    test_scenario "Redis Unavailable" \
        "Tests graceful degradation when Redis cache is unavailable" \
        "scenario_redis_unavailable" && ((passed++)) || ((failed++))
    
    test_scenario "TS Engine Unavailable" \
        "Tests handling when TypeScript engine server is down" \
        "scenario_ts_unavailable" && ((passed++)) || ((failed++))
    
    test_scenario "Ephemeris Missing" \
        "Tests fallback when Swiss Ephemeris files are missing" \
        "scenario_ephemeris_missing" && ((passed++)) || ((failed++))
    
    test_scenario "High Load" \
        "Tests system behavior under concurrent request pressure" \
        "scenario_high_load" && ((passed++)) || ((failed++))
    
    test_scenario "Edge Cases" \
        "Tests handling of boundary values and unusual inputs" \
        "scenario_edge_cases" && ((passed++)) || ((failed++))
    
    echo -e "\n${BLUE}======================================${NC}"
    echo -e "${BLUE}Chaos Test Results${NC}"
    echo -e "${BLUE}======================================${NC}"
    echo -e "${GREEN}Passed: $passed${NC}"
    echo -e "${RED}Failed: $failed${NC}"
    
    if [ $failed -eq 0 ]; then
        echo -e "\n${GREEN}All chaos scenarios passed!${NC}"
        return 0
    else
        echo -e "\n${RED}Some chaos scenarios failed${NC}"
        return 1
    fi
}

# Main
case "${1:-all}" in
    all)
        run_all_scenarios
        ;;
    redis)
        test_scenario "Redis Unavailable" "" "scenario_redis_unavailable"
        ;;
    ts)
        test_scenario "TS Engine Unavailable" "" "scenario_ts_unavailable"
        ;;
    ephemeris)
        test_scenario "Ephemeris Missing" "" "scenario_ephemeris_missing"
        ;;
    load)
        test_scenario "High Load" "" "scenario_high_load"
        ;;
    edge)
        test_scenario "Edge Cases" "" "scenario_edge_cases"
        ;;
    *)
        echo "Usage: $0 [all|redis|ts|ephemeris|load|edge]"
        exit 1
        ;;
esac
