# Scoring Rubric: Pattern Adherence

Measures how well the generated code follows documented patterns.

## Score Definitions

### 10 - Exemplary
- Uses all documented patterns exactly as specified
- Response format matches `{ success: true, data: {...} }`
- Uses validateBody/validateParams/validateQuery correctly
- Uses AppError for all error cases
- Service layer pattern followed
- Repository pattern followed

### 9 - Excellent
- All major patterns followed
- One very minor deviation (e.g., slightly different variable name)

### 8 - Very Good
- All major patterns followed
- Minor deviation in one area that doesn't affect functionality

### 7 - Good
- Most patterns followed
- One pattern slightly deviated but reasonable

### 6 - Acceptable
- Core patterns followed
- Multiple minor deviations
- Functional but not ideal

### 5 - Marginal
- Some patterns followed
- Mix of correct and incorrect patterns
- Shows awareness of patterns but inconsistent

### 4 - Below Standard
- Few patterns followed correctly
- Multiple pattern violations
- Response format incorrect

### 3 - Poor
- Major patterns ignored
- Creates own patterns instead of following docs
- Error handling non-standard

### 2 - Very Poor
- Almost no pattern adherence
- Completely different approach than documented

### 1 - Minimal
- Only accidental pattern matches

### 0 - None
- No pattern adherence whatsoever

## Checklist

| Item | Points | Present? |
|------|--------|----------|
| Response format correct | 2 | [ ] |
| Uses validation middleware | 2 | [ ] |
| Uses AppError | 2 | [ ] |
| Service layer pattern | 2 | [ ] |
| Repository pattern | 1 | [ ] |
| Async/await consistency | 1 | [ ] |

## Anti-Pattern Deductions

| Anti-Pattern | Deduction |
|--------------|-----------|
| .then() mixed with await | -1 |
| Direct error response (not AppError) | -2 |
| Wrong response format | -2 |
| Skips validation middleware | -3 |
