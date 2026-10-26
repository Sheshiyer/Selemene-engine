# @selemene/engine-sdk: public npm release and consumer switch-over

Decision (2026-10-27, operator): publish `@selemene/engine-sdk` to public npm; hold `@noesis/sdk` (no consumer imports it).

## Consumer map (verified 2026-10-27)

| Repo | Today | After release |
|---|---|---|
| sankalpa | `file:vendor/selemene-engine-sdk-0.2.0.tgz` (dist byte-identical to main@dcc1a046) | `^0.2.0` from npm, vendor tarball removed |
| urania-137 | hand-copied types in `src/types/index.ts` (7-field provenance, in sync) | optional follow-up PR to import types from the SDK |
| noesis-raycast | hand-copied types in `src/lib/types.ts` (in sync) | optional follow-up; Raycast store needs public-registry deps, which npm satisfies |
| antahkarana | Rust `selemene_health` via gateway worker, no TS client | no change |
| others (6) | no Selemene client code | no change |

## Release (operator runs; npm may prompt for 2FA)

1. `npm login` (the token on this Mac returned 401 on 2026-10-27).
2. Confirm the `@selemene` scope belongs to your npm account or org (registry search shows no packages under it, which does not prove the name is free).
3. `DRY_RUN=1 scripts/release/publish-engine-sdk.sh` then `scripts/release/publish-engine-sdk.sh`.
   The script: clean detached checkout of origin/main, refuses if the version already exists, bun install, typecheck, test, build, checks no src/tests in the tarball, `npm publish --access public`, registry readback.

Pre-verified on main@dcc1a046: typecheck ok, 40 tests pass, 26 files / 22.6 KB packed (dist + README + package.json), secret scan clean, installs and imports under plain Node, `EngineClient.capabilities()` present.

No npm provenance attestation: that needs a CI OIDC token, and CI is disabled by policy.

## Switch-over

- sankalpa branch `chore/engine-sdk-from-npm`: dependency to `^0.2.0`, delete `vendor/selemene-engine-sdk-0.2.0.tgz`, regenerate the lockfile after the package is live, vitest + typecheck, PR.
- urania-137 #180 and noesis-raycast: separate optional PRs replacing copied types with `import type { ... } from "@selemene/engine-sdk"`.

## Receipt (2026-10-27)

- Published `@selemene/engine-sdk@0.2.0` (dist-tag latest) by `tryambakam-noesis`, owner of the `@selemene` npm org. shasum `070ccf5d4f35eb24253c45cd8261f8b775fc7142`, 26 files, 22.6 kB, from Selemene-engine main@dcc1a046 via `scripts/release/publish-engine-sdk.sh` (operator approved 2FA in the browser).
- npm normalised `repository.url` to `git+https://...` on publish; fix in source with the next version bump (`npm pkg fix`).
- Sankalpa PR Sheshiyer/sankalpa#19: dependency `^0.2.0`, lockfile resolves from the registry (integrity matches), vendor tarball removed; `npm ci`, typecheck and 144 vitest tests pass.
- `@noesis/sdk` held (no consumer). The `@selemene` org also has access to an unrelated `@selemene/sdk` package not visible in public search.
