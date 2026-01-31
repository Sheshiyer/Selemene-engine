# Control Trial 2

## Key Implementation Decisions:
- Used `validateQuery(taskFilterSchema)` middleware
- Created Zod schema with enum validation
- Status filter only

## Code (route):
```typescript
router.get('/', validateQuery(taskFilterSchema), async (req: Request, res: Response) => {
  const userId = req.user!.id;
  const filter = req.query;
  const tasks = await taskService.getTasksByUser(userId, filter);
  res.status(200).json({ success: true, data: { tasks } });
});
```

## Observations:
- DID use validateQuery
- DID create proper Zod schema
- Very similar to Trial 1
