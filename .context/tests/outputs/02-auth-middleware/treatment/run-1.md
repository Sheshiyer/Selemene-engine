# Test 02: Treatment Group Output

**Prompt**: Add a password change endpoint for authenticated users.

## Generated Code

Based on .context/ documentation:

```typescript
// src/schemas/auth.ts - add new schema
import { z } from 'zod';

export const changePasswordSchema = z.object({
  currentPassword: z.string().min(1, 'Current password is required'),
  newPassword: z.string().min(8, 'Password must be at least 8 characters')
});

export type ChangePasswordInput = z.infer<typeof changePasswordSchema>;
```

```typescript
// src/routes/auth.ts - add new endpoint
import { authenticate } from '../middleware/authenticate';
import { validateBody } from '../middleware/validation';
import { changePasswordSchema } from '../schemas/auth';
import { AppError } from '../errors/app-error';
import argon2 from 'argon2';

// PATCH /api/v1/auth/password
router.patch(
  '/password',
  authenticate,
  validateBody(changePasswordSchema),
  async (req: Request, res: Response) => {
    const { currentPassword, newPassword } = req.body;
    const userId = req.user!.id;

    // Get user (from auth/overview.md pattern)
    const user = await authService.findUserById(userId);
    if (!user) {
      throw new AppError('USER_NOT_FOUND', 'User not found', 404);
    }

    // Verify current password using argon2.verify (auth/security.md)
    const isValid = await argon2.verify(user.passwordHash, currentPassword);
    if (!isValid) {
      throw new AppError('INVALID_CREDENTIALS', 'Current password is incorrect', 401);
    }

    // Hash new password with Argon2id (auth/security.md - exact pattern)
    const newHash = await argon2.hash(newPassword, {
      type: argon2.argon2id,
      memoryCost: 65536,  // 64 MB
      timeCost: 3,        // 3 iterations
      parallelism: 4      // 4 threads
    });

    // Update password
    await authService.updatePassword(userId, newHash);

    // Response format from api/responses.md
    res.status(200).json({
      success: true,
      data: { message: 'Password changed successfully' }
    });
  }
);
```

## Documentation Referenced
- `auth/security.md` - Exact Argon2id code pattern with import and config
- `auth/security.md` - Security checklist: "Validate all inputs with Zod", "Use Argon2id for passwords"
- `validation.md` - validateBody middleware pattern
- `api/responses.md` - Success response format
- `anti-patterns.md` - Shows BAD (bcrypt) vs GOOD (Argon2id) with full code

## Security Checklist Applied
- [x] Validate all inputs with Zod
- [x] Use Argon2id for passwords
- [x] Verify JWT on protected routes (authenticate middleware)
- [x] Never log sensitive data
