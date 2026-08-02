---
title: Authentication
sidebar_position: 2
---

Noesis API supports two auth mechanisms for programmatic access.

## 1) JWT Bearer

Pass a bearer token obtained from Cloudflare Access sign-in:

```http
Authorization: Bearer <jwt>
```

## 2) API Key

Pass API key in header (preferred for CLI, scripts, server-to-server):

```http
X-API-Key: nk_xxx
```

API keys are prefixed with `nk_`. Each key is a unique user identity — rate limits and credit balances are per-key.

## Cloudflare Access

Human and admin authentication is enforced by Cloudflare Zero Trust. The Rust API validates the Access token signature, issuer, and audience before mapping identity into local `user_roles`.

Role mapping:

- Supported identity-provider group names map directly.
- An IdP `selemene-admin` claim maps to `platform-admin`.
- Cloudflare Access rule groups are policy collections, not JWT identity groups.
  Exact administrators therefore require a second, fail-closed match in
  `CF_PLATFORM_ADMIN_EMAILS` after token validation.
- Unmatched identities retain `viewer`.

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
