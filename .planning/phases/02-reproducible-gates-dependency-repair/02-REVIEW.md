---
phase: 02-reproducible-gates-dependency-repair
reviewed: 2026-09-06T12:44:38Z
depth: deep
files_reviewed: 22
files_reviewed_list:
  - .github/workflows/deploy.yaml
  - .github/workflows/release.yml
  - contracts/release/v1/fixtures/current-production-incomplete.json
  - contracts/release/v1/fixtures/eligible-source-redeploy.json
  - contracts/release/v1/fixtures/mutation-cases.json
  - contracts/release/v1/manifest.json
  - contracts/release/v1/receipt.schema.json
  - contracts/v1/manifest.json
  - contracts/v1/registries/engines.json
  - crates/noesis-core/tests/contract_v1_authority.rs
  - crates/noesis-orchestrator/src/lib.rs
  - package.json
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
  critical: 10
  warning: 4
  info: 0
  total: 14
status: issues_found
---

# Phase 02: Code Review Report

**Reviewed:** 2026-09-06T12:44:38Z
**Depth:** deep
**Files Reviewed:** 22
**Status:** issues_found

## Summary

Plans 06-07 establish useful source-enumeration checks and a release-receipt shape, but the operational gate is not yet an authorization boundary for production mutation. A receipt can be reused indefinitely, does not authorize a release tag, and accepts deployment facts that cannot be known until after the gated mutation. The deploy workflow can also report success without deploying, deploys only one of the service roles whose deployed state the receipt claims, and has several partial-mutation paths without role-complete rollback evidence.

The focused Python suites passed (72 tests), the contract validator accepted the canonical authority, and the receipt fixture suite accepted its two receipts and nine mutation cases. Those green results do not cover the failures below. Direct in-memory calls to the production validator reproduced four bypasses: a receipt dated in 2000 returned no errors, a source-redeploy receipt with a fabricated candidate deployment ID returned no errors, an immutable-image operational receipt with no tag/version field returned no errors, and required asset source/integrity values changed to fabricated values returned no errors. A disposable-copy probe also changed a registry evidence reference to `repo://definitely/missing/file.rs#nonexistent`; the canonical contract validator still exited 0.

Production promotion should remain HOLD.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: Release authorization is not bound to the triggering semantic version

**Classification:** Critical
**File:** `contracts/release/v1/receipt.schema.json:8-22`; `.github/workflows/release.yml:104-115`; `.github/workflows/release.yml:140-162`
**Issue:** The receipt schema has no release tag or version identity, and the workflow validates only source SHA, promotion mode, target profile, and image digests. The tag is read only after validation, when it is used to move the version, minor, major, and `latest` aliases. Consequently, one reviewed receipt for a commit/digest pair authorizes every `vMAJOR.MINOR.PATCH` tag that points at that commit. A tag pusher can publish an unreviewed version and move public aliases without obtaining a receipt for that release identity. The adversarial probe converted the positive fixture to the operational immutable-image profile, left it tagless, and `validate_release_receipt` returned an empty error list.
**Fix:** Add a required canonical `release.tag` (and, if useful, parsed version) to the schema. Add `--expected-release-tag` to the validator, pass `"$GITHUB_REF_NAME"` at lines 108-115, reject noncanonical or mismatched values before `promote-images`, and add a negative test proving a receipt for `v1.2.3` cannot authorize `v1.2.4`.

### CR-02: Operational receipts never expire and can be replayed

**Classification:** Critical
**File:** `scripts/validate_release_receipt.py:91-95`; `scripts/validate_release_receipt.py:114-132`; `scripts/validate_release_receipt.py:441-459`; `.github/workflows/deploy.yaml:134-198`
**Issue:** `generated_at` and `rollback.tested_at` are checked only for timestamp syntax. There is no maximum age, expiry, workflow-run binding, operation nonce, or consumed-receipt check, despite the workflow claiming that stale evidence fails. A repository variable can therefore retain an old receipt and every rerun for the same source/digests can republish tags and start another production deployment. The adversarial probe changed both timestamps to `2000-01-01T00:00:00Z`; operational validation still returned no errors.
**Fix:** Require `issued_at`, `expires_at`, and a deployment/release operation identifier. Enforce an explicit maximum age against a UTC clock supplied to the validator, bind the receipt to the current workflow/run or a one-use nonce, and reject stale rollback rehearsals separately. Add frozen-clock boundary and replay tests.

