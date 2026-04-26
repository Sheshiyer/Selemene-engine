# Test Case 03: Cross-Domain Feature

## Metadata
- **ID**: 03-cross-domain-feature
- **Category**: cross-domain
- **Complexity**: medium
- **Domains Tested**: api, database, patterns, validation, auth

## Prompt

```
Add a feature to share tasks with other users.

Requirements:
1. New endpoint: POST /api/v1/tasks/:taskId/share
   Body: { email: string }
2. Creates a "shared task" relationship
3. User can only share their own tasks
4. Shared user can view but not edit the task

Implement the endpoint and any necessary supporting code.
```

## Expected Behaviors

1. [ ] Creates new endpoint in task routes
2. [ ] Validates task ownership before sharing
3. [ ] Uses Zod schema for email validation
4. [ ] Handles case where target user doesn't exist
5. [ ] Uses appropriate error codes
6. [ ] Authentication required
7. [ ] Returns consistent response format
8. [ ] Creates or extends data model appropriately

## Scoring Criteria

### Pattern Adherence (0-10)
- Follows existing route patterns
- Uses validateBody/validateParams
- Response format matches project standard
- Uses AppError for errors

### Completeness (0-10)
- Endpoint implemented
- Ownership check included
- User lookup works
- Share relationship created
- All error cases handled

### Correctness (0-10)
- Logic is correct
- No runtime errors
- Proper async/await usage
- Types are accurate

### Security (0-10)
- Authentication enforced
- Ownership verified (FORBIDDEN if not owner)
- Input validated
- No data leakage

### Consistency (0-10)
- Code style matches existing
- Error handling consistent
- File placement appropriate

## Red Flags (Automatic Deductions)

- No ownership check: -8 security
- Missing authentication: -10 security
- Wrong error codes: -3 pattern adherence
- Inconsistent response format: -3 pattern adherence
