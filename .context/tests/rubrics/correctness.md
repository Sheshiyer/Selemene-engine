# Scoring Rubric: Correctness

Measures whether the code works correctly without bugs.

## Score Definitions

### 10 - Flawless
- No bugs or logic errors
- Handles all input cases correctly
- TypeScript types accurate
- No runtime errors possible

### 9 - Excellent
- No functional bugs
- One very minor type annotation issue

### 8 - Very Good
- No functional bugs
- Minor type issues
- Works correctly in all tested cases

### 7 - Good
- No critical bugs
- Minor logic issue in edge case
- Works for common cases

### 6 - Acceptable
- Works for happy path
- One bug in error handling
- Minor logic issues

### 5 - Marginal
- Works for basic case
- Multiple minor bugs
- Some error paths broken

### 4 - Below Standard
- Partially works
- One significant bug
- Error handling broken

### 3 - Poor
- Major bugs present
- May crash on valid input
- Logic fundamentally flawed

### 2 - Very Poor
- Does not work correctly
- Multiple critical bugs
- Would fail basic testing

### 1 - Minimal
- Almost non-functional
- Crashes immediately

### 0 - Broken
- Does not compile/run
- Syntax errors
- Completely broken

## Bug Severity Classification

| Severity | Description | Deduction |
|----------|-------------|-----------|
| Critical | Crashes, data loss, security breach | -4 to -10 |
| Major | Feature doesn't work | -3 |
| Moderate | Works but incorrectly | -2 |
| Minor | Edge case failure | -1 |
| Trivial | Cosmetic, type warnings | -0.5 |

## Verification Checklist

- [ ] Code compiles without errors
- [ ] Happy path works correctly
- [ ] Error cases return appropriate errors
- [ ] Types are accurate
- [ ] Async operations handled correctly
- [ ] No obvious logic errors
