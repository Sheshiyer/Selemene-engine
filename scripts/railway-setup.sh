#!/bin/bash
set -e  # Exit on error

# =============================================================================
# Railway CLI Automated Setup Script
# =============================================================================
# Purpose: Configure all environment variables for Selemene Engine deployment
# Usage: ./scripts/railway-setup.sh
# Prerequisites: Railway CLI installed and authenticated (railway whoami)
# =============================================================================

echo "🚂 Railway CLI Setup for Selemene Engine"
echo "========================================"
echo ""

# Check if Railway CLI is installed
if ! command -v railway &> /dev/null; then
    echo "❌ Railway CLI not found. Install it first:"
    echo "   npm install -g @railway/cli"
    echo "   or: brew install railway"
    exit 1
fi

# Check if authenticated
if ! railway whoami &> /dev/null; then
    echo "❌ Not authenticated with Railway."
    echo "   Run: railway login"
    exit 1
fi

echo "✅ Railway CLI installed and authenticated"
echo ""

# Check if project is linked
if ! railway status &> /dev/null; then
    echo "📎 No Railway project linked to this directory."
    echo ""
    echo "Options:"
    echo "  1. Link to existing project: railway link"
    echo "  2. Create new project: railway init"
    echo ""
    read -p "Press Enter after linking project, or Ctrl+C to exit..."

    # Verify link succeeded
    if ! railway status &> /dev/null; then
        echo "❌ Project still not linked. Exiting."
        exit 1
    fi
fi

echo "✅ Railway project linked"
railway status
echo ""

# =============================================================================
# Environment Variables Setup
# =============================================================================

echo "🔧 Setting environment variables..."
echo ""

# Prompt for required credentials
echo "📝 Please provide the following credentials:"
echo ""

# Supabase DATABASE_URL
read -p "Supabase DATABASE_URL (from Supabase Dashboard → Settings → Database → Connection Pooling): " DATABASE_URL
if [ -z "$DATABASE_URL" ]; then
    echo "❌ DATABASE_URL is required"
    exit 1
fi

# Railway URL (optional, can be set later)
read -p "Production domain (default: tryambakam.space): " PROD_DOMAIN
PROD_DOMAIN=${PROD_DOMAIN:-tryambakam.space}

echo ""
echo "🚀 Configuring Railway environment variables..."
echo ""

# JWT Secret — generate a unique secret for your deployment:
#   openssl rand -base64 48
# Then set it via Railway dashboard or uncomment and paste below:
JWT_SECRET="${JWT_SECRET:?ERROR: JWT_SECRET must be set. Generate one with: openssl rand -base64 48}"
EXTRA_ALLOWED_ORIGINS="${EXTRA_ALLOWED_ORIGINS:-}"

ALLOWED_ORIGINS="https://$PROD_DOMAIN,https://*.railway.app"
if [ -n "$EXTRA_ALLOWED_ORIGINS" ]; then
  ALLOWED_ORIGINS="$ALLOWED_ORIGINS,$EXTRA_ALLOWED_ORIGINS"
fi

# Set all environment variables using Railway CLI
railway variables \
  --set "RUST_ENV=production" \
  --set "SERVER_HOST=0.0.0.0" \
  --set "SERVER_PORT=8080" \
  --set "RUST_LOG=info" \
  --set "LOG_FORMAT=json" \
  --set "JWT_SECRET=$JWT_SECRET" \
  --set "JWT_EXPIRY=3600" \
  --set "DATABASE_URL=$DATABASE_URL" \
  --set "ALLOWED_ORIGINS=$ALLOWED_ORIGINS" \
  --set "SWISS_EPHEMERIS_PATH=/app/data/ephemeris" \
  --set "DATA_PATH=/app/data" \
  --set "WISDOM_DOCS_PATH=/app/data/wisdom-docs" \
  --set "RATE_LIMIT_REQUESTS=100" \
  --set "RATE_LIMIT_WINDOW=60" \
  --set "ENABLE_METRICS=true" \
  --set "ENABLE_WITNESS=true" \
  --set "CACHE_L1_SIZE=268435456" \
  --set "CACHE_L1_TTL=3600" \
  --set "CACHE_L2_TTL=86400"

echo ""
echo "✅ Environment variables set successfully!"
echo ""

# =============================================================================
# Verification
# =============================================================================

echo "🔍 Verifying configuration..."
echo ""

# Show all variables
echo "Current environment variables:"
railway variables --kv | head -20
echo "... (showing first 20 variables)"
echo ""

# =============================================================================
# Redis Addon Reminder
# =============================================================================

echo "📦 IMPORTANT: Redis Add-on Required"
echo ""
echo "You need to provision Railway Redis add-on:"
echo "  1. Go to Railway Dashboard → Your Project"
echo "  2. Click '+ New' → Database → Redis"
echo "  3. Railway will auto-inject REDIS_URL"
echo ""
read -p "Have you provisioned Redis? (y/n): " redis_provisioned

if [ "$redis_provisioned" != "y" ]; then
    echo ""
    echo "⚠️  Provision Redis before deploying, or the app will fail to start."
    echo ""
fi

# =============================================================================
# Deployment
# =============================================================================

echo ""
echo "🚀 Ready to deploy?"
echo ""
echo "Options:"
echo "  1. Trigger deployment now: railway up"
echo "  2. Push to GitHub (auto-deploys if webhook configured)"
echo "  3. Manual deploy via Railway Dashboard"
echo ""
read -p "Deploy now via Railway CLI? (y/n): " deploy_now

if [ "$deploy_now" == "y" ]; then
    echo ""
    echo "🚂 Deploying to Railway..."
    railway up --detach
    echo ""
    echo "✅ Deployment initiated!"
    echo ""
    echo "Monitor progress:"
    echo "  railway logs"
    echo "  railway status"
    echo "  railway open  # Opens dashboard in browser"
else
    echo ""
    echo "✅ Setup complete! Deploy when ready with:"
    echo "   railway up"
fi

echo ""
echo "==============================================="
echo "✨ Railway configuration complete!"
echo "==============================================="
echo ""
echo "Next steps:"
echo "1. ✅ Environment variables configured"
echo "2. 📦 Provision Redis add-on (if not done)"
echo "3. 🚀 Deploy: railway up or git push"
echo "4. 🔍 Monitor: railway logs"
echo "5. 🌐 Configure Cloudflare DNS (see RAILWAY_SETUP_CHECKLIST.md)"
echo ""
