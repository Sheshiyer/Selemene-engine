import express from 'express';
import { authRouter } from './routes/auth';
import { taskRouter } from './routes/tasks';
import { errorHandler } from './middleware/error-handler';
import { requestLogger } from './middleware/request-logger';

const app = express();

app.use(express.json());
app.use(requestLogger);

app.use('/api/v1/auth', authRouter);
app.use('/api/v1/tasks', taskRouter);

app.use(errorHandler);

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});

export { app };
