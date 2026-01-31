# API Endpoints

## Authentication

### POST /api/v1/auth/register
Create a new user account.

**Request**:
```json
{
  "email": "user@example.com",
  "password": "securepassword",
  "name": "John Doe"
}
```

**Response** (201):
```json
{
  "success": true,
  "data": {
    "user": { "id": "uuid", "email": "user@example.com", "name": "John Doe" },
    "token": "jwt-token"
  }
}
```

### POST /api/v1/auth/login
Authenticate and receive JWT token.

**Request**:
```json
{
  "email": "user@example.com",
  "password": "securepassword"
}
```

**Response** (200):
```json
{
  "success": true,
  "data": {
    "user": { "id": "uuid", "email": "user@example.com", "name": "John Doe" },
    "token": "jwt-token"
  }
}
```

## Tasks (Requires Authentication)

### GET /api/v1/tasks
List all tasks for authenticated user.

### POST /api/v1/tasks
Create a new task.

### GET /api/v1/tasks/:taskId
Get a specific task.

### PATCH /api/v1/tasks/:taskId
Update a task.

### DELETE /api/v1/tasks/:taskId
Delete a task.
