#!/usr/bin/env bash
# Manual local smoke; provision sidecars and dependencies before execution.
set -euo pipefail
sudo apt-get update -qq && sudo apt-get install -y -qq jq
cd python-services
python -m uvicorn biofield_cv_service.main:app --host 127.0.0.1 --port 8002 --log-level error &
UV_PID=$!
cleanup() {
  kill "$UV_PID" 2>/dev/null || true
  wait "$UV_PID" 2>/dev/null || true
}
trap cleanup EXIT

echo "=== BIOFIELD HEALTH (T-004 capture contract) ==="
ready=0
for attempt in {1..20}; do
  if curl --fail-with-body --show-error --silent --max-time 3 \
      --output "$RUNNER_TEMP/biofield-health.json" \
      http://127.0.0.1:8002/health \
    && jq -e '
      .status == "healthy" and
      .service == "biofield-cv" and
      .opencv_available == true and
      .numpy_available == true
    ' "$RUNNER_TEMP/biofield-health.json" >/dev/null; then
    ready=1
    break
  fi
  echo "Biofield readiness attempt ${attempt}/20 failed"
  sleep 1
done
if [[ "$ready" -ne 1 ]]; then
  echo "Biofield sidecar did not become ready within 20 seconds" >&2
  exit 1
fi

export BIOFIELD_SMOKE_IMAGE="$RUNNER_TEMP/biofield-smoke.png"
python - <<'PY'
import os
import cv2
import numpy as np

image = np.zeros((64, 64, 3), dtype=np.uint8)
cv2.circle(image, (32, 32), 20, (180, 160, 140), -1)
cv2.rectangle(image, (20, 32), (44, 60), (160, 140, 120), -1)
if not cv2.imwrite(os.environ["BIOFIELD_SMOKE_IMAGE"], image):
    raise SystemExit("failed to write biofield smoke fixture")
PY

echo "=== BIOFIELD ANALYZE (dual-biofield real-cv/v1 + 11 metrics) ==="
curl --fail-with-body --show-error --silent --max-time 30 \
  --output "$RUNNER_TEMP/biofield-analyze.json" \
  -X POST http://127.0.0.1:8002/analyze \
  -F "image=@$BIOFIELD_SMOKE_IMAGE;type=image/png"
jq -e '
  .contract_version == "biofield-cv/v1" and
  .analysis_version == "real-cv/v1" and
  (.metrics | keys | length == 11) and
  (.metrics.light_quanta_density | type == "number") and
  (.metrics.energy_analysis.low | type == "number") and
  (.metrics.fractal_dimension | type == "number") and
  (.metrics.body_symmetry | type == "number") and
  (.algorithms_run | length == 11) and
  (.quality_assessment.sufficient_quality | type == "boolean") and
  (.processing_time_ms | type == "number")
' "$RUNNER_TEMP/biofield-analyze.json"

