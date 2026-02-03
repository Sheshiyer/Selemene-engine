#!/bin/bash
# Test script for legacy API endpoints

set -e

BASE_URL="http://localhost:8080"

echo "Testing legacy Panchanga endpoint..."
echo "POST $BASE_URL/api/legacy/panchanga/calculate"

curl -s -X POST "$BASE_URL/api/legacy/panchanga/calculate" \
  -H "Content-Type: application/json" \
  -d '{
    "date": "1991-08-13",
    "time": "13:31",
    "latitude": 12.9716,
    "longitude": 77.5946,
    "timezone": "Asia/Kolkata"
  }' | jq '.'

echo ""
echo "Testing legacy Ghati current endpoint..."
echo "GET $BASE_URL/api/legacy/ghati/current"

curl -s -X GET "$BASE_URL/api/legacy/ghati/current" \
  -H "Content-Type: application/json" \
  -d '{
    "latitude": 12.9716,
    "longitude": 77.5946
  }' | jq '.'

echo ""
echo "All legacy endpoint tests completed!"
