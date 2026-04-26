# Test 01: Control Group Output

**Prompt**: Add a new endpoint to get the currently authenticated user's profile.

## Generated Code

Based on the CLAUDE.md documentation, I'll create a new route file for users:

```typescript
// src/routes/users.ts
import { Router, Request, Response } from 'express';
import { authenticate } from '../middleware/authenticate';

const router = Router();

// All routes require authentication
router.use(authenticate);

// GET /api/v1/users/me
router.get('/me', async (req: Request, res: Response) => {
  const user = req.user!;

  res.status(200).json({
    success: true,
    data: {
      user: {
        id: user.id,
        email: user.email,
        name: user.name
      }
    }
  });
});

export { router as userRouter };
```

Then register in index.ts:
```typescript
import { userRouter } from './routes/users';
app.use('/api/v1/users', userRouter);
```

## Documentation Referenced
- "Creating New Endpoints" section
- "Route Handler Template"
- "API Response Format"
- "Database Models > User" (to know which fields to return)
- "Anti-Patterns" (don't return passwordHash)
