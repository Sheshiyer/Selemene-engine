---
phase: 02-reproducible-gates-dependency-repair
reviewed: 2026-09-06T14:38:09Z
depth: deep
baseline: 5846fd6
head: 2bcaf77f24fe08ac5ad959560b3dba421b77285d
files_reviewed: 30
files_reviewed_list:
  - .github/workflows/deploy.yaml
  - .github/workflows/release.yml
  - .github/workflows/test.yml
  - apps/admin-web/next.config.mjs
  - contracts/release/v1/fixtures/current-production-incomplete.json
  - contracts/release/v1/fixtures/eligible-source-redeploy.json
  - contracts/release/v1/fixtures/mutation-cases.json
  - contracts/release/v1/manifest.json
  - contracts/release/v1/receipt.schema.json
  - contracts/v1/manifest.json
  - contracts/v1/registries/engines.json
  - crates/noesis-api/src/handlers/admin.rs
  - crates/noesis-api/src/lib.rs
  - crates/noesis-api/tests/capability_route_tests.rs
  - crates/noesis-core/tests/contract_v1_authority.rs
  - crates/noesis-orchestrator/src/lib.rs
  - docs/baseline/api-route-inventory.json
  - package.json
  - python-services/Dockerfile.mediapipe
  - requirements-gates.txt
  - scripts/validate_contracts.py
  - scripts/validate_release_receipt.py
  - tests/scripts/test_gate_wiring.py
  - tests/scripts/test_validate_contracts.py
  - tests/scripts/test_validate_release_receipt.py
  - ts-engines/src/index.ts
  - ts-engines/src/server/__tests__/registry-authority.test.ts
  - ts-engines/src/server/app.ts
  - ts-engines/src/server/index.ts
  - ts-engines/src/server/registry.ts
findings:
  critical: 0
  warning: 1
  info: 0
  total: 1
status: issues_found
production_promotion: HOLD
---

# Phase 02: Deep Code Review Recheck

**Reviewed:** 2026-09-06T14:38:09Z
**Depth:** deep
**Review range:** complete Phase 02 source scope, with focused review of `5846fd6..2bcaf77f24fe08ac5ad959560b3dba421b77285d`
**Files Reviewed:** 30
**Status:** issues_found
**Production promotion:** HOLD

## Summary

The recheck independently traced the final receipt validator, contract authority, workflow job graph, provider scripts, runtime registration seams, and the tests added after review baseline `5846fd6`. Thirteen prior findings are fully resolved or deliberately fail closed. WR-03 is only partially resolved: `repo://` paths are contained within the repository, but a source anchor is accepted when its name occurs only in a comment. This leaves one Warning and no Critical issues.

Both production mutation profiles remain explicitly disabled in `contracts/release/v1/manifest.json:90-131`, and both workflow callers use operational validation. `scripts/validate_release_receipt.py:239-263` therefore rejects every current production authorization before emitting provider selectors or allowing a mutating job to run. The repository is not production-promotable from these workflows; that fail-closed HOLD is the intended final behavior.

The existing review and fix ledger were read before this recheck:

- `.planning/phases/02-reproducible-gates-dependency-repair/02-REVIEW.md`
- `.planning/phases/02-reproducible-gates-dependency-repair/02-REVIEW-FIXES.md`

## Narrative Findings (AI reviewer)

## Warnings

### WR-R01: Source evidence anchors accept comment-only symbol names

**Classification:** WARNING
**File:** `/Volumes/madara/2026/Projects/thoughtseed/.superset-worktrees/b7b13f10-dbc6-4dc5-b24a-4b03607ae607/codex/selemene-phase-recovery-20260905/scripts/validate_contracts.py:185-193`
**Related test gap:** `/Volumes/madara/2026/Projects/thoughtseed/.superset-worktrees/b7b13f10-dbc6-4dc5-b24a-4b03607ae607/codex/selemene-phase-recovery-20260905/tests/scripts/test_validate_contracts.py:387-400`

**Issue:** Non-Markdown anchors are validated by searching for each anchor component as a word anywhere in the source text. Comments and string literals count as matches. A temporary source file containing only `// registerTypeScriptRuntimeEngines removed from production` was accepted for `repo://comment-only.ts#registerTypeScriptRuntimeEngines`. A stale evidence row can therefore remain contract-valid after the referenced declaration or implementation has been removed, as long as its old name survives in prose. The current negative test proves rejection only when the token is completely absent.

