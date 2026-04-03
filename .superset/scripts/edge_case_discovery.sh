#!/bin/bash
# Edge Case Discovery - Find boundary conditions and failure modes across all 16 engines
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

EDGE_CASES=(
  # Extreme dates
  '{"birth_data":{"date":"1900-01-01","time":"00:00:00","latitude":0,"longitude":0,"timezone":"UTC"}}'
  '{"birth_data":{"date":"2100-12-31","time":"23:59:59","latitude":90,"longitude":180,"timezone":"Pacific/Apia"}}'
  
  # Null/undefined fields
  '{"birth_data":{"date":null,"time":"14:30:00","latitude":28.6,"longitude":77.2}}'
  '{"birth_data":{"date":"1990-06-15","time":null,"latitude":28.6,"longitude":77.2}}'
  
  # Extreme coordinates
  '{"birth_data":{"date":"1990-06-15","time":"14:30:00","latitude":-90,"longitude":-180,"timezone":"UTC"}}'
  '{"birth_data":{"date":"1990-06-15","time":"14:30:00","latitude":90,"longitude":180,"timezone":"UTC"}}'
  
  # Invalid but plausible values
  '{"birth_data":{"date":"1990-06-15","time":"14:30:00","latitude":28.6139,"longitude":77.2090,"timezone":"Invalid/Timezone"}}'
  
  # Empty strings
  '{"birth_data":{"date":"","time":"14:30:00","latitude":28.6,"longitude":77.2}}'
  '{"birth_data":{"date":"1990-06-15","time":"","latitude":28.6,"longitude":77.2}}'
  
  # Very long strings
  '{"birth_data":{"date":"1990-06-15","time":"14:30:00","latitude":28.6,"longitude":77.2,"timezone":"'$(printf 'A%.0s' {1..500})'"}}'
)

echo "============================================"
echo "  EDGE CASE DISCOVERY"
echo "  Testing boundary conditions & failure modes"
echo "============================================"
echo ""

found_issues=()

for engine in "${ENGINES[@]}"; do
  echo "Testing $engine..."
  
  for i in "${!EDGE_CASES[@]}"; do
    case_num=$((i+1))
    
    response=$(curl -s -w "\n%{http_code}" -X POST "$API_BASE/engines/$engine/calculate" \
      -H "Content-Type: application/json" \
      -d "${EDGE_CASES[$i]}" 2>/dev/null)
    
    http_code=$(echo "$response" | tail -1)
    body=$(echo "$response" | head -n -1)
    
    # Check for interesting behaviors
    if [[ "$http_code" == "500" ]]; then
      echo "  └─ Case $case_num: 500 Internal Error"
      found_issues+=("$engine: case $case_num returned 500")
    elif [[ "$http_code" == "200" ]]; then
      # Check if result contains warnings or fallbacks
      if echo "$body" | grep -qi "fallback\|warning\|error\|unknown"; then
        echo "  └─ Case $case_num: Graceful degradation detected"
      fi
    elif [[ "$http_code" == "422" ]]; then
      # Good - proper validation
      :
    elif [[ "$http_code" == "000" ]]; then
      echo "  └─ Case $case_num: Connection failed (engine down?)"
    fi
  done
done

echo ""
echo "============================================"
echo "  Summary"
echo "============================================"

if [ ${#found_issues[@]} -eq 0 ]; then
  echo "✓ All edge cases handled gracefully"
else
  echo "Found ${#found_issues[@]} potential issues:"
  for issue in "${found_issues[@]}"; do
    echo "  - $issue"
  done
fi

# Test timezone edge cases specifically
echo ""
echo "Testing timezone edge cases..."
tz_cases=(
  "Pacific/Apia"  # Day before
  "Pacific/Honolulu"  # Very early
  "Asia/Kolkata"  # Common
  "Europe/London"  # DST boundary
  "Antarctica/DumontDUrville"  # Weird timezone
)

for tz in "${tz_cases[@]}"; do
  response=$(curl -s -w "%{http_code}" -o /tmp/tz_$tz.json \
    -X POST "$API_BASE/engines/panchanga/calculate" \
    -H "Content-Type: application/json" \
    -d '{"birth_data":{"date":"1990-06-15","time":"14:30:00","latitude":28.6,"longitude":77.2,"timezone":"'$tz'"}}')
  echo "  $tz: $response"
done

echo ""
echo "Edge case discovery complete."
