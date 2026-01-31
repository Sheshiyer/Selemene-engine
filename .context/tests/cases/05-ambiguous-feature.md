# Test Case 05: Ambiguous Feature Request

## Metadata
- **ID**: 05-ambiguous-feature
- **Category**: ambiguous
- **Complexity**: medium
- **Domains Tested**: patterns, api, validation

## Prompt

```
Add task filtering.
```

## Expected Behaviors

This test evaluates how well the AI infers requirements from documentation when the prompt is vague.

1. [ ] Adds query parameters to GET /api/v1/tasks
2. [ ] Implements filtering by status (matches Task model)
3. [ ] Uses validateQuery middleware
4. [ ] Follows existing patterns
5. [ ] Uses Zod for query validation
6. [ ] Handles empty results gracefully

## Scoring Criteria

### Pattern Adherence (0-10)
- Uses validateQuery middleware
- Follows existing route patterns
- Response format correct
- Uses Zod schema

### Completeness (0-10)
- Filter by status implemented
- Handles no filters case
- Optional filters work correctly
- Returns filtered results

### Correctness (0-10)
- Filtering logic works
- No runtime errors
- Types are correct

### Security (0-10)
- Input validated
- Only filters user's own tasks
- No injection vulnerabilities

### Consistency (0-10)
- Matches existing code style
- Uses existing patterns
- Naming conventions followed

## Evaluation Notes

This test specifically measures:
- Can the AI infer appropriate filter fields from the Task model?
- Does it follow the validation patterns without being told?
- Does it maintain consistency with existing code?

The vague prompt should result in WORSE scores for the control group (single CLAUDE.md) because:
- The structured documentation makes patterns more discoverable
- Domain-specific files provide clearer guidance
- Anti-patterns are explicitly documented

## Red Flags (Automatic Deductions)

- Filters all tasks (not just user's): -8 security
- No validation: -5 completeness
- Ignores existing patterns: -5 pattern adherence
