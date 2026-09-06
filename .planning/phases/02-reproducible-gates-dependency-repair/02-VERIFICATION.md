---
phase: 02-reproducible-gates-dependency-repair
status: passed
verified: 2026-09-06T16:50:15+00:00
source_commit: ba2d149106345bb9637e1d31fec8160e703c5501
---
# Phase 2 verification

| Requirement | Status | Evidence or gap |
|---|---|---|
| GATE-01 | Pass locally and remotely | The database-free repository gate passes at code source `4305265`, including 255 script tests and every Rust/API/SDK/verification/TypeScript stage. GitHub [run 34046002390](https://github.com/Sheshiyer/Selemene-engine/actions/runs/34046002390) then passed all 16 jobs, including `CI Gate`, at exact evidence head `ba2d149`. The pull-request merge commit and branch head have the same tree object `3e693be8778b7f9eccff18112f5504cbf0dea512`. |
| GATE-02 | Pass locally | Node production and complete-graph audits report zero findings after the direct TypeScript parser dependency; Rust reports zero advisories with one recorded inactive yank warning. |
| GATE-03 | Pass | A fresh archive installs all eight Node workspaces offline from the frozen lock and resolves exact root `typescript@5.9.3`; `uv.lock`, Python audit, Python 3.11/3.12 contract/smoke and Linux image imports retain their recorded passing evidence. |
| GATE-04 | Source and read-only provider pass | Railway project/environment/service selectors match the release manifest; all seven latest deployments report successful/running. Four Cloudflare configs bind account `9d9d23b27f32b2df8c6cdc1321aa2c0f10` and dry-run, while the default Wrangler login is a different account and cannot refresh live 9d9d inventory. No provider mutation was performed. |
| GATE-05 | Pass locally; production held | The 19-row executable registry and versioned receipt authority now bind release tag, short-lived operation identity, exact artifacts and service roles, role-keyed rollback, manifest-owned health targets and computed repository asset trees. Unsupported or unprovable production mutation is disabled before provider output or writes. |
| GATE-06 | Pass | User-approved strict CI Gate remains required by ruleset 15597830 with verified readback. Production promotion remains a separate HOLD. |

## Goal Truths

| Truth | Status | Evidence |
|---|---|---|
| Release authorization is exact and short-lived | Verified locally | A canonical semver tag, workflow, run/attempt operation ID, issue/expiry window and rollback freshness are required. Cross-tag, stale, expired and replayed-attempt probes fail. Durable one-use consumption is required by policy and both operational profiles stay disabled until it exists. |
| Pre-mutation authorization cannot claim future deployment identity | Verified locally | Receipt schema and validator reject candidate `deployed` claims. Provider-returned deployment ID/source/status is a separate required post-deploy attestation; its absence keeps `deploy-production` disabled. |
| Every owned mutation role has exact target and rollback authority | Verified locally | API and TypeScript are deployment/artifact roles; biofield CV is topology-only. Roots, configs, Railway selectors, health authority and role-keyed rollback sets are exact. Executed shims record the two Railway calls and both health probes. |
| Mutable multi-artifact publication cannot partially advance | Verified locally | Deploy publishes only immutable `sha-$GITHUB_SHA` candidates after validation. Release permissions are read-only and an always-failing hold job replaces alias/GitHub-release mutation until atomic multi-registry promotion with compensation exists. |
| Required assets are repository-bound | Verified locally with operational hold | Canonical paths and deterministic `sha256-tree-v1` digests/file counts are recomputed. Build recipe and container path are authoritative, while operational inclusion fails without source-bound post-build inspection attestation. |
| Verification uses local, disposable and shimmed resources | Verified | No workflow, deployment, publication, release, tag, cloud/DNS, GitHub setting, schema/data, production database, secret read or provider write ran. |

## Artifact and Wiring Verification

- `.planning/phases/02-reproducible-gates-dependency-repair/02-REVIEW-FIXES.md` maps all ten Critical and four Warning findings to code, tests, commits and explicit fail-closed dispositions. Five independent adversarial rechecks followed; the final exact-snapshot report records zero Critical, Warning, or Info findings.
- All workflow actions remain immutable-SHA pinned, and every workflow YAML file parses successfully.
- Deploy source admission executes for `main` and canonical semver tags and rejects feature refs or unsupported environments before prebuilds. The final result job cannot pass when authoritative Railway deployment is skipped.
- Provider tests evaluate actual job conditions and execute extracted Docker, Railway, Kustomize, Kubectl, curl and release scripts under recording shims with exact argv/cwd assertions.
- `RAILWAY_TOKEN` is absent from checkout and CLI installation; only credentialed scope/deploy steps receive it.
- `register_database_conditional_engines` is called by both production app-state builders and the focused no-pool/lazy-pool tests.
- Registry provenance validation resolves safe `repo://` paths and supported Markdown/source-symbol anchors; missing, traversing, stale, parser-ambiguous and declaration-decoy targets fail closed. JavaScript and TypeScript use the locked compiler AST; bounded Rust lexical scope has compile-backed macro, literal, generic and callable-bound coverage.

## Exact-Source Local Verification

At code source `4305265acee96461c40594fbb2689306d357f59d`:

- Fifth independent resolver/release recheck: 181 passed, including 141 contract and 40 release tests; 0 findings.
- Independent compile-backed Rust adjacency matrix: 9 of 9 valid variants compiled and resolved.
- `cargo test -p noesis-api database_conditional_registration --locked`: 2 passed.
- `python3 scripts/validate_action_pins.py`: exit 0.
- PyYAML parsing of every `.github/workflows/*.{yml,yaml}` file: exit 0.
- `python3 scripts/validate_contracts.py`: `schemas=6 fixtures=5 registries=1 engines=19`.
- `python3 scripts/validate_release_receipt.py --validate-fixtures`: `receipts=2 mutation_cases=9`.
- The current-production negative fixture exits 1 with 29 explicit unavailable facts.
- `env -u DATABASE_URL -u TEST_DATABASE_URL pnpm run gate:scripts`: 255 passed plus contract, receipt, migration and Docker validators.
- `env -u DATABASE_URL -u TEST_DATABASE_URL pnpm run gate`: exit 0 with 255 script, 9 core, 4 OpenAPI, 16 API integration, 35 engine SDK, 11 Noesis SDK, 36 verification and 94 TypeScript tests; all builds/typechecks pass.
- Forced CodeGraph rebuild and follow-up sync: up to date at 867 files, 15,164 nodes and 37,101 edges; functional queries resolved all three runtime registration seams and both release-authority validators.

## Exact-Head Remote Verification

At evidence head `ba2d149106345bb9637e1d31fec8160e703c5501`:

- GitHub `CI - Test & Lint` run [34046002390](https://github.com/Sheshiyer/Selemene-engine/actions/runs/34046002390) completed successfully at `2026-09-06T16:50:15Z`.
- All 16 jobs passed: required admin web, security audit, both Python images, secret scanning, lint, workspace gate, Node dependency audit, TS engines, Python 3.11 and 3.12 sidecars, workflow registry parity, tests, build, integration tests and aggregate `CI Gate`.
- The generated pull-request merge commit `e163c86b7070484e193b178741ab290c7c1ad199` has parents `main@ae3e2cef402bd0cf28d1c7a102800d215cc5f2c2` and candidate `ba2d149`; its tree equals the candidate tree exactly.
- Draft PR [#1488](https://github.com/Sheshiyer/Selemene-engine/pull/1488) remained open, draft and mergeable. No merge, auto-merge, deployment or release was performed.

## Production Hold

Phase 2 is verified complete: all seven plans executed, all deep-review findings were closed, five independent rechecks ended clean, the complete local gate passed, and the exact pushed evidence head passed the full remote CI matrix.

Production promotion remains **HOLD** independently of remote CI. Re-enabling repository-owned production mutation requires durable one-use receipt consumption, Railway-returned deployment/source/status attestation, atomic API/TypeScript coordination, source-bound image asset attestation and atomic multi-registry alias promotion with compensation. Vercel provider-side main deployment protection, exact staging/Kubernetes profiles, real production schema/rollback evidence and per-platform native-binary authority remain later work.
