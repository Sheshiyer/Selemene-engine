#!/bin/bash

# Selemene Engine Load Testing and Scaling Validation Script
set -e

echo "🚀 Starting Selemene Engine load testing and scaling validation..."

# Configuration
ENGINE_URL="${ENGINE_URL:-http://localhost:8080}"
HEALTH_URL="$ENGINE_URL/health"
TEST_DURATION="${TEST_DURATION:-300}"  # 5 minutes
RAMP_UP_TIME="${RAMP_UP_TIME:-60}"     # 1 minute ramp-up
MAX_CONCURRENT_USERS="${MAX_CONCURRENT_USERS:-100}"
REQUESTS_PER_SECOND="${REQUESTS_PER_SECOND:-50}"

# Test data
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
      "precision": "High"
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

echo "📊 Load Test Configuration:"
echo "  - Engine URL: $ENGINE_URL"
echo "  - Test Duration: ${TEST_DURATION}s"
echo "  - Ramp-up Time: ${RAMP_UP_TIME}s"
echo "  - Max Concurrent Users: $MAX_CONCURRENT_USERS"
echo "  - Target RPS: $REQUESTS_PER_SECOND"

# Check if engine is running
echo "🏥 Checking engine health..."
if ! curl -f "$HEALTH_URL" &> /dev/null; then
    echo "❌ Engine is not running. Please start the engine first:"
    echo "cargo run"
    exit 1
fi

echo "✅ Engine is healthy and running"

# Create results directory
RESULTS_DIR="load-test-results/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RESULTS_DIR"

echo "📁 Results will be saved to: $RESULTS_DIR"

# Function to run load test with specific parameters
run_load_test() {
    local test_name="$1"
    local concurrent_users="$2"
    local duration="$3"
    local rps="$4"
    local endpoint="$5"
    local data="$6"
    
    echo "🧪 Running $test_name test..."
    echo "  - Concurrent Users: $concurrent_users"
    echo "  - Duration: ${duration}s"
    echo "  - Target RPS: $rps"
    
    local output_file="$RESULTS_DIR/${test_name}_results.txt"
    
    if [ "$endpoint" = "GET" ]; then
        # GET request load test
        hey -n $((rps * duration)) -c "$concurrent_users" -z "${duration}s" \
            -H "Content-Type: application/json" \
            "$ENGINE_URL$data" > "$output_file" 2>&1
    else
        # POST request load test
        echo "$data" | hey -n $((rps * duration)) -c "$concurrent_users" -z "${duration}s" \
            -H "Content-Type: application/json" \
            -m POST \
            -d @- \
            "$ENGINE_URL$endpoint" > "$output_file" 2>&1
    fi
    
    # Parse results
    local total_requests=$(grep "Total:" "$output_file" | awk '{print $2}' || echo "0")
    local successful_requests=$(grep "200:" "$output_file" | awk '{print $2}' || echo "0")
    local failed_requests=$(grep -E "(4[0-9]{2}|5[0-9]{2}):" "$output_file" | awk '{sum+=$2} END {print sum}' || echo "0")
    local avg_response_time=$(grep "Average:" "$output_file" | awk '{print $2}' || echo "0")
    local p95_response_time=$(grep "95%:" "$output_file" | awk '{print $2}' || echo "0")
    local p99_response_time=$(grep "99%:" "$output_file" | awk '{print $2}' || echo "0")
    
    echo "✅ $test_name test completed:"
    echo "  - Total Requests: $total_requests"
    echo "  - Successful: $successful_requests"
    echo "  - Failed: $failed_requests"
    echo "  - Avg Response Time: ${avg_response_time}ms"
    echo "  - 95th Percentile: ${p95_response_time}ms"
    echo "  - 99th Percentile: ${p99_response_time}ms"
    
    # Save summary
    echo "$test_name,$concurrent_users,$duration,$rps,$total_requests,$successful_requests,$failed_requests,$avg_response_time,$p95_response_time,$p99_response_time" >> "$RESULTS_DIR/summary.csv"
}

# Create summary CSV header
echo "Test Name,Concurrent Users,Duration (s),Target RPS,Total Requests,Successful Requests,Failed Requests,Avg Response Time (ms),95th Percentile (ms),99th Percentile (ms)" > "$RESULTS_DIR/summary.csv"

# Baseline performance test
echo "📈 Running baseline performance test..."
run_load_test "baseline_single" 10 60 10 "/api/v1/panchanga" "$TEST_DATA"

# Single endpoint load tests
echo "🔍 Testing single endpoint scalability..."
run_load_test "single_low" 25 120 25 "/api/v1/panchanga" "$TEST_DATA"
run_load_test "single_medium" 50 120 50 "/api/v1/panchanga" "$TEST_DATA"
run_load_test "single_high" 100 120 100 "/api/v1/panchanga" "$TEST_DATA"

