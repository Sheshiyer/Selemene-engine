import { Router, Request, Response } from 'express';
import { TaskService } from '../services/task-service';
import { authenticate } from '../middleware/authenticate';
import { validateBody, validateParams } from '../middleware/validation';
import { createTaskSchema, updateTaskSchema, taskIdSchema } from '../schemas/task';
import { AppError } from '../errors/app-error';

const router = Router();
const taskService = new TaskService();

// All routes require authentication
router.use(authenticate);

// GET /api/v1/tasks
router.get('/', async (req: Request, res: Response) => {
  const userId = req.user!.id;
  const tasks = await taskService.getTasksByUser(userId);

  res.status(200).json({
    success: true,
    data: { tasks }
  });
});

// POST /api/v1/tasks
router.post('/', validateBody(createTaskSchema), async (req: Request, res: Response) => {
  const userId = req.user!.id;
  const { title, description, dueDate } = req.body;

  const task = await taskService.createTask({
    userId,
    title,
    description,
    dueDate: dueDate ? new Date(dueDate) : undefined
  });

  res.status(201).json({
    success: true,
    data: { task }
  });
});

// GET /api/v1/tasks/:taskId
router.get('/:taskId', validateParams(taskIdSchema), async (req: Request, res: Response) => {
  const userId = req.user!.id;
  const { taskId } = req.params;

  const task = await taskService.getTaskById(taskId);

  if (!task) {
    throw new AppError('TASK_NOT_FOUND', 'Task not found', 404);
  }

  if (task.userId !== userId) {
    throw new AppError('FORBIDDEN', 'Access denied', 403);
  }

  res.status(200).json({
    success: true,
    data: { task }
  });
});

// PATCH /api/v1/tasks/:taskId
router.patch('/:taskId', validateParams(taskIdSchema), validateBody(updateTaskSchema), async (req: Request, res: Response) => {
  const userId = req.user!.id;
  const { taskId } = req.params;

  const task = await taskService.getTaskById(taskId);

  if (!task) {
    throw new AppError('TASK_NOT_FOUND', 'Task not found', 404);
  }

  if (task.userId !== userId) {
    throw new AppError('FORBIDDEN', 'Access denied', 403);
  }

  const updated = await taskService.updateTask(taskId, req.body);

  res.status(200).json({
    success: true,
    data: { task: updated }
  });
});

// DELETE /api/v1/tasks/:taskId
router.delete('/:taskId', validateParams(taskIdSchema), async (req: Request, res: Response) => {
  const userId = req.user!.id;
  const { taskId } = req.params;

  const task = await taskService.getTaskById(taskId);

  if (!task) {
    throw new AppError('TASK_NOT_FOUND', 'Task not found', 404);
  }

  if (task.userId !== userId) {
    throw new AppError('FORBIDDEN', 'Access denied', 403);
  }

  await taskService.deleteTask(taskId);

  res.status(200).json({
    success: true,
    data: null
  });
});

export { router as taskRouter };
