---
phase: 02-reproducible-gates-dependency-repair
status: gaps_found
verified: 2026-09-06T13:42:29+00:00
source_commit: 941e3cfd059831330f7f3adb2f90f526384426c5
---
# Phase 2 verification

| Requirement | Status | Evidence or gap |
|---|---|---|
| GATE-01 | Prior pass; current source pending | Complete 16-job run 33973728459 passed at source 7a5793d and its identical merge tree. Plans 06–07 and the deep-review fixes changed the candidate after that run; the authorized no-push execution means current-source remote CI does not yet exist. |
| GATE-02 | Pass locally | Node production and development audits reported zero findings; Rust reported zero advisories with its recorded yank disposition preserved. |
| GATE-03 | Pass | Frozen Node install, `uv.lock`, Python audit, Python 3.11/3.12 contract/smoke and Linux image imports passed at the recorded source. |
| GATE-04 | Source pass | Railway schema/dry runs and Cloudflare 9d9d ownership plus targeted DNS browser evidence remain recorded. No provider mutation was performed in this review-fix pass. |
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

- `.planning/phases/02-reproducible-gates-dependency-repair/02-REVIEW-FIXES.md` maps all ten Critical and four Warning findings to code, tests, commits and explicit fail-closed dispositions.
- All workflow actions remain immutable-SHA pinned, and every workflow YAML file parses successfully.
- Deploy source admission executes for `main` and canonical semver tags and rejects feature refs or unsupported environments before prebuilds. The final result job cannot pass when authoritative Railway deployment is skipped.
- Provider tests evaluate actual job conditions and execute extracted Docker, Railway, Kustomize, Kubectl, curl and release scripts under recording shims with exact argv/cwd assertions.
- `RAILWAY_TOKEN` is absent from checkout and CLI installation; only credentialed scope/deploy steps receive it.
- `register_database_conditional_engines` is called by both production app-state builders and the focused no-pool/lazy-pool tests.
- Registry provenance validation resolves safe `repo://` paths and supported Markdown/source-symbol anchors; missing, traversing and stale targets fail closed.

## Exact-Source Local Verification

At code source `941e3cfd059831330f7f3adb2f90f526384426c5`:

- `python3 -m pytest tests/scripts/test_validate_release_receipt.py tests/scripts/test_gate_wiring.py tests/scripts/test_validate_contracts.py -q`: 95 passed.
- `cargo test -p noesis-api database_conditional_registration --locked`: 2 passed.
- `python3 scripts/validate_action_pins.py`: exit 0.
- PyYAML parsing of every `.github/workflows/*.{yml,yaml}` file: exit 0.
- `python3 scripts/validate_contracts.py`: `schemas=6 fixtures=5 registries=1 engines=19`.
- `python3 scripts/validate_release_receipt.py --validate-fixtures`: `receipts=2 mutation_cases=9`.
- The current-production negative fixture exits 1 with 29 explicit unavailable facts.
- `env -u DATABASE_URL -u TEST_DATABASE_URL pnpm run gate:scripts`: 140 passed plus contract, receipt, migration and Docker validators.
- `env -u DATABASE_URL -u TEST_DATABASE_URL pnpm run gate`: exit 0 with 140 script, 9 core, 4 OpenAPI, 16 API integration, 35 engine SDK, 11 Noesis SDK, 36 verification and 94 TypeScript tests; all builds/typechecks pass.

## Remaining Gap

Phase 2 does not transition to verified complete until the parent reconciles and pushes this candidate and a full current-source GitHub Actions run passes. GATE-01 therefore remains pending even though all seven execution plans and all deep-review findings have local evidence.

Production promotion remains **HOLD** independently of remote CI. Re-enabling repository-owned production mutation requires durable one-use receipt consumption, Railway-returned deployment/source/status attestation, atomic API/TypeScript coordination, source-bound image asset attestation and atomic multi-registry alias promotion with compensation. Vercel provider-side main deployment protection, exact staging/Kubernetes profiles, real production schema/rollback evidence and per-platform native-binary authority remain later work.