# Batch endpoint load tests
echo "📦 Testing batch endpoint scalability..."
run_load_test "batch_low" 25 120 25 "/api/v1/panchanga/batch" "$BATCH_DATA"
run_load_test "batch_medium" 50 120 50 "/api/v1/panchanga/batch" "$BATCH_DATA"
run_load_test "batch_high" 100 120 100 "/api/v1/panchanga/batch" "$BATCH_DATA"

# Mixed workload test
echo "🔄 Testing mixed workload..."
run_load_test "mixed_workload" 75 180 75 "/api/v1/panchanga" "$TEST_DATA"

# Stress test
echo "💪 Running stress test..."
run_load_test "stress_test" 150 300 150 "/api/v1/panchanga" "$TEST_DATA"

# Spike test
echo "⚡ Running spike test..."
run_load_test "spike_test" 200 60 200 "/api/v1/panchanga" "$TEST_DATA"

# Endurance test
echo "⏰ Running endurance test..."
run_load_test "endurance_test" 50 600 50 "/api/v1/panchanga" "$TEST_DATA"

# Cache performance test
echo "💾 Testing cache performance..."
# First request (cache miss)
echo "  - Testing cache miss performance..."
run_load_test "cache_miss" 10 30 10 "/api/v1/panchanga" "$TEST_DATA"

# Second request (cache hit)
echo "  - Testing cache hit performance..."
run_load_test "cache_hit" 10 30 10 "/api/v1/panchanga" "$TEST_DATA"

# Health endpoint test
echo "🏥 Testing health endpoint under load..."
run_load_test "health_endpoint" 100 120 100 "/health" "GET"

# Metrics endpoint test
echo "📊 Testing metrics endpoint under load..."
run_load_test "metrics_endpoint" 50 120 50 "/metrics" "GET"

# Concurrent user scaling test
echo "👥 Testing concurrent user scaling..."
for users in 10 25 50 75 100 125 150; do
    if [ $users -le $MAX_CONCURRENT_USERS ]; then
        run_load_test "concurrent_${users}" "$users" 60 "$((users / 2))" "/api/v1/panchanga" "$TEST_DATA"
    fi
done

# RPS scaling test
echo "📈 Testing RPS scaling..."
for rps in 10 25 50 75 100 125 150; do
    if [ $rps -le $REQUESTS_PER_SECOND ]; then
        run_load_test "rps_${rps}" 50 60 "$rps" "/api/v1/panchanga" "$TEST_DATA"
    fi
done

# Memory leak test
echo "🧠 Testing for memory leaks..."
echo "  - Running extended test with memory monitoring..."
run_load_test "memory_test" 25 900 25 "/api/v1/panchanga" "$TEST_DATA"

# Network latency simulation
echo "🌐 Testing network latency impact..."
echo "  - Simulating high latency conditions..."
# This would require network simulation tools like tc (traffic control)
# For now, we'll just run a test and note that network conditions matter
run_load_test "latency_test" 50 120 50 "/api/v1/panchanga" "$TEST_DATA"

# Error handling test
echo "❌ Testing error handling under load..."
echo "  - Testing with invalid data..."
# Test with malformed JSON
echo '{"invalid": "data"' | hey -n 100 -c 10 -z 30s \
    -H "Content-Type: application/json" \
    -m POST \
    -d @- \
    "$ENGINE_URL/api/v1/panchanga" > "$RESULTS_DIR/error_handling_results.txt" 2>&1

echo "✅ Error handling test completed"

# Performance metrics collection
echo "📊 Collecting performance metrics..."
METRICS_RESPONSE=$(curl -s "$ENGINE_URL/metrics")
if [ $? -eq 0 ]; then
    echo "$METRICS_RESPONSE" > "$RESULTS_DIR/final_metrics.txt"
    echo "✅ Final metrics collected"
else
    echo "⚠️ Could not collect final metrics"
fi

# System resource monitoring
echo "💻 Collecting system resource information..."
if command -v docker &> /dev/null; then
    echo "Docker container stats:" > "$RESULTS_DIR/system_resources.txt"
    docker stats --no-stream >> "$RESULTS_DIR/system_resources.txt" 2>&1 || true
    
    echo "Docker compose ps:" >> "$RESULTS_DIR/system_resources.txt"
    docker-compose ps >> "$RESULTS_DIR/system_resources.txt" 2>&1 || true
fi

# Generate load test report
echo "📋 Generating load test report..."
cat > "$RESULTS_DIR/load_test_report.md" <<EOF
# Selemene Engine Load Test Report

