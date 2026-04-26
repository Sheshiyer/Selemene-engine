# Scoring Rubric: Consistency

Measures how well the code matches existing codebase style.

## Score Definitions

### 10 - Identical
- Indistinguishable from existing code
- Same patterns, naming, formatting
- Uses existing utilities
- File placement correct

### 9 - Excellent
- Very close to existing style
- One minor stylistic difference

### 8 - Very Good
- Consistent with existing code
- Few minor differences
- Would pass code review easily

### 7 - Good
- Generally consistent
- Some stylistic differences
- Overall matches codebase

### 6 - Acceptable
- Core style matches
- Multiple minor differences
- Needs some cleanup

### 5 - Marginal
- Some consistency
- Mix of matching and different styles
- Noticeable differences

### 4 - Below Standard
- Partial consistency
- Different approach visible
- Would require significant review

### 3 - Poor
- Inconsistent with codebase
- Different patterns used
- Stands out as different

### 2 - Very Poor
- Almost completely different style
- Different conventions throughout

### 1 - Minimal
- Only accidental consistency

### 0 - None
- Completely different coding style

## Consistency Factors

| Factor | Weight | Check |
|--------|--------|-------|
| Naming conventions | 2 | camelCase, descriptive names |
| File structure | 2 | Correct directory, file naming |
| Import style | 1 | Named imports, order |
| Error handling | 2 | Uses AppError consistently |
| Response format | 2 | Matches existing format |
| TypeScript usage | 1 | Proper types, interfaces |

## Anti-Pattern Deductions

| Item | Deduction |
|------|-----------|
| Uses `var` instead of `const/let` | -2 |
| Uses `.then()` with async/await | -2 |
| Includes `console.log` | -2 |
| Different naming convention | -1 |
| Different response format | -2 |
| Direct repository access (skips service) | -2 |
| Magic strings/numbers | -1 |
