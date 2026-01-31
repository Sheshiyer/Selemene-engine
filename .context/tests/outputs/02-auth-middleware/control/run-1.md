# Test 02: Control Group Output

**Prompt**: Add a password change endpoint for authenticated users.

## Generated Code

Based on CLAUDE.md documentation:

```typescript
// src/schemas/auth.ts - add new schema
import { z } from 'zod';

export const changePasswordSchema = z.object({
  currentPassword: z.string().min(1, 'Current password is required'),
  newPassword: z.string().min(8, 'Password must be at least 8 characters')
});
```

```typescript
// src/routes/auth.ts - add new endpoint
import { changePasswordSchema } from '../schemas/auth';
import argon2 from 'argon2';

// PATCH /api/v1/auth/password
router.patch('/password', authenticate, validateBody(changePasswordSchema), async (req: Request, res: Response) => {
  const { currentPassword, newPassword } = req.body;
  const userId = req.user!.id;

  // Get user with password hash
  const user = await authService.findUserById(userId);
  if (!user) {
    throw new AppError('USER_NOT_FOUND', 'User not found', 404);
  }

  // Verify current password
  const isValid = await argon2.verify(user.passwordHash, currentPassword);
  if (!isValid) {
    throw new AppError('INVALID_CREDENTIALS', 'Invalid current password', 401);
  }

  // Hash new password with Argon2id (as per docs: memoryCost=65536, timeCost=3, parallelism=4)
  const newHash = await argon2.hash(newPassword, {
    type: argon2.argon2id,
    memoryCost: 65536,
    timeCost: 3,
    parallelism: 4
  });

  // Update user
  await authService.updatePassword(userId, newHash);

  res.status(200).json({
    success: true,
    data: { message: 'Password updated successfully' }
  });
});
```

## Documentation Referenced
- "Password Hashing" section: Argon2id with memoryCost=65536, timeCost=3, parallelism=4
- "Validation" section: Use Zod
- "Error Codes" section: INVALID_CREDENTIALS for wrong password
- "Anti-Patterns": NEVER use bcrypt
