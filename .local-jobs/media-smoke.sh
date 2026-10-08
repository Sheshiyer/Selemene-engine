#!/usr/bin/env bash
# Manual local smoke; provision sidecars and dependencies before execution.
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
command -v jq >/dev/null || { echo "Install jq before running this smoke" >&2; exit 1; }
RUNNER_TEMP="$(mktemp -d "${TMPDIR:-/tmp}/selemene-smoke.XXXXXX")"
cd "$REPO_ROOT/ts-engines"
bun run start &
SERVER_PID=$!
cleanup() {
  rm -rf "$RUNNER_TEMP"
  kill "$SERVER_PID" 2>/dev/null || true
  wait "$SERVER_PID" 2>/dev/null || true
}
trap cleanup EXIT

echo "=== HEALTH ==="
ready=0
for attempt in {1..20}; do
  if curl --fail-with-body --show-error --silent \
      --output "$RUNNER_TEMP/ts-health.json" \
      http://localhost:3001/health \
    && jq -e '.status == "healthy"' "$RUNNER_TEMP/ts-health.json" >/dev/null; then
    ready=1
    break
  fi
  echo "TS engine readiness attempt ${attempt}/20 failed"
  sleep 1
done
if [[ "$ready" -ne 1 ]]; then
  echo "TS engine did not become healthy within 20 seconds" >&2
  exit 1
fi
jq -c '{status, engines, version}' "$RUNNER_TEMP/ts-health.json"

echo "=== RAAGA (strudel audio per T-005 FROZEN; refs bootstrap + resources-and-assets + gaps + goal-understanding + P1W1-CONTRACTS-FROZEN) ==="
curl --fail-with-body --show-error --silent \
  --output "$RUNNER_TEMP/raaga.json" \
  -X POST http://localhost:3001/engines/raaga/calculate \
  -H 'Content-Type: application/json' \
  -d '{"consciousness_level":2,"parameters":{"melakarta":1},"consent":{"granted":true,"scopes":["raaga-audio"],"timestamp":"2026-07-17T12:00:00Z"}}'
jq -e '
  .engine_id == "raaga" and
  .result.melakarta.num == 1 and
  (.generated_audio.strudel_ratios | type == "array" and length == 8) and
  .generated_audio.metadata.engine == "raaga"
' "$RUNNER_TEMP/raaga.json"

echo "=== SIGIL (deterministic guidance-only contract; provider media remains separately mocked/config-gated per T-003 FROZEN) ==="
curl --fail-with-body --show-error --silent \
  --output "$RUNNER_TEMP/sigil.json" \
  -X POST http://localhost:3001/engines/sigil-forge/calculate \
  -H 'Content-Type: application/json' \
  -d '{"consciousness_level":1,"parameters":{"intention":"witness patterns"},"consent":{"granted":true,"scopes":["sigil-gen"],"timestamp":"2026-07-17T12:00:00Z"}}'
jq -e '
  .engine_id == "sigil-forge" and
  (.result.method.id | type == "string" and length > 0) and
  .result.svg_preview.status == "absent" and
  .result.generated_image == null
' "$RUNNER_TEMP/sigil.json"

