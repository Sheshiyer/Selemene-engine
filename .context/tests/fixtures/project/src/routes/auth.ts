import { Router, Request, Response } from 'express';
import { AuthService } from '../services/auth-service';
import { validateBody } from '../middleware/validation';
import { loginSchema, registerSchema } from '../schemas/auth';
import { AppError } from '../errors/app-error';

const router = Router();
const authService = new AuthService();

// POST /api/v1/auth/register
router.post('/register', validateBody(registerSchema), async (req: Request, res: Response) => {
  const { email, password, name } = req.body;

  const existingUser = await authService.findUserByEmail(email);
  if (existingUser) {
    throw new AppError('EMAIL_EXISTS', 'Email already registered', 409);
  }

  const user = await authService.createUser({ email, password, name });
  const token = authService.generateToken(user);

  res.status(201).json({
    success: true,
    data: {
      user: { id: user.id, email: user.email, name: user.name },
      token
    }
  });
});

// POST /api/v1/auth/login
router.post('/login', validateBody(loginSchema), async (req: Request, res: Response) => {
  const { email, password } = req.body;

  const user = await authService.findUserByEmail(email);
  if (!user) {
    throw new AppError('INVALID_CREDENTIALS', 'Invalid email or password', 401);
  }

  const isValid = await authService.verifyPassword(password, user.passwordHash);
  if (!isValid) {
    throw new AppError('INVALID_CREDENTIALS', 'Invalid email or password', 401);
  }

  const token = authService.generateToken(user);

  res.status(200).json({
    success: true,
    data: {
      user: { id: user.id, email: user.email, name: user.name },
      token
    }
  });
});

export { router as authRouter };
