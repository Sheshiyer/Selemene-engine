import { User } from '../types/user';

// In-memory store for testing (would be PostgreSQL in production)
const users: Map<string, User> = new Map();

export class UserRepository {
  async create(data: Omit<User, 'id' | 'createdAt' | 'updatedAt'>): Promise<User> {
    const user: User = {
      id: crypto.randomUUID(),
      ...data,
      createdAt: new Date(),
      updatedAt: new Date()
    };
    users.set(user.id, user);
    return user;
  }

  async findById(id: string): Promise<User | null> {
    return users.get(id) || null;
  }

  async findByEmail(email: string): Promise<User | null> {
    for (const user of users.values()) {
      if (user.email === email) {
        return user;
      }
    }
    return null;
  }

  async update(id: string, data: Partial<User>): Promise<User> {
    const user = users.get(id);
    if (!user) {
      throw new Error('User not found');
    }
    const updated = { ...user, ...data, updatedAt: new Date() };
    users.set(id, updated);
    return updated;
  }

  async delete(id: string): Promise<void> {
    users.delete(id);
  }
}
