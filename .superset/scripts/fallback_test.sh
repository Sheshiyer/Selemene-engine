#!/bin/bash
# Graceful Fallback Testing - Verify error handling and degradation behavior
# Production URL: https://selemene.tryambakam.space

set -e

API_BASE="${API_BASE:-https://selemene.tryambakam.space/api/v1}"

echo "============================================"
echo "  GRACEFUL FALLBACK TEST"
echo "  Production: https://selemene.tryambakam.space"
echo "  Testing error handling and degradation"
echo "============================================"
echo ""

# Check for API key
if [ -n "$NOESIS_API_KEY" ]; then
  AUTH_FLAG="-H X-API-Key: $NOESIS_API_KEY"
elif [ -n "$API_KEY" ]; then
  AUTH_FLAG="-H X-API-Key: $API_KEY"
else
  AUTH_FLAG=""
  echo "Note: No API key set. Some tests may fail."
fi

# Test 1: Service unavailability
echo "Test 1: Service unavailability scenarios"
echo "  (Assumes you can simulate service downtime)"

# Test 2: Invalid engine name - should return helpful error
echo ""
echo "Test 2: Invalid engine name handling..."
response=$(curl -s -w "\n%{http_code}" -X POST "$API_BASE/engines/nonexistent-engine/calculate" \
  -H "Content-Type: application/json" \
  -d '{"birth_data":{"date":"1990-06-15","time":"14:30:00"}}')

http_code=$(echo "$response" | tail -1)
body=$(echo "$response" | head -n -1)

if [[ "$http_code" == "404" ]]; then
  echo "  ✓ Invalid engine returns 404"
  if echo "$body" | grep -qi "available\|engine"; then
    echo "  ✓ Error message is helpful"
  fi
else
  echo "  ✗ Unexpected response: $http_code"
fi

# Test 3: Malformed JSON - should not crash
echo ""
echo "Test 3: Malformed JSON handling..."
response=$(curl -s -w "\n%{http_code}" -X POST "$API_BASE/engines/panchanga/calculate" \
  -H "Content-Type: application/json" \
  -d 'not valid json at all')

if [[ "$http_code" == "400" || "$http_code" == "422" ]]; then
  echo "  ✓ Malformed JSON properly rejected: $http_code"
else
  echo "  ? Unexpected: $http_code"
fi

# Test 4: Missing required fields - graceful validation
echo ""
echo "Test 4: Missing required fields..."
response=$(curl -s -w "\n%{http_code}" -X POST "$API_BASE/engines/panchanga/calculate" \
  -H "Content-Type: application/json" \
  -d '{"birth_data":{"date":"1990-06-15"}}')  # missing time, lat, lon

http_code=$(echo "$response" | tail -1)
if [[ "$http_code" == "400" || "$http_code" == "422" ]]; then
  echo "  ✓ Missing fields properly validated: $http_code"
else
  echo "  ? Unexpected: $http_code"
fi

# Test 5: Concurrent requests - should handle gracefully
echo ""
echo "Test 5: Concurrent request handling..."
for i in {1..5}; do
  curl -s -X POST "$API_BASE/engines/vimshottari/calculate" \
    -H "Content-Type: application/json" \
    -d '{"birth_data":{"date":"1990-06-15","time":"14:30:00","latitude":28.6,"longitude":77.2}}' &
done
wait

echo "  ✓ Concurrent requests completed"

# Test 6: Large payload - should handle or reject gracefully
echo ""
echo "Test 6: Large payload handling..."
large_payload=$(jq -n '{birth_data: {date: "1990-06-15", time: "14:30:00", latitude: 28.6, longitude: 77.2}, extra_data: '$(jq -n '{}' | head -c 10000)') 2>/dev/null || \
large_payload='{"birth_data":{"date":"1990-06-15","time":"14:30:00","latitude":28.6,"longitude":77.2},"extra":"'$(printf 'x%.0s' {1..10000})'"}'

response=$(curl -s -w "\n%{http_code}" -X POST "$API_BASE/engines/panchanga/calculate" \
  -H "Content-Type: application/json" \
  --max-time 10 \
  -d "$large_payload")

http_code=$(echo "$response" | tail -1)
if [[ "$http_code" == "413" || "$http_code" == "400" || "$http_code" == "200" ]]; then
  echo "  ✓ Large payload handled: $http_code"
else
  echo "  ? Response: $http_code"
fi

# Test 7: Timeout handling
echo ""
echo "Test 7: Request timeout handling..."
start=$(date +%s%N)
response=$(curl -s -w "\n%{http_code}" --max-time 5 \
  -X POST "$API_BASE/engines/panchanga/calculate" \
  -H "Content-Type: application/json" \
  -d '{"birth_data":{"date":"1990-06-15","time":"14:30:00","latitude":28.6,"longitude":77.2}}')
end=$(date +%s%N)

duration=$(( (end - start) / 1000000 ))
echo "  Request took ${duration}ms"

if [[ "$response" == *"000"* ]]; then
  echo "  ✓ Timeout handled (connection failed)"
else
  http_code=$(echo "$response" | tail -1)
  echo "  Response: $http_code"
fi

echo ""
echo "============================================"
echo "  Fallback Test Complete"
echo "============================================"
