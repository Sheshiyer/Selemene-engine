# Authentication Overview

## JWT Strategy

- Access tokens expire in 24 hours
- Tokens contain `sub` (user ID) and `email` claims
- Authorization header format: `Bearer <token>`

## Token Structure

```typescript
interface JwtPayload {
  sub: string;   // User ID
  email: string; // User email
  iat: number;   // Issued at
  exp: number;   // Expiry
}
```

## Auth Flow

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│  Client  │────▶│  /login  │────▶│   JWT    │
└──────────┘     └──────────┘     └────┬─────┘
                                       │
┌──────────────────────────────────────┘
│
▼
┌──────────────────────────────────────┐
│  Authorization: Bearer <token>        │
│  All protected endpoints              │
└──────────────────────────────────────┘
```

## Using Authentication Middleware

```typescript
import { authenticate } from '../middleware/authenticate';

router.use(authenticate); // All routes below require auth

router.get('/', async (req, res) => {
  const userId = req.user!.id; // User is guaranteed
});
```
