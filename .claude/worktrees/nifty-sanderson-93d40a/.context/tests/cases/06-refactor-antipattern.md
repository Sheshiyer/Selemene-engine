# Test Case 06: Refactor Anti-Pattern

## Metadata
- **ID**: 06-refactor-antipattern
- **Category**: refactoring
- **Complexity**: medium
- **Domains Tested**: patterns, anti-patterns

## Prompt

```
Refactor this code to follow project standards:

```typescript
router.post('/tasks', async (req, res) => {
  var title = req.body.title;
  var desc = req.body.description;

  if (!title) {
    res.status(400).send({ error: 'Title required' });
    return;
  }

  const task = await taskRepository.create({
    userId: req.user.id,
    title: title,
    description: desc,
    status: 'pending',
    createdAt: new Date()
  }).then(t => t);

  console.log('Created task:', task);

  res.json(task);
});
```

Fix all issues according to project patterns.
```

## Expected Behaviors

1. [ ] Changes var to const/let
2. [ ] Uses validateBody middleware with Zod schema
3. [ ] Removes console.log
4. [ ] Changes response format to `{ success: true, data: {...} }`
5. [ ] Removes .then() (uses pure async/await)
6. [ ] Uses TaskService instead of direct repository access
7. [ ] Uses destructuring for req.body
8. [ ] Returns 201 for creation

## Scoring Criteria

### Pattern Adherence (0-10)
- Uses validateBody middleware
- Response format correct
- Service layer pattern used
- Error handling uses AppError

### Completeness (0-10)
- All anti-patterns fixed
- Functionality preserved
- Proper status codes

### Correctness (0-10)
- Code works correctly
- No type errors
- Logic unchanged

### Security (0-10)
- Input validation added
- Uses existing auth middleware
- No data exposure

### Consistency (0-10)
- Matches project style exactly
- No var usage
- No console.log
- Pure async/await

## Red Flags (Automatic Deductions)

- Keeps var: -2 consistency
- Keeps console.log: -2 consistency
- Keeps .then(): -2 consistency
- Wrong response format: -3 pattern adherence
- Direct repository access: -3 pattern adherence
