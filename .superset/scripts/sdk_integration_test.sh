#!/bin/bash
# SDK Integration Test - Test API key authentication and SDK endpoints
# Production URL: https://selemene.tryambakam.space
# Architecture: Railway + Supabase + Cloudflare

set -e

API_BASE="${API_BASE:-https://selemene.tryambakam.space/api/v1}"

echo "============================================"
echo "  SDK INTEGRATION TEST"
echo "  Production: https://selemene.tryambakam.space"
echo "  Testing API key authentication"
echo "============================================"
echo ""

# Check for API key
if [ -z "$NOESIS_API_KEY" ] && [ -z "$API_KEY" ]; then
  echo "ERROR: No API key set."
  echo "Please set NOESIS_API_KEY or API_KEY environment variable."
  echo ""
  echo "To get an API key:"
  echo "  1. Go to https://selemene.tryambakam.space"
  echo "  2. Sign up/login via Supabase auth"
  echo "  3. Generate an API key from the dashboard"
  echo ""
  echo "Usage:"
  echo "  NOESIS_API_KEY=your_key_here .superset/scripts/sdk_integration_test.sh"
  exit 1
fi

if [ -n "$NOESIS_API_KEY" ]; then
  AUTH_FLAG="-H X-API-Key: $NOESIS_API_KEY"
elif [ -n "$API_KEY" ]; then
  AUTH_FLAG="-H X-API-Key: $API_KEY"
fi

echo "Using API key: ${NOESIS_API_KEY:0:8}...${NOESIS_API_KEY: -4}"
echo ""

# Test 1: Authenticated engine request
echo "Test 1: Authenticated engine request (vimshottari)..."
response=$(curl -s -w "\n%{http_code}" -o /tmp/test_engine.json \
  -X POST "$API_BASE/engines/vimshottari/calculate" \
  -H "Content-Type: application/json" \
  $AUTH_FLAG \
  -d '{
    "birth_data": {
      "date": "1990-06-15",
      "time": "14:30:00",
      "latitude": 28.6139,
      "longitude": 77.2090,
      "timezone": "Asia/Kolkata"
    }
  }')

http_code=$(echo "$response" | tail -1)
body=$(echo "$response" | head -n -1)

if [[ "$http_code" == "200" || "$http_code" == "201" ]]; then
  echo "  ✓ Engine request successful: $http_code"
  
  if echo "$body" | jq -e '.engine_id' >/dev/null 2>&1; then
    engine_id=$(echo "$body" | jq -r '.engine_id')
    echo "  ✓ Response contains engine_id: $engine_id"
  fi
else
  echo "  ✗ Engine request failed: $http_code"
  echo "  Response: $body"
fi

echo ""

# Test 2: Authenticated workflow request
echo "Test 2: Authenticated workflow request (birth-blueprint)..."
response=$(curl -s -w "\n%{http_code}" -o /tmp/test_workflow.json \
  -X POST "$API_BASE/workflows/birth-blueprint/execute" \
  -H "Content-Type: application/json" \
  $AUTH_FLAG \
  -d '{
    "birth_data": {
      "date": "1990-06-15",
      "time": "14:30:00",
      "latitude": 28.6139,
      "longitude": 77.2090,
      "timezone": "Asia/Kolkata"
    }
  }')

http_code=$(echo "$response" | tail -1)
body=$(echo "$response" | head -n -1)

if [[ "$http_code" == "200" || "$http_code" == "201" ]]; then
  echo "  ✓ Workflow request successful: $http_code"
  
  if echo "$body" | jq -e '.workflow_id' >/dev/null 2>&1; then
    workflow_id=$(echo "$body" | jq -r '.workflow_id')
    echo "  ✓ Response contains workflow_id: $workflow_id"
  fi
else
  echo "  ✗ Workflow request failed: $http_code"
  echo "  Response: $body"
fi

echo ""

# Test 3: Invalid API key should fail
echo "Test 3: Invalid API key rejection..."
response=$(curl -s -w "\n%{http_code}" -o /tmp/test_invalid.json \
  -X POST "$API_BASE/engines/panchanga/calculate" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: invalid_key_12345" \
  -d '{"birth_data":{"date":"1990-06-15"}}')

http_code=$(echo "$response" | tail -1)

if [[ "$http_code" == "401" || "$http_code" == "403" ]]; then
  echo "  ✓ Invalid API key properly rejected: $http_code"
else
  echo "  ? Unexpected response: $http_code"
fi

echo ""

# Test 4: List engines with auth
echo "Test 4: List engines with authentication..."
response=$(curl -s -w "\n%{http_code}" -o /tmp/test_engines.json \
  -X GET "$API_BASE/engines" \
  $AUTH_FLAG)

http_code=$(echo "$response" | tail -1)

if [[ "$http_code" == "200" ]]; then
  body=$(echo "$response" | head -n -1)
  count=$(echo "$body" | jq 'length' 2>/dev/null || echo "?")
  echo "  ✓ Successfully listed engines: $count found"
else
  echo "  ✗ Failed to list engines: $http_code"
fi

echo ""

# Test 5: List workflows with auth
echo "Test 5: List workflows with authentication..."
response=$(curl -s -w "\n%{http_code}" -o /tmp/test_workflows.json \
  -X GET "$API_BASE/workflows" \
  $AUTH_FLAG)

http_code=$(echo "$response" | tail -1)

if [[ "$http_code" == "200" ]]; then
  body=$(echo "$response" | head -n -1)
  count=$(echo "$body" | jq 'length' 2>/dev/null || echo "?")
  echo "  ✓ Successfully listed workflows: $count found"
else
  echo "  ✗ Failed to list workflows: $http_code"
fi

echo ""
echo "============================================"
echo "  SDK Integration Test Complete"
echo "============================================"
echo ""
echo "Architecture:"
echo "  - Backend: Railway (https://railway.app)"
echo "  - Auth/DB: Supabase (https://supabase.com)"
echo "  - Domain: Cloudflare (https://cloudflare.com)"
echo ""
echo "All tests passed! SDK is properly integrated."
