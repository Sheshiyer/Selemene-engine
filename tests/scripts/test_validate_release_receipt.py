from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
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
        "--required-promotion-mode",
        "immutable-image",
    )

    assert result.returncode == 0, f"{result.stdout}{result.stderr}"
    assert "mode=immutable-image" in result.stdout


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

    for forbidden in ("requests", "urllib.request", "subprocess", "socket", "curl", "railway"):
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
