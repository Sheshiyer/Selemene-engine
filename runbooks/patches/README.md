# Patches archive

Patches recovered from in-flight work that was not (yet) ready to merge. Kept
here so the engineering intent isn't lost when the originating stash, branch,
or worktree is cleaned up.

## Convention

- Filename: `<topic>-<YYYY-MM>.patch`
- Format: `git format-patch` or `git stash show -p` output
- Each patch must have a sibling note in this README explaining:
  - **What** the patch does
  - **Why** it exists outside main
  - **Replay instructions** (whether it can be `git apply`'d cleanly today,
    or needs re-derivation against current main)

These are NOT auto-applied by anything. They are reference material.

---

## Index

### `admin-keys-coalesce-2026-05.patch` (2026-05-06)

**What:** Adds `COALESCE(...)` defaults to every `SELECT` on `api_keys` in
`crates/noesis-data/src/repositories/admin_repository.rs` so the admin API-keys
endpoints don't return 500 when joined `users` rows or optional `api_keys`
columns (`tier`, `permissions`, `consciousness_level`, `rate_limit`,
`is_active`) are missing/NULL on legacy databases. Also includes a few `cargo
fmt` whitespace fixes.

**Why outside main:** Originally stashed on `codex/admin-portal-foundation-p0`
during a 500-investigation. Since stashing, main grew the same file from 1210
to 2040+ lines and added an existing `missing_api_keys_optional_columns()`
retry path that handles legacy schemas via a fallback query. The two
strategies overlap but aren't identical:

- This patch: defensive-by-default — every column gets a `COALESCE` default
  inline, no fallback query needed.
- Main today: optimistic-by-default + fallback to a legacy query when the
  optimistic query errors with the specific column-missing signature.

The patch can't be `git apply`'d directly because the surrounding code has
moved; re-derivation requires reading the current file structure and porting
the `COALESCE` calls into the live `SELECT` statements.

**Replay instructions:**
1. `git checkout -b fix/admin-keys-coalesce main`
2. Read this patch as a guide for which columns to wrap.
3. Read `crates/noesis-data/src/repositories/admin_repository.rs` (current
   shape — note the `list_api_keys` and `get_api_key_*` paths).
4. Decide whether to add `COALESCE` inline OR keep the existing
   optimistic+fallback structure.
5. Run `cargo test -p noesis-data admin_repository::` to confirm no
   regression.

**Why both might still be worth it:** Defense-in-depth for query failures vs.
the existing fallback. If admin endpoints ever 500 on a fresh staging DB or
during a partial migration, this is the first place to look.
