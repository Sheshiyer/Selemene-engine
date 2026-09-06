---
phase: 02-reproducible-gates-dependency-repair
status: gaps_found
verified: 2026-09-06T12:13:42+00:00
source_commit: 0441d03b8c0fcde4333d2d0a32c48e51939fa054
---
# Phase 2 verification

| Requirement | Status | Evidence or gap |
|---|---|---|
| GATE-01 | Prior pass; current source pending | Complete 16-job run 33973728459 passed at source 7a5793d and its identical merge tree. Plans 06–07 changed the candidate after that run; user-authorized no-push execution means current-source remote CI does not yet exist. |
| GATE-02 | Pass locally | Node production and development audits reported zero findings; Rust reported zero advisories with its yank disposition preserved. |
| GATE-03 | Pass | Frozen Node install, `uv.lock`, Python audit, Python 3.11/3.12 contract/smoke and Linux image imports passed at the recorded source. |
| GATE-04 | Source pass | Railway schema/dry runs and Cloudflare 9d9d ownership plus targeted DNS browser evidence remain recorded. |
| GATE-05 | Pass locally | Plans 06–07 provide the 19-row executable registry plus versioned, offline release receipt authority. Exact source, artifact digests, schema, service project/environment/ID, checks, dependencies, assets and rollback are fail-closed; both workflows retain immutable Action pins and gate all six detected repository-controlled mutations. |
| GATE-06 | Pass | User-approved strict CI Gate remains required by ruleset 15597830 with verified readback. Receipt-gated candidates are reviewable; production promotion remains a separate HOLD. |

## Goal Truths

| Truth | Status | Evidence |
|---|---|---|
| Release eligibility fails closed on missing or mismatched required identity | Verified locally | 49 focused tests, nine mutation cases and the current-production negative fixture; the latter exits 1 with 21 unavailable facts. |
| Built and deployed artifact identity remain separate | Verified locally | Source-redeploy fixtures require deployed source but permit justified non-applicable image identity; immutable mode requires built, deployed and registry-read digests to match. |
| Verification uses fixtures, mocks or disposable/local resources | Verified | No workflow, deploy, release, tag, provider write, production secret, database or schema/data mutation ran. Plan sync ran in dry-run mode only. |

## Artifact and Wiring Verification

- `gsd-sdk query verify.artifacts .../02-07-PLAN.md`: 2/2 artifacts pass.
- `gsd-sdk query verify.key-links .../02-07-PLAN.md`: 1/1 workflow-to-validator link passes.
- Both immutable Action-pin checks pass.
- Static workflow analysis finds six repository-controlled mutation jobs. Every one depends on receipt validation; deploy image pushes also follow a local digest comparison in the same job.
- Mocked workflow execution sends no mutation-spy calls for missing, synthetic, wrong-source, wrong-target, wrong-service or wrong-digest receipts. Eligible synthetic policy fixtures reach only the declared mocked mutation graph.

## Exact-Source Local Verification

At source `0441d03b8c0fcde4333d2d0a32c48e51939fa054`:

- `python3 -m pytest tests/scripts/test_validate_release_receipt.py tests/scripts/test_gate_wiring.py -q`: 49 passed.
- `env -u DATABASE_URL -u TEST_DATABASE_URL pnpm run gate:scripts`: 117 passed plus contract, receipt, migration and Docker validators.
- `env -u DATABASE_URL -u TEST_DATABASE_URL pnpm run gate`: exit 0 with 117 script, 9 core, 4 OpenAPI, 16 API integration, 35 engine SDK, 11 Noesis SDK, 36 verification and 94 TypeScript tests; all builds/typechecks pass.
- Independent adversarial review first returned BLOCK, the findings were remediated, and final review returned GO with no internal blockers or warnings.

## Remaining Gap

Phase 2 does not transition to verified complete until the parent reconciles and pushes this candidate and a full current-source GitHub Actions run passes. GATE-01 therefore remains unchecked even though all seven execution plans have summaries.

Production promotion remains HOLD independently of that source gate. Actual production API source/build identity, applied schema, asset inclusion/retention, exercised rollback and Vercel provider-side deployment protection remain Phase 7 evidence or decision work. Staging, Kubernetes and native-binary publication also remain fail-closed until exact authority is added.
