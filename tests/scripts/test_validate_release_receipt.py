from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

import pytest
from jsonschema import Draft202012Validator

from .conftest import REPO_ROOT, run_python_script


RELEASE_ROOT = REPO_ROOT / "contracts/release/v1"
ELIGIBLE_SOURCE = RELEASE_ROOT / "fixtures/eligible-source-redeploy.json"
CURRENT_PRODUCTION = RELEASE_ROOT / "fixtures/current-production-incomplete.json"
MUTATION_CASES = RELEASE_ROOT / "fixtures/mutation-cases.json"
SYNTHETIC_SOURCE = "1111111111111111111111111111111111111111"

SPEC = importlib.util.spec_from_file_location(
    "validate_release_receipt",
    REPO_ROOT / "scripts/validate_release_receipt.py",
)
assert SPEC is not None and SPEC.loader is not None
validate_release_receipt_module = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(validate_release_receipt_module)
apply_json_pointer = validate_release_receipt_module.apply_json_pointer
validate_release_receipt = validate_release_receipt_module.validate_release_receipt


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def run_receipt(path: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return run_python_script("scripts/validate_release_receipt.py", path, *args)


def materialize_case(case_id: str) -> dict:
    cases = load_json(MUTATION_CASES)
    receipt = load_json(RELEASE_ROOT / cases["base"])
    case = next(item for item in cases["cases"] if item["id"] == case_id)
    for mutation in case["mutations"]:
        apply_json_pointer(receipt, mutation)
    return receipt


def operational_receipt(*, workflow: str = ".github/workflows/deploy.yaml") -> dict:
    receipt = load_json(ELIGIBLE_SOURCE)
    receipt.pop("test_fixture")
    now = datetime.now(timezone.utc).replace(microsecond=0)
    receipt["issued_at"] = (now - timedelta(seconds=30)).strftime("%Y-%m-%dT%H:%M:%SZ")
    receipt["expires_at"] = (now + timedelta(minutes=10)).strftime("%Y-%m-%dT%H:%M:%SZ")
    receipt["operation"] = {
        "id": "github-run-90000000001-attempt-1",
        "workflow": workflow,
        "run_id": "90000000001",
        "run_attempt": 1,
    }
    return receipt


def test_release_receipt_schema_is_valid_draft_2020_12() -> None:
    schema = load_json(RELEASE_ROOT / "receipt.schema.json")

    Draft202012Validator.check_schema(schema)
    assert schema["$id"] == "https://schemas.selemene.dev/release/v1/receipt.schema.json"
    assert schema["additionalProperties"] is False


def test_declared_fixture_suite_falsifies_negative_cases() -> None:
    result = run_python_script(
        "scripts/validate_release_receipt.py", "--validate-fixtures"
    )

    assert result.returncode == 0, f"{result.stdout}{result.stderr}"
    assert "receipts=2 mutation_cases=9" in result.stdout


def test_complete_source_redeploy_receipt_is_eligible() -> None:
    result = run_receipt(
        ELIGIBLE_SOURCE,
        "--expected-source",
        SYNTHETIC_SOURCE,
        "--required-promotion-mode",
        "source-redeploy",
    )

    assert result.returncode == 0, f"{result.stdout}{result.stderr}"
    assert "mode=source-redeploy" in result.stdout


def test_immutable_image_receipt_requires_matching_built_and_deployed_digests(
    tmp_path: Path,
) -> None:
    receipt_path = tmp_path / "immutable.json"
    receipt_path.write_text(
        json.dumps(materialize_case("eligible-immutable-image")), encoding="utf-8"
    )

    result = run_receipt(
        receipt_path,
        "--expected-source",
        SYNTHETIC_SOURCE,
        "--expected-release-tag",
        "v1.2.3",
        "--required-promotion-mode",
        "immutable-image",
    )

    assert result.returncode == 0, f"{result.stdout}{result.stderr}"
    assert "mode=immutable-image" in result.stdout


def test_release_receipt_cannot_authorize_a_different_semantic_tag(
    tmp_path: Path,
) -> None:
    receipt_path = tmp_path / "release.json"
    receipt_path.write_text(
        json.dumps(materialize_case("eligible-immutable-image")), encoding="utf-8"
    )

    result = run_receipt(
        receipt_path,
        "--expected-source",
        SYNTHETIC_SOURCE,
        "--expected-release-tag",
        "v1.2.4",
        "--required-promotion-mode",
        "immutable-image",
    )

    assert result.returncode == 1
    assert "release.tag does not match expected release tag" in result.stderr


@pytest.mark.parametrize(
    ("case_id", "expected_error"),
    [
        ("wrong-source", "source.validated_revision must equal source.revision"),
        ("wrong-image", "deployed image digest must equal built image digest"),
        ("missing-build-identity", "artifacts.api.built.build_id evidence is unavailable"),
        ("wrong-service-role", "missing service roles"),
        ("wrong-required-check", "missing required check identity"),
        ("missing-schema", "schema_identity"),
        ("missing-rollback", "rollback"),
        ("unknown-asset", "unknown asset IDs"),
    ],
)
def test_negative_mutation_cases_fail_closed(
    tmp_path: Path, case_id: str, expected_error: str
) -> None:
    receipt_path = tmp_path / f"{case_id}.json"
    receipt_path.write_text(json.dumps(materialize_case(case_id)), encoding="utf-8")

    result = run_receipt(receipt_path)

    assert result.returncode == 1
    assert expected_error in result.stderr


def test_current_production_snapshot_remains_ineligible() -> None:
    result = run_receipt(CURRENT_PRODUCTION)

    assert result.returncode == 1
    assert "source.revision evidence is unavailable" in result.stderr
    assert "schema_identity.applied_revision evidence is unavailable" in result.stderr
    assert "rollback.previous_source_revision evidence is unavailable" in result.stderr
    assert "assets.ephemeris-data.integrity evidence is unavailable" in result.stderr


def test_expected_source_is_bound_by_the_cli() -> None:
    result = run_receipt(
        ELIGIBLE_SOURCE,
        "--expected-source",
        "2222222222222222222222222222222222222222",
    )

    assert result.returncode == 1
    assert "does not match expected source" in result.stderr


def test_source_redeploy_cannot_satisfy_immutable_image_workflow() -> None:
    result = run_receipt(
        ELIGIBLE_SOURCE,
        "--required-promotion-mode",
        "immutable-image",
    )

    assert result.returncode == 1
    assert "promotion_mode does not match the workflow" in result.stderr


def test_operational_validation_rejects_synthetic_test_receipt() -> None:
    result = run_receipt(
        ELIGIBLE_SOURCE,
        "--operational",
        "--target-profile",
        "deploy-production",
        "--expected-artifact-digest",
        "api=sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "--expected-artifact-digest",
        "typescript-engines=sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    )

    assert result.returncode == 1
    assert "rejects receipts containing test_fixture metadata" in result.stderr
    assert "target.environment does not match workflow authority" in result.stderr


def test_operational_validation_requires_expected_source(tmp_path: Path) -> None:
    receipt = operational_receipt()
    receipt["target"]["environment"] = "production"
    receipt_path = tmp_path / "operational.json"
    receipt_path.write_text(json.dumps(receipt), encoding="utf-8")

    result = run_receipt(
        receipt_path,
        "--operational",
        "--target-profile",
        "deploy-production",
        "--expected-operation-id",
        "github-run-90000000001-attempt-1",
        "--expected-workflow",
        ".github/workflows/deploy.yaml",
        "--now",
        datetime.now(timezone.utc).replace(microsecond=0).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "--expected-artifact-digest",
        "api=sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "--expected-artifact-digest",
        "typescript-engines=sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    )

    assert result.returncode == 1
    assert "operational validation requires an expected source revision" in result.stderr


def test_target_profile_rejects_wrong_provider_scope() -> None:
    receipt = load_json(ELIGIBLE_SOURCE)
    receipt.pop("test_fixture")
    receipt["target"] = {
        "environment": "production",
        "provider_scope": ["railway"],
    }

    errors = validate_release_receipt(
        receipt,
        REPO_ROOT,
        target_profile="deploy-production",
    )

    assert "missing target providers: ['ghcr']" in errors


def test_service_target_identity_is_bound_to_manifest_authority() -> None:
    receipt = load_json(ELIGIBLE_SOURCE)
    api = next(row for row in receipt["service_roles"] if row["role"] == "api")
    api["provider"] = "vercel"
    api["project_id"]["value"] = "fabricated-project"
    api["service_id"]["value"] = "fabricated-api-service"
    api["environment_id"]["value"] = "fabricated-environment"

    errors = validate_release_receipt(receipt, REPO_ROOT)

    assert "service_roles.api.provider does not match release authority" in errors
    assert "service_roles.api.project_id does not match release authority" in errors
    assert "service_roles.api.service_id does not match release authority" in errors
    assert "service_roles.api.environment_id does not match release authority" in errors


def test_operational_validation_emits_only_manifest_bound_deploy_selector(
    tmp_path: Path,
) -> None:
    receipt = operational_receipt()
    receipt["target"]["environment"] = "production"
    receipt_path = tmp_path / "operational.json"
    output_path = tmp_path / "github-output"
    receipt_path.write_text(json.dumps(receipt), encoding="utf-8")

    result = run_receipt(
        receipt_path,
        "--expected-source",
        SYNTHETIC_SOURCE,
        "--required-promotion-mode",
        "source-redeploy",
        "--target-profile",
        "deploy-production",
        "--expected-operation-id",
        "github-run-90000000001-attempt-1",
        "--expected-workflow",
        ".github/workflows/deploy.yaml",
        "--now",
        datetime.now(timezone.utc).replace(microsecond=0).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "--expected-artifact-digest",
        "api=sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "--expected-artifact-digest",
        "typescript-engines=sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "--github-output",
        str(output_path),
        "--operational",
    )

    assert result.returncode == 0, f"{result.stdout}{result.stderr}"
    assert output_path.read_text(encoding="utf-8").splitlines() == [
        "railway_project_id=11eedde4-41e6-4f51-b86b-cf77111cf592",
        "railway_environment_id=702b945e-2c66-4d5a-bae1-4c67ea14c3bb",
        "railway_service_id=48b3bd23-5620-4f7b-8e5d-96bc5c5d7fc4",
    ]


def test_operational_validation_binds_every_actual_artifact_digest() -> None:
    receipt = operational_receipt()
    receipt["target"] = {
        "environment": "production",
        "provider_scope": ["ghcr", "railway"],
    }

    errors = validate_release_receipt(
        receipt,
        REPO_ROOT,
        target_profile="deploy-production",
        expected_artifact_digests={
            "api": "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        },
        expected_operation_id="github-run-90000000001-attempt-1",
        expected_workflow=".github/workflows/deploy.yaml",
        now=datetime.now(timezone.utc).replace(microsecond=0),
        operational=True,
    )

    assert "missing expected artifact digest roles: ['typescript-engines']" in errors
    assert (
        "artifacts.api.built.image_digest does not match workflow artifact" in errors
    )


def test_operational_receipt_rejects_expiry_boundary_and_replayed_attempt() -> None:
    receipt = operational_receipt()
    receipt["target"]["environment"] = "production"
    expiry = datetime.fromisoformat(receipt["expires_at"].replace("Z", "+00:00"))
    common = {
        "expected_source": SYNTHETIC_SOURCE,
        "target_profile": "deploy-production",
        "expected_artifact_digests": {
            "api": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "typescript-engines": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        },
        "expected_workflow": ".github/workflows/deploy.yaml",
        "operational": True,
    }

    before_expiry = validate_release_receipt(
        receipt,
        REPO_ROOT,
        expected_operation_id="github-run-90000000001-attempt-1",
        now=expiry - timedelta(seconds=1),
        **common,
    )
    at_expiry = validate_release_receipt(
        receipt,
        REPO_ROOT,
        expected_operation_id="github-run-90000000001-attempt-1",
        now=expiry,
        **common,
    )
    replayed_attempt = validate_release_receipt(
        receipt,
        REPO_ROOT,
        expected_operation_id="github-run-90000000001-attempt-2",
        now=expiry - timedelta(seconds=1),
        **common,
    )

    assert before_expiry == []
    assert "release authorization is expired" in at_expiry
    assert any("operation.id does not match expected workflow operation" in error for error in replayed_attempt)


def test_operational_receipt_rejects_stale_authorization_and_rollback() -> None:
    receipt = operational_receipt()
    receipt["target"]["environment"] = "production"
    receipt["issued_at"] = "2000-01-01T00:00:00Z"
    receipt["expires_at"] = "2000-01-01T00:15:00Z"
    receipt["rollback"]["tested_at"]["value"] = "2000-01-01T00:00:00Z"

    errors = validate_release_receipt(
        receipt,
        REPO_ROOT,
        expected_source=SYNTHETIC_SOURCE,
        target_profile="deploy-production",
        expected_artifact_digests={
            "api": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "typescript-engines": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        },
        expected_operation_id="github-run-90000000001-attempt-1",
        expected_workflow=".github/workflows/deploy.yaml",
        now=datetime(2026, 9, 6, tzinfo=timezone.utc),
        operational=True,
    )

    assert "release authorization is stale" in errors
    assert "release authorization is expired" in errors
    assert "rollback rehearsal is stale" in errors


@pytest.mark.parametrize(
    ("path", "value"),
    [
        (("schema_identity", "applied_revision", "value"), "x"),
        (("schema_identity", "rollback_compatibility", "value"), "x"),
        (("rollback", "schema_restore", "value"), "x"),
        (("rollback", "procedure", "value"), "x"),
        (("rollback", "tested_at", "value"), "x"),
    ],
)
def test_schema_and_rollback_identity_reject_meaningless_strings(
    path: tuple[str, ...],
    value: str,
) -> None:
    receipt = load_json(ELIGIBLE_SOURCE)
    target = receipt
    for part in path[:-1]:
        target = target[part]
    target[path[-1]] = value

    errors = validate_release_receipt(receipt, REPO_ROOT)

    assert any(
        "is not valid" in error
        or "is not one of" in error
        or "does not match" in error
        or "is not a" in error
        for error in errors
    )


def test_unavailable_optional_dependency_is_unknown_not_not_applicable() -> None:
    receipt = load_json(ELIGIBLE_SOURCE)
    mediapipe = next(row for row in receipt["dependencies"] if row["id"] == "mediapipe")
    mediapipe["state"] = {
        "status": "unavailable",
        "reason": "the optional dependency state was not observed",
    }

    errors = validate_release_receipt(receipt, REPO_ROOT)

    assert any("dependencies.mediapipe.state evidence is unavailable" in error for error in errors)


def test_not_applicable_is_rejected_for_required_dependency() -> None:
    receipt = load_json(ELIGIBLE_SOURCE)
    postgresql = next(row for row in receipt["dependencies"] if row["id"] == "postgresql")
    postgresql["state"] = {
        "status": "not-applicable",
        "reason": "synthetic attempt to waive a required dependency",
    }

    errors = validate_release_receipt(receipt, REPO_ROOT)

    assert any("dependencies.postgresql.state evidence is not-applicable" in error for error in errors)


def test_registry_identity_must_match_plan_06_authority() -> None:
    receipt = load_json(ELIGIBLE_SOURCE)
    receipt["authority"]["engine_registry"]["sha256"] = (
        "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    )

    errors = validate_release_receipt(receipt, REPO_ROOT)

    assert "authority.engine_registry.sha256 does not match the release manifest" in errors


def test_validator_has_no_network_or_provider_client_path() -> None:
    source = (REPO_ROOT / "scripts/validate_release_receipt.py").read_text(
        encoding="utf-8"
    )

    for forbidden in (
        "requests",
        "urllib.request",
        "subprocess",
        "socket",
        "curl",
        "https://backboard.railway.com",
        "railway up",
    ):
        assert forbidden not in source


def test_non_object_receipt_fails_without_traceback(tmp_path: Path) -> None:
    receipt_path = tmp_path / "array.json"
    receipt_path.write_text("[]", encoding="utf-8")

    result = subprocess.run(
        [sys.executable, str(REPO_ROOT / "scripts/validate_release_receipt.py"), str(receipt_path)],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 2
    assert "receipt must be a JSON object" in result.stderr
    assert "Traceback" not in result.stderr
