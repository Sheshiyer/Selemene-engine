#!/bin/bash

# Selemene Engine Performance Benchmarking Script
set -e

echo "🚀 Starting Selemene Engine performance benchmarks..."

# Check if the engine is running
ENGINE_URL="http://localhost:8080"
HEALTH_URL="$ENGINE_URL/health"

echo "🏥 Checking engine health..."
if ! curl -f "$HEALTH_URL" &> /dev/null; then
    echo "❌ Engine is not running. Please start the engine first:"
    echo "cargo run"
    exit 1
fi

echo "✅ Engine is healthy and running"

# Set benchmark parameters
BENCHMARK_ITERATIONS=100
CONCURRENT_REQUESTS=10
BENCHMARK_DURATION=60

echo "📊 Benchmark Parameters:"
echo "  - Iterations: $BENCHMARK_ITERATIONS"
echo "  - Concurrent Requests: $CONCURRENT_REQUESTS"
echo "  - Duration: ${BENCHMARK_DURATION}s"

# Create test data
echo "📝 Creating test data..."
TEST_DATA=$(cat <<EOF
{
  "date": "2025-01-27",
  "coordinates": {
    "latitude": 19.0760,
    "longitude": 72.8777
  },
  "precision": "Standard",
  "timezone": null
}
EOF
)

# Benchmark single calculation
echo "🧮 Benchmarking single calculation..."
SINGLE_START=$(date +%s.%N)

for i in $(seq 1 $BENCHMARK_ITERATIONS); do
    curl -s -X POST "$ENGINE_URL/api/v1/panchanga" \
        -H "Content-Type: application/json" \
        -d "$TEST_DATA" > /dev/null
    
    if [ $((i % 10)) -eq 0 ]; then
        echo "  Completed $i/$BENCHMARK_ITERATIONS iterations"
    fi
done

SINGLE_END=$(date +%s.%N)
SINGLE_DURATION=$(echo "$SINGLE_END - $SINGLE_START" | bc -l)
SINGLE_AVERAGE=$(echo "$SINGLE_DURATION / $BENCHMARK_ITERATIONS" | bc -l)

echo "✅ Single calculation benchmark completed:"
echo "  - Total time: ${SINGLE_DURATION}s"
echo "  - Average time: ${SINGLE_AVERAGE}s"
echo "  - Throughput: $(echo "$BENCHMARK_ITERATIONS / $SINGLE_DURATION" | bc -l) req/s"

# Benchmark batch calculations
echo "📦 Benchmarking batch calculations..."
BATCH_START=$(date +%s.%N)

# Create batch request
BATCH_DATA=$(cat <<EOF
{
  "requests": [
    {
      "date": "2025-01-27",
      "coordinates": {"latitude": 19.0760, "longitude": 72.8777},
      "precision": "Standard"
    },
    {
      "date": "2025-06-15",
      "coordinates": {"latitude": 28.6139, "longitude": 77.2090},
      "precision": "Standard"
    },
    {
      "date": "2025-12-21",
      "coordinates": {"latitude": 12.9716, "longitude": 77.5946},
      "precision": "Standard"
    }
  ]
}
EOF
)

for i in $(seq 1 $((BENCHMARK_ITERATIONS / 3))); do
    curl -s -X POST "$ENGINE_URL/api/v1/panchanga/batch" \
        -H "Content-Type: application/json" \
        -d "$BATCH_DATA" > /dev/null
    
    if [ $((i % 5)) -eq 0 ]; then
        echo "  Completed $i/$((BENCHMARK_ITERATIONS / 3)) batch iterations"
    fi
done

BATCH_END=$(date +%s.%N)
BATCH_DURATION=$(echo "$BATCH_END - $BATCH_START" | bc -l)
BATCH_AVERAGE=$(echo "$BATCH_DURATION / ($BENCHMARK_ITERATIONS / 3)" | bc -l)

echo "✅ Batch calculation benchmark completed:"
echo "  - Total time: ${BATCH_DURATION}s"
echo "  - Average time: ${BATCH_AVERAGE}s"
echo "  - Throughput: $(echo "($BENCHMARK_ITERATIONS / 3) / $BATCH_DURATION" | bc -l) batch/s"

