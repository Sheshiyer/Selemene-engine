# Authentication

## Overview

Noesis supports two authentication methods:

1) **API Key** (recommended for server-to-server and agents)
2) **JWT Bearer Token** (issued on login)

Each API key is treated as a unique user identity.

## API Key

Send your key in the `X-API-Key` header:

```http
X-API-Key: nk_<api_key>
```

## JWT Bearer Token

Send your token in the `Authorization` header:

```http
Authorization: Bearer <jwt_token>
```

## Register

```
POST /api/v1/auth/register
```

```json
{
  "email": "user@example.com",
  "password": "StrongPass1",
  "full_name": "First Last"
}
```

## Login

```
POST /api/v1/auth/login
```

```json
{
  "email": "user@example.com",
  "password": "StrongPass1"
}
```

**Response:**

```json
{
  "token": "<jwt>",
  "user_id": "<uuid>",
  "email": "user@example.com",
  "tier": "premium"
}
```

## Forgot Password

```
POST /api/v1/auth/forgot-password
```

```json
{
  "email": "user@example.com"
}
```

## Reset Password

```
POST /api/v1/auth/reset-password
```

```json
{
  "token": "<reset_token>",
  "new_password": "NewStrongPass1"
}
```

## Change Password (Authenticated)

```
POST /api/v1/auth/change-password
```

```json
{
  "current_password": "StrongPass1",
  "new_password": "NewStrongPass1"
}
```

## Error Format

```json
{
  "error": "Invalid or expired API key",
  "error_code": "UNAUTHORIZED",
  "details": {"auth_method": "api_key"}
}
```