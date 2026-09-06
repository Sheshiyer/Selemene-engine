---
phase: 02-reproducible-gates-dependency-repair
plan: "07"
subsystem: release-engineering
tags: [release-receipt, github-actions, railway, ghcr, fail-closed]

requires:
  - phase: 02-reproducible-gates-dependency-repair
    plan: "06"
    provides: versioned 19-row engine registry and executable runtime drift gates
provides:
  - versioned source-bound release receipt schema with explicit unavailable evidence
  - offline fail-closed operational eligibility validator and negative fixtures
  - mutation-ordered deployment and publication workflows bound to exact digests and targets
affects: [phase-03-capability-contract-closure, phase-07-deployment-operational-proof]

tech-stack:
  added: []
  patterns:
    - operational receipts bind source, target profile, artifact digests, schema, checks, assets and rollback
    - local image bytes are digest-checked before registry publication
    - provider selectors are emitted only after receipt eligibility succeeds

key-files:
  created:
    - contracts/release/v1/receipt.schema.json
    - contracts/release/v1/manifest.json
    - contracts/release/v1/fixtures/eligible-source-redeploy.json
    - contracts/release/v1/fixtures/current-production-incomplete.json
    - contracts/release/v1/fixtures/mutation-cases.json
    - scripts/validate_release_receipt.py
    - tests/scripts/test_validate_release_receipt.py
  modified:
    - .github/workflows/deploy.yaml
    - .github/workflows/release.yml
    - tests/scripts/test_gate_wiring.py
    - package.json

key-decisions:
  - "Operational receipts reject marked test fixtures and require an exact source, workflow target profile and every required artifact digest."
  - "Main deployment and semver publication use distinct receipt variables; release.yml is the only semver tag owner."
  - "Railway project, environment and service selectors come from validated receipt and manifest authority, with project-token scope checked before deployment."
  - "Release receipt v1 covers container images only; native binary publication stays disabled until per-platform digest authority exists."
  - "Vercel native main deployment remains outside the repository gate, so production promotion stays HOLD."

patterns-established:
  - "Mutation-order gate: prebuild without publication, validate the receipt, compare the same locally loaded bytes, then publish."
  - "Evidence boundary: synthetic fixtures can prove policy behavior but can never satisfy operational validation."

requirements-completed: [GATE-05]
requirements-progressed: [GATE-01]

duration: 69min
completed: 2026-09-06
---

# Phase 02 Plan 07: Release Receipt Authority Summary

**Versioned release receipts now reject incomplete production evidence and bind every repository-controlled deployment or publication mutation to exact source, artifact, schema, service, check, asset and rollback identity.**

## Performance

- **Duration:** 69 min
- **Started:** 2026-09-06T11:05:25Z
- **Completed:** 2026-09-06T12:14:51Z
- **Tasks:** 2
- **Files modified:** 17 including execution evidence and state bookkeeping

## Source Precondition

Execution began from `cfeb34a69bd47c486ee8c9487f255a41691330c9`, the committed Plan 02-06 result with 19 runtime IDs, 17 public groups, 12 native / 1 database-conditional / 6 TypeScript registrations and passing registry drift gates. Production promotion was HOLD and remained HOLD throughout Plan 02-07.

## Accomplishments

- Added release receipt v1 authority for source revision, separate built and deployed artifacts, schema identity, exact Railway project/environment/service roles, required CI checks, dependency state, assets and rollback evidence.
- Added realistic synthetic eligible and mutation fixtures plus an incomplete current-production snapshot that fails with 21 explicit unavailable facts; no unknown source, schema, asset or rollback value was guessed.
- Made operational validation reject marked fixtures and require the exact source revision, target profile, provider scope and digest of every required container artifact.
- Rewired main deployment so nonpublishing prebuilds feed receipt eligibility, locally loaded images are checked against those authorized digests before `docker push`, and Railway token scope plus all three exact selectors are checked before `railway up`.
- Made `release.yml` the sole semver tag owner, resolve existing source-tag images by digest, validate them, and promote the exact registry manifests without rebuilding. Removed unbound main changelog writes and native binary publication outside v1 artifact authority.
- Added static graph and mocked execution proofs: missing, synthetic, wrong-source, wrong-target, wrong-service and wrong-digest receipts make zero mocked mutation calls; eligible synthetic policy fixtures reach only the declared mocked mutation graph.

## Task Commits

Each task was committed atomically, followed by a bounded audit-remediation commit:

1. **Task 1: Define and falsify release eligibility** — `2541aca` (feat)
2. **Task 2: Place receipt validation before release mutations** — `83440c3` (fix)
3. **Independent audit remediation for Tasks 1–2** — `0441d03` (fix)

## Files Created/Modified