### CR-03: The pre-mutation gate accepts fabricated post-deployment identity

**Classification:** Critical
**File:** `contracts/release/v1/fixtures/eligible-source-redeploy.json:57-107`; `scripts/validate_release_receipt.py:205-250`; `.github/workflows/deploy.yaml:177-198`; `.github/workflows/deploy.yaml:485-492`
**Issue:** A source-redeploy receipt must claim candidate `deployed.deployment_id` and candidate deployed source for both artifacts before the workflow calls `railway up`. The actual deployment ID cannot describe the upcoming mutation at that point. The validator only checks that the ID is a nonempty string and that the claimed source equals the candidate SHA; it never binds the ID to Railway. Replacing the API deployment ID with `fabricated-before-deployment` and its source with a self-assertion still produced zero operational validation errors. The gate therefore turns unverifiable future facts into eligibility evidence.
**Fix:** Split pre-deploy authorization from post-deploy attestation. The pre-deploy document should bind intended source, artifact digests, exact selectors, and rollback inputs. After `railway up`, capture the provider-returned deployment ID (prefer machine-readable CLI/API output), query its source/status for each target, validate that attestation, and fail the workflow if it cannot be established.

### CR-04: A production workflow dispatch on a non-main ref can finish green without deploying

**Classification:** Critical
**File:** `.github/workflows/deploy.yaml:3-15`; `.github/workflows/deploy.yaml:200-273`; `.github/workflows/deploy.yaml:287-355`; `.github/workflows/deploy.yaml:436-442`; `tests/scripts/test_gate_wiring.py:413-430`
**Issue:** `workflow_dispatch` is available on an arbitrary selected ref. With `environment=production` and a matching receipt, both image jobs run and publish registry tags because they have no ref condition. The Railway and Kubernetes jobs then skip unless the ref is main or a tag. The smoke jobs also skip because their required deploy did not succeed, and skipped jobs do not make the workflow fail. This recreates a green “deployment” that mutated GHCR but shipped nothing. The test named `workflow_dispatch_cannot_bypass_receipt_validation` only inspects the receipt job and never evaluates the ref-dependent job conditions.
**Fix:** Reject unsupported refs in `validate-source` before prebuild/receipt validation, or apply one shared allowed-ref predicate to every mutation job. Add a final required deployment-result job that fails when production was requested but the authoritative deploy job was skipped. Test main, tag, and feature-branch dispatch contexts by evaluating job conditions.

### CR-05: The receipt claims multi-service deployed state but CD deploys only the API service

**Classification:** Critical
**File:** `contracts/release/v1/manifest.json:17-44`; `contracts/release/v1/manifest.json:47-55`; `scripts/validate_release_receipt.py:304-344`; `scripts/validate_release_receipt.py:575-605`; `.github/workflows/deploy.yaml:443-516`
**Issue:** The authority requires API and TypeScript artifacts plus API, TypeScript, and biofield-CV service identities. The operational receipt also requires a deployed source/deployment ID for both artifacts. Despite that scope, `deploy-production` names one singular `deployment_service_role`, output emission selects only that role, and CD invokes `railway up` once for the API service and probes only the API endpoint. A successful run cannot establish or update the TypeScript deployment represented by the receipt, so the six TypeScript registry changes can remain unshipped while CD is green.
**Fix:** Model `deployment_service_roles` as an exact list and emit a selector for every service that this workflow owns. Deploy API and TypeScript from their correct roots/configurations, capture each provider deployment ID, and probe each service. If CV is topology-only for this release mode, encode that distinction instead of claiming a common deployed-artifact state.

### CR-06: Rollback evidence is singular and unbound for a multi-artifact, multi-service release

**Classification:** Critical
**File:** `contracts/release/v1/receipt.schema.json:172-190`; `contracts/release/v1/manifest.json:17-44`; `scripts/validate_release_receipt.py:441-480`; `contracts/release/v1/fixtures/eligible-source-redeploy.json:293-323`
**Issue:** The schema records one previous deployment ID and one previous artifact digest even though the release authority covers two artifacts and three services. The validator only requires those strings/digest to exist and differ from the candidate; it does not bind them to any role, provider target, or actual previous state. The positive fixture supplies only an API-shaped previous deployment. Eligibility can therefore pass with no TypeScript rollback identity, leaving no deterministic recovery for a partially updated TS service or image alias.
**Fix:** Replace the singular fields with role-keyed `rollback.artifacts` and `rollback.services` collections whose sets must exactly match the mutated roles. Bind every prior deployment to project/environment/service, every prior digest to its artifact repository, and validate the rehearsal timestamp/procedure per rollback unit.

