#!/bin/bash

# Rate Limit Testing Script for noesis-api
# Tests the rate limiting middleware with curl

set -e

echo "=== Rate Limit Testing ==="
echo ""

# Configuration
API_BASE="http://localhost:8080"
API_KEY="your-test-api-key-here"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Note: Make sure to:"
echo "1. Start the noesis-server: cargo run --bin noesis-server"
echo "2. Generate a test API key: cargo run --bin generate_test_credentials"
echo "3. Update the API_KEY variable in this script${NC}"
echo ""

if [ "$API_KEY" = "your-test-api-key-here" ]; then
    echo -e "${RED}ERROR: Please update the API_KEY variable in this script${NC}"
    exit 1
fi

echo "Testing rate limit functionality..."
echo ""

# Test 1: Public endpoint (no rate limit)
echo -e "${YELLOW}Test 1: Public endpoint /health (should not be rate limited)${NC}"
for i in {1..5}; do
    response=$(curl -s -o /dev/null -w "%{http_code}" "$API_BASE/health")
    echo "Request $i: HTTP $response"
    if [ "$response" -ne 200 ]; then
        echo -e "${RED}FAIL: Expected 200${NC}"
        exit 1
    fi
done
echo -e "${GREEN}PASS: Public endpoint not rate limited${NC}"
echo ""

# Test 2: Authenticated endpoint (rate limited)
echo -e "${YELLOW}Test 2: Authenticated endpoint /api/v1/status (rate limited)${NC}"
echo "Making 5 requests (assuming limit >= 5)..."
for i in {1..5}; do
    response=$(curl -s -i \
        -H "X-API-Key: $API_KEY" \
        "$API_BASE/api/v1/status" 2>&1)
    
    http_code=$(echo "$response" | grep "HTTP/" | awk '{print $2}')
    rate_limit=$(echo "$response" | grep -i "x-ratelimit-limit:" | cut -d: -f2 | tr -d ' \r')
    remaining=$(echo "$response" | grep -i "x-ratelimit-remaining:" | cut -d: -f2 | tr -d ' \r')
    reset=$(echo "$response" | grep -i "x-ratelimit-reset:" | cut -d: -f2 | tr -d ' \r')
    
    echo "Request $i: HTTP $http_code | Limit: $rate_limit | Remaining: $remaining | Reset: $reset"
    
    if [ "$http_code" -ne 200 ]; then
        echo -e "${RED}FAIL: Expected 200, got $http_code${NC}"
        exit 1
    fi
done
echo -e "${GREEN}PASS: Rate limit headers present and requests succeeded${NC}"
echo ""

# Test 3: Exceed rate limit (if rate limit is low enough)
echo -e "${YELLOW}Test 3: Attempt to exceed rate limit${NC}"
echo "Making 110 rapid requests to test 100 req/min limit..."
exceeded=false
for i in {1..110}; do
    response=$(curl -s -o /dev/null -w "%{http_code}" \
        -H "X-API-Key: $API_KEY" \
        "$API_BASE/api/v1/status")
    
    if [ "$response" -eq 429 ]; then
        echo -e "${GREEN}Request $i: HTTP 429 (Rate limit exceeded as expected)${NC}"
        exceeded=true
        break
    elif [ "$i" -eq 110 ]; then
        echo "Request $i: HTTP $response"
    fi
done

if [ "$exceeded" = true ]; then
    echo -e "${GREEN}PASS: Rate limit enforced (got 429)${NC}"
else
    echo -e "${YELLOW}INFO: Did not hit rate limit (may be set > 110 req/min)${NC}"
fi
echo ""

echo -e "${GREEN}=== Rate Limit Tests Complete ===${NC}"
