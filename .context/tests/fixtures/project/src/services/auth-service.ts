import jwt from 'jsonwebtoken';
import argon2 from 'argon2';
import { UserRepository } from '../repositories/user-repository';
import { User, CreateUserInput } from '../types/user';

const JWT_SECRET = process.env.JWT_SECRET || 'dev-secret-change-in-production';
const JWT_EXPIRES_IN = '24h';

export class AuthService {
  private userRepository: UserRepository;

  constructor() {
    this.userRepository = new UserRepository();
  }

  async createUser(input: CreateUserInput): Promise<User> {
    const passwordHash = await argon2.hash(input.password, {
      type: argon2.argon2id,
      memoryCost: 65536,
      timeCost: 3,
      parallelism: 4
    });

    return this.userRepository.create({
      email: input.email,
      name: input.name,
      passwordHash
    });
  }

  async findUserByEmail(email: string): Promise<User | null> {
    return this.userRepository.findByEmail(email);
  }

  async verifyPassword(password: string, hash: string): Promise<boolean> {
    return argon2.verify(hash, password);
  }

  generateToken(user: User): string {
    return jwt.sign(
      { sub: user.id, email: user.email },
      JWT_SECRET,
      { expiresIn: JWT_EXPIRES_IN }
    );
  }

  verifyToken(token: string): { sub: string; email: string } | null {
    try {
      return jwt.verify(token, JWT_SECRET) as { sub: string; email: string };
    } catch {
      return null;
    }
  }
}
