# Control Trial 3

## Key Implementation Decisions:
- DID NOT use validateQuery middleware
- Used inline `safeParse` validation instead
- Added dueDate filter (more complete)

## Code (route):
```typescript
const filterSchema = z.object({
  status: z.enum(['pending', 'in_progress', 'completed']).optional(),
  dueDate: z.string().datetime().optional(),
});

router.get('/', async (req: Request, res: Response) => {
  const userId = req.user!.id;
  const filters = {
    status: req.query.status as string | undefined,
    dueDate: req.query.dueDate as string | undefined,
  };

  const validation = filterSchema.safeParse(filters);
  if (!validation.success) {
    return res.status(400).json({
      success: false,
      error: { code: 'VALIDATION_ERROR', message: validation.error.message }
    });
  }

  const tasks = await taskService.getTasksByUser(userId, validation.data);
  res.status(200).json({ success: true, data: { tasks } });
});
```

## Observations:
- DID NOT use validateQuery middleware (inconsistent with existing patterns)
- Used inline validation approach
- Added dueDate filter
- Manual error response (not using AppError)
