# Authentication API

## Overview

Selemene Engine uses JWT (JSON Web Tokens) for stateless authentication. API keys can be exchanged for short-lived JWTs.

## Authentication Methods

### 1. JWT Bearer Token (Recommended)

Include the JWT in the Authorization header:

```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### 2. API Key

For server-to-server communication, use the X-API-Key header:

```
X-API-Key: sk_live_your_api_key_here
```

---

## Obtaining a Token

### Endpoint
```
POST /api/v1/auth/token
```

### Request
```json
{
  "api_key": "sk_live_your_api_key_here"
}
```

### Response
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "expires_at": "2025-01-15T13:00:00Z"
}
```

### cURL Example
```bash
curl -X POST http://localhost:8080/api/v1/auth/token \
  -H "Content-Type: application/json" \
  -d '{"api_key": "sk_live_your_api_key_here"}'
```

---

## JWT Claims

Tokens include the following claims:

| Claim | Description |
|-------|-------------|
| `sub` | User ID |
| `iat` | Issued at (Unix timestamp) |
| `exp` | Expiration (Unix timestamp) |
| `tier` | Access tier (free, premium, enterprise) |
| `phase` | Consciousness phase level (0-5) |
| `scope` | Permitted scopes |

### Example Decoded Token

```json
{
  "sub": "user_123456",
  "iat": 1705323600,
  "exp": 1705327200,
  "tier": "premium",
  "phase": 3,
  "scope": ["engines:read", "workflows:execute"]
}
```

---

## Refreshing Tokens

### Endpoint
```
POST /api/v1/auth/refresh
```

### Request
```json
{
  "refresh_token": "rt_your_refresh_token_here"
}
```

### Response
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "rt_new_refresh_token"
}
```

---

## Validating Tokens

### Endpoint
```
GET /api/v1/auth/validate
```

### Request
Include token in Authorization header.

### Response
```json
{
  "valid": true,
  "user_id": "user_123456",
  "tier": "premium",
  "phase": 3,
  "expires_at": "2025-01-15T13:00:00Z"
}
```

---

## Phase-Based Access Control

Engines require minimum consciousness phase levels:

| Phase | Access Level | Engines Available |
|-------|--------------|-------------------|
| 0 | Public | panchanga, numerology, biorhythm, vedic-clock |
| 1 | Basic | + human-design, gene-keys, vimshottari, tarot, i-ching, enneagram |
| 2 | Intermediate | + biofield, face-reading, sacred-geometry, sigil-forge |
| 3 | Advanced | Full engine access + advanced features |
| 4 | Expert | + experimental features |
| 5 | Master | Full system access |

### Phase Access Denied Response

```json
{
  "success": false,
  "error": {
    "code": "PHASE_ACCESS_DENIED",
    "message": "Access denied: requires phase 2, current phase 1",
    "required_phase": 2,
    "current_phase": 1
  }
}
```

---

## API Key Management

### Create API Key (Admin)
```
POST /api/v1/admin/api-keys
```

```json
{
  "name": "My Application",
  "tier": "premium",
  "phase": 3,
  "expires_at": "2026-01-15T00:00:00Z"
}
```

### List API Keys (Admin)
```
GET /api/v1/admin/api-keys
```

### Revoke API Key (Admin)
```
DELETE /api/v1/admin/api-keys/{key_id}
```

---

## Security Best Practices

### Token Storage
- Store tokens securely (not in localStorage for web apps)
- Use HttpOnly cookies when possible
- Never log or expose tokens

### Token Rotation
- Access tokens expire in 1 hour
- Use refresh tokens for long-lived sessions
- Rotate API keys periodically

### Transport Security
- Always use HTTPS in production
- Validate SSL certificates
- Use TLS 1.2+

### Rate Limiting
- Respect rate limit headers
- Implement exponential backoff
- Cache responses when appropriate

---

## Error Responses

### 401 Unauthorized
Missing or invalid authentication:

```json
{
  "success": false,
  "error": {
    "code": "AUTHENTICATION_ERROR",
    "message": "Invalid or expired token"
  }
}
```

### 403 Forbidden
Valid authentication but insufficient permissions:

```json
{
  "success": false,
  "error": {
    "code": "AUTHORIZATION_ERROR",
    "message": "Insufficient permissions for this resource"
  }
}
```

### 429 Too Many Requests
Rate limit exceeded:

```json
{
  "success": false,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded. Try again in 45 seconds.",
    "retry_after": 45
  }
}
```

---

## Example: Full Authentication Flow

### 1. Exchange API Key for Token
```bash
TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/auth/token \
  -H "Content-Type: application/json" \
  -d '{"api_key": "sk_live_xxx"}' | jq -r '.access_token')
```

### 2. Use Token for API Calls
```bash
curl -X POST http://localhost:8080/api/v1/engines/human-design/calculate \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"birth_data": {...}}'
```

### 3. Refresh When Needed
```bash
NEW_TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{"refresh_token": "rt_xxx"}' | jq -r '.access_token')
```

---

**Last Updated**: 2026-01
