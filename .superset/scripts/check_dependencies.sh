#!/bin/bash
# Dependency Health Check - Verify production API health and dependencies
# Production URL: https://selemene.tryambakam.space

echo "============================================"
echo "  DEPENDENCY HEALTH CHECK"
echo "  Production: https://selemene.tryambakam.space"
echo "  Checking all required services"
echo "============================================"
echo ""

API_BASE="${API_BASE:-https://selemene.tryambakam.space/api/v1}"

# Check for API key
if [ -n "$NOESIS_API_KEY" ]; then
  AUTH_FLAG="-H X-API-Key: $NOESIS_API_KEY"
elif [ -n "$API_KEY" ]; then
  AUTH_FLAG="-H X-API-Key: $API_KEY"
else
  AUTH_FLAG=""
fi

# Check application health endpoint
echo "Application:"
if command -v curl >/dev/null 2>&1; then
# Check application health endpoint
echo "Application:"
if command -v curl >/dev/null 2>&1; then
  # Health endpoint
  health_response=$(curl -s -w "\n%{http_code}" "$API_BASE/health" 2>/dev/null)
  health_code=$(echo "$health_response" | tail -1)
  
  if [[ "$health_code" == "200" ]]; then
    echo "✓ /health endpoint: OK"
    
    health_body=$(echo "$health_response" | head -n -1)
    if command -v jq >/dev/null 2>&1; then
      echo "  Status: $(echo "$health_body" | jq -r '.status // "unknown"')"
      echo "  Version: $(echo "$health_body" | jq -r '.version // "unknown"')"
      echo "  Engines: $(echo "$health_body" | jq -r '.engines_loaded // "unknown"')"
      echo "  Workflows: $(echo "$health_body" | jq -r '.workflows_loaded // "unknown"')"
    fi
  else
    echo "✗ /health endpoint: $health_code"
  fi
  
  # Check engines endpoint
  engines_response=$(curl -s -w "\n%{http_code}" "$API_BASE/engines" $AUTH_FLAG 2>/dev/null)
  engines_code=$(echo "$engines_response" | tail -1)
  
  if [[ "$engines_code" == "200" ]]; then
    echo "✓ /engines endpoint: OK"
    
    engines_body=$(echo "$engines_response" | head -n -1)
    if command -v jq >/dev/null 2>&1; then
      count=$(echo "$engines_body" | jq 'length' 2>/dev/null || echo "?")
      echo "  Available engines: $count"
    fi
  elif [[ "$engines_code" == "401" || "$engines_code" == "403" ]]; then
    echo "⚠ /engines endpoint: auth required ($engines_code)"
    echo "  Set NOESIS_API_KEY to test authenticated endpoints"
  else
    echo "✗ /engines endpoint: $engines_code"
  fi
  
  # Check workflows endpoint
  workflows_response=$(curl -s -w "\n%{http_code}" "$API_BASE/workflows" $AUTH_FLAG 2>/dev/null)
  workflows_code=$(echo "$workflows_response" | tail -1)
  
  if [[ "$workflows_code" == "200" ]]; then
    echo "✓ /workflows endpoint: OK"
    
    workflows_body=$(echo "$workflows_response" | head -n -1)
    if command -v jq >/dev/null 2>&1; then
      count=$(echo "$workflows_body" | jq 'length' 2>/dev/null || echo "?")
      echo "  Available workflows: $count"
    fi
  elif [[ "$workflows_code" == "401" || "$workflows_code" == "403" ]]; then
    echo "⚠ /workflows endpoint: auth required ($workflows_code)"
  else
    echo "✗ /workflows endpoint: $workflows_code"
  fi
else
  echo "  (curl not available)"
fi

echo ""
echo "============================================"
echo "  Health Check Complete"
echo "============================================"
echo ""
echo "To check with your API key:"
echo "  NOESIS_API_KEY=your_key .superset/scripts/check_dependencies.sh"
