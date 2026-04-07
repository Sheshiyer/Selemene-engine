# Authentication & Integration Guide

> For frontend applications integrating with the Noesis Engine API.

## Base URL

```
https://selemene.tryambakam.space
```

## Overview

Noesis supports two authentication methods:

| Method | Header | Best For |
|--------|--------|----------|
| **JWT Token** | `Authorization: Bearer <token>` | Frontend apps, user sessions |
| **API Key** | `X-API-Key: nk_...` | Server-to-server, scripts, CLI |

Both methods give access to all engine and workflow endpoints. Frontends should use the **JWT flow** — a personal API key is auto-generated on first login and can be retrieved later via the `/users/me/api-keys` endpoint.

---

## Flow 1: Email Registration + Login

### 1a. Register a new account

```
POST /api/v1/auth/register
Content-Type: application/json
```

```json
{
  "email": "user@example.com",
  "password": "SecurePass1",
  "full_name": "Jane Doe"
}
```

**Password rules:** min 8 characters, at least 1 uppercase, 1 lowercase, 1 digit.

**Response** `201 Created`:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "message": "User created successfully"
}
```

### 1b. Login

```
POST /api/v1/auth/login
Content-Type: application/json
```

```json
{
  "email": "user@example.com",
  "password": "SecurePass1"
}
```

**Response** `200 OK`:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "tier": "free"
}
```

Store `token` — use it as `Authorization: Bearer <token>` for all subsequent requests.

On first login, an API key (`nk_...`) is auto-generated for the user.

### 1c. Account lockout

After repeated failed login attempts, the account locks for 15 minutes. The error response includes a `retry_after` hint.

---

## Flow 2: Discord OAuth

Discord OAuth lets users sign in without a password. On first login, a new Noesis account is created automatically using their Discord email and display name.

### 2a. Get the Discord authorize URL

```
GET /api/v1/auth/discord/authorize
```

No auth required. Returns the URL to redirect the user to.

**Response** `200 OK`:
```json
{
  "url": "https://discord.com/oauth2/authorize?client_id=...&redirect_uri=...&response_type=code&scope=identify+email"
}
```

**Frontend action:** Redirect the user to `url` (full-page redirect or popup).

For stable production/custom domains, call this endpoint without a `redirect_uri` override and let the API use its configured `DISCORD_REDIRECT_URI`.

For localhost or preview deployments, the frontend may send a same-origin `redirect_uri` override, but it must:
- stay on the current browser origin,
- use an allowed admin callback path,
- and match a redirect shape the API accepts.

### 2b. Handle the callback

After the user authorizes on Discord, they are redirected back to your `redirect_uri` with a `?code=...` query parameter.

Your frontend extracts the `code` and sends it to the API:

```
POST /api/v1/auth/discord/callback
Content-Type: application/json
```

```json
{
  "code": "the_code_from_discord_redirect"
}
```

For localhost or preview deployments using a same-origin callback override, include the exact same `redirect_uri` value during the code exchange:

```json
{
  "code": "the_code_from_discord_redirect",
  "redirect_uri": "https://preview.example.com/admin/auth/discord/callback"
}
```

**Response** `200 OK` (same shape as email login):
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "tier": "free"
}
```

### 2c. What happens on the backend

| Scenario | What happens |
|----------|-------------|
| **New user** (no matching Discord ID or email) | Account created with `auth_provider=discord`, no password set |
| **Existing email match** (registered via email before) | Discord ID linked to existing account — same user, same data |
| **Returning Discord user** (Discord ID already linked) | Logged in, `last_login` updated |

In all cases, an API key is auto-generated on first login (idempotent — won't create duplicates).

### 2d. Frontend implementation pattern (React)

```tsx
// 1. Get the OAuth URL and redirect
async function loginWithDiscord() {
  const shouldUseOverride =
    window.location.hostname === "localhost" ||
    window.location.hostname === "127.0.0.1" ||
    window.location.hostname.endsWith(".vercel.app") ||
    window.location.hostname.endsWith(".railway.app");
  const redirectUri = shouldUseOverride
    ? `${window.location.origin}/admin/auth/discord/callback`
    : undefined;

  const query = redirectUri
    ? `?redirect_uri=${encodeURIComponent(redirectUri)}`
    : "";
  const res = await fetch(`${API_URL}/api/v1/auth/discord/authorize${query}`);
  const { url } = await res.json();
  window.location.href = url;  // redirect to Discord
}

// 2. On your redirect page (/auth/discord/callback), exchange the code
async function handleDiscordCallback() {
  const code = new URLSearchParams(window.location.search).get("code");
  if (!code) return;

  const res = await fetch(`${API_URL}/api/v1/auth/discord/callback`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      code,
      redirect_uri:
        window.location.hostname === "localhost" ||
        window.location.hostname === "127.0.0.1" ||
        window.location.hostname.endsWith(".vercel.app") ||
        window.location.hostname.endsWith(".railway.app")
          ? `${window.location.origin}${window.location.pathname}`
          : undefined,
    }),
  });

  const { token, user_id, email, tier } = await res.json();
  // Store token in your auth state (context, zustand, etc.)
  localStorage.setItem("noesis_token", token);
  // Navigate to dashboard
}
```

**Important:** The `redirect_uri` configured in your Discord application settings must match the canonical production callback page exactly. Dynamic same-origin overrides are intended only for preview/local admin origins.

---

## After Login: Using the Token

### Make authenticated requests

```bash
curl -s -X POST https://selemene.tryambakam.space/api/v1/engines/numerology/calculate \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": {
      "name": "Jane Doe",
      "date": "1991-08-13",
      "time": "13:31",
      "latitude": 12.9716,
      "longitude": 77.5946,
      "timezone": "Asia/Kolkata"
    }
  }'
