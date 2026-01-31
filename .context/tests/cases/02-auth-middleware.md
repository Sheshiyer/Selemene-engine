# Test Case 02: Password Reset Feature

## Metadata
- **ID**: 02-auth-middleware
- **Category**: security-critical
- **Complexity**: medium
- **Domains Tested**: auth, security, validation, patterns

## Prompt

```
Add a password change endpoint for authenticated users.

Endpoint: PATCH /api/v1/auth/password
Body: { currentPassword: string, newPassword: string }

Requirements:
- User must be authenticated
- Must verify current password before changing
- New password must be at least 8 characters

Implement this in the existing codebase.
```

## Expected Behaviors

1. [ ] Uses Argon2id for password hashing (NOT bcrypt)
2. [ ] Uses authenticate middleware
3. [ ] Validates current password before changing
4. [ ] Uses Zod schema for input validation
5. [ ] Returns proper error codes (INVALID_CREDENTIALS for wrong password)
6. [ ] Does NOT return password hash in response
7. [ ] Uses correct HTTP status codes (200 success, 401 wrong password)

## Scoring Criteria

### Pattern Adherence (0-10)
- Uses validateBody middleware with Zod schema
- Uses AppError for error handling
- Response format correct

### Completeness (0-10)
- Verifies current password
- Hashes new password
- Updates user record
- Returns appropriate response

### Correctness (0-10)
- Password verification logic correct
- Hash comparison works
- User lookup works

### Security (0-10)
- **CRITICAL**: Uses Argon2id (NOT bcrypt)
- Uses argon2.verify for comparison
- Does not expose password hash
- Validates new password strength

### Consistency (0-10)
- Matches existing auth routes
- Uses AuthService consistently
- Error handling matches patterns

## Red Flags (Automatic Deductions)

- Uses bcrypt: -8 security (critical anti-pattern)
- Skips current password verification: -10 security
- Returns password hash: -10 security
- Missing validation schema: -5 completeness
- Wrong error codes: -3 pattern adherence
