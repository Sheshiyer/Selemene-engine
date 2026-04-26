# Input Validation

## Using Zod

All input validation uses Zod schemas:

```typescript
import { z } from 'zod';

export const createTaskSchema = z.object({
  title: z.string().min(1).max(200),
  description: z.string().max(2000).optional(),
  dueDate: z.string().datetime().optional()
});
```

## Validation Middleware

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

## Common Patterns

### UUID Validation
```typescript
const idSchema = z.object({
  id: z.string().uuid()
});
```

### Email Validation
```typescript
const emailSchema = z.string().email();
```

### Enum Validation
```typescript
const statusSchema = z.enum(['pending', 'in_progress', 'completed']);
```

### Optional with Default
```typescript
const pageSchema = z.number().int().positive().default(1);
```

## Error Response

Validation errors return 400 with details:

```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid request body",
    "details": {
      "errors": [
        { "path": ["email"], "message": "Invalid email format" }
      ]
    }
  }
}
```
