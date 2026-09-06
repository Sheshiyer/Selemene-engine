---
phase: 02-reproducible-gates-dependency-repair
fixed_at: 2026-09-06T13:42:29Z
review_path: .planning/phases/02-reproducible-gates-dependency-repair/02-REVIEW.md
iteration: 1
findings_in_scope: 14
fixed: 14
skipped: 0
recheck_findings: 1
recheck_fixed: 1
rechecked_at: 2026-09-06T14:49:30Z
second_recheck_findings: 5
second_recheck_fixed: 5
second_rechecked_at: 2026-09-06T15:08:34Z
status: all_fixed
production_promotion: HOLD
---

# Phase 02 Deep Review Fixes

The ten Critical and four Warning findings in `02-REVIEW.md` are closed in source or by an explicit fail-closed disposition. No production deployment, registry alias, release, tag, provider, schema, data, DNS, repository setting or secret mutation was performed.

## Finding Dispositions

| Finding | Disposition | Code and authority | Regression proof | Commit |
|---|---|---|---|---|
| CR-01 | Fixed | Release receipts require a canonical `release.tag`; the release workflow passes `GITHUB_REF_NAME` and the validator binds it before eligibility. | Cross-tag authorization is rejected by `test_release_receipt_cannot_authorize_a_different_semantic_tag`; workflow wiring checks the exact CLI argument. | `8069ca9` |
| CR-02 | Fixed with fail-closed production hold | Receipts bind `issued_at`, `expires_at`, workflow, run/attempt operation identity and rollback freshness. The manifest declares durable one-use consumption mandatory; both production profiles remain disabled because stateless CI cannot prove consumption. | Frozen-clock expiry, stale rollback and replayed-attempt cases fail in `test_operational_receipt_rejects_expiry_boundary_and_replayed_attempt` and `test_operational_receipt_rejects_stale_authorization_and_rollback`. | `952b291` |
| CR-03 | Fixed with fail-closed production hold | Pre-mutation receipts contain intended source, exact artifact digests, selectors and rollback inputs, and reject a fabricated future deployment identity. Railway post-deploy source/status attestation is a separate required authority and the deploy profile stays disabled until it exists. | `test_pre_deploy_receipt_rejects_fabricated_future_deployment_identity` and `test_operational_validation_refuses_outputs_while_provider_attestation_is_unavailable`. | `5839e96` |
| CR-04 | Fixed | `validate-source` rejects non-production dispatch environments and refs other than `main` or canonical semver tags before image work. An always-run deployment-result job fails if the authoritative Railway job did not succeed. | Executable main/tag/feature contexts and skipped-deploy result cases in `test_deploy_ref_admission_executes_for_main_tag_and_feature` and `test_final_deployment_result_cannot_pass_when_railway_is_skipped`. | `507e7d1` |
| CR-05 | Fixed with fail-closed production hold | The manifest distinguishes deployed roles (`api`, `typescript-engines`) from topology-only `biofield-cv`, binds each root/config/selector, and the Railway script addresses both repository-owned deploy roles. Atomic cross-service coordination and provider attestations are still required before production authorization. | Exact role outputs and exact two-service Railway argv are exercised by `test_deploy_profile_emits_every_owned_service_selector_and_root` and `test_actual_railway_script_records_exact_multi_service_argv`. | `db15032` |
| CR-06 | Fixed | Rollback authority is role-keyed for every mutated artifact and service, including repository, provider target, prior source/digest/deployment, procedure and rehearsal time. | Missing roles and wrong targets fail in `test_rollback_requires_every_mutated_role_and_exact_target`. | `97a2a5a` |
| CR-07 | Fixed | Deploy publication is limited to immutable `sha-$GITHUB_SHA` candidates for both repositories; branch, `latest` and semver aliases are absent. | `test_deploy_publishes_only_source_immutable_candidate_tags` and exact Docker shim argv. | `b5295c7` |
| CR-08 | Fixed with fail-closed production hold | The release workflow is serialized, read-only and ends in an explicit failing hold job. Alias and GitHub release mutations are absent until atomic multi-registry promotion with compensation and durable one-use authority exist. | `test_release_mutation_is_absent_until_atomic_authority_exists` and `test_actual_release_scripts_are_read_only_and_end_in_hold`. | `0f22e90` |
| CR-09 | Fixed with fail-closed production hold | Health origins, paths, status fields and source-revision fields come from manifest-bound service authority. The workflow probes both deployed roles and requires the reported revision to equal `GITHUB_SHA`. Production remains disabled until provider-returned deployment identity and deployed source markers are attested. | `test_health_checks_use_only_manifest_outputs_and_require_source_markers` and the executable failure cases in `test_actual_health_script_requires_both_bound_source_markers`. | `89d1849` |
| CR-10 | Fixed with fail-closed production hold | Required assets use canonical repository paths and deterministic `sha256-tree-v1` digests/file counts, with artifact role, build recipe and container path authority. Operational eligibility refuses self-asserted image inclusion until a source-bound post-build inspection attestation exists. | Wrong path/digest cases fail in `test_required_asset_source_and_integrity_match_repository_tree`; missing image attestation fails in `test_operational_asset_inclusion_fails_without_postbuild_attestation`. | `ddfe062` |
| WR-01 | Fixed | Tests evaluate actual job conditions and execute extracted provider scripts under recording Docker, Railway, Kustomize, Kubectl, curl and release shims, asserting exact argv, cwd and outcomes. Concurrent pipe participants use isolated logs. | Provider-script tests in `test_gate_wiring.py`; removing commands or changing conditions now breaks direct assertions. | `2ba59c4`, `941e3cf` |
| WR-02 | Fixed | Both production app-state builders call `register_database_conditional_engines`; tests exercise the same seam with no pool and a lazy local pool. | `cargo test -p noesis-api database_conditional_registration --locked`: 2 passed without production database access. | `a28a983` |
| WR-03 | Fixed; strengthened after recheck | `repo://` validation rejects absolute/traversing/missing paths and resolves supported Markdown anchors. Stale TypeScript and database-conditional references now name real symbols; the WR-R01 follow-up below makes source resolution declaration-aware. | Missing path, missing anchor and traversal cases fail in `test_validate_contracts.py`; canonical registry validation passes. | `c6e6a0f` plus the containing WR-R01 follow-up commit |
| WR-04 | Fixed | Railway CLI installation and version verification run without `RAILWAY_TOKEN`; the token is injected only into credentialed scope and deployment steps. | Static wiring and exact credentialed script execution assertions in `test_gate_wiring.py`. | `c67635e` |