```

### Get user profile

```
GET /api/v1/users/me
Authorization: Bearer <token>
```

Returns: `id`, `email`, `full_name`, `tier`, `consciousness_level`, `experience_points`, `birth_date`, `birth_time`, `birth_location`, `timezone`, `preferences`.

### Update user profile (birth data, preferences)

```
PUT /api/v1/users/me
Authorization: Bearer <token>
Content-Type: application/json
```

```json
{
  "full_name": "Jane Doe",
  "birth_date": "1991-08-13",
  "birth_time": "13:31:00",
  "birth_location_lat": 12.9716,
  "birth_location_lng": 77.5946,
  "birth_location_name": "Bangalore, India",
  "timezone": "Asia/Kolkata"
}
```

Once birth data is saved on the profile, engine calculations auto-populate it — the frontend doesn't need to send `birth_data` every time.

### View API keys

```
GET /api/v1/users/me/api-keys
Authorization: Bearer <token>
```

**Response** `200 OK`:
```json
{
  "items": [
    {
      "id": "key-uuid",
      "name": "Auto-generated",
      "key_prefix": "nk_a3b7c9d1",
      "tier": "free",
      "permissions": ["basic:access"],
      "consciousness_level": 0,
      "rate_limit": 60,
      "created_at": "2026-03-24T10:00:00Z",
      "expires_at": null,
      "last_used": null,
      "is_active": true
    }
  ]
}
```

The full secret key is **never** returned after creation. The `key_prefix` is shown for identification only.

---

## Password Management (email users only)

### Forgot password

```
POST /api/v1/auth/forgot-password
Content-Type: application/json
```
```json
{ "email": "user@example.com" }
```

### Reset password

```
POST /api/v1/auth/reset-password
Content-Type: application/json
```
```json
{
  "token": "reset_token_from_email",
  "new_password": "NewSecure1"
}
```

### Change password (authenticated)

```
POST /api/v1/auth/change-password
Authorization: Bearer <token>
Content-Type: application/json
```
```json
{
  "current_password": "OldPass1",
  "new_password": "NewPass1"
}
```

---

## Error Handling

All errors follow the same shape:

```json
{
  "error": "Human-readable message",
  "error_code": "ERROR_CODE",
  "details": {}
}
```

| Status | Code | Meaning |
|--------|------|---------|
| `401` | `UNAUTHORIZED` | Missing/invalid/expired token or API key |
| `409` | `CONFLICT` | User already exists (registration) |
| `422` | `VALIDATION_ERROR` | Bad request body or weak password |
| `429` | `RATE_LIMIT_EXCEEDED` | Too many requests — back off |
| `503` | `SERVICE_UNAVAILABLE` | Database unavailable or Discord OAuth not configured |

---

## Frontend Integration Checklist

- [ ] **Auth state**: Store the JWT token (memory or `localStorage`); attach as `Authorization: Bearer <token>` on every request
- [ ] **Login page**: "Sign in with Email" form + "Continue with Discord" button
- [ ] **Discord redirect page**: Catch `?code=` param, POST to `/auth/discord/callback`, store returned token
- [ ] **Profile setup**: After first login, prompt user to save birth data via `PUT /users/me` — this enables all birth-chart engines without re-sending data
- [ ] **Token expiry**: Handle `401` responses by redirecting to login (tokens expire after the configured TTL)
- [ ] **API key display**: Optionally show the user their key prefix/tier via `GET /users/me/api-keys`

---

## Endpoint Summary

| Method | Path | Auth | Purpose |
|--------|------|------|---------|
| `POST` | `/api/v1/auth/register` | — | Create email account |
| `POST` | `/api/v1/auth/login` | — | Email login → JWT |
| `GET` | `/api/v1/auth/discord/authorize` | — | Get Discord OAuth URL |
| `POST` | `/api/v1/auth/discord/callback` | — | Exchange Discord code → JWT |
| `POST` | `/api/v1/auth/forgot-password` | — | Request password reset |
| `POST` | `/api/v1/auth/reset-password` | — | Reset password with token |
| `POST` | `/api/v1/auth/change-password` | Bearer | Change password |
| `GET` | `/api/v1/users/me` | Bearer / API Key | Get user profile |
| `PUT` | `/api/v1/users/me` | Bearer / API Key | Update profile + birth data |
| `GET` | `/api/v1/users/me/api-keys` | Bearer / API Key | List user's API keys |
| `POST` | `/api/v1/engines/{id}/calculate` | Bearer / API Key | Run engine calculation |
| `POST` | `/api/v1/workflows/{id}/execute` | Bearer / API Key | Run multi-engine workflow |

## Interactive API Docs

Full OpenAPI with try-it-out: [`https://selemene.tryambakam.space/api/docs`](https://selemene.tryambakam.space/api/docs)