**Fix:** Resolve source anchors against declarations or definitions rather than raw text. Use language-aware parsing for the supported Rust, TypeScript, JavaScript, and Python suffixes, or a narrowly defined declaration matcher that strips comments and strings before matching. For qualified Rust anchors such as `BridgeManager::new`, verify the implementation/type context as well as the terminal name. Add negative fixtures for a comment-only token, a string-literal-only token, and a token declared under the wrong qualifier.

## Prior Finding Disposition Recheck

| Prior finding | Disposition | Direct evidence |
|---|---|---|
| CR-01 — release/tag binding | VERIFIED | `contracts/release/v1/receipt.schema.json:62-71` requires a canonical release tag; `scripts/validate_release_receipt.py:280-302` binds it to promotion mode and expected tag; `.github/workflows/release.yml:108-123` supplies `GITHUB_REF_NAME`. |
| CR-02 — freshness and replay binding | VERIFIED, FAIL-CLOSED | `contracts/release/v1/manifest.json:17-22` declares age/skew and one-use authority; `scripts/validate_release_receipt.py:170-204` validates clock, run ID, attempt, and workflow. Durable one-use consumption remains unavailable, so both production profiles are disabled at `contracts/release/v1/manifest.json:93-123` and operational use is denied at `scripts/validate_release_receipt.py:257-263`. |
| CR-03 — pre/post provider identity | VERIFIED, FAIL-CLOSED | Receipt evidence is pre-mutation and no future provider deployment ID is trusted. The unavailable provider-returned post-deploy attestation is named as required authority at `contracts/release/v1/manifest.json:93-101`; operational validation denies the profile at `scripts/validate_release_receipt.py:257-263`. |
| CR-04 — unsupported refs and false-green skips | VERIFIED | `.github/workflows/deploy.yaml:39-58` rejects non-production environments and non-main/noncanonical-version refs; `.github/workflows/deploy.yaml:687-724` requires source, receipt, both builds, Railway, and both smoke jobs to succeed. |
| CR-05 — complete multi-service target | VERIFIED, FAIL-CLOSED | `contracts/release/v1/manifest.json:23-88` declares artifact and service roles; `scripts/validate_release_receipt.py:375-470` checks role classifications and target authority; `.github/workflows/deploy.yaml:545-564` invokes both API and TypeScript Railway services. Atomic coordination is still absent and keeps deployment disabled at `contracts/release/v1/manifest.json:93-101`. |
| CR-06 — role-keyed rollback authority | VERIFIED | `contracts/release/v1/receipt.schema.json:447-482` defines role-keyed artifact and service rollback records; `scripts/validate_release_receipt.py:672-759` requires every mutated role, exact repository/provider/project/environment/service identity, fresh rehearsal evidence, and a previous source distinct from the candidate. |
| CR-07 — mutable deploy tags | VERIFIED | Prebuilds are unpublished at `.github/workflows/deploy.yaml:60-90,105-133`; mutating builds publish only `sha-<exact source>` tags at `.github/workflows/deploy.yaml:253-297,337-378`. No `latest`, branch, or semver alias is written by this workflow. |
| CR-08 — partial release promotion | VERIFIED, FAIL-CLOSED | `.github/workflows/release.yml:52-70` only resolves immutable source-tag digests, and `.github/workflows/release.yml:125-141` has read-only permissions and ends in an unconditional production hold. There is no release/tag/alias mutation job. |
| CR-09 — health/source binding | VERIFIED, FAIL-CLOSED | Manifest-owned origins, paths, expected status, and source fields are emitted only after successful validation at `scripts/validate_release_receipt.py:870-926`; `.github/workflows/deploy.yaml:566-598` requires both responses to report the exact `GITHUB_SHA`. The API maps `/health/live` to a response that lacks `source_revision` and reports `ok` (`crates/noesis-api/src/lib.rs:1118,1144-1151,1952-1963`), while the TypeScript response lacks `source_revision` (`ts-engines/src/server/app.ts:83-90`). Thus the proposed check fails closed until real source-bound provider attestation is implemented, consistent with the disabled profile. |
| CR-10 — asset integrity and inclusion | VERIFIED, FAIL-CLOSED | `scripts/validate_release_receipt.py:555-655` recomputes repository-tree digests, verifies file counts and build recipes, and rejects operational inclusion without source-bound post-build attestation. The missing attestation remains named in both disabled profiles at `contracts/release/v1/manifest.json:93-123`. |
| WR-01 — provider commands not executed by tests | VERIFIED | `tests/scripts/test_gate_wiring.py:769-842` executes the extracted two-service Railway script against a recording shim; `tests/scripts/test_gate_wiring.py:845-933` executes the Docker publication and Kubernetes scripts; `tests/scripts/test_gate_wiring.py:936-1044` executes source-health and token-scope scripts and asserts exact arguments. |
| WR-02 — database-conditional test seam bypass | VERIFIED | `crates/noesis-api/src/lib.rs:3973-3986` centralizes conditional registration; production builders invoke it at `crates/noesis-api/src/lib.rs:4084-4086,4221-4223`; tests exercise the same helper at `crates/noesis-api/src/lib.rs:4361-4400`. |
| WR-03 — repository provenance validation | PARTIAL; WR-R01 remains | `scripts/validate_contracts.py:133-170` safely constrains and resolves repository paths and `scripts/validate_contracts.py:172-183` validates Markdown headings. Source anchors at `scripts/validate_contracts.py:185-193` only require a raw word occurrence, which the adversarial comment-only probe bypassed. |
| WR-04 — Railway secret scope | VERIFIED | `RAILWAY_TOKEN` is absent from job-level environment and from the CLI installation step at `.github/workflows/deploy.yaml:466-495`; it is supplied only to configuration, scope-check, and provider-mutation steps at `.github/workflows/deploy.yaml:502-547`. |

