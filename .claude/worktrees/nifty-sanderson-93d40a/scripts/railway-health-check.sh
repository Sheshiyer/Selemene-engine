#!/bin/bash
# Railway Health Check with Cold Start Retry Logic
#
# Railway deployments sleep after idle periods and take 30-60s to wake up.
# This script retries with exponential backoff to handle cold starts gracefully.

set -e

URL="${1:-https://selemene.tryambakam.space/health/live}"
MAX_ATTEMPTS=3
DELAYS=(0 10 30)  # Retry delays in seconds

echo "🔍 Checking Railway deployment health: $URL"
echo "   (Accounting for ~30-60s cold start delay)"
echo ""

for i in $(seq 0 $((MAX_ATTEMPTS - 1))); do
    attempt=$((i + 1))
    delay=${DELAYS[$i]}

    if [ $delay -gt 0 ]; then
        echo "⏳ Waiting ${delay}s before attempt $attempt/$MAX_ATTEMPTS (cold start delay)..."
        sleep $delay
    fi

    echo "🔄 Attempt $attempt/$MAX_ATTEMPTS: Calling $URL..."

    response=$(curl -s -w "\n%{http_code}" "$URL" 2>&1 || echo "000")
    http_code=$(echo "$response" | tail -n 1)
    body=$(echo "$response" | sed '$d')  # Remove last line (http_code)

    if [ "$http_code" = "200" ]; then
        echo "✅ SUCCESS! Railway deployment is healthy."
        echo ""
        echo "Response:"
        echo "$body" | jq . 2>/dev/null || echo "$body"
        exit 0
    elif [ "$http_code" = "502" ] && [ $i -lt $((MAX_ATTEMPTS - 1)) ]; then
        echo "⚠️  Got 502 (Application failed to respond) - likely cold start"
        echo "   Railway is waking up the service..."
    else
        echo "❌ HTTP $http_code"
        echo "$body"

        if [ $i -eq $((MAX_ATTEMPTS - 1)) ]; then
            echo ""
            echo "💡 All $MAX_ATTEMPTS attempts failed. Possible causes:"
            echo "   1. Service is still deploying (check: railway logs)"
            echo "   2. Application crashed on startup (check: railway logs)"
            echo "   3. Health endpoint path is wrong"
            echo "   4. DATABASE_URL or other env var missing"
            exit 1
        fi
    fi
    echo ""
done
