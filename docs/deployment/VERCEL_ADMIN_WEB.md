# Admin Web Vercel Deployment Runbook

Last updated: 2026-02-28

## Target

- App: `apps/admin-web`
- Vercel project: `sheshiyers-projects/enantiodromia-engine-dashboard`
- Production domain: `https://enantiodromia-engine-dashboard.vercel.app`
- Framework preset: `Next.js`
- Root directory: `.` (set by linking from `apps/admin-web`)
- Build command: `npm run build`

## Required environment variables

- `NEXT_PUBLIC_API_BASE_URL=https://selemene.tryambakam.space`
- `NEXT_PUBLIC_ADMIN_DEV_MODE=false`

Set these at minimum in **Production**. Mirror to **Preview** when preview deployments are used.

## Clean setup flow

```bash
cd apps/admin-web
vercel link --yes --scope sheshiyers-projects --project enantiodromia-engine-dashboard
vercel project inspect enantiodromia-engine-dashboard --scope sheshiyers-projects
```

```bash
cd apps/admin-web
vercel env add NEXT_PUBLIC_API_BASE_URL production --value 'https://selemene.tryambakam.space' --yes --scope sheshiyers-projects
vercel env add NEXT_PUBLIC_ADMIN_DEV_MODE production --value 'false' --yes --scope sheshiyers-projects
vercel env add NEXT_PUBLIC_API_BASE_URL preview --value 'https://selemene.tryambakam.space' --yes --scope sheshiyers-projects
vercel env add NEXT_PUBLIC_ADMIN_DEV_MODE preview --value 'false' --yes --scope sheshiyers-projects
vercel env ls --scope sheshiyers-projects
```

```bash
cd apps/admin-web
vercel --prod --yes --scope sheshiyers-projects
```

## Smoke checks

After deploy, verify:

1. `GET /` returns `307` to `/admin/login`
2. `GET /admin/login` returns `200`
3. Backend liveness endpoint is reachable:
   - `https://selemene.tryambakam.space/health/live`

## Notes

- `NEXT_PUBLIC_*` variables are public by design in browser bundles.
- `next.config.mjs` uses `basePath: "/admin"`, so routes are expected under `/admin/*`.
- Avoid deploying from repo root for this app; deploy from `apps/admin-web` or set monorepo root directory accordingly in Vercel.