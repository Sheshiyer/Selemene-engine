# Database Models

## User

| Column | Type | Constraints |
|--------|------|-------------|
| id | UUID | PRIMARY KEY |
| email | VARCHAR(255) | UNIQUE, NOT NULL |
| name | VARCHAR(100) | NOT NULL |
| passwordHash | VARCHAR(255) | NOT NULL |
| createdAt | TIMESTAMP | NOT NULL |
| updatedAt | TIMESTAMP | NOT NULL |

```typescript
interface User {
  id: string;
  email: string;
  name: string;
  passwordHash: string;
  createdAt: Date;
  updatedAt: Date;
}
```

## Task

| Column | Type | Constraints |
|--------|------|-------------|
| id | UUID | PRIMARY KEY |
| userId | UUID | FOREIGN KEY (users.id) |
| title | VARCHAR(200) | NOT NULL |
| description | TEXT | NULLABLE |
| status | ENUM | DEFAULT 'pending' |
| dueDate | TIMESTAMP | NULLABLE |
| createdAt | TIMESTAMP | NOT NULL |
| updatedAt | TIMESTAMP | NOT NULL |

```typescript
type TaskStatus = 'pending' | 'in_progress' | 'completed';

interface Task {
  id: string;
  userId: string;
  title: string;
  description?: string;
  status: TaskStatus;
  dueDate?: Date;
  createdAt: Date;
  updatedAt: Date;
}
```

## Relationships

```
┌──────────┐       ┌──────────┐
│   User   │──────<│   Task   │
└──────────┘       └──────────┘
     1                  *
```
