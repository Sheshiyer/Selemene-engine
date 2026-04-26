#!/bin/bash

# Run .context vs CLAUDE.md comparison tests using Claude Code
#
# Usage:
#   ./run-with-claude-code.sh                    # Run all tests
#   ./run-with-claude-code.sh 01-simple-endpoint # Run single test
#   ./run-with-claude-code.sh --score-only       # Only score existing outputs

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

CASES=(
  "01-simple-endpoint"
  "02-auth-middleware"
  "03-cross-domain-feature"
  "04-security-fix"
  "05-ambiguous-feature"
  "06-refactor-antipattern"
)
GROUPS=("control" "treatment")
RUNS_PER_TEST=3  # Reduced for manual runs

# Parse arguments
SINGLE_CASE=""
SCORE_ONLY=false

while [[ $# -gt 0 ]]; do
  case $1 in
    --score-only)
      SCORE_ONLY=true
      shift
      ;;
    *)
      SINGLE_CASE="$1"
      shift
      ;;
  esac
done

if [[ -n "$SINGLE_CASE" ]]; then
  CASES=("$SINGLE_CASE")
fi

echo "=============================================="
echo ".context vs CLAUDE.md Comparison Test"
echo "=============================================="
echo "Cases: ${CASES[*]}"
echo "Groups: ${GROUPS[*]}"
echo "Runs per test: $RUNS_PER_TEST"
echo ""

if [[ "$SCORE_ONLY" == "false" ]]; then
  echo "Running tests with Claude Code..."
  echo ""
  echo "For each test, run in Claude Code:"
  echo ""

  for case_id in "${CASES[@]}"; do
    for group in "${GROUPS[@]}"; do
      for ((run=1; run<=RUNS_PER_TEST; run++)); do
        echo "  /run-context-test $case_id $group"
      done
    done
  done

  echo ""
  echo "After running all tests, execute:"
  echo "  ./run-with-claude-code.sh --score-only"
fi

if [[ "$SCORE_ONLY" == "true" ]]; then
  echo "Scoring outputs..."
  echo ""
  echo "For each output in tests/outputs/, run:"
  echo ""

  for case_id in "${CASES[@]}"; do
    for group in "${GROUPS[@]}"; do
      output_dir="tests/outputs/$case_id/$group"
      if [[ -d "$output_dir" ]]; then
        for output_file in "$output_dir"/*.md; do
          if [[ -f "$output_file" ]]; then
            run_id=$(basename "$output_file" .md)
            echo "  /score-context-test $case_id $group $run_id"
          fi
        done
      fi
    done
  done

  echo ""
  echo "After scoring, run analysis:"
  echo "  cd tests && npx ts-node scripts/analyze.ts"
fi

echo ""
echo "=============================================="
