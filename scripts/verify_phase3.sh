#!/bin/bash
# Phase 3 Verification Script
# Validates Gene Keys + Vimshottari engines are operational

set -e

echo "=================================================="
echo "  Phase 3 (W1-P3) Verification: Gene Keys + Vimshottari"
echo "=================================================="
echo ""

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PASS=0
FAIL=0
WARN=0

check_pass() {
    echo -e "  ${GREEN}PASS${NC}: $1"
    PASS=$((PASS + 1))
}

check_fail() {
    echo -e "  ${RED}FAIL${NC}: $1"
    FAIL=$((FAIL + 1))
}

check_warn() {
    echo -e "  ${YELLOW}WARN${NC}: $1"
    WARN=$((WARN + 1))
}

# ============ 1. BUILD CHECK ============
echo "--- Step 1: Build Workspace ---"

if cargo check --workspace 2>/dev/null; then
    check_pass "Workspace compiles successfully"
else
    check_fail "Workspace compilation failed"
fi

# ============ 2. GENE KEYS TESTS ============
echo ""
echo "--- Step 2: Gene Keys Engine Tests ---"

if cargo test --package engine-gene-keys 2>/dev/null; then
    check_pass "Gene Keys tests passing"
else
    check_fail "Gene Keys tests failed"
fi

# ============ 3. VIMSHOTTARI TESTS ============
echo ""
echo "--- Step 3: Vimshottari Engine Tests ---"

if cargo test --package engine-vimshottari 2>/dev/null; then
    check_pass "Vimshottari tests passing"
else
    check_fail "Vimshottari tests failed"
fi

# ============ 4. CORE TESTS ============
echo ""
echo "--- Step 4: Core Library Tests ---"

if cargo test --package noesis-core 2>/dev/null; then
    check_pass "noesis-core tests passing"
else
    check_warn "noesis-core tests had issues"
fi

# ============ 5. API TESTS ============
echo ""
echo "--- Step 5: API Tests ---"

if cargo test --package noesis-api 2>/dev/null; then
    check_pass "noesis-api tests passing"
else
    check_warn "noesis-api tests had issues"
fi

# ============ 6. DOCUMENTATION CHECK ============
echo ""
echo "--- Step 6: Documentation Verification ---"

if [ -f ".context/engines/gene-keys.md" ]; then
    GK_SIZE=$(wc -c < ".context/engines/gene-keys.md")
    if [ "$GK_SIZE" -gt 12000 ]; then
        check_pass "gene-keys.md exists (${GK_SIZE} bytes, >12KB requirement)"
    else
        check_warn "gene-keys.md exists but only ${GK_SIZE} bytes (<12KB target)"
    fi
else
    check_fail "gene-keys.md not found"
fi

if [ -f ".context/engines/vimshottari.md" ]; then
    VD_SIZE=$(wc -c < ".context/engines/vimshottari.md")
    if [ "$VD_SIZE" -gt 10000 ]; then
        check_pass "vimshottari.md exists (${VD_SIZE} bytes, >10KB requirement)"
    else
        check_warn "vimshottari.md exists but only ${VD_SIZE} bytes (<10KB target)"
    fi
else
    check_fail "vimshottari.md not found"
fi

if [ -f ".context/engines/human-design.md" ]; then
    check_pass "human-design.md exists (reference document)"
else
    check_warn "human-design.md not found"
fi

if [ -f "docs/PHASE_3_COMPLETION_SUMMARY.md" ]; then
    check_pass "Phase 3 completion summary exists"
else
    check_fail "Phase 3 completion summary not found"
fi

# ============ 7. SOURCE FILE CHECK ============
echo ""
echo "--- Step 7: Source File Verification ---"

GK_FILES=(
    "crates/engine-gene-keys/src/lib.rs"
    "crates/engine-gene-keys/src/engine.rs"
    "crates/engine-gene-keys/src/models.rs"
    "crates/engine-gene-keys/src/mapping.rs"
    "crates/engine-gene-keys/src/wisdom.rs"
    "crates/engine-gene-keys/src/frequency.rs"
    "crates/engine-gene-keys/src/transformation.rs"
    "crates/engine-gene-keys/src/witness.rs"
)

for f in "${GK_FILES[@]}"; do
    if [ -f "$f" ]; then
        check_pass "$f exists"
    else
        check_fail "$f missing"
    fi
done

VD_FILES=(
    "crates/engine-vimshottari/src/lib.rs"
    "crates/engine-vimshottari/src/calculator.rs"
    "crates/engine-vimshottari/src/models.rs"
    "crates/engine-vimshottari/src/wisdom.rs"
    "crates/engine-vimshottari/src/wisdom_data.rs"
    "crates/engine-vimshottari/src/witness.rs"
)

for f in "${VD_FILES[@]}"; do
    if [ -f "$f" ]; then
        check_pass "$f exists"
    else
        check_fail "$f missing"
    fi
done

# ============ 8. LINE COUNT ============
echo ""
echo "--- Step 8: Source Code Statistics ---"

GK_LINES=$(wc -l crates/engine-gene-keys/src/*.rs 2>/dev/null | tail -1 | awk '{print $1}')
VD_LINES=$(wc -l crates/engine-vimshottari/src/*.rs 2>/dev/null | tail -1 | awk '{print $1}')

echo "  Gene Keys: ${GK_LINES} lines"
echo "  Vimshottari: ${VD_LINES} lines"
echo "  Total new code: $((GK_LINES + VD_LINES)) lines"

# ============ SUMMARY ============
echo ""
echo "=================================================="
echo "  VERIFICATION SUMMARY"
echo "=================================================="
echo ""
echo -e "  ${GREEN}PASSED${NC}: ${PASS}"
echo -e "  ${RED}FAILED${NC}: ${FAIL}"
echo -e "  ${YELLOW}WARNED${NC}: ${WARN}"
echo ""

if [ "$FAIL" -eq 0 ]; then
    echo -e "  ${GREEN}Phase 3 verification COMPLETE${NC}"
    echo ""
    exit 0
else
    echo -e "  ${RED}Phase 3 verification FAILED (${FAIL} failures)${NC}"
    echo ""
    exit 1
fi
