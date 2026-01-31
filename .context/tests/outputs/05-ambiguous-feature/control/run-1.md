# Test 05: Control Group Output

**Prompt**: Add task filtering.

## Generated Code

Based on CLAUDE.md documentation:

```typescript
// src/routes/tasks.ts - modify GET /api/v1/tasks

// GET /api/v1/tasks
router.get('/', async (req: Request, res: Response) => {
  const userId = req.user!.id;
  const { status } = req.query;

  let tasks = await taskService.getTasksByUser(userId);

  // Filter by status if provided
  if (status) {
    tasks = tasks.filter(task => task.status === status);
  }

  res.status(200).json({
    success: true,
    data: { tasks }
  });
});
```

## Documentation Referenced
- "Database Models > Task" section: has status field (pending, in_progress, completed)
- "API Response Format" section: response format
- Existing route patterns from template

## Notes
- Documentation mentions validateBody but unclear about query parameter validation
- Used basic filtering approach
- Task model has status enum which is logical to filter by
