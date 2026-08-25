# Selemene Gate Foundation Evidence

Date: 2026-08-26
Branch: `codex/selemene-gate-foundation`
Scope: repository-owned gates only; no deployment, push, merge, secret, branch-protection, or live-service mutation.

## Guarantees established

### Migration history

- `migrations/history.sha256` is the immutable filename-and-SHA ledger for every SQL migration currently present through `037`.
- The exact historical filename-and-digest shape through `037` is anchored by a hardcoded SHA-256 of a deterministic canonical representation, so coordinated SQL-and-ledger edits fail.
- Only the historical duplicate `007` is accepted; historical gaps `015`/`016` must remain absent from both disk and ledger.
- Every SQL file, including future `038+` files, must have a ledger entry and matching SHA-256.
- Ledgered extensions must be unique and continuous, beginning at `038`.
- Deletion, rename, checksum drift, malformed names, historical insertion, untracked extension, duplicate extension, and extension gaps fail closed.
- `scripts/apply-migrations.sh` journals exact filename, SHA-256, state, and application time while preserving lexical order, `psql -X`, and `ON_ERROR_STOP=1`.
- Every runner mode validates `migrations/history.sha256` before the first database probe. Appending `038+` therefore requires a matching ledger checksum and continuous legal numbering before any SQL can run.
- Exact already-applied files are skipped, including both distinct `007` filenames; checksum drift fails before SQL is replayed, and pending `038+` files remain incremental.
- Ordinary migrations apply and record atomically under a transaction-scoped advisory lock. Transaction-incompatible `CREATE INDEX CONCURRENTLY` history uses a session advisory lock plus explicit `applying` → `applied` state; failure leaves a visible dirty row and every subsequent run refuses operator-unaudited continuation.
- A nonempty pre-journal schema is never replayed blindly, including when an empty journal table already exists. The explicit `--adopt-through NNN` path first validates the canonical ledger, then atomically records audited filename/checksum history; empty databases bootstrap normally.
- PostgreSQL journal probes emit canonical `t`/`f`, while the parser defensively accepts both PostgreSQL short and word-form boolean text.
- Both database lanes in CI use this canonical runner instead of masked raw loops.
- The Suno bridge orchestrator requires an exported `DATABASE_URL` and `psql`, always invokes the incremental runner even when persistent state says migration is done, and records omission or failure as a nonzero failed run.

### Docker workspace cache

- `scripts/validate_docker_workspace.py` derives workspace members and explicit `[[bin]]`, `[[bench]]`, and `[[example]]` targets from Cargo manifests, then structurally checks manifest `COPY` coverage and actual dependency-cache stub paths.
- `noesis-sdk`, binary-only `noesis-tui`, `engine-biofield-capture`, and `engine-financial-biosensor` now participate in the cache layer.
- All currently declared explicit targets, including both Vimshottari benches, have cache stubs; crate names in comments cannot satisfy the validator.
- The dependency cache build no longer suppresses `cargo build` failures with `|| true`.

### TypeScript truth

- TypeScript no longer uses removed `baseUrl`; the `@/*` alias is relative.
- The engine metadata route has a concrete `EngineMetadata | ErrorResponse` boundary.
- Test JSON is narrowed through explicit shape parsers, and nullable image assertions use runtime guards.
- Strict typecheck passes without exclusions, `any`, `ts-ignore`, or reduced compiler strictness.
- Bun remains pinned to `1.3.13` in CI and the existing 88-test engine suite remains green.
- The CI media smoke uses bounded health readiness, fail-on-HTTP-error requests, and `jq -e` assertions over real Raaga audio plus Sigil guidance-only method, absent-SVG, and null-image fields; only EXIT cleanup may suppress errors.
- The Python biofield smoke uses bounded readiness and a generated offline image upload, then fails closed unless the real `biofield-cv/v1` / `real-cv/v1` response exposes all 11 algorithms and expected metrics/quality fields.

### Canonical CI and delivery source

- `pnpm run gate` composes migration/Docker behavior gates, witness-pipeline build, existing verification tests/typecheck, and TS typecheck/tests.
- `requirements-gates.txt` pins the Python dependency used by repository gate tests.
- `test.yml` exposes `workflow_call`, runs the workspace gate, and ends with an always-running `CI Gate` that fails unless every required Rust, security, secret-scan, build, TS, and Python lane succeeded.
- `deploy.yaml` calls the complete reusable CI workflow before witnessing the triggering Git SHA.
- Every image build and deployment checkout uses that witnessed SHA. Container metadata includes an explicit full-SHA image tag, and Kubernetes references that same tag.
- Railway deployment and tag release advertising require successful API and TypeScript image builds; the release body advertises the exact produced `sha-${validated SHA}` tags rather than nonexistent ref-name tags.
- Railway `build.watchPatterns` includes `migrations/**`, so migration-only changes trigger a rebuild; this does not apply migrations.

