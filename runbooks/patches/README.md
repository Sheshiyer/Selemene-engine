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

---

### `witness-retirement-pass2-2026-07.patch` (2026-07-01)

**What:** Pass 2 of the witness-agents retirement into Selemene. Captures all
changes from commit `0b14df5c`:
- Documentation updates positioning Selemene as canonical (ENGINES.md,
  PROJECT_OVERVIEW.md, API docs, SDK README).
- Retirement artifacts added to witness-agents repo (README banner +
  RETIREMENT.md with migration guidance and asset table).
- Persona regression tests (7 new deterministic tests) in noesis-witness.
- Verification matrix run and design doc checklist closed.

**Why outside main:** This patch is the clean archival snapshot of the
two-pass parallel dispatch work. The work is already committed on main
(0b14df5c). The patch exists here per the runbooks/patches convention for
traceability and potential re-derivation in isolated worktrees or audits.

**Replay instructions:**
1. The changes are already on main at `0b14df5c`.
2. To inspect or re-apply in a clean context:
   - `git checkout -b audit/witness-retirement-pass2`
   - `git format-patch -1 0b14df5c --stdout | git apply --check`
   - Or: `git show 0b14df5c | git apply`
3. Run the critical matrix to verify:
   - `cargo test -p noesis-witness`
   - `cargo test -p noesis-api --test assets_generate_contract_test`
   - `cd packages/witness-pipeline && npm test`
   - `cd packages/noesis-sdk-ts && npm test`
4. All frozen public contracts (`/witness/interpret` 6-field shape, engine
   envelopes, SDK `interpretWitness`) must remain byte-identical.

**Why it exists:** Documents the two-pass parallel dispatch workflow used for
this retirement (Pass 1 = wiring + additive surfaces; Pass 2 = docs +
retirement artifacts + quality gates + verification). Serves as reference for
future similar efforts.

**References:**
- Design: `docs/plans/2026-07-01-witness-agents-retirement-minimal-arch-design.md`
- Pass 1 commit: `4ff25d71`
- Patch file: `runbooks/patches/witness-retirement-pass2-2026-07.patch` (1606 lines)
