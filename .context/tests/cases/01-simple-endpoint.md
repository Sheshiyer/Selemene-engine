# Test Case 01: Simple Endpoint

## Metadata
- **ID**: 01-simple-endpoint
- **Category**: simple
- **Complexity**: low
- **Domains Tested**: api, patterns, validation

## Prompt

```
Add a new endpoint to get the currently authenticated user's profile.

Endpoint: GET /api/v1/users/me
Returns: The authenticated user's profile data

Implement this in the existing codebase.
```

## Expected Behaviors

1. [ ] Creates route handler in appropriate location (src/routes/)
2. [ ] Uses `authenticate` middleware
3. [ ] Returns response in correct format: `{ success: true, data: { user: {...} } }`
4. [ ] Does NOT return passwordHash in response
5. [ ] Uses correct HTTP status (200)
6. [ ] Follows existing code patterns (async/await, Request/Response types)

## Scoring Criteria

### Pattern Adherence (0-10)
- Uses Router pattern matching existing routes
- Response format matches `{ success: true, data: {...} }`
- Uses async/await consistently
- Follows existing file structure

### Completeness (0-10)
- Endpoint returns user profile
- Authentication is enforced
- Correct HTTP method (GET)
- Proper route path

### Correctness (0-10)
- No runtime errors
- User data retrieved correctly
- Proper TypeScript types

### Security (0-10)
- Uses authenticate middleware
- Does NOT expose passwordHash
- Accesses req.user safely

### Consistency (0-10)
- Code style matches existing routes
- Naming conventions followed
- Import style consistent

## Red Flags (Automatic Deductions)

- Returns passwordHash: -10 security
- Missing authentication: -10 security
- Wrong response format: -3 pattern adherence
- Uses .then() instead of await: -2 consistency
