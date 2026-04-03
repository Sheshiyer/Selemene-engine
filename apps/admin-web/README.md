# Admin Web (Vercel)

This app is the Vercel-hosted admin dashboard for Selemene Engine.

## Runtime contract

- App base path: `/admin` (configured in `next.config.mjs`)
- Root `/` is redirected to `/admin/login` via `next.config.mjs` redirects
- Backend API origin: `NEXT_PUBLIC_API_BASE_URL`
- Auth flow:
  - Login: `POST /api/v1/auth/login`
  - Session check: `GET /api/v1/admin/session`
  - Discord authorize: `GET /api/v1/auth/discord/authorize`
  - Discord callback exchange: `POST /api/v1/auth/discord/callback`
  - UI callback routes: `/admin/login/discord-callback` and `/admin/auth/discord/callback`
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

If `NEXT_PUBLIC_API_BASE_URL` is missing in a non-local deployment, login/session calls now fail fast with a clear `ADMIN_ENV_MISCONFIG` error instead of silently targeting `http://localhost:8080`.

Discord OAuth settings:

- The API deployment must have `DISCORD_CLIENT_ID`, `DISCORD_CLIENT_SECRET`, and `DISCORD_REDIRECT_URI` configured.
- `DISCORD_REDIRECT_URI` must exactly match the callback URL registered in the Discord developer portal.
- The admin web supports the callback alias `/admin/auth/discord/callback` in addition to the existing `/admin/login/discord-callback`, so either path can be used as long as the Discord app config and API env use the same exact URI.
- The login UI now sends its current dashboard callback URI to the API, and the API accepts it only when it stays on the current browser origin and one of the allowed admin callback paths. This keeps Discord login working across the main dashboard origin and compatible preview/local admin origins without opening arbitrary redirect targets.
- For localhost debugging against the Railway API, the deployed API `ALLOWED_ORIGINS` must include `http://localhost:3001` (and/or `http://localhost:3000` if you run Next there).
- If the custom API domain does not return localhost CORS headers, prefer the direct Railway public API URL for local testing, for example `NEXT_PUBLIC_API_BASE_URL=https://selemene-engine-production.up.railway.app`.

## Vercel setup

- Root Directory: `apps/admin-web`
- Build Command: `npm run build`
- Output: default Next.js output
- `vercel.json` pins framework preset to Next.js
- Required env:
  - `NEXT_PUBLIC_API_BASE_URL` -> Railway API base URL
  - `NEXT_PUBLIC_ADMIN_DEV_MODE=false`

Current project wiring:

- Project: `sheshiyers-projects/enantiodromia-engine-dashboard`
- Production alias: `https://enantiodromia-engine-dashboard.vercel.app`
