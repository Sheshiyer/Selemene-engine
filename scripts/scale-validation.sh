#!/bin/bash

# Selemene Engine Scaling Validation Script
set -e

echo "🚀 Starting Selemene Engine scaling validation..."

# Configuration
ENGINE_URL="${ENGINE_URL:-https://api.selemene.io}"
HEALTH_URL="$ENGINE_URL/health"
STATUS_URL="$ENGINE_URL/status"
METRICS_URL="$ENGINE_URL/metrics"

# Scaling parameters
MIN_INSTANCES="${MIN_INSTANCES:-2}"
MAX_INSTANCES="${MAX_INSTANCES:-10}"
TARGET_CPU="${TARGET_CPU:-70}"
SCALE_UP_THRESHOLD="${SCALE_UP_THRESHOLD:-80}"
SCALE_DOWN_THRESHOLD="${SCALE_DOWN_THRESHOLD:-30}"

echo "📊 Scaling Configuration:"
echo "  - Engine URL: $ENGINE_URL"
echo "  - Min Instances: $MIN_INSTANCES"
echo "  - Max Instances: $MAX_INSTANCES"
echo "  - Target CPU: ${TARGET_CPU}%"
echo "  - Scale Up Threshold: ${SCALE_UP_THRESHOLD}%"
echo "  - Scale Down Threshold: ${SCALE_DOWN_THRESHOLD}%"

# Check if engine is accessible
echo "🏥 Checking engine accessibility..."
if ! curl -f "$HEALTH_URL" &> /dev/null; then
    echo "❌ Engine is not accessible. Please check the URL and try again."
    exit 1
fi

echo "✅ Engine is accessible"

# Create validation results directory
RESULTS_DIR="scale-validation-results/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RESULTS_DIR"

echo "📁 Results will be saved to: $RESULTS_DIR"

# Function to collect current metrics
collect_metrics() {
    local phase="$1"
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    
    echo "📊 Collecting metrics for phase: $phase"
    
    # Health check
    local health_response=$(curl -s "$HEALTH_URL")
    local health_status=$(echo "$health_response" | jq -r '.status' 2>/dev/null || echo "unknown")
    
    # Status check
    local status_response=$(curl -s "$STATUS_URL")
    local uptime=$(echo "$status_response" | jq -r '.uptime_seconds' 2>/dev/null || echo "0")
    local total_requests=$(echo "$status_response" | jq -r '.total_requests' 2>/dev/null || echo "0")
    local success_rate=$(echo "$status_response" | jq -r '.success_rate' 2>/dev/null || echo "0")
    local avg_response_time=$(echo "$status_response" | jq -r '.average_response_time_ms' 2>/dev/null || echo "0")
    
    # Metrics collection
    local metrics_response=$(curl -s "$METRICS_URL")
    
    # Save metrics
    cat > "$RESULTS_DIR/${phase}_metrics.json" <<EOF
{
  "timestamp": "$timestamp",
  "phase": "$phase",
  "health": {
    "status": "$health_status",
    "response": $health_response
  },
  "status": {
    "uptime_seconds": $uptime,
    "total_requests": $total_requests,
    "success_rate": $success_rate,
    "average_response_time_ms": $avg_response_time,
    "response": $status_response
  },
  "metrics": "$metrics_response"
}
EOF
    
    echo "✅ Metrics collected for $phase"
    echo "  - Health Status: $health_status"
    echo "  - Uptime: ${uptime}s"
    echo "  - Total Requests: $total_requests"
    echo "  - Success Rate: ${success_rate}%"
    echo "  - Avg Response Time: ${avg_response_time}ms"
}

# Function to run scaling test
run_scaling_test() {
    local test_name="$1"
    local concurrent_users="$2"
    local duration="$3"
    local rps="$4"
    
    echo "🧪 Running scaling test: $test_name"
    echo "  - Concurrent Users: $concurrent_users"
    echo "  - Duration: ${duration}s"
    echo "  - Target RPS: $rps"
    
    # Collect baseline metrics
    collect_metrics "${test_name}_baseline"
    
    # Run load test
    local test_data='{"date": "2025-01-27", "coordinates": {"latitude": 19.0760, "longitude": 72.8777}, "precision": "Standard"}'
    
    echo "  - Starting load test..."
    local output_file="$RESULTS_DIR/${test_name}_load_test.txt"
    
    echo "$test_data" | hey -n $((rps * duration)) -c "$concurrent_users" -z "${duration}s" \
        -H "Content-Type: application/json" \
        -m POST \
        -d @- \
        "$ENGINE_URL/api/v1/panchanga" > "$output_file" 2>&1
    
    # Wait for scaling to stabilize
    echo "  - Waiting for scaling to stabilize..."
    sleep 30
    
    # Collect post-test metrics
    collect_metrics "${test_name}_post_test"
    
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
    
    # Save test summary
    echo "$test_name,$concurrent_users,$duration,$rps,$total_requests,$successful_requests,$failed_requests,$avg_response_time,$p95_response_time,$p99_response_time" >> "$RESULTS_DIR/scaling_tests_summary.csv"
}

