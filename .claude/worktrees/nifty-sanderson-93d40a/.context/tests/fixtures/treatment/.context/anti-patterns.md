# Anti-Patterns to Avoid

## Authentication

### DO NOT use bcrypt

```typescript
// BAD - uses bcrypt
import bcrypt from 'bcrypt';
const hash = await bcrypt.hash(password, 10);

// GOOD - uses Argon2id
import argon2 from 'argon2';
const hash = await argon2.hash(password, {
  type: argon2.argon2id,
  memoryCost: 65536,
  timeCost: 3,
  parallelism: 4
});
```

### DO NOT store plain text passwords

### DO NOT return password hash in responses

```typescript
// BAD
res.json({ user });

// GOOD
res.json({
  user: { id: user.id, email: user.email, name: user.name }
});
```

## Error Handling

### DO NOT expose stack traces

```typescript
// BAD
res.status(500).json({ error: err.stack });

// GOOD
res.status(500).json({
  success: false,
  error: { code: 'INTERNAL_ERROR', message: 'An error occurred' }
});
```

### DO NOT use generic errors for known cases

```typescript
// BAD
if (!user) throw new Error('Not found');

// GOOD
if (!user) throw new AppError('USER_NOT_FOUND', 'User not found', 404);
```

## Validation

### DO NOT trust client input

```typescript
// BAD
const { id } = req.params;
const task = await taskService.getById(id);

// GOOD
router.get('/:id', validateParams(idSchema), async (req, res) => {
  const { id } = req.params; // Validated
});
```

## Code Style

### DO NOT mix async patterns

```typescript
// BAD
async function getData() {
  return fetch(url).then(r => r.json());
}

// GOOD
async function getData() {
  const response = await fetch(url);
  return response.json();
}
```

### DO NOT use var

```typescript
// BAD
var count = 0;

// GOOD
const count = 0;
let mutableCount = 0;
```

### DO NOT use magic strings

```typescript
// BAD
if (status === 'pending') { ... }

// GOOD
const TaskStatus = { PENDING: 'pending' } as const;
if (status === TaskStatus.PENDING) { ... }
```
