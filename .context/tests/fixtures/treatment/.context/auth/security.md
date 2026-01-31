# Security Guidelines

## Password Hashing

**REQUIRED**: Use Argon2id algorithm with these settings:

```typescript
import argon2 from 'argon2';

const hash = await argon2.hash(password, {
  type: argon2.argon2id,
  memoryCost: 65536,  // 64 MB
  timeCost: 3,        // 3 iterations
  parallelism: 4      // 4 threads
});
```

**NEVER use**:
- bcrypt
- MD5
- SHA-1/SHA-256 without salt
- Plain text storage

## Token Verification

Always verify before accessing protected resources:

```typescript
const payload = authService.verifyToken(token);
if (!payload) {
  throw new AppError('TOKEN_INVALID', 'Invalid token', 401);
}
```

## Resource Ownership

Always check ownership before allowing access:

```typescript
const task = await taskService.getTaskById(taskId);

if (task.userId !== req.user!.id) {
  throw new AppError('FORBIDDEN', 'Access denied', 403);
}
```

## Security Checklist

- [ ] Validate all inputs with Zod
- [ ] Use Argon2id for passwords
- [ ] Verify JWT on protected routes
- [ ] Check resource ownership
- [ ] Never log sensitive data
- [ ] Use constant-time comparison
