# API Response Formats

## Success Response

```json
{
  "success": true,
  "data": { ... }
}
```

## Error Response

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

## Error Codes Reference

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

## Using AppError

```typescript
import { AppError } from '../errors/app-error';

// Throw structured errors
throw new AppError('TASK_NOT_FOUND', 'Task not found', 404);

// With details
throw new AppError('VALIDATION_ERROR', 'Invalid input', 400, {
  errors: [{ field: 'email', message: 'Invalid format' }]
});
```
