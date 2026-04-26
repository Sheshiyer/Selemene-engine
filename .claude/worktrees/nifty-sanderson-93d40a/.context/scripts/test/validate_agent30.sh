#!/bin/bash
# Quick validation script for Agent 30 deliverables

echo "============================================"
echo "Agent 30: Deliverable Validation"
echo "============================================"
echo ""

cd /Volumes/madara/2026/witnessos/Selemene-engine

echo "[1/4] Checking file existence..."

FILES=(
    "crates/noesis-api/tests/gene_keys_integration.rs"
    "run_agent30_tests.sh"
    "AGENT30_COMPLETION_REPORT.md"
)

for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "✓ $file exists"
    else
        echo "✗ $file missing!"
        exit 1
    fi
done

echo ""
echo "[2/4] Counting test functions..."

# Count #[tokio::test] annotations
integration_tests=$(grep -c "#\[tokio::test\]" crates/noesis-api/tests/gene_keys_integration.rs || echo "0")
echo "  - gene_keys_integration.rs: $integration_tests tests"

# Check cross-engine tests in integration_tests.rs
cross_engine_tests=$(grep -c "test_hd_to_gene_keys\|test_gene_keys_directly\|test_gene_keys_consciousness_level_affects" crates/noesis-api/tests/integration_tests.rs || echo "0")
echo "  - Cross-engine tests: $cross_engine_tests tests"

total_api_tests=$((integration_tests + cross_engine_tests))
echo "  - Total API tests: $total_api_tests"

# Archetypal depth tests (will be created by run script)
echo "  - Archetypal depth tests: Will be created by run_agent30_tests.sh"

echo ""
echo "[3/4] Validating test structure..."

# Check for required test patterns
echo "  Checking for key test patterns..."

if grep -q "test_gene_keys_with_birth_data" crates/noesis-api/tests/gene_keys_integration.rs; then
    echo "  ✓ birth_data mode test present"
fi

if grep -q "test_gene_keys_with_hd_gates" crates/noesis-api/tests/gene_keys_integration.rs; then
    echo "  ✓ hd_gates mode test present"
fi

if grep -q "test_consciousness_level_adaptation" crates/noesis-api/tests/gene_keys_integration.rs; then
    echo "  ✓ consciousness level test present"
fi

if grep -q "test_witness_prompt" crates/noesis-api/tests/gene_keys_integration.rs; then
    echo "  ✓ witness prompt test present"
fi

if grep -q "test_archetypal_depth" crates/noesis-api/tests/gene_keys_integration.rs; then
    echo "  ✓ archetypal depth test present"
fi

if grep -q "test_hd_to_gene_keys_workflow" crates/noesis-api/tests/integration_tests.rs; then
    echo "  ✓ cross-engine workflow test present"
fi

echo ""
echo "[4/4] Checking data files..."

if [ -f "data/gene-keys/archetypes.json" ]; then
    echo "  ✓ archetypes.json exists"
    
    # Quick validation of JSON structure
    keys_count=$(grep -o '"number":' data/gene-keys/archetypes.json | wc -l | tr -d ' ')
    echo "  ✓ Found $keys_count Gene Keys in dataset"
    
    if [ "$keys_count" -ge 64 ]; then
        echo "  ✓ All 64 Gene Keys present"
    else
        echo "  ⚠ Warning: Expected 64 keys, found $keys_count"
    fi
else
    echo "  ✗ archetypes.json missing!"
    exit 1
fi

echo ""
echo "============================================"
echo "✓ Agent 30 deliverables validated"
echo "============================================"
echo ""
echo "To run tests:"
echo "  chmod +x run_agent30_tests.sh"
echo "  ./run_agent30_tests.sh"
echo ""
echo "Or run individually:"
echo "  cargo test -p noesis-api --test gene_keys_integration"
echo "  cargo test -p engine-gene-keys --test archetypal_depth_validation"
echo "  cargo test -p noesis-api --test integration_tests -- test_hd_to_gene_keys"
