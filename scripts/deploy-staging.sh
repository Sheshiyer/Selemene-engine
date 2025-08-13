#!/bin/bash

# Selemene Engine Staging Deployment Script
set -e

echo "🚀 Starting Selemene Engine staging deployment..."

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
export ENVIRONMENT=staging
export RUST_LOG=debug
export CACHE_SIZE_MB=256
export MAX_CONCURRENT_CALCULATIONS=100

echo "📋 Environment: $ENVIRONMENT"
echo "🔧 RUST_LOG: $RUST_LOG"
echo "💾 Cache Size: ${CACHE_SIZE_MB}MB"
echo "⚡ Max Concurrent: $MAX_CONCURRENT_CALCULATIONS"

# Run tests before deployment
echo "🧪 Running tests..."
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

# Deploy to Railway staging
echo "🚂 Deploying to Railway staging..."
railway up --service selemene-staging

# Wait for deployment
echo "⏳ Waiting for deployment to complete..."
sleep 60

# Health check
echo "🏥 Running health checks..."
HEALTH_URL="https://selemene-staging.railway.app/health"
STATUS_URL="https://selemene-staging.railway.app/status"
METRICS_URL="https://selemene-staging.railway.app/metrics"

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

# Test basic API functionality
echo "🧪 Testing basic API functionality..."
TEST_RESPONSE=$(curl -s -X POST "$HEALTH_URL/api/v1/panchanga" \
    -H "Content-Type: application/json" \
    -d '{"date": "2025-01-27"}')

if echo "$TEST_RESPONSE" | grep -q "success"; then
    echo "✅ API test passed"
else
    echo "❌ API test failed"
    echo "Response: $TEST_RESPONSE"
    exit 1
fi

echo "🎉 Staging deployment completed successfully!"
echo "🌐 Staging URL: https://selemene-staging.railway.app"
echo "📊 Metrics: https://selemene-staging.railway.app/metrics"
echo "🏥 Health: https://selemene-staging.railway.app/health"

# Optional: Run load tests
if command -v hey &> /dev/null; then
    echo "📈 Running basic load test..."
    hey -n 100 -c 10 "$HEALTH_URL"
else
    echo "💡 Install 'hey' for load testing: go install github.com/rakyll/hey@latest"
fi

echo "✨ Staging deployment script completed!"
