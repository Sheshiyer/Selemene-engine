# Admin Web (Vercel)

This app is the Vercel-hosted admin dashboard for Selemene Engine.

## Runtime contract

- App base path: `/admin` (configured in `next.config.mjs`)
- Backend API origin: `NEXT_PUBLIC_API_BASE_URL`
- Auth flow:
  - Login: `POST /api/v1/auth/login`
  - Session check: `GET /api/v1/admin/session`
  - Discord OAuth UI: `/auth/discord/callback` public callback shell
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
# Optional Discord OAuth frontend shell:
NEXT_PUBLIC_DISCORD_OAUTH_CLIENT_ID=
NEXT_PUBLIC_DISCORD_OAUTH_REDIRECT_URI=
NEXT_PUBLIC_DISCORD_OAUTH_SCOPE="identify email guilds"
NEXT_PUBLIC_DISCORD_OAUTH_PROMPT=consent
# Optional override; defaults to Discord's standard authorize endpoint:
NEXT_PUBLIC_DISCORD_OAUTH_AUTHORIZE_URL=https://discord.com/oauth2/authorize
```

`NEXT_PUBLIC_ADMIN_DEV_MODE=true` allows a `basic:access` token to pass UI route guards for local scaffolding only.

Discord OAuth notes:

- If `NEXT_PUBLIC_DISCORD_OAUTH_CLIENT_ID` is unset, the Discord CTA remains visible but disabled with an explanatory message.
- If `NEXT_PUBLIC_DISCORD_OAUTH_REDIRECT_URI` is unset, the app derives it from the current browser origin plus `/admin/auth/discord/callback`.
- This wave is frontend-only: Discord can authorize and return to the callback page, but backend code exchange and admin session issuance are still pending.

## Vercel setup

- Root Directory: `apps/admin-web`
- Build Command: `npm run build`
- Output: default Next.js output
- `vercel.json` pins framework preset to Next.js
- Required env:
  - `NEXT_PUBLIC_API_BASE_URL` -> Railway API base URL
  - `NEXT_PUBLIC_ADMIN_DEV_MODE=false`

Current project wiring:

- Project: `sheshiyers-projects/selemene-admin-dashboard`
- Production alias: `https://selemene-admin-dashboard.vercel.app`
