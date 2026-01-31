import { Task } from '../types/task';

// In-memory store for testing (would be PostgreSQL in production)
const tasks: Map<string, Task> = new Map();

export class TaskRepository {
  async create(data: Omit<Task, 'id'>): Promise<Task> {
    const task: Task = {
      id: crypto.randomUUID(),
      ...data
    };
    tasks.set(task.id, task);
    return task;
  }

  async findById(id: string): Promise<Task | null> {
    return tasks.get(id) || null;
  }

  async findByUserId(userId: string): Promise<Task[]> {
    const result: Task[] = [];
    for (const task of tasks.values()) {
      if (task.userId === userId) {
        result.push(task);
      }
    }
    return result;
  }

  async update(id: string, data: Partial<Task>): Promise<Task> {
    const task = tasks.get(id);
    if (!task) {
      throw new Error('Task not found');
    }
    const updated = { ...task, ...data };
    tasks.set(id, updated);
    return updated;
  }

  async delete(id: string): Promise<void> {
    tasks.delete(id);
  }
}
