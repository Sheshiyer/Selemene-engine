# Admin Web (Vercel)

This app is the Vercel-hosted admin dashboard for Selemene Engine.

## Runtime contract

- App base path: `/admin` (configured in `next.config.mjs`)
- Backend API origin: `NEXT_PUBLIC_API_BASE_URL`
- Auth flow:
  - Login: `POST /api/v1/auth/login`
  - Session check: `GET /api/v1/admin/session`
  - Token transport: `Authorization: Bearer <token>`

## Local development

```bash
cd apps/admin-web
npm install
npm run dev
```

Default URL: `http://localhost:3001/admin/login`

## Environment variables

```bash
NEXT_PUBLIC_API_BASE_URL=http://localhost:8080
# Optional local bootstrap mode for non-admin tokens:
NEXT_PUBLIC_ADMIN_DEV_MODE=false
```

`NEXT_PUBLIC_ADMIN_DEV_MODE=true` allows a `basic:access` token to pass UI route guards for local scaffolding only.

## Vercel setup

- Root Directory: `apps/admin-web`
- Build Command: `npm run build`
- Output: default Next.js output
- Required env:
  - `NEXT_PUBLIC_API_BASE_URL` -> Railway API base URL
  - `NEXT_PUBLIC_ADMIN_DEV_MODE=false`
