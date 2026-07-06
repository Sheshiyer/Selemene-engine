# Retire `biofield-web`, `noesis-web`, and `dyad-ui` from Selemene-engine

## Goal

Remove the web frontend apps `apps/biofield-web` and `apps/noesis-web`, plus the shared package `packages/dyad-ui`, from `Selemene-engine`. Retain only `apps/admin-web` as the web frontend. Shift the repository's focus to SDKs, tooling, Rust engines, Python services, and the admin operator surface.

## Context

- `sankalpa` (`../sankalpa/`) is a standalone Electron app that already unifies the client-safe surfaces from `biofield-web` and `noesis-web`.
- `sankalpa` passes `npm test` (35 tests) and `npm run typecheck`.
- `dyad-ui` files are already mirrored 1:1 in `sankalpa/src/renderer/dyad/`.
- `apps/noesis-web/public/` still contains assets not yet in `sankalpa`: `depth-reading/meshes/`, `depth-reading/mockups/`, `design-system/`, and `icons/`.

## Approach

Hard delete the retired apps and package after migrating all remaining assets to `sankalpa`. Clean up workspace references, CI/CD, infra docs, and runbooks so the repo remains buildable and deployable.

## Asset Migration

Copy the remaining `apps/noesis-web/public/` tree into `sankalpa/public/` while preserving relative paths:

- `depth-reading/meshes/*.glb` → `sankalpa/public/depth-reading/meshes/`
- `depth-reading/mockups/*.png` → `sankalpa/public/depth-reading/mockups/`
- `design-system/**` → `sankalpa/public/design-system/`
- `icons/**` → `sankalpa/public/icons/`
- `depth-reading/README.md` → `sankalpa/public/depth-reading/README.md`

Skip `.DS_Store`, `.gitkeep`, and macOS metadata. `apps/biofield-web/public/` has no additional assets beyond the character PNGs already in `sankalpa`.

## Deletion Targets

- `apps/biofield-web/`
- `apps/noesis-web/`
- `packages/dyad-ui/`

`pnpm-workspace.yaml` uses globs (`apps/*`, `packages/*`), so no workspace config edit is required. Root `package.json` scripts reference only `@noesis/verification`, so they stay untouched.

## Lockfile Regeneration

After deletion, regenerate lockfiles so they no longer contain entries for removed packages:

- `pnpm install --lockfile-only`
- `npm install --package-lock-only`

## Reference Cleanup

### Must-fix

- `.github/workflows/test.yml` — remove the `noesis-web` test job.
- `.github/workflows/deploy.yaml` — update the Vercel-native-deploy comment to reflect only `admin-web`.
- `infra/suno-bridge/` — update `run-all.sh`, `deploy.sh`, and `README.md` to point at `sankalpa` instead of `apps/noesis-web`.
- `tools/alpha-witnesses.py` — repoint character paths to `sankalpa/public/depth-reading/characters`.

### Should-fix

- `docs/runbooks/biofield-capture-analysis.md` — rewrite to use `sankalpa` or mark as archived.
- `docs/engines/*.md` — update renderer paths to `sankalpa/src/renderer/...` or add a historical note.
- `docs/plans/*biofield-web*.md`, `*noesis-web*.md` — leave as historical but add an "archived" header.
- `README.md` — add a note that `sankalpa` is the desktop client successor.

### Low-priority

- `packages/noesis-sdk-ts/src/billing.ts` comments mentioning `biofield-web`.
- `RELEASE_NOTES_v3.3.0.md` and `CHANGELOG.md` — keep as history, no edit needed.
- `raagaegnin/*.md` — update only if they describe active workflows.

## Verification

1. Asset parity in `sankalpa` via `diff -r` against source `public/` trees.
2. `sankalpa` health: `npm test` and `npm run typecheck` pass.
3. `Selemene-engine` workspace installs cleanly with regenerated lockfiles.
4. `apps/admin-web` builds and typechecks.
5. CI green: `test.yml` and `deploy.yaml` run without missing jobs.
6. Reference audit: `grep -R "apps/biofield-web\|apps/noesis-web\|@selemene/dyad-ui"` returns only historical docs and changelog entries.
7. Smoke checks: `scripts/smoke_admin_web.sh` and `scripts/smoke_biofield_web.sh` still pass against the API.

## Risks

- `apps/noesis-web` contained Next.js API routes and client libraries (e.g., Raaga/Suno). These are intentionally out of scope for `sankalpa` and remain backend-owned, but any external docs or runbooks that assume those routes exist will break.
- `docs/engines/*.md` has many renderer paths pointing into `apps/noesis-web`. These need batch updates or archive headers.
- `@noesis/witness-pipeline` is used by both `apps/noesis-web` and `packages/verification`, so it must **not** be deleted.

## Rollback

If deletion causes unexpected failures, restore the directories from the Git commit immediately preceding the deletion commit. All asset migration is committed first, so no data is lost.
