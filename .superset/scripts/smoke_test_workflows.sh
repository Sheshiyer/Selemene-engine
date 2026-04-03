#!/bin/bash
# Smoke Test for All 6 Workflows
# Tests each workflow endpoint with valid inputs
# Production URL: https://selemene.tryambakam.space

set -e

API_BASE="${API_BASE:-https://selemene.tryambakam.space/api/v1}"

# All 6 workflows
WORKFLOWS=(
  "birth-blueprint"
  "creative-expression"
  "daily-practice"
  "decision-support"
  "full-spectrum"
  "self-inquiry"
)

# Test data
VALID_INPUTS='{
  "birth_data": {
    "date": "1990-06-15",
    "time": "14:30:00",
    "latitude": 28.6139,
    "longitude": 77.2090,
    "timezone": "Asia/Kolkata"
  },
  "options": {
    "precision": "Standard"
  }
}'

echo "============================================"
echo "  SMOKE TEST: All 6 Workflows"
echo "  Production: https://selemene.tryambakam.space"
echo "============================================"
echo ""

# Check for API key
if [ -n "$NOESIS_API_KEY" ]; then
  AUTH_FLAG="-H X-API-Key: $NOESIS_API_KEY"
  echo "Using X-API-Key authentication"
elif [ -n "$API_KEY" ]; then
  AUTH_FLAG="-H X-API-Key: $API_KEY"
  echo "Using X-API-Key authentication (API_KEY)"
else
  echo "Warning: No API key set. Set NOESIS_API_KEY or API_KEY."
  AUTH_FLAG=""
fi

passed=0
failed=0

for workflow in "${WORKFLOWS[@]}"; do
  echo -n "Testing $workflow... "
  
  response=$(curl -s -w "%{http_code}" -o /tmp/workflow_$workflow.json \
    -X POST "$API_BASE/workflows/$workflow/execute" \
    -H "Content-Type: application/json" \
    $AUTH_FLAG \
    -d "$VALID_INPUTS" 2>/dev/null || echo "000")
  
  http_code=$(echo "$response")
  
  if [[ "$http_code" == "200" || "$http_code" == "201" ]]; then
    echo "✓ ($http_code)"
    ((passed++))
  elif [[ "$http_code" == "401" || "$http_code" == "403" ]]; then
    echo "⚠ (auth required: $http_code)"
    ((failed++))
  elif [[ "$http_code" == "404" ]]; then
    echo "⚠ (not found: $http_code)"
    ((failed++))
  else
    echo "✗ ($http_code)"
    ((failed++))
  fi
done

echo ""
echo "============================================"
echo "  Results: $passed passed, $failed failed"
echo "============================================"
echo ""
echo "To run with your API key:"
echo "  NOESIS_API_KEY=your_key .superset/scripts/smoke_test_workflows.sh"

exit $failed
