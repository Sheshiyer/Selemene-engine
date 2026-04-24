#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

echo "[1/4] Running tarot engine unit and HTTP integration tests"
(
  cd ts-engines
  bun test src/engines/tarot/engine.test.ts tests/integration.test.ts
)

echo "[2/4] Running decision-support workflow tests"
cargo test -p noesis-orchestrator decision_support -- --nocapture

echo "[3/4] Running targeted persistence regression"
cargo test -p noesis-data save_reading_preserves_nested_tarot_result_payloads -- --nocapture

echo "[4/4] Spot-checking docs and fixtures updated for tarot integration"
grep -n 'yes_no' docs/api/engines.md docs/api/workflows.md docs/portal/docs/engines/tarot.md docs/portal/docs/workflows/decision-support.md tests/fixtures/expected_outputs/engine_schemas.json > /dev/null

echo "Engine/workflow feature integration checks completed successfully"
