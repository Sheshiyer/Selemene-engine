---
phase: 02-reproducible-gates-dependency-repair
plan: 07
subsystem: release-engineering
tags: [release-receipt, github-actions, railway, ghcr, fail-closed]

requires:
  - phase: 02-reproducible-gates-dependency-repair
    plan: 06
    provides: versioned 19-row engine registry and executable runtime drift gates
provides:
  - versioned pre-mutation receipt bound to source, semver tag and short-lived operation identity
  - exact artifact/deployment roles, role-keyed rollback, computed asset trees and repository provenance
  - fail-closed workflows that expose immutable candidates only and disable unprovable production mutation
affects: [phase-03-capability-contract-closure, phase-07-deployment-operational-proof]

tech-stack:
  added: []
  patterns:
    - pre-mutation authorization and provider post-deploy attestation are separate evidence classes
    - unavailable one-use, atomicity or image-inclusion proof disables the affected mutation profile
    - provider scripts are executed under recording shims with exact condition, argv and cwd assertions

key-files:
  created:
    - contracts/release/v1/receipt.schema.json
    - contracts/release/v1/manifest.json
    - contracts/release/v1/fixtures/eligible-source-redeploy.json
    - contracts/release/v1/fixtures/current-production-incomplete.json
    - contracts/release/v1/fixtures/mutation-cases.json
    - scripts/validate_release_receipt.py
    - tests/scripts/test_validate_release_receipt.py
    - .planning/phases/02-reproducible-gates-dependency-repair/02-REVIEW-FIXES.md
  modified:
    - .github/workflows/deploy.yaml
    - .github/workflows/release.yml
    - contracts/v1/registries/engines.json
    - scripts/validate_contracts.py
    - crates/noesis-api/src/lib.rs
    - tests/scripts/test_gate_wiring.py

key-decisions:
  - Production receipts require canonical release identity, bounded time and exact run-attempt operation identity.
  - Durable one-use consumption, provider attestation and atomic multi-artifact guarantees cannot be self-asserted by stateless CI.
  - Deploy publication is limited to immutable source candidates; release alias and GitHub-release mutation is absent.
  - API and TypeScript are deployed roles; biofield CV is topology-only for this authority.
  - Vercel native main deployment remains outside the repository gate, so production promotion stays HOLD.

patterns-established:
  - Evidence boundary: synthetic fixtures prove policy behavior but never satisfy operational validation.
  - Safe disposition: retain a failing workflow/profile until its named external authority can be observed.

requirements-completed: [GATE-05]
requirements-progressed: [GATE-01]

duration: 69min
completed: 2026-09-06
---

# Phase 02 Plan 07: Release Receipt Authority Summary

**Release eligibility is now exact and fail-closed locally. Repository workflows cannot perform production promotion because the remaining one-use, provider-attestation, atomicity and image-inclusion guarantees are not yet provable.**

## Source and Evidence Boundary

Plan 02-07 began from `cfeb34a69bd47c486ee8c9487f255a41691330c9`. Its initial implementation and audit remediation ended at `0441d03`; the committed deep review at `5846fd6` then identified ten Critical and four Warning defects. The review-fix code validated here ends at `941e3cfd059831330f7f3adb2f90f526384426c5`.

All work used repository fixtures, local builds, lazy/disposable resources or command shims. No workflow, deployment, publication, release, tag, provider, cloud/DNS, GitHub setting, schema/data, production database or secret mutation ran.

## Final Behavior

- Receipt v1 binds canonical release tag, source revision, workflow, run/attempt operation ID, issued/expiry window, exact built artifacts, service roles, required checks, dependencies, computed asset trees and role-keyed rollback.
- Pre-mutation authorization rejects candidate deployment IDs. Railway-returned deployment ID, source and status must be captured and checked as separate post-deploy attestation.
- API and TypeScript engines are the exact repository-owned deployment roles, with distinct roots, Railway configs, selectors and health authorities. Biofield CV is encoded as topology-only.
- Deploy source admission rejects unsupported refs and environments before prebuild. A final result job fails whenever the authoritative deployment is skipped or unsuccessful.
- Deploy image publication contains only immutable `sha-$GITHUB_SHA` candidates. Mutable branch, `latest` and semver aliases are absent.
- Release permissions are read-only, runs are serialized and the workflow ends in an explicit failure. Alias and GitHub-release writes remain absent until atomic multi-registry promotion with compensation exists.
- Required ephemeris and wisdom assets have canonical repository paths, deterministic `sha256-tree-v1` digests/file counts, artifact roles, build recipe and container paths. Operational authorization still requires source-bound post-build inclusion attestation.
- Health origins, paths, expected status and source-revision fields come from manifest authority; both deploy roles must report the triggering revision.
- Railway CLI installation runs without credentials. `RAILWAY_TOKEN` exists only for the scope and deployment steps.
- Registry provenance validation rejects absolute, traversing, missing and unsupported `repo://` targets and verifies Markdown anchors or source symbols.
- Both production app-state builders and focused tests share the database-conditional registration seam; the tests use no pool and a lazy local pool.

