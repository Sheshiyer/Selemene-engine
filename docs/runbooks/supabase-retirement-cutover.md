# Supabase Retirement Cutover Runbook

## Preflight

- Railway Postgres is provisioned and reachable.
- Cloudflare Access application protects admin-web and API routes that require human/admin auth.
- The Access policy uses the `selemene-admin` rule group, while
  `CF_PLATFORM_ADMIN_EMAILS` explicitly names the validated identities that
  become local `platform-admin` users.
- R2 bucket exists: `selemene-raga-clips`.
- Runtime secrets are set: `DATABASE_URL`, `CF_ACCESS_ISSUER`,
  `CF_ACCESS_AUDIENCE`, `CF_PLATFORM_ADMIN_EMAILS`, Dodo envs,
  `INTERNAL_SERVICE_KEY`, and R2 envs for TS generation jobs.

## Database Migration

1. Take final Supabase backup.
2. Dry-run schema pass:
   `scripts/supabase-to-railway-migrate.sh --dry-run --schema-only --source "$SUPABASE_DATABASE_URL" --target "$RAILWAY_DATABASE_URL"`
3. Run schema pass:
   `scripts/supabase-to-railway-migrate.sh --yes --schema-only --source "$SUPABASE_DATABASE_URL" --target "$RAILWAY_DATABASE_URL"`
4. Run data pass:
   `scripts/supabase-to-railway-migrate.sh --yes --data-only --source "$SUPABASE_DATABASE_URL" --target "$RAILWAY_DATABASE_URL"`
5. Verify row counts and partitions from script output.

## Deploy

1. Deploy API with `DATABASE_URL` pointing to Railway Postgres and CF validation envs set.
2. Deploy admin-web behind Cloudflare Access.
3. Deploy TS engine script changes for R2 upload.

## Smoke Tests

- `curl -fsS https://selemene.tryambakam.space/health/live`
- `curl -fsS https://selemene.tryambakam.space/health/ready`
- A fresh CF-authenticated allowlisted admin session returns `platform-admin`;
  an authenticated non-allowlisted identity remains `viewer`.
- Dodo webhook forward path still accepts valid `X-Forward-Secret`.
- Raga generation uploads to R2 and `/internal/raga/clip` persists the row.

## Cleanup

- Unset Supabase secrets from Railway, local shells, and CI.
- Remove Discord OAuth app credentials from secret stores.
- Confirm `oauth_accounts` has been dropped.