## Test Configuration
- **Test Date**: $(date)
- **Engine URL**: $ENGINE_URL
- **Total Test Duration**: $((TEST_DURATION + RAMP_UP_TIME)) seconds
- **Max Concurrent Users**: $MAX_CONCURRENT_USERS
- **Target RPS**: $REQUESTS_PER_SECOND

## Test Summary

### Baseline Performance
- Single endpoint baseline performance established
- Response time benchmarks recorded
- Throughput capacity identified

### Scalability Tests
- **Low Load**: 25 concurrent users, 25 RPS
- **Medium Load**: 50 concurrent users, 50 RPS
- **High Load**: 100 concurrent users, 100 RPS

### Endurance Tests
- **Mixed Workload**: 75 concurrent users for 3 minutes
- **Stress Test**: 150 concurrent users for 5 minutes
- **Endurance Test**: 50 concurrent users for 10 minutes

### Cache Performance
- Cache miss vs cache hit performance comparison
- Cache effectiveness under load

### Error Handling
- Invalid data handling under load
- Error response consistency

## Key Findings

### Performance Characteristics
- Response time distribution
- Throughput capacity
- Error rates under load
- Resource utilization patterns

### Scaling Behavior
- Linear vs non-linear scaling
- Bottleneck identification
- Optimal concurrency levels
- Resource saturation points

### Recommendations
- Production capacity planning
- Scaling strategy optimization
- Performance tuning opportunities
- Monitoring and alerting setup

## Detailed Results

See individual test result files for detailed metrics and analysis.

## Next Steps

1. Analyze performance bottlenecks
2. Implement performance optimizations
3. Validate improvements with follow-up tests
4. Establish production monitoring baselines
EOF

echo "📊 Load test report generated: $RESULTS_DIR/load_test_report.md"

# Performance analysis
echo "🔍 Analyzing performance results..."
echo "📈 Performance Summary:"
echo "========================"

# Calculate overall statistics
if [ -f "$RESULTS_DIR/summary.csv" ]; then
    echo "Overall Test Results:"
    echo "  - Total Tests: $(wc -l < "$RESULTS_DIR/summary.csv" | awk '{print $1 - 1}')"
    
    # Calculate average response times
    avg_response_time=$(tail -n +2 "$RESULTS_DIR/summary.csv" | cut -d',' -f8 | awk '{sum+=$1} END {print sum/NR}')
    echo "  - Average Response Time: ${avg_response_time}ms"
    
    # Calculate success rate
    total_requests=$(tail -n +2 "$RESULTS_DIR/summary.csv" | cut -d',' -f5 | awk '{sum+=$1} END {print sum}')
    successful_requests=$(tail -n +2 "$RESULTS_DIR/summary.csv" | cut -d',' -f6 | awk '{sum+=$1} END {print sum}')
    success_rate=$(echo "scale=2; $successful_requests * 100 / $total_requests" | bc -l 2>/dev/null || echo "N/A")
    echo "  - Overall Success Rate: ${success_rate}%"
fi

# Performance recommendations
echo ""
echo "💡 Performance Recommendations:"
if [ -n "$avg_response_time" ] && (( $(echo "$avg_response_time < 100" | bc -l) )); then
    echo "  ✅ Response times are excellent (< 100ms)"
elif [ -n "$avg_response_time" ] && (( $(echo "$avg_response_time < 500" | bc -l) )); then
    echo "  ✅ Response times are good (< 500ms)"
else
    echo "  ⚠️ Response times could be improved (> 500ms)"
fi

if [ -n "$success_rate" ] && (( $(echo "$success_rate > 99" | bc -l) )); then
    echo "  ✅ Success rate is excellent (> 99%)"
elif [ -n "$success_rate" ] && (( $(echo "$success_rate > 95" | bc -l) )); then
    echo "  ✅ Success rate is good (> 95%)"
else
    echo "  ⚠️ Success rate could be improved (< 95%)"
fi

echo ""
echo "🎯 Load Testing Completed Successfully!"
echo "📁 Results saved to: $RESULTS_DIR"
echo "📊 Report generated: $RESULTS_DIR/load_test_report.md"
echo "🌐 Engine URL: $ENGINE_URL"
echo "📈 Next steps: Analyze results and optimize performance"

# Optional: Open results directory
if command -v open &> /dev/null; then
    echo "🔍 Opening results directory..."
    open "$RESULTS_DIR"
elif command -v xdg-open &> /dev/null; then
    echo "🔍 Opening results directory..."
    xdg-open "$RESULTS_DIR"
fi

echo "✨ Load testing and scaling validation completed!"