## Atomic Commits

Initial Plan 02-07 commits:

1. `2541aca` — define and falsify release eligibility.
2. `83440c3` — place receipt validation before release mutations.
3. `0441d03` — bind the initial gates to exact targets.

Deep-review remediation commits:

1. `a28a983` — WR-02 exercise production conditional registration.
2. `c6e6a0f` — WR-03 validate registry provenance targets.
3. `8069ca9` — CR-01 bind receipts to release tags.
4. `952b291` — CR-02 expire and bind release operations.
5. `5839e96` — CR-03 separate predeploy authorization evidence.
6. `db15032` — CR-05 model every deployment service role.
7. `97a2a5a` — CR-06 bind rollback evidence by role.
8. `b5295c7` — CR-07 publish immutable deploy candidates only.
9. `0f22e90` — CR-08 disable non-atomic release promotion.
10. `507e7d1` — CR-04 reject unsupported deployment dispatches.
11. `89d1849` — CR-09 bind health checks to service authority.
12. `ddfe062` — CR-10 bind required assets to computed trees.
13. `c67635e` — WR-04 scope Railway token to credentialed steps.
14. `2ba59c4` — WR-01 execute provider scripts under shims.
15. `941e3cf` — WR-01 isolate concurrent shim logs.

Finding-by-finding code, tests and fail-closed dispositions are recorded in `02-REVIEW-FIXES.md`.

## Verification Evidence

| Command or probe | Result |
|---|---|
| `python3 -m pytest tests/scripts/test_validate_release_receipt.py tests/scripts/test_gate_wiring.py tests/scripts/test_validate_contracts.py -q` | 95 passed |
| `cargo test -p noesis-api database_conditional_registration --locked` | 2 passed |
| `python3 scripts/validate_action_pins.py` | Passed |
| Parse every `.github/workflows/*.{yml,yaml}` with PyYAML | Passed |
| `python3 scripts/validate_contracts.py` | `schemas=6 fixtures=5 registries=1 engines=19` |
| `python3 scripts/validate_release_receipt.py --validate-fixtures` | `receipts=2 mutation_cases=9` |
| Current-production negative fixture | Exit 1 with 29 unavailable facts |
| `env -u DATABASE_URL -u TEST_DATABASE_URL pnpm run gate:scripts` | 140 passed; contract, receipt, migration and Docker validators passed |
| `env -u DATABASE_URL -u TEST_DATABASE_URL pnpm run gate` | Exit 0: 140 script, 9 core, 4 OpenAPI, 16 API integration, 35 engine SDK, 11 Noesis SDK, 36 verification and 94 TypeScript tests; all builds/typechecks passed |
| `git diff --check` | Passed |

## Review Outcome

All CR-01 through CR-10 and WR-01 through WR-04 have tested code fixes or an explicit disabled mutation disposition. The authoritative review artifact remains unchanged as the record of what was found; `02-REVIEW-FIXES.md` records what closed each finding.

## Production Mutation Hold

`deploy-production` and `release-production` both declare `mutation_policy.status: disabled`. The validator refuses operational outputs or authorization before any repository-controlled provider write.

Re-enabling production requires observed and reviewed authority for:

- durable one-use receipt consumption;
- Railway-returned deployment ID, source and status for API and TypeScript;
- atomic API/TypeScript service coordination;
- source-bound image asset inclusion attestation;
- atomic multi-registry alias promotion with rollback compensation;
- real production schema and rollback evidence;
- provider-side protection for Vercel native main deployment.

Staging, Kubernetes and native-binary publication also remain fail-closed until exact target or per-platform artifact authority is added.

## User Setup Required

No setup was performed. Supplying a repository variable alone cannot enable either disabled production profile.

## Remaining Phase Gap

Phase verification remains `gaps_found` because no full current-source GitHub Actions run exists for the review-fixed candidate. Production promotion remains **HOLD** independently of that remote CI gap.

## Self-Check: PASSED

- All fourteen review findings appear once in `02-REVIEW-FIXES.md`.
- All fifteen remediation commits are present in repository history.
- Focused, script and full local gates pass with database variables removed.
- Production profiles are disabled and release mutation is absent.
- No external or production mutation was performed.

---
*Phase: 02-reproducible-gates-dependency-repair*
*Completed: 2026-09-06*
