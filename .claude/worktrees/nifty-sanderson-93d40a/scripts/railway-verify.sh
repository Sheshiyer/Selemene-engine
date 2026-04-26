#!/bin/bash

# =============================================================================
# Railway Deployment Verification Script
# =============================================================================
# Purpose: Verify Railway deployment is configured and working correctly
# Usage: ./scripts/railway-verify.sh
# =============================================================================

set -e

echo "🔍 Railway Deployment Verification"
echo "===================================="
echo ""

# Check Railway CLI
if ! command -v railway &> /dev/null; then
    echo "❌ Railway CLI not installed"
    exit 1
fi
echo "✅ Railway CLI installed ($(railway --version))"

# Check authentication
if ! railway whoami &> /dev/null; then
    echo "❌ Not authenticated with Railway"
    exit 1
fi
echo "✅ Authenticated as: $(railway whoami)"

# Check project link
if ! railway status &> /dev/null; then
    echo "❌ No Railway project linked"
    echo "   Run: railway link"
    exit 1
fi
echo "✅ Project linked"
echo ""

# Show project status
echo "📊 Project Status:"
railway status
echo ""

# =============================================================================
# Verify Environment Variables
# =============================================================================

echo "🔐 Checking Environment Variables..."
echo ""

required_vars=(
    "RUST_ENV"
    "JWT_SECRET"
    "DATABASE_URL"
    "REDIS_URL"
    "ALLOWED_ORIGINS"
    "FREE_ASTROLOGY_API_KEY"
    "VEDIC_ENGINE_PROVIDER"
)

missing_vars=()
vars_output=$(railway variables --kv 2>&1)

for var in "${required_vars[@]}"; do
    if echo "$vars_output" | grep -q "^$var="; then
        echo "✅ $var is set"
    else
        echo "❌ $var is MISSING"
        missing_vars+=("$var")
    fi
done

echo ""

if [ ${#missing_vars[@]} -gt 0 ]; then
    echo "⚠️  Missing required variables: ${missing_vars[*]}"
    echo "   Run: ./scripts/railway-setup.sh"
    echo ""
fi

# =============================================================================
# Check Redis Add-on
# =============================================================================

echo "📦 Checking Redis Add-on..."
if echo "$vars_output" | grep -q "^REDIS_URL="; then
    redis_url=$(echo "$vars_output" | grep "^REDIS_URL=" | cut -d'=' -f2)
    echo "✅ Redis provisioned: ${redis_url:0:30}..."
else
    echo "❌ REDIS_URL not found - Redis add-on not provisioned"
    echo "   Provision via: Railway Dashboard → + New → Database → Redis"
fi
echo ""

# =============================================================================
# Check Deployment Status
# =============================================================================

echo "🚀 Checking Deployment Status..."
echo ""

# Get latest deployment info
deployment_info=$(railway status 2>&1)

if echo "$deployment_info" | grep -q "No deployments"; then
    echo "⚠️  No deployments yet"
    echo "   Deploy with: railway up"
else
    echo "$deployment_info"
fi

echo ""

# =============================================================================
# Get Railway URL
# =============================================================================

echo "🌐 Getting Railway URL..."
echo ""

# Try to extract URL from status or use railway open --no-open
railway_url=$(railway status 2>&1 | grep -oE "https://[a-zA-Z0-9.-]+\.railway\.app" | head -1 || echo "")

if [ -n "$railway_url" ]; then
    echo "✅ Railway URL: $railway_url"
    echo ""

    # Test health endpoint
    echo "🏥 Testing Health Endpoint..."
    echo "   Endpoint: $railway_url/health"
    echo ""

    health_response=$(curl -s -w "\n%{http_code}" "$railway_url/health" 2>&1 || echo "000")
    http_code=$(echo "$health_response" | tail -1)
    response_body=$(echo "$health_response" | head -n -1)

    if [ "$http_code" == "200" ]; then
        echo "✅ Health check passed!"
        echo "   Response: $response_body"
    elif [ "$http_code" == "000" ]; then
        echo "❌ Cannot reach endpoint (deployment may be in progress)"
    else
        echo "⚠️  Health check returned $http_code"
        echo "   Response: $response_body"
    fi
else
    echo "⚠️  Railway URL not found (may need to deploy first)"
fi

echo ""

# =============================================================================
# Summary
# =============================================================================

echo "==============================================="
echo "📋 Verification Summary"
echo "==============================================="
echo ""

if [ ${#missing_vars[@]} -eq 0 ] && echo "$vars_output" | grep -q "^REDIS_URL="; then
    echo "✅ All required configuration present"
    echo ""
    echo "Next steps:"
    echo "  1. Deploy: railway up"
    echo "  2. Monitor logs: railway logs"
    echo "  3. Open dashboard: railway open"
    echo "  4. Test health: curl https://[railway-url]/health"
else
    echo "⚠️  Configuration incomplete"
    echo ""
    echo "Action items:"
    if [ ${#missing_vars[@]} -gt 0 ]; then
        echo "  - Set missing env vars: ./scripts/railway-setup.sh"
    fi
    if ! echo "$vars_output" | grep -q "^REDIS_URL="; then
        echo "  - Provision Redis add-on via Railway Dashboard"
    fi
fi

echo ""
