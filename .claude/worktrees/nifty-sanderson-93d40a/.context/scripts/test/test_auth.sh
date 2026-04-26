#!/bin/bash

# Test script for authentication and authorization middleware
# Tests JWT and API key authentication with consciousness level checks

BASE_URL="http://localhost:8080"

echo "=== Testing Noesis Authentication & Authorization ==="
echo ""

# Test 1: No authentication (should return 401 UNAUTHORIZED)
echo "Test 1: Request without authentication"
echo "Expected: 401 UNAUTHORIZED with error_code: UNAUTHORIZED"
curl -s -X POST "${BASE_URL}/api/v1/engines/panchanga/calculate" \
  -H "Content-Type: application/json" \
  -d '{"birth_data": {"date": "1991-08-13"}}' | jq '.'
echo ""
echo "---"
echo ""

# Test 2: Invalid JWT token (should return 401 UNAUTHORIZED)
echo "Test 2: Request with invalid JWT token"
echo "Expected: 401 UNAUTHORIZED with error_code: UNAUTHORIZED"
curl -s -X POST "${BASE_URL}/api/v1/engines/panchanga/calculate" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer invalid_token_here" \
  -d '{"birth_data": {"date": "1991-08-13"}}' | jq '.'
echo ""
echo "---"
echo ""

# Test 3: Invalid API key (should return 401 UNAUTHORIZED)
echo "Test 3: Request with invalid API key"
echo "Expected: 401 UNAUTHORIZED with error_code: UNAUTHORIZED"
curl -s -X POST "${BASE_URL}/api/v1/engines/panchanga/calculate" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: invalid_api_key_123" \
  -d '{"birth_data": {"date": "1991-08-13"}}' | jq '.'
echo ""
echo "---"
echo ""

# Test 4: Health check should work without authentication
echo "Test 4: Health check (no auth required)"
echo "Expected: 200 OK"
curl -s "${BASE_URL}/health" | jq '.'
echo ""
echo "---"
echo ""

# Test 5: Legacy API should work without authentication
echo "Test 5: Legacy API endpoint (no auth required)"
echo "Expected: 200 OK or appropriate legacy response"
curl -s -X POST "${BASE_URL}/api/legacy/panchanga/calculate" \
  -H "Content-Type: application/json" \
  -d '{
    "date": "1991-08-13",
    "time": "13:31",
    "latitude": 12.9629,
    "longitude": 77.5775,
    "timezone": "Asia/Kolkata"
  }' | jq '.' || echo "Legacy endpoint might not be fully implemented yet"
echo ""
echo "---"
echo ""

echo "=== Test Summary ==="
echo "✓ Test 1-3 should return 401 with UNAUTHORIZED error_code"
echo "✓ Test 4 should return 200 with health status"
echo "✓ Test 5 should work without auth (legacy compatibility)"
echo ""
echo "To test with valid credentials:"
echo "1. Start the server: cargo run"
echo "2. Generate a valid JWT or API key"
echo "3. Use the token in Authorization header or X-API-Key header"
echo ""
