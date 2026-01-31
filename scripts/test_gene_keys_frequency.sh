#!/bin/bash
# Test script for Gene Keys frequency assessment and transformation pathways

set -e

cd "$(dirname "$0")/.."

echo "════════════════════════════════════════════════════════════"
echo "  Testing Gene Keys Frequency Assessment (W1-S5-05)"
echo "════════════════════════════════════════════════════════════"
echo ""

echo "Building engine-gene-keys..."
cargo build -p engine-gene-keys

echo ""
echo "Running unit tests..."
cargo test -p engine-gene-keys --lib

echo ""
echo "Running frequency module tests..."
cargo test -p engine-gene-keys --lib frequency

echo ""
echo "Running transformation module tests..."
cargo test -p engine-gene-keys --lib transformation

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Testing Gene Keys Example"
echo "════════════════════════════════════════════════════════════"
echo ""

echo "Running frequency assessment example..."
cargo run --example gene_keys_frequency_assessment

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  All Tests Passed ✓"
echo "════════════════════════════════════════════════════════════"