- `contracts/release/v1/receipt.schema.json` — Draft 2020-12 receipt contract with constrained timestamps, schema/rollback states and project/service evidence.
- `contracts/release/v1/manifest.json` — Registry, migration, artifact, workflow-target and exact Railway service authority.
- `contracts/release/v1/fixtures/*.json` — Eligible synthetic, current-production incomplete and nine negative mutation cases.
- `scripts/validate_release_receipt.py` — Offline schema, authority, operational target/digest and evidence validator with post-eligibility GitHub outputs.
- `tests/scripts/test_validate_release_receipt.py` — Positive, negative, operational and target-output tests.
- `tests/scripts/test_gate_wiring.py` — Static workflow graph, step ordering and executable mocked mutation-spy tests.
- `.github/workflows/deploy.yaml` — Main-only receipt-gated local-build publication and exact Railway source deployment.
- `.github/workflows/release.yml` — Sole tag-triggered, receipt-gated immutable image promotion and GitHub release.
- `package.json` — Adds release fixture validation to `gate:scripts`.
- `ISA.md`, `02-VERIFICATION.md`, `STATE.md`, `ROADMAP.md`, `REQUIREMENTS.md` and this summary — Evidence and execution bookkeeping.

## Verification Evidence

All validation used repository fixtures, mocks or local builds. Database-sensitive commands ran with `DATABASE_URL` and `TEST_DATABASE_URL` removed. No workflow, tag, release, deployment, provider mutation or production secret was used.

| Command or review | Result |
|---|---|
| `python3 -m pytest tests/scripts/test_validate_release_receipt.py tests/scripts/test_gate_wiring.py -q` | 49 passed |
| `python3 scripts/validate_release_receipt.py --validate-fixtures` | `receipts=2 mutation_cases=9` |
| `python3 scripts/validate_release_receipt.py contracts/release/v1/fixtures/current-production-incomplete.json` | Exit 1 with 21 unavailable source/build, schema, asset and rollback facts |
| `python3 scripts/validate_action_pins.py --path .github/workflows/deploy.yaml` | Exit 0 |
| `python3 scripts/validate_action_pins.py --path .github/workflows/release.yml` | Exit 0 |
| `env -u DATABASE_URL -u TEST_DATABASE_URL pnpm run gate:scripts` | 117 passed; contract, receipt, migration and Docker validators pass |
| `env -u DATABASE_URL -u TEST_DATABASE_URL pnpm run gate` | Exit 0: 117 script, 9 core, 4 OpenAPI, 16 API integration, 35 engine SDK, 11 Noesis SDK, 36 verification and 94 TypeScript tests; all builds/typechecks pass |
| `gsd-sdk query verify.artifacts .../02-07-PLAN.md` | 2/2 artifacts pass |
| `gsd-sdk query verify.key-links .../02-07-PLAN.md` | 1/1 key link verified |
| `scripts/sync-plans-to-github-issues.sh --repo Sheshiyer/Selemene-engine` | Dry-run; no matching plan JSON and no issue mutation |
| Independent adversarial review | Initial BLOCK exposed six ordering/authority defects; final exact-source verdict GO with no internal blockers or warnings |
| `git diff --check` | Exit 0 |

## Decisions Made

- Source redeployment and immutable image promotion remain different receipt modes. A source deployment cannot claim that its separately built image was deployed.
- `DEPLOY_RELEASE_RECEIPT_B64` and `PUBLISH_RELEASE_RECEIPT_B64` are intentionally separate, and neither accepts a receipt containing `test_fixture` metadata.
- Main deploy prebuilds do not publish or upload build records. After receipt validation, the rebuilt image is loaded locally, its digest is compared before mutation, and those same tags are pushed.
- Railway CLI is pinned to `5.41.0`. The project token is read-only queried for project/environment scope, while manifest-bound project, environment and service IDs are passed explicitly to the deployment command.
- Semver publication reads the exact `sha-$GITHUB_SHA` registry digests and promotes `repository@digest` inputs. It does not rebuild images or write back to mutable `main`.
- Receipt v1 explicitly excludes native binaries. Restoring their publication requires a later authority version that records each platform build and digest before release mutation.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Loaded the standalone validator directly in tests**
- **Found during:** Task 1
- **Issue:** `scripts/` is not a Python package, so negative fixture tests could not import mutation helpers through a package path.
- **Fix:** Loaded the validator by exact file path with `importlib.util` while keeping the production script standalone and offline.
- **Files modified:** `tests/scripts/test_validate_release_receipt.py`
- **Committed in:** `2541aca`

**2. [Rule 1 - Bug] Corrected a negative-mode test expectation**
- **Found during:** Task 1 verification
- **Issue:** One mutation exercised the validator's required workflow mode rather than the fixture mode expected by the assertion.
- **Fix:** Bound the assertion to the actual fail-closed promotion-mode error.
- **Files modified:** `tests/scripts/test_validate_release_receipt.py`
- **Committed in:** `2541aca`

