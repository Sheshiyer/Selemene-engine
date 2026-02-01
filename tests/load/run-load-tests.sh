#!/bin/bash
# tests/load/run-load-tests.sh
# W2-S8-03: Run all k6 load tests
#
# Usage: ./tests/load/run-load-tests.sh [quick|full]
#   quick - Run smoke tests only (5 minutes)
#   full  - Run full load tests (30+ minutes)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
K6_DIR="$SCRIPT_DIR/k6"
RESULTS_DIR="$K6_DIR/results"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check for required tools
check_requirements() {
    if ! command -v k6 &> /dev/null; then
        echo -e "${RED}Error: k6 is not installed${NC}"
        echo "Install with: brew install k6 (macOS) or see https://k6.io/docs/getting-started/installation/"
        exit 1
    fi
    
    if ! command -v curl &> /dev/null; then
        echo -e "${RED}Error: curl is not installed${NC}"
        exit 1
    fi
}

# Generate JWT token if needed
generate_token() {
    if [ -z "$JWT_TOKEN" ]; then
        echo -e "${YELLOW}No JWT_TOKEN set. Attempting to generate...${NC}"
        
        # Try to run the token generator
        if [ -f "$SCRIPT_DIR/generate_token.rs" ]; then
            cargo run --manifest-path "$SCRIPT_DIR/../../Cargo.toml" --bin generate_test_credentials 2>/dev/null || true
        fi
        
        # If still no token, warn user
        if [ -z "$JWT_TOKEN" ]; then
            echo -e "${YELLOW}Warning: JWT_TOKEN not set. Tests may fail authentication.${NC}"
            echo "Set with: export JWT_TOKEN='your-token'"
        fi
    fi
}

# Check if API is accessible
check_api() {
    local api_url="${API_URL:-http://localhost:8080}"
    echo -e "${YELLOW}Checking API at $api_url ...${NC}"
    
    if curl -s --connect-timeout 5 "$api_url/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ API is accessible${NC}"
    else
        echo -e "${RED}✗ API is not accessible at $api_url${NC}"
        echo "Please ensure the API server is running:"
        echo "  cargo run --release"
        exit 1
    fi
}

# Create results directory
mkdir -p "$RESULTS_DIR"

# Run smoke test
run_smoke_test() {
    echo -e "\n${YELLOW}=== Running Smoke Test ===${NC}"
    k6 run --vus 5 --duration 30s \
        -e API_URL="${API_URL:-http://localhost:8080}" \
        -e JWT_TOKEN="$JWT_TOKEN" \
        "$K6_DIR/engine-load.js" 2>&1 | head -50
}

# Run engine load test
run_engine_load() {
    echo -e "\n${YELLOW}=== Running Engine Load Test ===${NC}"
    echo "Target: 1000 VUs, p95 < 1s"
    k6 run \
        -e API_URL="${API_URL:-http://localhost:8080}" \
        -e JWT_TOKEN="$JWT_TOKEN" \
        "$K6_DIR/engine-load.js"
}

# Run workflow load test
run_workflow_load() {
    echo -e "\n${YELLOW}=== Running Workflow Load Test ===${NC}"
    echo "Target: 500 VUs, p95 < 2s"
    k6 run \
        -e API_URL="${API_URL:-http://localhost:8080}" \
        -e JWT_TOKEN="$JWT_TOKEN" \
        "$K6_DIR/workflow-load.js"
}

# Run full spectrum load test
run_full_spectrum_load() {
    echo -e "\n${YELLOW}=== Running Full Spectrum Load Test ===${NC}"
    echo "Target: 100 VUs, p95 < 5s"
    k6 run \
        -e API_URL="${API_URL:-http://localhost:8080}" \
        -e JWT_TOKEN="$JWT_TOKEN" \
        "$K6_DIR/full-spectrum.js"
}

# Main execution
main() {
    local mode="${1:-quick}"
    
    echo -e "${GREEN}======================================${NC}"
    echo -e "${GREEN}Selemene Engine Load Tests${NC}"
    echo -e "${GREEN}======================================${NC}"
    
    check_requirements
    generate_token
    check_api
    
    case "$mode" in
        quick|smoke)
            echo -e "\n${YELLOW}Running quick smoke tests...${NC}"
            run_smoke_test
            ;;
        full)
            echo -e "\n${YELLOW}Running full load test suite...${NC}"
            echo "This will take approximately 30 minutes."
            
            run_engine_load
            run_workflow_load
            run_full_spectrum_load
            
            echo -e "\n${GREEN}======================================${NC}"
            echo -e "${GREEN}All load tests completed!${NC}"
            echo -e "${GREEN}Results saved to: $RESULTS_DIR${NC}"
            echo -e "${GREEN}======================================${NC}"
            ;;
        engine)
            run_engine_load
            ;;
        workflow)
            run_workflow_load
            ;;
        full-spectrum)
            run_full_spectrum_load
            ;;
        *)
            echo "Usage: $0 [quick|full|engine|workflow|full-spectrum]"
            echo ""
            echo "Modes:"
            echo "  quick         - Run 30s smoke test (default)"
            echo "  full          - Run all load tests (~30 min)"
            echo "  engine        - Run engine load test only"
            echo "  workflow      - Run workflow load test only"
            echo "  full-spectrum - Run full-spectrum load test only"
            exit 1
            ;;
    esac
}

main "$@"
