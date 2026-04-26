# Supabase Migrations

The canonical migration authority for this repository is the root [`migrations/`](../migrations) directory.

`supabase/migrations/` mirrors that canonical set so a fresh Supabase bootstrap yields the same schema expected by runtime repositories.

## Naming Convention

Each Supabase migration mirrors one root migration file:

- Supabase: `<timestamp>_<root-file-name>.sql`
- Root: `<root-file-name>.sql`

Example:

- `supabase/migrations/20260225000001_001_initial_users.sql`
- `migrations/001_initial_users.sql`

## Drift Check

Run this before opening migration-related PRs:

```bash
bash scripts/check_migration_drift.sh
```

The check fails when:

- a root migration has no Supabase mirror,
- a mirror exists but content differs,
- multiple mirrors target the same root migration, or
- extra Supabase-only migration files are present.
