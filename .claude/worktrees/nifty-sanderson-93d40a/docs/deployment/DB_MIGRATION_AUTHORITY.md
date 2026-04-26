# Database Migration Authority

## Decision

`/migrations` at repo root is the **canonical migration source of truth** for runtime schema used by the API and services.

## Why

The runtime Rust services (`noesis-api`, `noesis-auth`, `noesis-data`) are built against the schema evolved by root migrations:

- `/migrations/001_initial_users.sql`
- `/migrations/002_password_reset.sql`
- `/migrations/003_user_progression.sql`
- `/migrations/004_api_keys.sql`
- `/migrations/005_readings.sql`
- `/migrations/006_auth_improvements.sql`

Using multiple migration authorities causes drift and production defects.

## Supabase Alignment Rule

`/supabase/migrations` is **not** an independent authority.

It must be kept aligned with canonical root migrations and used only as a projection for Supabase CLI workflows.

Allowed pattern:

1. Introduce migration in `/migrations` first.
2. Validate runtime behavior/tests.
3. Mirror equivalent change to `/supabase/migrations` if needed for Supabase-managed environments.

Disallowed pattern:

- Adding schema changes only in `/supabase/migrations` without a corresponding root migration.

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
- If Supabase files are updated, PR must reference the matching root migration IDs.
- Periodically run schema diff checks between a database bootstrapped from `/migrations` and Supabase-projected schema.
