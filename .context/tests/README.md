# Empirical Test Suite: .context vs CLAUDE.md

This test suite provides empirical evidence about AI code generation quality based on documentation structure.

## Key Finding

**Explicit code examples matter more than documentation organization.**

When documentation contains identical information, both single-file (CLAUDE.md) and modular (.context/) approaches achieve equal pattern adherence.

## Study Results

### Study 2: Fair Comparison (Identical Information)

**Test conducted**: 2025-01-06 | **n=10 per group**

| Metric | Control (CLAUDE.md) | Treatment (.context/) |
|--------|:-------------------:|:---------------------:|
| Mean Score | 10.0 / 10 | 10.0 / 10 |
| Pattern Adherence | **100% (10/10)** | **100% (10/10)** |
| Effect Size | | **0.00 (None)** |

**Conclusion**: No difference when information is identical.

### Study 1: Original (Information Asymmetry - Flawed)

The original study showed 40% vs 100% adherence, but this was due to **information asymmetry**, not organizational benefits:
- Control CLAUDE.md only showed `validateBody(schema)` example
- Treatment validation.md showed ALL THREE: `validateBody`, `validateParams`, AND `validateQuery`

This was not a fair comparison of organization - it was a comparison of completeness.

## What Actually Matters

Based on rigorous testing:

1. **Explicit code examples** are the key factor for AI pattern adherence
2. **Documentation organization** (single file vs modular) does not significantly impact code quality
3. **Modular documentation MAY help** because it naturally encourages completeness per domain
4. **The real value of .context/** is as a framework for ensuring comprehensive documentation, not as a superior format

## Practical Recommendations

Whether you use CLAUDE.md or .context/:

| Do This | Not This |
|---------|----------|
| Show explicit code examples for EVERY pattern | Describe patterns in prose only |
| Include all variations (validateBody, validateParams, validateQuery) | Show one example and expect inference |
| Document what NOT to do with bad examples | Only document correct approaches |

## Quick Start

### Running Tests

```bash
# In Claude Code, run each test:
/run-context-test 05-ambiguous-feature control
/run-context-test 05-ambiguous-feature treatment

# Score the outputs:
/score-context-test 05-ambiguous-feature control run-1234567890
```

## Test Methodology

### Experimental Design

```
┌─────────────────────────────────────────────────────────────┐
│                    CONTROLLED VARIABLES                      │
│  - Same synthetic project (tests/fixtures/project/)          │
│  - Same prompts (tests/cases/*.md)                          │
│  - Same model (Claude Sonnet)                               │
│  - Same temperature (0 for reproducibility)                  │
│  - SAME INFORMATION CONTENT                                  │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼
┌─────────────────────────┐     ┌─────────────────────────┐
│    CONTROL GROUP        │     │   TREATMENT GROUP       │
│                         │     │                         │
│  Single CLAUDE.md       │     │  Structured .context/   │
│  (all info in one file) │     │  (modular files)        │
│                         │     │                         │
│  INCLUDES validateQuery │     │  INCLUDES validateQuery │
│  example explicitly     │     │  example explicitly     │
└─────────────────────────┘     └─────────────────────────┘
```

### Why Fair Comparison Matters

The original study was methodologically flawed:
- Treatment had MORE information (3 validation examples vs 1)
- This measured information completeness, not organization

The corrected study:
- Both groups have IDENTICAL information
- Only difference is organization (1 file vs multiple files)
- Result: No difference in AI performance

## Directory Structure

```
tests/
├── README.md                    # This file
├── fixtures/
│   ├── project/                 # Synthetic codebase
│   ├── control/
│   │   └── CLAUDE.md           # Single-file (with all examples)
│   └── treatment/
│       └── .context/           # Modular files
├── cases/                       # Test prompts
├── rubrics/                     # Scoring criteria
└── results/                     # Study data
```

## Interpreting Results

### What This Study Shows

1. **AI agents follow explicit examples** - if you show a pattern, they use it
2. **AI agents may not infer patterns** - if you only show validateBody, they might not guess validateQuery exists
3. **Organization is secondary** - the information content matters more than how it's structured

### What This Study Does NOT Show

- That .context/ is useless (it may help with completeness)
- That CLAUDE.md is better (it requires equal diligence)
- That you should abandon modular documentation

## Contributing

To replicate:
1. Ensure both control and treatment have IDENTICAL information
2. Run 10+ trials per group to account for variance
3. Score blindly using the rubrics
4. Report both successes and failures

Full data: [`results/full-study-scores.md`](results/full-study-scores.md) | [`results/statistics.md`](results/statistics.md)