**3. [Rule 2 - Missing critical functionality] Closed independent audit findings before mutation**
- **Found during:** Task 2 independent review
- **Issue:** The first implementation could accept marked synthetic receipts operationally, did not bind exact provider/service targets or actual artifacts, gave two workflows the same tag/release ownership, checked rebuilt image drift only after push, allowed a mutable Railway service selector and retained an unbound mutable-main changelog write.
- **Fix:** Added operational source/fixture/profile/digest enforcement, exact project/environment/service authority, pre-push local digest checks, manifest-derived Railway outputs and token-scope readback; consolidated tag ownership and removed the changelog write.
- **Files modified:** Release manifest/schema/fixtures, validator/tests and both workflows.
- **Committed in:** `0441d03`

**4. [Rule 2 - Missing critical functionality] Constrained rollback identity and publication scope**
- **Found during:** Task 2 independent review
- **Issue:** Free-form schema and rollback strings could satisfy eligibility, and native binaries could be uploaded without per-platform receipt identity.
- **Fix:** Added constrained migration, compatibility, restore, runbook and UTC timestamp evidence. Declared receipt v1 container-only and disabled native publication pending exact digest authority.
- **Files modified:** Release manifest/schema/fixtures, validator/tests and release workflow.
- **Committed in:** `0441d03`

**5. [Rule 3 - Blocking] Added native UTC validation and pinned Railway CLI**
- **Found during:** Task 2 remediation
- **Issue:** The installed jsonschema format checker did not reject malformed date-time strings, and a floating Railway CLI could change selector semantics.
- **Fix:** Added Python UTC parsing after schema validation and pinned the reviewed Railway CLI at `5.41.0` with a static regression test.
- **Files modified:** `scripts/validate_release_receipt.py`, `tests/scripts/test_validate_release_receipt.py`, `.github/workflows/deploy.yaml`, `tests/scripts/test_gate_wiring.py`
- **Committed in:** `0441d03`

---

**Total deviations:** 5 auto-fixed (1 bug, 2 blocking, 2 missing critical functionality)

**Impact on plan:** Each change strengthens the stated fail-closed and pre-mutation contract. No additional production surface was activated.

## Issues Encountered

- The first independent review returned BLOCK despite green tests. Its adversarial probes identified target, artifact and trigger gaps; all were reproduced, fixed and re-reviewed to GO.
- `gsd-sdk query verify.decisions` is unavailable in the installed SDK and its compatibility fallback. The plan contains no checkpoint decision block, so artifact and key-link verification were run directly and this missing optional subcommand did not reduce evidence.

## Known Stubs

None. The `unavailable` values in `current-production-incomplete.json` are deliberate negative evidence and are required to keep that production snapshot ineligible.

## Threat Flags

| Flag | File | Description |
|---|---|---|
| threat_flag: repository-variable-input | `.github/workflows/deploy.yaml`, `.github/workflows/release.yml` | Base64 receipt input is decoded to a private temporary file and rejected unless strict offline schema and authority checks pass. |
| threat_flag: provider-token-scope-read | `.github/workflows/deploy.yaml` | A Railway project token is used only after receipt validation for a read-only project/environment scope query, then an exact manifest-bound deployment. |
| threat_flag: registry-tag-mutation | `.github/workflows/deploy.yaml`, `.github/workflows/release.yml` | GHCR tags move only after exact digest eligibility; deploy pushes already-checked loaded bytes and release promotes registry-read `repository@digest` manifests. |

## User Setup Required

No setup was performed. A future authorized deployment or publication requires a reviewed operational receipt in the appropriate repository variable and the already-defined provider credentials.

## Remaining Critical Production Decisions

- Decide and apply provider-side Vercel deployment protection, or disable native production auto-deploy, so a push to `main` cannot bypass receipt authority.
- Establish the real production API source/build identity, applied migration revision, asset integrity/retention/inclusion and an exercised rollback target/procedure/timestamp. The current snapshot remains ineligible until all are observed.
- Review and set exact-source `DEPLOY_RELEASE_RECEIPT_B64` or `PUBLISH_RELEASE_RECEIPT_B64` only for a concrete candidate and its actual prebuilt or registry-read digests.
- Prove on a current-source remote run that BuildKit's locally checked digest is preserved by GHCR publication and that Railway's token-scope query plus exact selector flags behave as designed.
- Add exact staging and Kubernetes target profiles before enabling either path; current workflow inputs fail closed because those authorities are absent.
- Add a later receipt version with per-platform binary build/digest identity before restoring native binary uploads.
- Run current-source GitHub CI after parent reconciliation and push. Until that evidence exists, GATE-01 and Phase 2 verification remain open even though all seven plans are executed.

## Self-Check: PASSED

- All seven created release authority, fixture, validator and test files exist.
- Task commits `2541aca`, `83440c3` and `0441d03` are present in repository history.
- ISA progress recomputes to 331 checked criteria out of 335 total.
- GSD artifact and key-link checks pass; documentation diff passes `git diff --check`.

---
*Phase: 02-reproducible-gates-dependency-repair*
*Completed: 2026-09-06*
