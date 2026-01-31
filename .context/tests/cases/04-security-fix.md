# Test Case 04: Security Fix

## Metadata
- **ID**: 04-security-fix
- **Category**: security-critical
- **Complexity**: high
- **Domains Tested**: security, auth, patterns

## Prompt

```
Review and fix the following code for security issues:

```typescript
router.post('/admin/users/:userId/reset-password', async (req, res) => {
  const { userId } = req.params;
  const newPassword = 'temporary123';

  const user = await userRepository.findById(userId);
  const hash = await bcrypt.hash(newPassword, 10);

  await userRepository.update(userId, { passwordHash: hash });

  res.json({
    success: true,
    data: {
      user,
      temporaryPassword: newPassword
    }
  });
});
```

Fix all security issues while maintaining the functionality.
```

## Expected Behaviors

1. [ ] Adds authentication middleware
2. [ ] Adds authorization check (admin role)
3. [ ] Changes bcrypt to Argon2id with correct config
4. [ ] Removes passwordHash from response
5. [ ] Does not return temporary password in response (should email instead)
6. [ ] Adds input validation for userId
7. [ ] Uses AppError for error handling
8. [ ] Generates secure random password

## Scoring Criteria

### Pattern Adherence (0-10)
- Uses project patterns for auth/validation
- Error handling uses AppError
- Response format correct

### Completeness (0-10)
- All security issues identified
- All issues fixed
- Functionality preserved

### Correctness (0-10)
- Fixes are valid TypeScript
- Logic works correctly
- No new bugs introduced

### Security (0-10)
- **CRITICAL**: Uses Argon2id
- **CRITICAL**: No password in response
- Authentication added
- Authorization added
- Input validated

### Consistency (0-10)
- Matches project style
- Uses existing middleware
- Follows naming conventions

## Red Flags (Automatic Deductions)

- Keeps bcrypt: -8 security
- Returns password: -10 security
- No authentication: -10 security
- Returns passwordHash: -10 security