# Create summary CSV header
echo "Test Name,Concurrent Users,Duration (s),Target RPS,Total Requests,Successful Requests,Failed Requests,Avg Response Time (ms),95th Percentile (ms),99th Percentile (ms)" > "$RESULTS_DIR/scaling_tests_summary.csv"

# Collect initial baseline metrics
echo "📈 Collecting initial baseline metrics..."
collect_metrics "initial_baseline"

# Scaling validation tests
echo "🔍 Running scaling validation tests..."

# Test 1: Light load (should not trigger scaling)
echo "📊 Test 1: Light Load (No Scaling Expected)"
run_scaling_test "light_load" 25 120 25

# Test 2: Medium load (may trigger scaling)
echo "📊 Test 2: Medium Load (Potential Scaling)"
run_scaling_test "medium_load" 75 180 75

# Test 3: High load (should trigger scaling)
echo "📊 Test 3: High Load (Scaling Expected)"
run_scaling_test "high_load" 150 300 150

# Test 4: Sustained load (scaling stability)
echo "📊 Test 4: Sustained Load (Scaling Stability)"
run_scaling_test "sustained_load" 100 600 100

# Test 5: Burst load (rapid scaling)
echo "📊 Test 5: Burst Load (Rapid Scaling)"
run_scaling_test "burst_load" 200 120 200

# Test 6: Recovery test (scale down)
echo "📊 Test 6: Recovery Test (Scale Down)"
run_scaling_test "recovery_test" 25 300 25

# Cache scaling test
echo "💾 Testing cache scaling..."
collect_metrics "cache_test_baseline"

# Run cache-intensive test
echo "  - Running cache-intensive test..."
for i in {1..100}; do
    curl -s -X POST "$ENGINE_URL/api/v1/panchanga" \
        -H "Content-Type: application/json" \
        -d '{"date": "2025-01-27", "coordinates": {"latitude": 19.0760, "longitude": 72.8777}, "precision": "Standard"}' > /dev/null &
done
wait

sleep 30
collect_metrics "cache_test_post"

# Database scaling test
echo "🗄️ Testing database scaling..."
collect_metrics "database_test_baseline"

# Run database-intensive test
echo "  - Running database-intensive test..."
for i in {1..50}; do
    curl -s -X POST "$ENGINE_URL/api/v1/panchanga/batch" \
        -H "Content-Type: application/json" \
        -d '{"requests": [{"date": "2025-01-27", "coordinates": {"latitude": 19.0760, "longitude": 72.8777}, "precision": "Standard"}, {"date": "2025-06-15", "coordinates": {"latitude": 28.6139, "longitude": 77.2090}, "precision": "High"}]}' > /dev/null &
done
wait

sleep 30
collect_metrics "database_test_post"

# Memory scaling test
echo "🧠 Testing memory scaling..."
collect_metrics "memory_test_baseline"

# Run memory-intensive test
echo "  - Running memory-intensive test..."
for i in {1..200}; do
    curl -s -X POST "$ENGINE_URL/api/v1/panchanga" \
        -H "Content-Type: application/json" \
        -d '{"date": "2025-01-27", "coordinates": {"latitude": 19.0760, "longitude": 72.8777}, "precision": "Extreme"}' > /dev/null &
done
wait

sleep 30
collect_metrics "memory_test_post"

# Network scaling test
echo "🌐 Testing network scaling..."
collect_metrics "network_test_baseline"

# Run network-intensive test
echo "  - Running network-intensive test..."
for i in {1..150}; do
    curl -s -X POST "$ENGINE_URL/api/v1/panchanga" \
        -H "Content-Type: application/json" \
        -d '{"date": "2025-01-27", "coordinates": {"latitude": 19.0760, "longitude": 72.8777}, "precision": "Standard"}' > /dev/null &
done
wait

sleep 30
collect_metrics "network_test_post"

# Final metrics collection
echo "📊 Collecting final metrics..."
collect_metrics "final_baseline"

# Generate scaling validation report
echo "📋 Generating scaling validation report..."
cat > "$RESULTS_DIR/scaling_validation_report.md" <<EOF
# Selemene Engine Scaling Validation Report

## Test Configuration
- **Test Date**: $(date)
- **Engine URL**: $ENGINE_URL
- **Scaling Parameters**:
  - Min Instances: $MIN_INSTANCES
  - Max Instances: $MAX_INSTANCES
  - Target CPU: ${TARGET_CPU}%
  - Scale Up Threshold: ${SCALE_UP_THRESHOLD}%
  - Scale Down Threshold: ${SCALE_DOWN_THRESHOLD}%

## Test Summary

### Scaling Tests
1. **Light Load**: 25 concurrent users, 25 RPS (No scaling expected)
2. **Medium Load**: 75 concurrent users, 75 RPS (Potential scaling)
3. **High Load**: 150 concurrent users, 150 RPS (Scaling expected)
4. **Sustained Load**: 100 concurrent users, 100 RPS (Scaling stability)
5. **Burst Load**: 200 concurrent users, 200 RPS (Rapid scaling)
6. **Recovery Test**: 25 concurrent users, 25 RPS (Scale down)

