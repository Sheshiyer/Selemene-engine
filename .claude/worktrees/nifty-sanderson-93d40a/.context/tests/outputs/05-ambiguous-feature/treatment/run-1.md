# Test 05: Treatment Group Output

**Prompt**: Add task filtering.

## Generated Code

Based on .context/ documentation:

```typescript
// src/schemas/task.ts - add filter schema
import { z } from 'zod';

// Enum validation pattern from validation.md
export const taskFilterSchema = z.object({
  status: z.enum(['pending', 'in_progress', 'completed']).optional(),
  dueBefore: z.string().datetime().optional(),
  dueAfter: z.string().datetime().optional()
});

export type TaskFilterInput = z.infer<typeof taskFilterSchema>;
```

```typescript
// src/routes/tasks.ts - update GET endpoint
import { validateQuery } from '../middleware/validation';
import { taskFilterSchema } from '../schemas/task';

// GET /api/v1/tasks (with optional filters)
router.get('/', validateQuery(taskFilterSchema), async (req: Request, res: Response) => {
  const userId = req.user!.id;
  const { status, dueBefore, dueAfter } = req.query as TaskFilterInput;

  // Get user's tasks (security: only own tasks)
  let tasks = await taskService.getTasksByUser(userId);

  // Apply filters
  if (status) {
    tasks = tasks.filter(task => task.status === status);
  }

  if (dueBefore) {
    const beforeDate = new Date(dueBefore);
    tasks = tasks.filter(task => task.dueDate && task.dueDate < beforeDate);
  }

  if (dueAfter) {
    const afterDate = new Date(dueAfter);
    tasks = tasks.filter(task => task.dueDate && task.dueDate > afterDate);
  }

  res.status(200).json({
    success: true,
    data: { tasks }
  });
});
```

## Documentation Referenced
- `validation.md` - "Query validation" pattern: `router.get('/', validateQuery(filterSchema), handler)`
- `validation.md` - "Enum Validation" pattern: `z.enum(['pending', 'in_progress', 'completed'])`
- `database/models.md` - Task model fields: status, dueDate
- `api/responses.md` - Success response format
- `auth/security.md` - Resource ownership: only filter user's own tasks

## Inferred Requirements from Documentation
1. Status filtering (Task has status enum with 3 values per database/models.md)
2. Date filtering (Task has optional dueDate per database/models.md)
3. Query validation (validation.md shows validateQuery middleware)
4. Security (auth/security.md: always filter by user ownership)
