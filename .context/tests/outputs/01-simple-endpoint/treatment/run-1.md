# Test 01: Treatment Group Output

**Prompt**: Add a new endpoint to get the currently authenticated user's profile.

## Generated Code

Based on the .context/ documentation, I'll create a users route following the documented patterns:

```typescript
// src/routes/users.ts
import { Router, Request, Response } from 'express';
import { authenticate } from '../middleware/authenticate';

const router = Router();

// All routes require authentication (from auth/overview.md pattern)
router.use(authenticate);

// GET /api/v1/users/me
router.get('/me', async (req: Request, res: Response) => {
  // User is guaranteed by authenticate middleware (auth/overview.md)
  const user = req.user!;

  // Return user without passwordHash (anti-patterns.md: DO NOT return password hash)
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
- `auth/overview.md` - "Using Authentication Middleware" section with exact pattern
- `architecture/patterns.md` - "Route Handler Pattern" for response structure
- `api/responses.md` - Success response format `{ success: true, data: {...} }`
- `database/models.md` - User interface showing available fields
- `anti-patterns.md` - Explicit example showing GOOD pattern: `{ id: user.id, email: user.email, name: user.name }`