## First Independent Recheck Follow-up

### WR-R01: Source evidence anchors accepted comment-only symbol names

**Disposition:** Fixed.

Non-Markdown `repo://` anchors resolve against language-specific declarations and definitions. This first repair rejected ordinary comment-only and string-only names, required an explicit owner for qualified Rust methods, and used the Python standard-library AST. The second adversarial recheck below narrowed the exact lexical-scope and literal guarantees.

The resolver strips C-style comments and conventional quoted/template literals before Rust and TypeScript/JavaScript matching. Python parsing inherently excludes comments and literal contents. A terminal method under another Rust type cannot satisfy a qualified anchor.

**Red/green evidence:** Before the resolver change, the focused adversarial selection produced nine failures: four comment-only cases, four string-only cases and one wrong-Rust-qualifier case. After the change, the same selection passed 20 cases, including preservation probes for every canonical qualified and unqualified source reference plus valid Rust, TypeScript, JavaScript and Python declarations.

**First-fix verification:**

- `python3 -m pytest tests/scripts/test_validate_contracts.py -q`: 45 passed.
- `python3 scripts/validate_contracts.py`: `schemas=6 fixtures=5 registries=1 engines=19`.
- `python3 -m py_compile scripts/validate_contracts.py tests/scripts/test_validate_contracts.py`: passed.

**Files:** `scripts/validate_contracts.py`, `tests/scripts/test_validate_contracts.py`.

**Commit:** Atomic WR-R01 follow-up commit containing this evidence entry.

## Second Adversarial Recheck Follow-up

### WR-R2-01: Nested and class-scoped decoys satisfied broader anchors

**Disposition:** Fixed.

Unqualified Rust, TypeScript and JavaScript anchors now scan only file-scope declarations; nested functions, local variables and impl/class members are blanked before declaration matching. Qualified Rust impls and TypeScript/JavaScript owners must themselves be at file scope. Python still uses its AST, but only module-scope declarations receive unqualified names; class members require their complete class qualifier, and function-local owners are not collected.

All twelve registry occurrences of the native registration method now use `WorkflowOrchestrator::register_native_runtime_engines`. The registry remains 158 anchored occurrences and seven unique references, all resolved by the canonical validator.

### WR-R2-02: Rust raw C strings and lifetime impls were mishandled

**Disposition:** Fixed.

The bounded Rust scanner now recognizes `r`, `br`, and `cr` raw literals, blanks normal string, byte-string, C-string and character contents, and leaves lifetime and label apostrophes intact. Compile-backed probes cover embedded quotes and newlines in all three raw forms, normal byte/C strings, characters, labels, lifetime-parameterized inherent impls, and lifetime-parameterized trait impls.

### WR-R2-03: TSX JSX text was interpreted as a declaration

