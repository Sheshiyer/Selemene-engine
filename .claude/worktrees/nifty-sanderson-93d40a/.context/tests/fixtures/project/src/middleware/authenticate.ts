import { Request, Response, NextFunction } from 'express';
import { AuthService } from '../services/auth-service';
import { AppError } from '../errors/app-error';
import { UserRepository } from '../repositories/user-repository';

const authService = new AuthService();
const userRepository = new UserRepository();

export async function authenticate(
  req: Request,
  res: Response,
  next: NextFunction
): Promise<void> {
  const authHeader = req.headers.authorization;

  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    throw new AppError('TOKEN_INVALID', 'Missing or invalid authorization header', 401);
  }

  const token = authHeader.slice(7);
  const payload = authService.verifyToken(token);

  if (!payload) {
    throw new AppError('TOKEN_INVALID', 'Invalid or expired token', 401);
  }

  const user = await userRepository.findById(payload.sub);

  if (!user) {
    throw new AppError('USER_NOT_FOUND', 'User not found', 401);
  }

  req.user = user;
  next();
}

// Extend Express Request type
declare global {
  namespace Express {
    interface Request {
      user?: {
        id: string;
        email: string;
        name: string;
      };
    }
  }
}
