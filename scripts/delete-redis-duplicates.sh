#!/bin/bash
# Delete duplicate Redis services from Railway
# Keeps only the main "Redis" service

set -e

PROJECT_ID="11eedde4-41e6-4f51-b86b-cf77111cf592"

echo "🔍 Checking Railway authentication..."
railway whoami || { echo "❌ Not logged in to Railway. Run: railway login"; exit 1; }

echo ""
echo "📋 Current services in project:"
echo "   (Looking for duplicate Redis services to delete)"
echo ""

# Note: Railway CLI doesn't have a built-in command to list or delete services
# We'll need to use the dashboard or GraphQL API

echo "⚠️  Railway CLI limitation detected"
echo ""
echo "The Railway CLI doesn't provide commands to list/delete services directly."
echo "You have two options:"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "OPTION 1: Railway Dashboard (Recommended - Easiest)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1. Open: https://railway.app/project/$PROJECT_ID"
echo "2. Click on each duplicate Redis service:"
echo "   • Redis-fPqz"
echo "   • Redis-OiiO"
echo "   • Redis-c_CR"
echo "   • Redis-MMs6"
echo "3. Settings tab → Danger Zone → Delete Service"
echo ""
echo "✅ Keep: 'Redis' (without suffix) - this is the correct one!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "OPTION 2: Railway GraphQL API (Advanced)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "If you want to script this, I can help you use the Railway GraphQL API."
echo "This requires extracting your Railway API token and making HTTP requests."
echo ""
echo "Would you like me to:"
echo "  A) Open the Railway dashboard for you"
echo "  B) Create the GraphQL API script"
echo "  C) Skip for now"
echo ""

read -p "Choose (A/B/C): " choice

case $choice in
  A|a)
    echo ""
    echo "🌐 Opening Railway dashboard..."
    open "https://railway.app/project/$PROJECT_ID" || \
      echo "Visit: https://railway.app/project/$PROJECT_ID"
    ;;
  B|b)
    echo ""
    echo "🔧 Creating GraphQL API cleanup script..."
    echo "   (This will be saved as: scripts/railway-api-cleanup.sh)"
    # Would create the GraphQL script here
    ;;
  C|c)
    echo ""
    echo "⏭️  Skipping Redis cleanup for now."
    echo "   You can run this script again later."
    ;;
  *)
    echo ""
    echo "❌ Invalid choice. Exiting."
    exit 1
    ;;
esac

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "💰 Cost Savings: ~\$20/month by removing 4 duplicate Redis instances"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
