# Code Patterns

## Repository Pattern

All data access goes through repository classes:

```typescript
export class TaskRepository {
  async findById(id: string): Promise<Task | null> {
    return tasks.get(id) || null;
  }

  async create(data: Omit<Task, 'id'>): Promise<Task> {
    const task = { id: crypto.randomUUID(), ...data };
    tasks.set(task.id, task);
    return task;
  }
}
```

## Service Layer

Business logic lives in service classes:

```typescript
export class TaskService {
  private taskRepository: TaskRepository;

  async createTask(input: CreateTaskInput): Promise<Task> {
    return this.taskRepository.create({
      ...input,
      status: 'pending',
      createdAt: new Date()
    });
  }
}
```

## Route Handler Pattern

```typescript
router.post('/path', validateBody(schema), async (req: Request, res: Response) => {
  const result = await service.doSomething(req.body);

  res.status(201).json({
    success: true,
    data: { result }
  });
});
```

## Error Handling Pattern

Always use AppError for known error cases:

```typescript
if (!task) {
  throw new AppError('TASK_NOT_FOUND', 'Task not found', 404);
}
```