# Benchmark cache performance
echo "💾 Benchmarking cache performance..."
CACHE_START=$(date +%s.%N)

# First request (cache miss)
curl -s -X POST "$ENGINE_URL/api/v1/panchanga" \
    -H "Content-Type: application/json" \
    -d "$TEST_DATA" > /dev/null

# Second request (cache hit)
curl -s -X POST "$ENGINE_URL/api/v1/panchanga" \
    -H "Content-Type: application/json" \
    -d "$TEST_DATA" > /dev/null

CACHE_END=$(date +%s.%N)
CACHE_DURATION=$(echo "$CACHE_END - $CACHE_START" | bc -l)

echo "✅ Cache performance benchmark completed:"
echo "  - Cache test duration: ${CACHE_DURATION}s"

# Benchmark concurrent requests
echo "⚡ Benchmarking concurrent requests..."
CONCURRENT_START=$(date +%s.%N)

# Use parallel to make concurrent requests
for i in $(seq 1 $CONCURRENT_REQUESTS); do
    (
        curl -s -X POST "$ENGINE_URL/api/v1/panchanga" \
            -H "Content-Type: application/json" \
            -d "$TEST_DATA" > /dev/null
        echo "Request $i completed"
    ) &
done

wait
CONCURRENT_END=$(date +%s.%N)
CONCURRENT_DURATION=$(echo "$CONCURRENT_END - $CONCURRENT_START" | bc -l)

echo "✅ Concurrent requests benchmark completed:"
echo "  - Total time: ${CONCURRENT_DURATION}s"
echo "  - Concurrent requests: $CONCURRENT_REQUESTS"

# Get performance metrics
echo "📈 Fetching performance metrics..."
METRICS_RESPONSE=$(curl -s "$ENGINE_URL/metrics")
if [ $? -eq 0 ]; then
    echo "✅ Metrics endpoint responding"
    echo "  - Metrics size: $(echo "$METRICS_RESPONSE" | wc -c) bytes"
else
    echo "⚠️ Metrics endpoint not responding"
fi

# Performance summary
echo ""
echo "🎯 Performance Benchmark Summary"
echo "================================"
echo "Single Calculation:"
echo "  - Total time: ${SINGLE_DURATION}s"
echo "  - Average time: ${SINGLE_AVERAGE}s"
echo "  - Throughput: $(echo "$BENCHMARK_ITERATIONS / $SINGLE_DURATION" | bc -l) req/s"
echo ""
echo "Batch Calculations:"
echo "  - Total time: ${BATCH_DURATION}s"
echo "  - Average time: ${BATCH_AVERAGE}s"
echo "  - Throughput: $(echo "($BENCHMARK_ITERATIONS / 3) / $BATCH_DURATION" | bc -l) batch/s"
echo ""
echo "Concurrent Performance:"
echo "  - Concurrent requests: $CONCURRENT_REQUESTS"
echo "  - Total concurrent time: ${CONCURRENT_DURATION}s"
echo ""
echo "Cache Performance:"
echo "  - Cache test duration: ${CACHE_DURATION}s"
echo ""

# Performance recommendations
echo "💡 Performance Recommendations:"
if (( $(echo "$SINGLE_AVERAGE < 0.001" | bc -l) )); then
    echo "  ✅ Single calculation performance is excellent (< 1ms)"
elif (( $(echo "$SINGLE_AVERAGE < 0.01" | bc -l) )); then
    echo "  ✅ Single calculation performance is good (< 10ms)"
else
    echo "  ⚠️ Single calculation performance could be improved (> 10ms)"
fi

if (( $(echo "$BATCH_AVERAGE < 0.01" | bc -l) )); then
    echo "  ✅ Batch calculation performance is excellent (< 10ms)"
elif (( $(echo "$BATCH_AVERAGE < 0.1" | bc -l) )); then
    echo "  ✅ Batch calculation performance is good (< 100ms)"
else
    echo "  ⚠️ Batch calculation performance could be improved (> 100ms)"
fi

echo ""
echo "✨ Performance benchmarking completed!"
echo "🌐 Engine URL: $ENGINE_URL"
echo "📊 Metrics: $ENGINE_URL/metrics"