### CR-07: Deploy image publication can leave one artifact's mutable tags advanced

**Classification:** Critical
**File:** `.github/workflows/deploy.yaml:226-273`; `.github/workflows/deploy.yaml:313-355`
**Issue:** API and TypeScript image jobs run independently and each pushes all metadata tags, including mutable branch/`latest` aliases on main. If either build, digest comparison, login, or push fails after the other job publishes, CD stops before Railway deployment but the successful repository has already advanced its public tags. The receipt contains only a singular rollback digest and the workflow has no compensation step, so the externally visible artifact set no longer represents one release candidate.
**Fix:** Publish immutable digest/source tags first for both repositories, verify both descriptors exist, then promote mutable aliases in a coordinated job. Record each previous alias digest and restore all aliases on a promotion failure, or leave aliases untouched and expose only the immutable candidate until deployment succeeds.

### CR-08: Release promotion can publish half of a two-image release

**Classification:** Critical
**File:** `.github/workflows/release.yml:119-162`; `.github/workflows/release.yml:164-211`
**Issue:** The release job promotes four API aliases and then four TypeScript aliases. A TS registry failure leaves every API alias moved while the GitHub release is never created; a later GitHub-release failure likewise leaves both repositories promoted without the release record. There is no preflight, serialization, alias snapshot, or compensation. Concurrent or out-of-order tag runs can also race `latest`, major, and minor aliases.
**Fix:** Add a release concurrency group and a monotonic-version guard, preflight write access and both source descriptors, snapshot every current alias digest, and promote aliases only after both immutable version tags are staged. On any later failure, restore the snapshots or create a draft release before alias promotion and finalize it only after the coordinated update.

### CR-09: Post-deploy health is not bound to the Railway target that was mutated

**Classification:** Critical
**File:** `.github/workflows/deploy.yaml:443-492`; `.github/workflows/deploy.yaml:494-516`
**Issue:** Project, environment, and service are manifest-bound for `railway up`, but health verification uses the independent repository variable `API_BASE_URL` (or a hardcoded fallback). The receipt/manifest does not bind that URL to the selected service. If the variable points to any healthy old, staging, or unrelated API, the check returns 200 even when the authorized service is unhealthy. Thus the workflow's final runtime evidence can concern a different target from the mutation.
**Fix:** Put the canonical health origin in the target authority and emit it only after exact validation, or query the deployed service/domain from Railway using the validated IDs. After deployment, verify both the service ID/deployment ID and a source/build marker at that bound endpoint rather than accepting a generic liveness 200.

### CR-10: Required asset integrity is accepted without binding it to any asset

**Classification:** Critical
**File:** `contracts/release/v1/manifest.json:86-92`; `contracts/release/v1/fixtures/eligible-source-redeploy.json:243-290`; `scripts/validate_release_receipt.py:408-439`
**Issue:** The manifest declares only asset IDs and required booleans. For each required asset, the validator checks that source, digest-shaped integrity, retention text, and an inclusion boolean are present, but it never resolves the source, computes or compares a digest, or verifies inclusion in either authorized image. Changing ephemeris source to `claimed://nonexistent-assets`, its digest to an arbitrary valid SHA-256 string, and retention to a self-assertion still produced zero operational errors. The gate therefore does not fail closed on mismatched required asset identity and can authorize images missing or carrying the wrong ephemeris/wisdom payload.
**Fix:** Put canonical asset paths and digest/manifest rules in the authority. Resolve repository asset sources within the checkout, compute deterministic tree/file digests, and verify image inclusion through a source-bound SBOM or post-build inspection attestation. Add wrong-source, wrong-integrity, and absent-from-image operational tests.

## Warnings

### WR-01: “Mocked workflow execution” never executes a mutation command or job condition

