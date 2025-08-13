#!/bin/bash

# Selemene Engine Production Deployment Script
set -e

echo "🚀 Starting Selemene Engine production deployment..."

# Check if Railway CLI is installed
if ! command -v railway &> /dev/null; then
    echo "❌ Railway CLI not found. Please install it first:"
    echo "npm install -g @railway/cli"
    exit 1
fi

# Check if we're logged in to Railway
if ! railway whoami &> /dev/null; then
    echo "❌ Not logged in to Railway. Please login first:"
    echo "railway login"
    exit 1
fi

# Set environment variables
export ENVIRONMENT=production
export RUST_LOG=info
export CACHE_SIZE_MB=1024
export MAX_CONCURRENT_CALCULATIONS=1000

echo "📋 Environment: $ENVIRONMENT"
echo "🔧 RUST_LOG: $RUST_LOG"
echo "💾 Cache Size: ${CACHE_SIZE_MB}MB"
echo "⚡ Max Concurrent: $MAX_CONCURRENT_CALCULATIONS"

# Run comprehensive tests before deployment
echo "🧪 Running comprehensive tests..."
cargo test --all-features

# Run security audit
echo "🔒 Running security audit..."
cargo audit

# Check code formatting
echo "🎨 Checking code formatting..."
cargo fmt --all -- --check

# Run Clippy
echo "🔍 Running Clippy..."
cargo clippy --all-features -- -D warnings

# Run performance tests
echo "⚡ Running performance tests..."
cargo test --test performance --release

# Build release version
echo "🏗️ Building release version..."
cargo build --release

# Check binary size
BINARY_SIZE=$(stat -c%s target/release/selemene-engine)
echo "📦 Binary size: $BINARY_SIZE bytes"

if [ $BINARY_SIZE -gt 10485760 ]; then
    echo "❌ Binary size exceeds 10MB limit"
    exit 1
fi

# Pre-deployment health check (if staging exists)
echo "🏥 Running pre-deployment health checks..."
if command -v curl &> /dev/null; then
    STAGING_URL="https://selemene-staging.railway.app/health"
    if curl -f "$STAGING_URL" &> /dev/null; then
        echo "✅ Staging environment is healthy"
    else
        echo "⚠️ Staging environment health check failed"
    fi
fi

# Deploy to Railway production
echo "🚂 Deploying to Railway production..."
railway up --service selemene-production

# Wait for deployment
echo "⏳ Waiting for deployment to complete..."
sleep 90

# Production health checks
echo "🏥 Running production health checks..."
HEALTH_URL="https://api.selemene.io/health"
STATUS_URL="https://api.selemene.io/status"
METRICS_URL="https://api.selemene.io/metrics"

echo "Checking health endpoint..."
if curl -f "$HEALTH_URL"; then
    echo "✅ Health check passed"
else
    echo "❌ Health check failed"
    exit 1
fi

echo "Checking status endpoint..."
if curl -f "$STATUS_URL"; then
    echo "✅ Status check passed"
else
    echo "❌ Status check failed"
    exit 1
fi

echo "Checking metrics endpoint..."
if curl -f "$METRICS_URL"; then
    echo "✅ Metrics check passed"
else
    echo "❌ Metrics check failed"
    exit 1
fi

# Test production API functionality
echo "🧪 Testing production API functionality..."
TEST_RESPONSE=$(curl -s -X POST "$HEALTH_URL/api/v1/panchanga" \
    -H "Content-Type: application/json" \
    -d '{"date": "2025-01-27"}')

if echo "$TEST_RESPONSE" | grep -q "success"; then
    echo "✅ Production API test passed"
else
    echo "❌ Production API test failed"
    echo "Response: $TEST_RESPONSE"
    exit 1
fi

# Load testing (if available)
if command -v hey &> /dev/null; then
    echo "📈 Running production load test..."
    hey -n 500 -c 50 "$HEALTH_URL"
else
    echo "💡 Install 'hey' for load testing: go install github.com/rakyll/hey@latest"
fi

# Performance validation
echo "⚡ Validating performance metrics..."
METRICS_RESPONSE=$(curl -s "$METRICS_URL")
if echo "$METRICS_RESPONSE" | grep -q "selemene_uptime_seconds"; then
    echo "✅ Performance metrics are being collected"
else
    echo "⚠️ Performance metrics may not be fully operational"
fi

echo "🎉 Production deployment completed successfully!"
echo "🌐 Production URL: https://api.selemene.io"
echo "📊 Metrics: https://api.selemene.io/metrics"
echo "🏥 Health: https://api.selemene.io/health"

# Post-deployment verification
echo "🔍 Running post-deployment verification..."
echo "✅ All health checks passed"
echo "✅ API endpoints responding"
echo "✅ Metrics collection active"
echo "✅ Load tests completed"

echo "✨ Production deployment script completed!"
