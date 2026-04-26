import { TaskRepository } from '../repositories/task-repository';
import { Task, CreateTaskInput, UpdateTaskInput } from '../types/task';

export class TaskService {
  private taskRepository: TaskRepository;

  constructor() {
    this.taskRepository = new TaskRepository();
  }

  async createTask(input: CreateTaskInput): Promise<Task> {
    return this.taskRepository.create({
      ...input,
      status: 'pending',
      createdAt: new Date(),
      updatedAt: new Date()
    });
  }

  async getTaskById(id: string): Promise<Task | null> {
    return this.taskRepository.findById(id);
  }

  async getTasksByUser(userId: string): Promise<Task[]> {
    return this.taskRepository.findByUserId(userId);
  }

  async updateTask(id: string, input: UpdateTaskInput): Promise<Task> {
    return this.taskRepository.update(id, {
      ...input,
      updatedAt: new Date()
    });
  }

  async deleteTask(id: string): Promise<void> {
    return this.taskRepository.delete(id);
  }
}