**Classification:** Warning
**File:** `tests/scripts/test_gate_wiring.py:312-328`; `tests/scripts/test_gate_wiring.py:331-387`; `tests/scripts/test_gate_wiring.py:646-663`
**Issue:** The helper executes only the receipt materialization and validator shell snippets. It then marks mutation jobs called by traversing `needs`; it does not run their commands, evaluate `if`, propagate step/job outputs, or model failures. Removing `railway up`, breaking its arguments, or making every mutation condition false still leaves the “correct receipt” test green. This is why CR-04 and CR-05 are invisible to the suite.
**Fix:** Execute extracted mutation scripts under fake `docker`, `railway`, `kubectl`, and release shims that record exact argv, or use a workflow interpreter with a fixed event context. Evaluate job/step conditions and dependency results. Add test mutations that remove the actual provider command and invert the ref condition, and prove those mutations make the tests fail.

### WR-02: The database-conditional test bypasses the production conditional registration path

**Classification:** Warning
**File:** `crates/noesis-orchestrator/src/lib.rs:1004-1025`
**Issue:** The test's “without DB” branch omits the engine, then its “with DB” branch directly calls `orchestrator.register_engine` with a lazy pool. It never supplies DB configuration to the application builder or executes the conditional registration used at runtime. The test would continue to pass if production startup registered biofield-capture unconditionally, stopped registering it, or keyed off the wrong configuration.
**Fix:** Extract the production conditional-registration decision into a helper used by the API builder and this test, or exercise `build_app_state_lazy_db` with `database_url=None` and a disposable/lazy test URL and assert the actual application orchestrator's engine set in both cases.

### WR-03: Registry provenance accepts nonexistent files and stale symbolic anchors

**Classification:** Warning
**File:** `scripts/validate_contracts.py:329-344`; `contracts/v1/registries/engines.json:126`; `contracts/v1/registries/engines.json:238`; `contracts/v1/registries/engines.json:519`; `contracts/v1/registries/engines.json:744`; `contracts/v1/registries/engines.json:801`; `contracts/v1/registries/engines.json:858`; `contracts/v1/registries/engines.json:915`; `ts-engines/src/server/registry.ts:78-86`
**Issue:** Evidence references are validated only for a `repo://` prefix. A disposable authority whose first declaration reference was changed to a definitely nonexistent file still passed. The six TypeScript integrated references also point to `ts-engines/src/index.ts#runtime-registration`, while the extracted registration authority now lives in `ts-engines/src/server/registry.ts:78-86`; the biofield conditional anchor is likewise only a symbolic phrase. The evidence ledger can silently rot while all contract gates remain green.
**Fix:** Parse every `repo://` URI, reject absolute/traversing paths, require the path to exist inside the repository, and resolve supported fragments against source symbols or documented anchors. Update the TypeScript references to `ts-engines/src/server/registry.ts#registerTypeScriptRuntimeEngines` and add missing-file/missing-anchor negative tests.

### WR-04: The Railway production token is exposed to dependency installation

**Classification:** Warning
**File:** `.github/workflows/deploy.yaml:436-483`
**Issue:** `RAILWAY_TOKEN` is job-level environment state, so it is present during checkout and `npm install -g @railway/cli@5.41.0`. A version pin does not integrity-pin the package's transitive dependency graph, and npm lifecycle code inherits the environment. A compromised package or install script would receive a production project token even though installation requires no Railway credential.
**Fix:** Remove the token from job-level `env`. Install and verify the CLI before introducing credentials, then inject `RAILWAY_TOKEN` only into the token-scope and deploy steps. Keep the manifest-derived nonsecret IDs job-scoped if desired.

## Verification Performed

- `python3 -m pytest tests/scripts/test_validate_release_receipt.py tests/scripts/test_gate_wiring.py tests/scripts/test_validate_contracts.py -q` — 72 passed.
- `python3 scripts/validate_contracts.py` — passed with 6 schemas, 5 fixtures, 1 registry, and 19 engines.
- `python3 scripts/validate_release_receipt.py --validate-fixtures` — passed with 2 receipts and 9 mutation cases.
- Direct operational-validator probes — stale timestamps, fabricated deployment ID, tagless release authorization, and fabricated required-asset identity each returned zero errors.
- Disposable contract-authority probe — a nonexistent `repo://` reference was accepted.
- `@railway/cli@5.41.0 railway up --help` — confirmed the pinned CLI exposes the project/environment/service/CI flags used by the workflow; no selector-option finding is reported.

---

_Reviewed: 2026-09-06T12:44:38Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
