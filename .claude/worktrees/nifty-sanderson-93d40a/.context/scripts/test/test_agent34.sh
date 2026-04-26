#!/bin/bash

# Agent 34 Test Script - Vimshottari Current Period Detection

set -e

echo "===================================="
echo "Agent 34: Testing Period Detection"
echo "===================================="
echo ""

cd /Volumes/madara/2026/witnessos/Selemene-engine

echo "Step 1: Building engine-vimshottari..."
cargo build -p engine-vimshottari --lib 2>&1 | tail -5

echo ""
echo "Step 2: Running unit tests..."
cargo test -p engine-vimshottari --lib 2>&1 | grep -E "test result:|test_find_current_period|test_upcoming_transitions|test_binary_search|test_transition"

echo ""
echo "Step 3: Running specific Agent 34 tests..."
cargo test -p engine-vimshottari --lib test_find_current_period_basic -- --nocapture
cargo test -p engine-vimshottari --lib test_upcoming_transitions_chronological_order -- --nocapture

echo ""
echo "===================================="
echo "Agent 34 Tests Complete"
echo "===================================="