**Disposition:** Fixed fail closed.

Anchored `.tsx` and `.jsx` references now fail as unsupported until a JSX-aware AST resolver exists. Regression cases cover child text, attribute strings, expressions, nested components, fragments, and `.jsx` files. Plain `.ts`, `.mts`, `.cts`, `.js`, `.mjs`, and `.cjs` retain declaration-aware support.

### WR-R2-04: Non-rendered Markdown headings satisfied anchors

**Disposition:** Fixed.

Markdown anchor collection now tracks backtick and tilde fences and removes multiline HTML comments before recognizing ATX headings. Headings inside either non-rendered region reject, while a rendered ATX heading resolves.

### WR-R2-05: The fix ledger overstated the first resolver

**Disposition:** Fixed.

The first follow-up language above now states only what its tests proved. This second entry records the independent recheck findings, exact scope restrictions, fail-closed JSX disposition, compile-backed Rust syntax matrix, refreshed authority digest, and current green results.

**Red/green evidence:** Before the second repair, the focused residual selection reported 11 failed, 3 passed and 45 deselected. After implementation and expansion to every exact reviewer case, the residual selection passed 25 cases with 47 deselected.

**Final verification:**

- `python3 -m pytest tests/scripts/test_validate_contracts.py -q`: 72 passed.
- Residual adversarial `-k` selection: 25 passed, 47 deselected.
- `python3 scripts/validate_contracts.py`: `schemas=6 fixtures=5 registries=1 engines=19`.
- `python3 -m pytest tests/scripts/test_validate_release_receipt.py -q`: 40 passed.
- `python3 scripts/validate_release_receipt.py --validate-fixtures`: `receipts=2 mutation_cases=9`.
- `python3 -m py_compile scripts/validate_contracts.py tests/scripts/test_validate_contracts.py`: passed.
- Registry digest recomputation: `sha256:c801fb49332f5c78cdf78417b6efd343cfcf30dc26e6ce7259cfc6157b707896`, matched by the release manifest and both digest-bearing fixtures.

**Files:** `scripts/validate_contracts.py`, `tests/scripts/test_validate_contracts.py`, `contracts/v1/registries/engines.json`, `contracts/release/v1/manifest.json`, `contracts/release/v1/fixtures/eligible-source-redeploy.json`, `contracts/release/v1/fixtures/current-production-incomplete.json`.

**Commit:** Atomic WR-R2 follow-up commit containing this evidence entry.

## Production Mutation Disposition

`deploy-production` and `release-production` both have `mutation_policy.status: disabled` in `contracts/release/v1/manifest.json`. Operational validation fails before provider outputs or writes. Re-enabling either profile requires repository-independent authority that this local repair cannot invent:

- durable one-use receipt consumption;
- Railway-returned deployment ID, source and status attestation for API and TypeScript services;
- atomic API/TypeScript service coordination;
- source-bound post-build asset inclusion attestation;
- atomic multi-registry alias promotion with rollback compensation for releases.

Vercel native main deployment protection, staging/Kubernetes target profiles and per-platform native binary publication also remain outside this authority. Production promotion remains **HOLD**.

## Verification

All commands ran with production database variables removed where applicable.

| Check | Result |
|---|---|
| `python3 -m pytest tests/scripts/test_validate_release_receipt.py tests/scripts/test_gate_wiring.py tests/scripts/test_validate_contracts.py -q` | 95 passed |
| `cargo test -p noesis-api database_conditional_registration --locked` | 2 passed |
| `python3 scripts/validate_action_pins.py` | Passed |
| Parse every `.github/workflows/*.{yml,yaml}` with PyYAML | Passed |
| `python3 scripts/validate_contracts.py` | `schemas=6 fixtures=5 registries=1 engines=19` |
| `python3 scripts/validate_release_receipt.py --validate-fixtures` | `receipts=2 mutation_cases=9` |
| Current-production negative fixture | Exit 1 with 29 unavailable facts |
| `env -u DATABASE_URL -u TEST_DATABASE_URL pnpm run gate:scripts` | 140 passed; all validators passed |
| `env -u DATABASE_URL -u TEST_DATABASE_URL pnpm run gate` | Exit 0; 140 script, 9 core, 4 OpenAPI, 16 API integration, 35 engine SDK, 11 Noesis SDK, 36 verification and 94 TypeScript tests; all builds/typechecks passed |
| `git diff --check` | Passed |

## Remaining Evidence Boundary

No Critical or Warning review finding remains accepted by an enabled mutation path. Phase 2 remains `gaps_found` until this exact candidate has current-source remote CI. The disabled production profiles and the production HOLD remain until their named external authorities are observed and reviewed.
