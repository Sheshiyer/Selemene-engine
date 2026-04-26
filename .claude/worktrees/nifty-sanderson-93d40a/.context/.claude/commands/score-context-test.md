You are scoring a test output for the .context vs CLAUDE.md comparison study.

## Test to Score

**Arguments**: $ARGUMENTS (e.g., "01-simple-endpoint control run-1704067200000")

Parse to get: case_id, group, run_id

## Instructions

1. **Load the test case** from `tests/cases/{case_id}.md` - get expected behaviors and scoring criteria

2. **Load the output** from `tests/outputs/{case_id}/{group}/{run_id}.md`

3. **Load the rubrics** from `tests/rubrics/`:
   - pattern-adherence.md
   - completeness.md
   - correctness.md
   - security.md
   - consistency.md

4. **Score the output** against each dimension (0-10)

5. **Save scores** to `tests/results/scores/{case_id}/{group}/{run_id}-scores.json`

Use this JSON format:
```json
{
  "case_id": "...",
  "group": "...",
  "run_id": "...",
  "scores": {
    "pattern_adherence": { "score": X, "notes": "...", "deductions": [] },
    "completeness": { "score": X, "notes": "...", "deductions": [] },
    "correctness": { "score": X, "notes": "...", "deductions": [] },
    "security": { "score": X, "notes": "...", "deductions": [] },
    "consistency": { "score": X, "notes": "...", "deductions": [] }
  },
  "total": X
}
```

## Scoring Guidelines

- Be objective and consistent
- Apply deductions as specified in rubrics
- Note specific examples supporting your scores
