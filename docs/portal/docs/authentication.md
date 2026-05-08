---
title: Authentication
sidebar_position: 2
---

Noesis API supports two auth mechanisms.

## 1) JWT Bearer

Pass a bearer token obtained from email/password or Discord OAuth sign-in:

```http
Authorization: Bearer <jwt>
```

## 2) API Key

Pass API key in header (preferred for CLI, scripts, server-to-server):

```http
X-API-Key: nk_xxx
```

API keys are prefixed with `nk_`. Each key is a unique user identity — rate limits and credit balances are per-key.

## Discord OAuth

Sign in with Discord at `/api/v1/auth/discord/authorize`. The callback is `/api/v1/auth/discord/callback`. On success, returns a JWT.

## Example

```bash
curl -X POST "https://selemene.tryambakam.space/api/v1/engines/numerology/calculate" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: nk_your_api_key" \
  -d '{
    "birth_data": {
      "date": "1990-05-15",
      "time": "14:30",
      "latitude": 12.9716,
      "longitude": 77.5946,
      "timezone": "Asia/Kolkata"
    }
  }'
```
