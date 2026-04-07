# Admin Web Vercel Deployment Runbook

Last updated: 2026-02-28

## Target

- App: `apps/admin-web`
- Vercel project (recommended): `sheshiyers-projects/selemene-admin-dashboard`
- Framework preset: `Next.js`
- Build command: `npm run build`

### Root directory modes (important)

Choose one setup mode and keep it consistent:

1. **Monorepo import from repo root**
   - Root Directory in Vercel: `apps/admin-web`
2. **Link/deploy from inside `apps/admin-web`**
   - Root Directory in Vercel: `.`

If Vercel says `apps/admin-web does not exist`, verify the connected GitHub repository is actually `Sheshiyer/Selemene-engine`.

## Required environment variables

- `NEXT_PUBLIC_API_BASE_URL=https://selemene-engine-production.up.railway.app`
- `NEXT_PUBLIC_ADMIN_DEV_MODE=false`

Set these at minimum in **Production**. Mirror to **Preview** when preview deployments are used.

## Clean setup flow

```bash
cd apps/admin-web
vercel link --yes --scope sheshiyers-projects --project selemene-admin-dashboard
vercel project inspect selemene-admin-dashboard --scope sheshiyers-projects
```

```bash
cd apps/admin-web
vercel env add NEXT_PUBLIC_API_BASE_URL production --value 'https://selemene-engine-production.up.railway.app' --yes --scope sheshiyers-projects
vercel env add NEXT_PUBLIC_ADMIN_DEV_MODE production --value 'false' --yes --scope sheshiyers-projects
vercel env add NEXT_PUBLIC_API_BASE_URL preview --value 'https://selemene-engine-production.up.railway.app' --yes --scope sheshiyers-projects
vercel env add NEXT_PUBLIC_ADMIN_DEV_MODE preview --value 'false' --yes --scope sheshiyers-projects
vercel env ls --scope sheshiyers-projects
```

```bash
cd apps/admin-web
vercel --prod --yes --scope sheshiyers-projects
```

## Smoke checks

After deploy, verify quickly with script:

```bash
ADMIN_WEB_URL=https://selemene-admin-dashboard.vercel.app \
API_BASE_URL=https://selemene-engine-production.up.railway.app \
bash scripts/smoke_admin_web.sh
```

Manual equivalent:

1. `GET /admin/login` returns `200`
2. Backend liveness endpoint is reachable: `GET /health/live` -> `200`
3. `GET /api/v1/admin/session` without token returns `401` (proves route exists + auth gate)
4. `GET /api/v1/admin/api-keys` without token returns `401` (proves route exists + auth gate)
5. `DELETE /api/v1/admin/api-keys/{key_id}` without token returns `401` (proves delete route exists + auth gate)

## Notes

- `NEXT_PUBLIC_*` variables are public by design in browser bundles.
- `next.config.mjs` uses `basePath: "/admin"`, so routes are expected under `/admin/*`.
- Avoid deploying from repo root for this app unless Vercel Root Directory is explicitly `apps/admin-web`.
- Backend API keys page depends on migration `009_api_keys_name_prefix.sql` (`api_keys.name`, `api_keys.key_prefix`). Ensure DB migrations are up to date before smoke testing.
