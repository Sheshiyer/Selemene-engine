---
title: Authentication
sidebar_position: 2
---

Selemene API supports two auth mechanisms:

## 1) JWT Bearer

Pass a bearer token:

```http
Authorization: Bearer <jwt>
```

## 2) API Key

Pass API key in header:

```http
X-API-Key: nk_xxx
```

## Example

```bash
curl -X POST "https://selemene-engine-production.up.railway.app/api/v1/engines/numerology/calculate" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: nk_your_api_key" \
  -d '{
    "birth_data": {"date":"1990-05-15","time":"14:30","latitude":12.9716,"longitude":77.5946,"timezone":"Asia/Kolkata"},
    "current_time":"2026-03-03T12:00:00Z",
    "precision":"standard",
    "options":{}
  }'
```
