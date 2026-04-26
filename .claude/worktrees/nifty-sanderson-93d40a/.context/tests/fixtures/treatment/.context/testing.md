# Testing Requirements

## Test Structure

- Unit tests for services
- Integration tests for routes
- Test both success and error cases
- Mock repositories for unit tests

## Service Unit Test Pattern

```typescript
describe('TaskService', () => {
  let taskService: TaskService;
  let mockRepository: jest.Mocked<TaskRepository>;

  beforeEach(() => {
    mockRepository = {
      findById: jest.fn(),
      create: jest.fn(),
      // ...
    };
    taskService = new TaskService(mockRepository);
  });

  it('should create a task', async () => {
    mockRepository.create.mockResolvedValue(mockTask);

    const result = await taskService.createTask(input);

    expect(result).toEqual(mockTask);
    expect(mockRepository.create).toHaveBeenCalledWith(
      expect.objectContaining({ title: input.title })
    );
  });
});
```

## Route Integration Test Pattern

```typescript
describe('POST /api/v1/tasks', () => {
  it('should create task when authenticated', async () => {
    const response = await request(app)
      .post('/api/v1/tasks')
      .set('Authorization', `Bearer ${validToken}`)
      .send({ title: 'Test Task' });

    expect(response.status).toBe(201);
    expect(response.body.success).toBe(true);
    expect(response.body.data.task.title).toBe('Test Task');
  });

  it('should return 401 without auth', async () => {
    const response = await request(app)
      .post('/api/v1/tasks')
      .send({ title: 'Test Task' });

    expect(response.status).toBe(401);
  });
});
```

## Test Coverage Requirements

- Services: 80% minimum
- Routes: 100% of endpoints
- Error cases: All error codes tested
