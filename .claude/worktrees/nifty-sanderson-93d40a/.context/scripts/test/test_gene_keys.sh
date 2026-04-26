#!/bin/bash

# Test Gene Keys Engine API Endpoint

API_BASE="http://localhost:8080/api/v1"

echo "=== Testing Gene Keys Engine ==="
echo ""

# Test 1: Mode 1 - with birth_data
echo "Test 1: Gene Keys with birth_data"
curl -X POST "${API_BASE}/engines/gene-keys/calculate" \
  -H "Content-Type: application/json" \
  -d '{
    "engine_id": "gene-keys",
    "birth_data": {
      "date": "1985-06-15",
      "time": "14:30:00",
      "latitude": 40.7128,
      "longitude": -74.0060,
      "timezone": "America/New_York"
    },
    "current_time": "2026-01-31T05:00:00Z",
    "precision": "Standard",
    "options": {
      "consciousness_level": 3
    }
  }' | jq .

echo ""
echo ""

# Test 2: Mode 2 - with hd_gates
echo "Test 2: Gene Keys with hd_gates"
curl -X POST "${API_BASE}/engines/gene-keys/calculate" \
  -H "Content-Type: application/json" \
  -d '{
    "engine_id": "gene-keys",
    "current_time": "2026-01-31T05:00:00Z",
    "precision": "Standard",
    "options": {
      "hd_gates": {
        "personality_sun": 17,
        "personality_earth": 18,
        "design_sun": 45,
        "design_earth": 26
      },
      "consciousness_level": 4
    }
  }' | jq .

echo ""
echo ""

# Test 3: Shadow level (consciousness_level 1)
echo "Test 3: Shadow prompts (level 1)"
curl -X POST "${API_BASE}/engines/gene-keys/calculate" \
  -H "Content-Type: application/json" \
  -d '{
    "engine_id": "gene-keys",
    "current_time": "2026-01-31T05:00:00Z",
    "precision": "Standard",
    "options": {
      "hd_gates": {
        "personality_sun": 17,
        "personality_earth": 18,
        "design_sun": 45,
        "design_earth": 26
      },
      "consciousness_level": 1
    }
  }' | jq .witness_prompt

echo ""
echo ""

# Test 4: Siddhi level (consciousness_level 6)
echo "Test 4: Siddhi prompts (level 6)"
curl -X POST "${API_BASE}/engines/gene-keys/calculate" \
  -H "Content-Type: application/json" \
  -d '{
    "engine_id": "gene-keys",
    "current_time": "2026-01-31T05:00:00Z",
    "precision": "Standard",
    "options": {
      "hd_gates": {
        "personality_sun": 17,
        "personality_earth": 18,
        "design_sun": 45,
        "design_earth": 26
      },
      "consciousness_level": 6
    }
  }' | jq .witness_prompt

echo ""
echo ""
echo "=== Tests Complete ==="
