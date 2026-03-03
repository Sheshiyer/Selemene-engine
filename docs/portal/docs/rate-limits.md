---
title: Rate Limits
sidebar_position: 3
---

Rate limits are enforced on authenticated endpoints.

## Headers

- `X-RateLimit-Limit`
- `X-RateLimit-Remaining`
- `X-RateLimit-Reset`
- `X-RateLimit-Daily-Remaining`
- `X-RateLimit-Daily-Reset`

## 429 response

When quota is exceeded the API returns:

```json
{
  "error": "Rate limit exceeded",
  "details": {
    "code": "rate_limit"
  }
}
```
