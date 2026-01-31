# TaskFlow API Documentation

This is the complete documentation for the TaskFlow API project.

## Project Overview

TaskFlow is a task management REST API built with Express.js, TypeScript, and PostgreSQL. It provides JWT-based authentication and CRUD operations for tasks.

## Architecture

### Layer Structure
- **Routes**: HTTP endpoint handlers in `src/routes/`
- **Services**: Business logic in `src/services/`
- **Repositories**: Data access in `src/repositories/`
- **Middleware**: Request processing in `src/middleware/`

### Patterns
- Repository pattern for data access
- Service layer for business logic
- Middleware for cross-cutting concerns
- Zod schemas for validation

## Authentication

### JWT Strategy
- Access tokens expire in 24 hours
- Tokens contain `sub` (user ID) and `email` claims
- Authorization header format: `Bearer <token>`

### Password Hashing
- Use Argon2id algorithm
- Configuration: memoryCost=65536, timeCost=3, parallelism=4
- NEVER use bcrypt or MD5

### Security Rules
- Always verify token before accessing protected resources
- Check resource ownership before allowing access
- Use constant-time comparison for sensitive data
- Never log passwords or tokens

## API Response Format

### Success Response
```json
{
  "success": true,
  "data": { ... }
}
```

### Error Response
```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable message",
    "details": { ... }
  }
}
```

## Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| INVALID_CREDENTIALS | 401 | Wrong email or password |
| TOKEN_INVALID | 401 | Missing or invalid JWT |
| TOKEN_EXPIRED | 401 | JWT has expired |
| FORBIDDEN | 403 | User lacks permission |
| USER_NOT_FOUND | 404 | User doesn't exist |
| TASK_NOT_FOUND | 404 | Task doesn't exist |
| EMAIL_EXISTS | 409 | Email already registered |
| VALIDATION_ERROR | 400 | Invalid request body |
| INTERNAL_ERROR | 500 | Unexpected server error |

## Validation

Use Zod for all input validation:

```typescript
import { z } from 'zod';

const schema = z.object({
  email: z.string().email(),
  password: z.string().min(8)
});
```

### Validation Middleware

Use the validation middleware for all endpoints:

```typescript
import { validateBody, validateParams, validateQuery } from '../middleware/validation';

// Body validation
router.post('/', validateBody(createSchema), handler);

// Params validation
router.get('/:id', validateParams(idSchema), handler);

// Query validation
router.get('/', validateQuery(filterSchema), handler);
```

### Common Validation Patterns

```typescript
// UUID validation
const idSchema = z.object({
  id: z.string().uuid()
});

// Enum validation
const statusSchema = z.enum(['pending', 'in_progress', 'completed']);

// Optional with default
const pageSchema = z.number().int().positive().default(1);
```

## Creating New Endpoints

1. Define Zod schema in `src/schemas/`
2. Create route handler in `src/routes/`
3. Use appropriate middleware (validateBody, authenticate)
4. Follow response format conventions
5. Use AppError for error handling

### Route Handler Template
```typescript
router.post('/path', validateBody(schema), async (req: Request, res: Response) => {
  // Business logic

  res.status(201).json({
    success: true,
    data: { ... }
  });
});
```

## Database Models

### User
- id: UUID (primary key)
- email: string (unique)
- name: string
- passwordHash: string
- createdAt: timestamp
- updatedAt: timestamp

### Task
- id: UUID (primary key)
- userId: UUID (foreign key)
- title: string
- description: string (optional)
- status: enum (pending, in_progress, completed)
- dueDate: timestamp (optional)
- createdAt: timestamp
- updatedAt: timestamp

## Anti-Patterns to Avoid

### Authentication
- DO NOT use bcrypt (use Argon2id)
- DO NOT store plain text passwords
- DO NOT return password hash in responses
- DO NOT use short JWT expiry without refresh tokens

### Error Handling
- DO NOT expose stack traces to clients
- DO NOT use generic 500 errors for known failure cases
- DO NOT catch errors without proper handling

### Validation
- DO NOT trust client input
- DO NOT skip validation on any endpoint
- DO NOT use `any` type for request bodies

### Code Style
- DO NOT mix async/await with .then()
- DO NOT use var (use const/let)
- DO NOT commit console.log statements
- DO NOT use magic numbers/strings

## Testing Requirements

- Unit tests for services
- Integration tests for routes
- Test both success and error cases
- Mock repositories for unit tests