## Authorization and Mutation Boundary

- The deploy workflow calls the validator with the exact source SHA, run ID/attempt, workflow, current UTC clock, promotion mode, target profile, both prebuild digests, `--github-output`, and `--operational` at `.github/workflows/deploy.yaml:201-225`.
- The release workflow supplies the equivalent immutable-image inputs, including the exact triggering tag, at `.github/workflows/release.yml:108-123`.
- Operationally disabled profiles always produce validation errors before `emit_github_outputs` can run (`scripts/validate_release_receipt.py:239-263,1029-1042`). Mutating deploy jobs all depend on that validation job. Production provider commands are therefore unreachable in the current profile state.
- This recheck performed no provider login, provider mutation, push, merge, deployment, release, tag, or external state change.

## Verification Performed

- `env -u DATABASE_URL -u TEST_DATABASE_URL python3 -m pytest tests/scripts/test_validate_release_receipt.py tests/scripts/test_gate_wiring.py tests/scripts/test_validate_contracts.py -q` — **95 passed**.
- `env -u DATABASE_URL -u TEST_DATABASE_URL cargo test -p noesis-api database_conditional_registration --locked` — **2 passed**, with unrelated tests filtered.
- `pnpm -C ts-engines test` — **94 passed, 0 failed**.
- Installed `railway 5.41.0`: `railway up --help` confirms the workflow's `--project`, `--environment`, `--service`, and `--ci` flags are supported; no provider CLI syntax finding is reported.
- `python3 scripts/validate_action_pins.py` — passed.
- `python3 scripts/validate_contracts.py` — passed (`schemas=6 fixtures=5 registries=1 engines=19`).
- `python3 scripts/validate_release_receipt.py --validate-fixtures` — passed (`receipts=2 mutation_cases=9`).
- `git diff --check 5846fd6..HEAD` — passed.
- Direct TypeScript health probe returned a healthy response containing six registered engines.
- Adversarial repository-reference probe reproduced WR-R01 by accepting a source file whose requested anchor appeared only in a comment.

## Final Counts

- **Critical / BLOCKER:** 0
- **Warning:** 1
- **Info:** 0
- **Total:** 1

---

_Reviewed: 2026-09-06T14:38:09Z_
_Reviewer: gsd-code-reviewer (independent final recheck)_
_Depth: deep_
