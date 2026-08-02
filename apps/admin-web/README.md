# Admin Web (Vercel)

This app is the Vercel-hosted admin dashboard for Selemene Engine.

## Authentication

Admin access is enforced by Cloudflare Zero Trust in front of the deployed admin surface. The Rust API validates the Access token signature, issuer, and audience before mapping identity into local `user_roles`.

Role mapping:

- Identity-provider groups map directly when they use a supported local role name.
- `selemene-admin` in an identity-provider `groups` claim maps to `platform-admin`.
- Access rule-group names are not identity claims. Exact human administrators
  must also be listed in the fail-closed `CF_PLATFORM_ADMIN_EMAILS` runtime
  allowlist; the match occurs only after Access JWT validation.
- An identity with neither a supported IdP group nor an allowlisted email is `viewer`.

Local development can use `RUST_ENV=development` with `CF_DEV_BYPASS_TOKEN` and the `x-noesis-dev-auth` header.

## Runtime contract

- App base path: `/admin` (configured in `next.config.mjs`)
- Root `/` is redirected to `/admin/login` via `next.config.mjs` redirects
- Backend API origin: `NEXT_PUBLIC_API_BASE_URL`
- Session check: `GET /api/v1/admin/session`
- Token transport: `Authorization: Bearer <token>`
- API key is not required for signed-in admin users

For browser/admin sessions, use JWT bearer tokens issued after Cloudflare Access validation.
Use API keys only for CLI usage, scripts, or other non-session interfaces.

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
# Optional override; defaults to the current public Urania host.
NEXT_PUBLIC_URANIA_URL=https://urania.tryambakam.space
# Optional local bootstrap mode for non-admin tokens:
NEXT_PUBLIC_ADMIN_DEV_MODE=false
```

`NEXT_PUBLIC_ADMIN_DEV_MODE=true` allows a `basic:access` token to pass UI route guards for local scaffolding only.

If `NEXT_PUBLIC_API_BASE_URL` is missing in a non-local deployment, login/session calls now fail fast with a clear `ADMIN_ENV_MISCONFIG` error instead of silently targeting `http://localhost:8080`.

## Vercel setup

- Root Directory: `apps/admin-web`
- Build Command: `npm run build`
- Output: default Next.js output
- `vercel.json` pins framework preset to Next.js
- Required env:
  - `NEXT_PUBLIC_API_BASE_URL` -> Railway API base URL
  - `NEXT_PUBLIC_URANIA_URL` -> Urania origin for invitation conversation handoff
  - `NEXT_PUBLIC_ADMIN_DEV_MODE=false`

Current project wiring:

- Project: `sheshiyers-projects/enantiodromia-engine-dashboard`
- Production alias: `https://enantiodromia-engine-dashboard.vercel.app`
