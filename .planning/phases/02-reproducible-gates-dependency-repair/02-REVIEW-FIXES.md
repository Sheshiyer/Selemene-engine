---
phase: 02-reproducible-gates-dependency-repair
fixed_at: 2026-09-06T13:42:29Z
review_path: .planning/phases/02-reproducible-gates-dependency-repair/02-REVIEW.md
iteration: 1
findings_in_scope: 14
fixed: 14
skipped: 0
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
| WR-03 | Fixed | `repo://` validation rejects absolute/traversing/missing paths and resolves supported Markdown anchors or source symbols. Stale TypeScript and database-conditional references now name real symbols; manifest registry digest was refreshed. | Missing path, missing anchor and traversal cases fail in `test_validate_contracts.py`; canonical registry validation passes. | `c6e6a0f` |
| WR-04 | Fixed | Railway CLI installation and version verification run without `RAILWAY_TOKEN`; the token is injected only into credentialed scope and deployment steps. | Static wiring and exact credentialed script execution assertions in `test_gate_wiring.py`. | `c67635e` |

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
