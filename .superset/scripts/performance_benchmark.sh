#!/bin/bash
# Performance Benchmark - Measure latency and throughput across all 16 engines
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

ITERATIONS=${1:-5}

echo "============================================"
echo "  PERFORMANCE BENCHMARK"
echo "  Production: https://selemene.tryambakam.space"
echo "  Testing latency across all 16 engines"
echo "  ($ITERATIONS iterations each)"
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

declare -A results

for engine in "${ENGINES[@]}"; do
  echo -n "Benchmarking $engine... "
  
  total_time=0
  min_time=999999
  max_time=0
  errors=0
  
  for i in $(seq 1 $ITERATIONS); do
    start=$(date +%s%N)
    
    response=$(curl -s -w "%{http_code}" -o /dev/null \
      -X POST "$API_BASE/engines/$engine/calculate" \
      -H "Content-Type: application/json" \
      -d "$TEST_INPUT" 2>/dev/null)
    
    end=$(date +%s%N)
    duration=$(( (end - start) / 1000000 ))
    
    if [[ "$response" == "200" || "$response" == "201" ]]; then
      total_time=$((total_time + duration))
      if [ $duration -lt $min_time ]; then
        min_time=$duration
      fi
      if [ $duration -gt $max_time ]; then
        max_time=$duration
      fi
    else
      ((errors++))
    fi
  done
  
  if [ $errors -eq $ITERATIONS ]; then
    echo "✗ All requests failed"
  else
    successful=$((ITERATIONS - errors))
    avg_time=$((total_time / successful))
    echo "✓ avg: ${avg_time}ms, min: ${min_time}ms, max: ${max_time}ms"
    
    results[$engine]="$avg_time"
  fi
done

echo ""
echo "============================================"
echo "  Latency Rankings (fastest to slowest)"
echo "============================================"

# Sort by latency
for key in "${!results[@]}"; do
  echo "$key: ${results[$key]}ms"
done | sort -t: -k2 -n

echo ""
echo "============================================"
echo "  Throughput Test"
echo "============================================"

# Quick throughput test - 10 concurrent requests
echo "Testing concurrent request handling (10 parallel)..."

for engine in vimshottari panchanga human-design; do
  start=$(date +%s%N)
  
  for i in {1..10}; do
    curl -s -X POST "$API_BASE/engines/$engine/calculate" \
      -H "Content-Type: application/json" \
      $AUTH_FLAG \
      -d "$TEST_INPUT" > /dev/null 2>&1 &
  done
  wait
  
  end=$(date +%s%N)
  total=$(( (end - start) / 1000000 ))
  throughput=$(( 10000 / total ))
  
  echo "  $engine: ${total}ms total, ~${throughput} req/sec"
done

echo ""
echo "Performance benchmark complete."
