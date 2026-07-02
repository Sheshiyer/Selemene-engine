# Database Migration Authority

## Decision

`/migrations` at repo root is the **canonical migration source of truth** for runtime schema used by the API and services.

The `supabase/` migration mirror and `scripts/check_migration_drift.sh` have been retired as part of the Supabase retirement plan. Root migrations remain the only schema authority.

## Why

The runtime Rust services (`noesis-api`, `noesis-auth`, `noesis-data`) are built against the schema evolved by root migrations:

- `/migrations/001_initial_users.sql`
- `/migrations/002_password_reset.sql`
- `/migrations/003_user_progression.sql`
- `/migrations/004_api_keys.sql`
- `/migrations/005_readings.sql`
- `/migrations/006_auth_improvements.sql`

Using multiple migration authorities causes drift and production defects.

## Supabase Alignment Rule (Retired)

The `/supabase/migrations` mirror and `scripts/check_migration_drift.sh` are retired. They were used only as a projection for Supabase CLI workflows and are no longer maintained. Root `/migrations` is the sole schema authority.

## Operational Workflow

### CI and local test setup

Apply root migrations in lexical order:

```bash
for migration in migrations/*.sql; do
  psql "$DATABASE_URL" -f "$migration"
done
```

### Production deployment

Deploy pipelines and runbooks must reference root migrations as canonical migration order.

## Drift Prevention

- Every schema PR must include migration files under `/migrations`.
- The retired `/supabase/migrations` mirror must not be updated; it exists only as historical reference.
