#!/bin/bash
# Consistency Check - Validate response structure across all 16 engines
# Production URL: https://selemene.tryambakam.space

set -e

API_BASE="${API_BASE:-https://selemene.tryambakam.space/api/v1}"

# All 16 engines
ENGINES=(
  "vimshottari"
  "human-design"
  "gene-keys"
  "transits"
  "vedic-clock"
  "panchanga"
  "biofield"
  "biorhythm"
  "face-reading"
  "numerology"
  "nadabrahman"
  "enneagram"
  "i-ching"
  "sacred-geometry"
  "sigil-forge"
  "tarot"
)

TEST_INPUT='{
  "birth_data": {
    "date": "1990-06-15",
    "time": "14:30:00",
    "latitude": 28.6139,
    "longitude": 77.2090,
    "timezone": "Asia/Kolkata"
  }
}'

echo "============================================"
echo "  ENGINE CONSISTENCY CHECK"
echo "  Production: https://selemene.tryambakam.space"
echo "  Validating response structure uniformity"
echo "============================================"
echo ""

# Check for API key
if [ -n "$NOESIS_API_KEY" ]; then
  AUTH_FLAG="-H X-API-Key: $NOESIS_API_KEY"
elif [ -n "$API_KEY" ]; then
  AUTH_FLAG="-H X-API-Key: $API_KEY"
else
  AUTH_FLAG=""
fi

# Expected fields that should be in every response
EXPECTED_FIELDS=(
  "engine_id"
  "status"
  "result"
)

echo "Testing response structure consistency..."
echo ""

inconsistent=()
consistent=0

for engine in "${ENGINES[@]}"; do
  response=$(curl -s -X POST "$API_BASE/engines/$engine/calculate" \
    -H "Content-Type: application/json" \
    -d "$TEST_INPUT" 2>/dev/null)
  
  if [ -z "$response" ] || [ "$response" == "null" ]; then
    echo "$engine: ✗ No response"
    inconsistent+=("$engine: no response")
    continue
  fi
  
  # Check for engine_id
  if echo "$response" | jq -e '.engine_id' >/dev/null 2>&1; then
    has_engine_id=1
  else
    has_engine_id=0
  fi
  
  # Check for status
  if echo "$response" | jq -e '.status' >/dev/null 2>&1; then
    has_status=1
  else
    has_status=0
  fi
  
  # Check for result
  if echo "$response" | jq -e '.result' >/dev/null 2>&1; then
    has_result=1
  else
    has_result=0
  fi
  
  # Check metadata present
  if echo "$response" | jq -e '.metadata' >/dev/null 2>&1; then
    has_metadata=1
  else
    has_metadata=0
  fi
  
  all_present=$((has_engine_id + has_status + has_result))
  
  if [ $all_present -eq 3 ]; then
    echo "$engine: ✓"
    ((consistent++))
  else
    echo "$engine: ✗ Missing fields (engine_id:$has_engine_id, status:$has_status, result:$has_result, metadata:$has_metadata)"
    inconsistent+=("$engine: missing standard fields")
  fi
done

echo ""
echo "============================================"
echo "  Structure Analysis"
echo "============================================"

# Check status value consistency
echo ""
echo "Checking status value consistency..."
status_values=$(echo "" | jq -n '[]')
for engine in "${ENGINES[@]}"; do
  status=$(curl -s -X POST "$API_BASE/engines/$engine/calculate" \
    -H "Content-Type: application/json" \
    -d "$TEST_INPUT" 2>/dev/null | jq -r '.status' 2>/dev/null || echo "error")
  echo "  $engine: $status"
done

# Check if metadata timing is present
echo ""
echo "Checking metadata completeness..."
for engine in "${ENGINES[@]}"; do
  metadata=$(curl -s -X POST "$API_BASE/engines/$engine/calculate" \
    -H "Content-Type: application/json" \
    -d "$TEST_INPUT" 2>/dev/null | jq -c '.metadata' 2>/dev/null)
  
  has_timing=$(echo "$metadata" | jq 'has("timing_ms") or has("timestamp")' 2>/dev/null || echo "false")
  echo "  $engine timing: $has_timing"
done

echo ""
echo "============================================"
echo "  Results: $consistent engines consistent"
echo "============================================"

if [ ${#inconsistent[@]} -gt 0 ]; then
  echo ""
  echo "Inconsistent engines:"
  for issue in "${inconsistent[@]}"; do
    echo "  - $issue"
  done
fi

# Summary
echo ""
echo "Consistency check complete."
