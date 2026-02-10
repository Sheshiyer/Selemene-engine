#!/usr/bin/env bash
# Selemene Hourly Panchanga — Vedic Time Snapshot
# Schedule: 0 * * * * (every hour on the hour)
#
# Captures current Vedic time qualities for monitoring/alerting.

set -euo pipefail

BRIDGE_URL="${BRIDGE_URL:-http://localhost:8000}"
OUTPUT_DIR="${OUTPUT_DIR:-/var/log/selemene/hourly}"
TIMESTAMP=$(date +%Y-%m-%d_%H%M)

mkdir -p "$OUTPUT_DIR"

curl -sf -X POST "$BRIDGE_URL/tools/execute" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"engine_calculate\",
    \"arguments\": {
      \"engine_id\": \"vedic-clock\",
      \"parameters\": {
        \"latitude\": 12.9716,
        \"longitude\": 77.5946
      },
      \"consciousness_level\": 0
    }
  }" > "$OUTPUT_DIR/vedic-clock-$TIMESTAMP.json" 2>&1 || \
  echo "{\"error\": \"Vedic clock unavailable\", \"timestamp\": \"$TIMESTAMP\"}" > "$OUTPUT_DIR/vedic-clock-$TIMESTAMP.json"

echo "[$(date)] Vedic clock snapshot saved"
