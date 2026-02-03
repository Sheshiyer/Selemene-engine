#!/bin/bash
set -e

echo "=== Building engine-human-design ==="
cd crates/engine-human-design
cargo build --lib
echo "✓ HD engine built successfully"

echo ""
echo "=== Running witness tests ==="
cargo test --lib witness::tests --no-fail-fast
echo "✓ Witness tests passed"

echo ""
echo "=== Running engine tests ==="
cargo test --lib engine::tests --no-fail-fast
echo "✓ Engine tests passed"

echo ""
echo "=== Building noesis-api ==="
cd ../noesis-api
cargo build
echo "✓ API built successfully"

echo ""
echo "=== Running HD integration tests ==="
cargo test --test integration_tests test_hd_engine --no-fail-fast
echo "✓ HD integration tests passed"

echo ""
echo "=== ✅ ALL TASKS COMPLETE ==="
echo "Tasks W1-S4-08, W1-S4-09, W1-S4-10 implemented successfully"