## TDD receipts

### RED

- `bun run typecheck`: exit `1`; `TS5102` (`baseUrl` removed) and `TS5090` (non-relative alias).
- Initial `python3 -m pytest tests/scripts -q`: 12 failures because the ledger, validators, and migration runner did not exist.
- After the config obstruction was removed, strict typecheck exposed 13 real errors: nullable generated-image access, an invalid generic metadata return type, an `unknown` metadata return, and unvalidated `response.json()` values.
- First root gate: exit `2`; witness-pipeline build failed `TS2307` because runtime import `playwright` was undeclared.
- Independent-review migration fixtures initially failed 2/12: coordinated historical SQL-and-ledger drift and a ledgered `015` gap both returned success.
- Independent-review Docker/wiring fixtures initially failed 5/8: comment-only and missing explicit target stubs, optional Suno migration wiring, masked media smokes, and API-only delivery dependencies all returned false green.
- Final adversarial journal/state/smoke/release fixtures initially failed 8/11: the runner replayed every file, persisted orchestration state hid migrations, Python smoke masked HTTP/contract failure, and release notes advertised tags the builders did not emit.
- A final empty-journal/nonempty-schema fixture also reproduced replay on its first RED run; the runner now refuses that partially initialized state.
- Real-PostgreSQL audit fixtures initially failed 2/2: word-form `true|false` probe output was rejected, and normal application contacted `psql` before rejecting an unledgered `038`.

### GREEN

- `python3 -m pytest tests/scripts -q`: 31 behavior tests pass, including persistent journal state across invocations, duplicate `007` names, second-run skips, pending `038`, drift refusal, transactional rollback, nontransactional dirty refusal, empty-journal refusal, audited baseline adoption, PostgreSQL boolean forms, pre-connection ledger validation, forced orchestration reruns, and sidecar/release wiring.
- `python3 scripts/validate_migrations.py`: current history valid through `037`; next legal version `038`.
- `python3 scripts/validate_docker_workspace.py`: current Docker cache coverage complete and fail-closed.
- `pnpm --filter @noesis/witness-pipeline build`: passes after declaring exact runtime dependency `playwright@1.61.1`.
- `cd ts-engines && bun run typecheck && bun test`: typecheck passes; 88 pass, 0 fail, 268 expectations.
- `pnpm run gate`: exit `0`; 31 script behaviors, 36 verification tests, both TypeScript typechecks, and all 88 Bun tests pass.
- Disposable PostgreSQL 16 integration: a fresh database applied and journaled all 36 historical files, including both `007` files and the nontransactional `030`; an immediate second run reported `applied=0 skipped=36` with 36 `applied` journal rows and no dirty state.
- Real Docker verification reached the unsuppressed dependency build. `linux/amd64` under QEMU was inconclusive after GCC `cc1` segfaulted compiling `libswisseph-sys` (~690 seconds); native `linux/arm64` deterministically exposed 12 pre-existing `libswisseph-sys 0.1.2` `c_char` pointer-signedness `E0308` errors. Unified CI now blocks on that production-build defect instead of passing it silently.

## Clean-worktree prerequisites and commands

```bash
python3 -m pip install --disable-pip-version-check -r requirements-gates.txt
pnpm install --frozen-lockfile
(cd ts-engines && bun install --frozen-lockfile)
pnpm run gate
```

Workflow syntax and patch hygiene:

```bash
python3 -c 'import yaml; [yaml.safe_load(open(path)) for path in [".github/workflows/test.yml", ".github/workflows/deploy.yaml"]]'
actionlint .github/workflows/test.yml .github/workflows/deploy.yaml  # when installed
git diff --check
```

## Remaining Wave 1 gaps

- `.github/workflows/release.yml` has not been brought under this gate or reviewed for equivalent fail-closed behavior.
- GitHub Actions remain version-tag pinned rather than immutable commit-SHA pinned.
- The admin smoke remains configuration-dependent, and full Python sidecar coverage beyond the current biofield contract lane remains open.
- Remote branch protection and required-check configuration were not changed or verified.
- Immutable digest-only promotion, deployment attestation, and tested rollback remain open; this slice establishes exact source SHA, not full supply-chain promotion.
- A production pre-deploy migrator is still open. The canonical runner is enforced in CI and the Suno bridge operator, but `deploy.yaml` still does not apply schema mutations before Railway deployment; Railway watch triggers rebuild only.
- No successful production image exists from this slice: the now-fail-closed cache build exposes the documented `libswisseph-sys` native-arm64 compile defect, while the amd64 QEMU attempt was environmentally inconclusive.

No remote or live deployment claim is made by this evidence file.