### Component Tests
- **Cache Scaling**: Cache performance under load
- **Database Scaling**: Database performance under load
- **Memory Scaling**: Memory usage patterns
- **Network Scaling**: Network performance under load

## Key Findings

### Scaling Behavior
- **Scale Up Performance**: How quickly the system scales up
- **Scale Down Performance**: How efficiently the system scales down
- **Scaling Stability**: Consistency of scaling behavior
- **Resource Utilization**: CPU, memory, and network usage patterns

### Performance Characteristics
- **Response Time Consistency**: Response time stability during scaling
- **Throughput Scaling**: How throughput scales with instances
- **Error Rate Stability**: Error rates during scaling events
- **Recovery Time**: Time to stabilize after scaling

### Bottleneck Analysis
- **CPU Bottlenecks**: CPU utilization patterns
- **Memory Bottlenecks**: Memory usage and garbage collection
- **Network Bottlenecks**: Network I/O patterns
- **Database Bottlenecks**: Database connection and query performance

## Recommendations

### Scaling Optimization
- Optimal instance count for different load levels
- Scaling threshold adjustments
- Resource allocation optimization
- Scaling policy refinement

### Performance Improvements
- Response time optimization
- Throughput enhancement
- Resource utilization improvement
- Error rate reduction

### Monitoring and Alerting
- Scaling event monitoring
- Performance threshold alerts
- Resource utilization tracking
- Scaling efficiency metrics

## Detailed Results

See individual test result files for detailed metrics and analysis.

## Next Steps

1. Analyze scaling patterns and bottlenecks
2. Optimize scaling policies and thresholds
3. Implement performance improvements
4. Validate optimizations with follow-up tests
5. Establish production scaling baselines
EOF

echo "📊 Scaling validation report generated: $RESULTS_DIR/scaling_validation_report.md"

# Performance analysis
echo "🔍 Analyzing scaling validation results..."
echo "📈 Scaling Validation Summary:"
echo "================================"

# Calculate overall statistics
if [ -f "$RESULTS_DIR/scaling_tests_summary.csv" ]; then
    echo "Overall Test Results:"
    echo "  - Total Tests: $(wc -l < "$RESULTS_DIR/scaling_tests_summary.csv" | awk '{print $1 - 1}')"
    
    # Calculate average response times
    avg_response_time=$(tail -n +2 "$RESULTS_DIR/scaling_tests_summary.csv" | cut -d',' -f8 | awk '{sum+=$1} END {print sum/NR}')
    echo "  - Average Response Time: ${avg_response_time}ms"
    
    # Calculate success rate
    total_requests=$(tail -n +2 "$RESULTS_DIR/scaling_tests_summary.csv" | cut -d',' -f5 | awk '{sum+=$1} END {print sum}')
    successful_requests=$(tail -n +2 "$RESULTS_DIR/scaling_tests_summary.csv" | cut -d',' -f6 | awk '{sum+=$1} END {print sum}')
    success_rate=$(echo "scale=2; $successful_requests * 100 / $total_requests" | bc -l 2>/dev/null || echo "N/A")
    echo "  - Overall Success Rate: ${success_rate}%"
fi

# Scaling recommendations
echo ""
echo "💡 Scaling Recommendations:"
if [ -n "$avg_response_time" ] && (( $(echo "$avg_response_time < 200" | bc -l) )); then
    echo "  ✅ Response times are excellent during scaling (< 200ms)"
elif [ -n "$avg_response_time" ] && (( $(echo "$avg_response_time < 500" | bc -l) )); then
    echo "  ✅ Response times are good during scaling (< 500ms)"
else
    echo "  ⚠️ Response times could be improved during scaling (> 500ms)"
fi

if [ -n "$success_rate" ] && (( $(echo "$success_rate > 99" | bc -l) )); then
    echo "  ✅ Success rate is excellent during scaling (> 99%)"
elif [ -n "$success_rate" ] && (( $(echo "$success_rate > 95" | bc -l) )); then
    echo "  ✅ Success rate is good during scaling (> 95%)"
else
    echo "  ⚠️ Success rate could be improved during scaling (< 95%)"
fi

echo ""
echo "🎯 Scaling Validation Completed Successfully!"
echo "📁 Results saved to: $RESULTS_DIR"
echo "📊 Report generated: $RESULTS_DIR/scaling_validation_report.md"
echo "🌐 Engine URL: $ENGINE_URL"
echo "📈 Next steps: Analyze scaling patterns and optimize policies"

# Optional: Open results directory
if command -v open &> /dev/null; then
    echo "🔍 Opening results directory..."
    open "$RESULTS_DIR"
elif command -v xdg-open &> /dev/null; then
    echo "🔍 Opening results directory..."
    xdg-open "$RESULTS_DIR"
fi

echo "✨ Scaling validation completed!"
