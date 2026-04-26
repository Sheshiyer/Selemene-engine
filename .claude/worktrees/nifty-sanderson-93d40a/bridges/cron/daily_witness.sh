#!/usr/bin/env bash
# Selemene Daily Witness — Brahma Muhurta Briefing
# Schedule: 0 4 * * * (4:00 AM daily, ~96 minutes before sunrise)
#
# Calls the universal bridge to run daily-practice workflow
# and generate a morning witnessing report.

set -euo pipefail

BRIDGE_URL="${BRIDGE_URL:-http://localhost:8000}"
OUTPUT_DIR="${OUTPUT_DIR:-/var/log/selemene/daily}"
DATE=$(date +%Y-%m-%d)

mkdir -p "$OUTPUT_DIR"

echo "[$(date)] Starting Brahma Muhurta daily witness..."

# Run daily-practice workflow (panchanga + vedic-clock + biorhythm)
RESULT=$(curl -sf -X POST "$BRIDGE_URL/tools/execute" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"workflow_execute\",
    \"arguments\": {
      \"workflow_id\": \"daily-practice\",
      \"parameters\": {
        \"latitude\": 12.9716,
        \"longitude\": 77.5946,
        \"timezone\": \"Asia/Kolkata\"
      },
      \"consciousness_level\": 0
    }
  }" 2>&1) || RESULT="{\"error\": \"Bridge unreachable\"}"

echo "$RESULT" > "$OUTPUT_DIR/witness-$DATE.json"
echo "[$(date)] Daily witness saved to $OUTPUT_DIR/witness-$DATE.json"

# Optional: Get panchanga for today
PANCHANGA=$(curl -sf -X POST "$BRIDGE_URL/tools/execute" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"engine_calculate\",
    \"arguments\": {
      \"engine_id\": \"panchanga\",
      \"parameters\": {
        \"latitude\": 12.9716,
        \"longitude\": 77.5946
      },
      \"consciousness_level\": 0
    }
  }" 2>&1) || PANCHANGA="{\"error\": \"Panchanga unavailable\"}"

echo "$PANCHANGA" > "$OUTPUT_DIR/panchanga-$DATE.json"
echo "[$(date)] Panchanga saved to $OUTPUT_DIR/panchanga-$DATE.json"
