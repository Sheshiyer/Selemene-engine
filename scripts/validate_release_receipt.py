#!/usr/bin/env python3
"""Fail-closed, offline validation for Selemene release receipts."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator, FormatChecker
from jsonschema.exceptions import SchemaError


RELEASE_ROOT = Path("contracts/release/v1")
MANIFEST_PATH = RELEASE_ROOT / "manifest.json"


class ReleaseReceiptError(ValueError):
    """Raised when release authority files cannot be read safely."""


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ReleaseReceiptError(f"{path}: {error}") from error


def sha256_path(path: Path) -> str:
    try:
        payload = path.read_bytes()
    except OSError as error:
        raise ReleaseReceiptError(f"{path}: {error}") from error
    return f"sha256:{hashlib.sha256(payload).hexdigest()}"


def evidence_value(
    evidence: dict[str, Any],
    label: str,
    errors: list[str],
    *,
    allow_not_applicable: bool = False,
) -> Any | None:
    status = evidence["status"]
    if status == "available":
        return evidence["value"]
    if status == "not-applicable" and allow_not_applicable:
        return None
    errors.append(f"{label} evidence is {status}: {evidence['reason']}")
    return None


def indexed_rows(
    rows: list[dict[str, Any]],
    key: str,
    label: str,
    errors: list[str],
) -> dict[str, dict[str, Any]]:
    indexed: dict[str, dict[str, Any]] = {}
    duplicates: set[str] = set()
    for row in rows:
        value = row[key]
        if value in indexed:
            duplicates.add(value)
        indexed[value] = row
    if duplicates:
        errors.append(f"duplicate {label}: {sorted(duplicates)}")
    return indexed


def set_difference_error(
    label: str,
    expected: set[str],
    actual: set[str],
    errors: list[str],
) -> None:
    missing = expected - actual
    unknown = actual - expected
    if missing:
        errors.append(f"missing {label}: {sorted(missing)}")
    if unknown:
        errors.append(f"unknown {label}: {sorted(unknown)}")


def validate_timestamp(value: str, label: str, errors: list[str]) -> datetime | None:
    try:
        parsed = datetime.fromisoformat(value.removesuffix("Z") + "+00:00")
    except ValueError:
        errors.append(f"{label} must be a valid UTC RFC3339 timestamp")
        return None
    if parsed.tzinfo is None or parsed.utcoffset() != timezone.utc.utcoffset(parsed):
        errors.append(f"{label} must be a valid UTC RFC3339 timestamp")
        return None
    return parsed


def schema_errors(receipt: Any, schema: dict[str, Any]) -> list[str]:
    try:
        Draft202012Validator.check_schema(schema)
    except SchemaError as error:
        raise ReleaseReceiptError(f"release receipt schema is invalid: {error.message}") from error

    validator = Draft202012Validator(schema, format_checker=FormatChecker())
    errors: list[str] = []
    for error in sorted(validator.iter_errors(receipt), key=lambda item: list(item.absolute_path)):
        location = "$"
        for part in error.absolute_path:
            location += f"[{part}]" if isinstance(part, int) else f".{part}"
        errors.append(f"{location}: {error.message}")
    return errors


def validate_release_receipt(
    receipt: dict[str, Any],
    repo_root: Path,
    *,
    expected_source: str | None = None,
    required_promotion_mode: str | None = None,
    target_profile: str | None = None,
    expected_artifact_digests: dict[str, str] | None = None,
    expected_release_tag: str | None = None,
    expected_operation_id: str | None = None,
    expected_workflow: str | None = None,
    now: datetime | None = None,
    operational: bool = False,
) -> list[str]:
    """Return every eligibility error; an empty list means eligible."""

    release_root = repo_root / RELEASE_ROOT
    manifest = load_json(repo_root / MANIFEST_PATH)
    schema = load_json(release_root / manifest["schema"])
    errors = schema_errors(receipt, schema)
    if errors:
        return errors
    issued_at = validate_timestamp(receipt["issued_at"], "issued_at", errors)
    expires_at = validate_timestamp(receipt["expires_at"], "expires_at", errors)
    policy = manifest["receipt_policy"]
    if issued_at is not None and expires_at is not None:
        lifetime = (expires_at - issued_at).total_seconds()
        if lifetime <= 0:
            errors.append("expires_at must be later than issued_at")
        if lifetime > policy["max_authorization_age_seconds"]:
            errors.append("receipt authorization lifetime exceeds release policy")
    if now is not None and issued_at is not None and expires_at is not None:
        skew = policy["max_clock_skew_seconds"]
        age = (now - issued_at).total_seconds()
        if age < -skew:
            errors.append("issued_at is too far in the future")
        if age > policy["max_authorization_age_seconds"]:
            errors.append("release authorization is stale")
        if now >= expires_at:
            errors.append("release authorization is expired")

    operation = receipt["operation"]
    canonical_operation_id = (
        f"github-run-{operation['run_id']}-attempt-{operation['run_attempt']}"
    )
    if operation["id"] != canonical_operation_id:
        errors.append("operation.id does not match run_id and run_attempt")
    if expected_operation_id is not None and operation["id"] != expected_operation_id:
        errors.append(
            "operation.id does not match expected workflow operation: "
            f"required={expected_operation_id} actual={operation['id']}"
        )
    if expected_workflow is not None and operation["workflow"] != expected_workflow:
        errors.append(
            "operation.workflow does not match expected workflow: "
            f"required={expected_workflow} actual={operation['workflow']}"
        )

    authority = receipt["authority"]
    if authority["contract_id"] != manifest["contract_id"]:
        errors.append("authority.contract_id does not match the release manifest")
    if authority["schema_id"] != manifest["schema_id"]:
        errors.append("authority.schema_id does not match the release manifest")

    receipt_registry = authority["engine_registry"]
    manifest_registry = manifest["engine_registry"]
    for key in ("path", "registry_version", "contract_version", "sha256"):
        if receipt_registry[key] != manifest_registry[key]:
            errors.append(f"authority.engine_registry.{key} does not match the release manifest")
    registry_path = repo_root / manifest_registry["path"]
    actual_registry_digest = sha256_path(registry_path)
    if actual_registry_digest != manifest_registry["sha256"]:
        errors.append(
            "release manifest engine registry digest is stale: "
            f"declared={manifest_registry['sha256']} actual={actual_registry_digest}"
        )

    target = receipt["target"]
    if operational and "test_fixture" in receipt:
        errors.append("operational validation rejects receipts containing test_fixture metadata")
    if operational and target_profile is None:
        errors.append("operational validation requires a target profile")
    if operational and expected_source is None:
        errors.append("operational validation requires an expected source revision")
    if operational and expected_operation_id is None:
        errors.append("operational validation requires an expected operation ID")
    if operational and expected_workflow is None:
        errors.append("operational validation requires an expected workflow")
    if operational and now is None:
        errors.append("operational validation requires an explicit UTC clock")

    profile = None
    if target_profile is not None:
        profile = manifest["workflow_targets"].get(target_profile)
        if profile is None:
            errors.append(f"unknown target profile: {target_profile}")
        else:
            if target["environment"] != profile["environment"]:
                errors.append(
                    "target.environment does not match workflow authority: "
                    f"required={profile['environment']} actual={target['environment']}"
                )
            expected_providers = set(profile["provider_scope"])
            actual_providers = set(target["provider_scope"])
            set_difference_error(
                "target providers", expected_providers, actual_providers, errors
            )
            if operation["workflow"] != profile["workflow"]:
                errors.append("operation.workflow does not match target profile")
            mutation_policy = profile["mutation_policy"]
            if operational and mutation_policy["status"] != "enabled":
                required = ", ".join(mutation_policy["required_authority"])
                errors.append(
                    f"target profile {target_profile} production mutation is disabled: "
                    f"{mutation_policy['reason']}; required_authority={required}"
                )

    source = receipt["source"]
    if source["repository"] != manifest["repository"]:
        errors.append("source.repository does not match the release manifest")
    source_revision = evidence_value(source["revision"], "source.revision", errors)
    validated_revision = evidence_value(
        source["validated_revision"], "source.validated_revision", errors
    )
    if source_revision is not None and validated_revision is not None:
        if source_revision != validated_revision:
            errors.append("source.validated_revision must equal source.revision")
        if expected_source is not None and source_revision != expected_source:
            errors.append(
                f"source.revision does not match expected source {expected_source}"
            )

    promotion_mode = receipt["promotion_mode"]
    release_tag = receipt["release"]["tag"]
    if promotion_mode == "source-redeploy" and release_tag is not None:
        errors.append("release.tag must be null for source-redeploy authorization")
    if promotion_mode == "immutable-image" and release_tag is None:
        errors.append("release.tag is required for immutable-image authorization")
    if expected_release_tag is not None and release_tag != expected_release_tag:
        errors.append(
            "release.tag does not match expected release tag: "
            f"required={expected_release_tag} actual={release_tag}"
        )
    if operational and promotion_mode == "immutable-image" and expected_release_tag is None:
        errors.append("operational immutable-image validation requires an expected release tag")
    if required_promotion_mode is not None and promotion_mode != required_promotion_mode:
        errors.append(
            "promotion_mode does not match the workflow: "
            f"required={required_promotion_mode} actual={promotion_mode}"
        )
    if profile is not None and promotion_mode != profile["promotion_mode"]:
        errors.append(
            "promotion_mode does not match target profile: "
            f"required={profile['promotion_mode']} actual={promotion_mode}"
        )

    artifact_rows = indexed_rows(receipt["artifacts"], "role", "artifact roles", errors)
    expected_artifacts = set(manifest["required_artifact_roles"])
    set_difference_error("artifact roles", expected_artifacts, set(artifact_rows), errors)
    if operational:
        supplied_digest_roles = set(expected_artifact_digests or {})
        set_difference_error(
            "expected artifact digest roles",
            expected_artifacts,
            supplied_digest_roles,
            errors,
        )
    built_digests: dict[str, str] = {}
    for role in sorted(expected_artifacts & set(artifact_rows)):
        artifact = artifact_rows[role]
        built = artifact["built"]
        evidence_value(built["build_id"], f"artifacts.{role}.built.build_id", errors)
        built_source = evidence_value(
            built["source_revision"], f"artifacts.{role}.built.source_revision", errors
        )
        built_digest = evidence_value(
            built["image_digest"], f"artifacts.{role}.built.image_digest", errors
        )
        if built_digest is not None:
            built_digests[role] = built_digest
        if source_revision is not None:
            if built_source is not None and built_source != source_revision:
                errors.append(f"artifacts.{role}.built.source_revision must equal source.revision")
        expected_digest = (expected_artifact_digests or {}).get(role)
        if expected_digest is not None and built_digest is not None:
            if built_digest != expected_digest:
                errors.append(
                    f"artifacts.{role}.built.image_digest does not match workflow artifact"
                )

    schema_identity = receipt["schema_identity"]
    manifest_history = manifest["migration_history"]
    if schema_identity["migration_history_path"] != manifest_history["path"]:
        errors.append("schema_identity.migration_history_path does not match the release manifest")
    history_digest = evidence_value(
        schema_identity["migration_history_digest"],
        "schema_identity.migration_history_digest",
        errors,
    )
    actual_history_digest = sha256_path(repo_root / manifest_history["path"])
    if manifest_history["sha256"] != actual_history_digest:
        errors.append(
            "release manifest migration history digest is stale: "
            f"declared={manifest_history['sha256']} actual={actual_history_digest}"
        )
    if history_digest is not None and history_digest != actual_history_digest:
        errors.append("schema_identity.migration_history_digest does not match repository history")
    applied_revision = evidence_value(
        schema_identity["applied_revision"], "schema_identity.applied_revision", errors
    )
    expected_applied_revision = f"migration-ledger-through-{manifest_history['head']}"
    if (
        applied_revision is not None
        and applied_revision != expected_applied_revision
    ):
        errors.append(
            "schema_identity.applied_revision does not match migration authority: "
            f"required={expected_applied_revision} actual={applied_revision}"
        )
    rollback_compatibility = evidence_value(
        schema_identity["rollback_compatibility"],
        "schema_identity.rollback_compatibility",
        errors,
    )

    service_rows = indexed_rows(
        receipt["service_roles"], "role", "service roles", errors
    )
    expected_services = set(manifest["required_service_roles"])
    set_difference_error("service roles", expected_services, set(service_rows), errors)
    service_authority = manifest["service_targets"]
    set_difference_error(
        "service target roles", expected_services, set(service_authority), errors
    )
    for role in sorted(expected_services & set(service_rows)):
        service = service_rows[role]
        project_id = evidence_value(
            service["project_id"], f"service_roles.{role}.project_id", errors
        )
        service_id = evidence_value(
            service["service_id"], f"service_roles.{role}.service_id", errors
        )
        environment_id = evidence_value(
            service["environment_id"], f"service_roles.{role}.environment_id", errors
        )
        expected_service = service_authority.get(role)
        if expected_service is not None:
            if service["provider"] != expected_service["provider"]:
                errors.append(
                    f"service_roles.{role}.provider does not match release authority"
                )
            if project_id is not None and project_id != expected_service["project_id"]:
                errors.append(
                    f"service_roles.{role}.project_id does not match release authority"
                )
            if service_id is not None and service_id != expected_service["service_id"]:
                errors.append(
                    f"service_roles.{role}.service_id does not match release authority"
                )
            if (
                environment_id is not None
                and environment_id != expected_service["environment_id"]
            ):
                errors.append(
                    f"service_roles.{role}.environment_id does not match release authority"
                )

    check_rows: dict[tuple[str, int], dict[str, Any]] = {}
    duplicate_checks: set[tuple[str, int]] = set()
    for check in receipt["required_checks"]:
        key = (check["name"], check["app_id"])
        if key in check_rows:
            duplicate_checks.add(key)
        check_rows[key] = check
        run_id = evidence_value(
            check["run_id"], f"required_checks.{check['name']}.run_id", errors
        )
        check_source = evidence_value(
            check["source_revision"],
            f"required_checks.{check['name']}.source_revision",
            errors,
        )
        if run_id is not None and not str(run_id).isdigit():
            errors.append(f"required_checks.{check['name']}.run_id must be numeric")
        if source_revision is not None and check_source is not None:
            if check_source != source_revision:
                errors.append(
                    f"required_checks.{check['name']}.source_revision must equal source.revision"
                )
        if check["conclusion"] != "success":
            errors.append(f"required_checks.{check['name']} conclusion must be success")
    if duplicate_checks:
        errors.append(f"duplicate required check identities: {sorted(duplicate_checks)}")
    for required_check in manifest["required_checks"]:
        key = (required_check["name"], required_check["app_id"])
        check = check_rows.get(key)
        if check is None:
            errors.append(
                "missing required check identity: "
                f"{required_check['name']} app_id={required_check['app_id']}"
            )
        elif check["workflow"] != required_check["workflow"]:
            errors.append(
                f"required_checks.{required_check['name']}.workflow does not match authority"
            )

    dependency_rows = indexed_rows(
        receipt["dependencies"], "id", "dependency IDs", errors
    )
    expected_dependencies = set(manifest["dependencies"])
    set_difference_error(
        "dependency IDs", expected_dependencies, set(dependency_rows), errors
    )
    for dependency_id in sorted(expected_dependencies & set(dependency_rows)):
        dependency = dependency_rows[dependency_id]
        expected_required = manifest["dependencies"][dependency_id]["required"]
        if dependency["required"] is not expected_required:
            errors.append(
                f"dependencies.{dependency_id}.required does not match release authority"
            )
        state = evidence_value(
            dependency["state"],
            f"dependencies.{dependency_id}.state",
            errors,
            allow_not_applicable=not expected_required,
        )
        if expected_required and state is not None and state != "enabled":
            errors.append(f"dependencies.{dependency_id}.state must be enabled")

    asset_rows = indexed_rows(receipt["assets"], "id", "asset IDs", errors)
    expected_assets = set(manifest["assets"])
    set_difference_error("asset IDs", expected_assets, set(asset_rows), errors)
    for asset_id in sorted(expected_assets & set(asset_rows)):
        asset = asset_rows[asset_id]
        expected_required = manifest["assets"][asset_id]["required"]
        if asset["required"] is not expected_required:
            errors.append(f"assets.{asset_id}.required does not match release authority")
        allow_na = not expected_required
        evidence_value(
            asset["source"], f"assets.{asset_id}.source", errors, allow_not_applicable=allow_na
        )
        evidence_value(
            asset["integrity"],
            f"assets.{asset_id}.integrity",
            errors,
            allow_not_applicable=allow_na,
        )
        evidence_value(
            asset["retention"],
            f"assets.{asset_id}.retention",
            errors,
            allow_not_applicable=allow_na,
        )
        included = evidence_value(
            asset["release_inclusion"],
            f"assets.{asset_id}.release_inclusion",
            errors,
            allow_not_applicable=allow_na,
        )
        if expected_required and included is not None and included is not True:
            errors.append(f"assets.{asset_id}.release_inclusion must be true")

    rollback = receipt["rollback"]
    previous_source = evidence_value(
        rollback["previous_source_revision"],
        "rollback.previous_source_revision",
        errors,
    )
    for field in ("previous_deployment_id", "procedure"):
        evidence_value(rollback[field], f"rollback.{field}", errors)
    previous_artifact_digest = evidence_value(
        rollback["previous_artifact_digest"],
        "rollback.previous_artifact_digest",
        errors,
    )
    schema_restore = evidence_value(
        rollback["schema_restore"], "rollback.schema_restore", errors
    )
    tested_at = evidence_value(rollback["tested_at"], "rollback.tested_at", errors)
    tested_at_value = None
    if tested_at is not None:
        tested_at_value = validate_timestamp(tested_at, "rollback.tested_at", errors)
    if now is not None and tested_at_value is not None:
        rollback_age = (now - tested_at_value).total_seconds()
        if rollback_age < -policy["max_clock_skew_seconds"]:
            errors.append("rollback.tested_at is too far in the future")
        if rollback_age > policy["max_rollback_age_seconds"]:
            errors.append("rollback rehearsal is stale")
    if source_revision is not None and previous_source == source_revision:
        errors.append("rollback.previous_source_revision must differ from source.revision")
    if (
        previous_artifact_digest is not None
        and previous_artifact_digest in built_digests.values()
    ):
        errors.append(
            "rollback.previous_artifact_digest must differ from candidate artifact digests"
        )
    if rollback_compatibility == "verified-backward-compatible":
        if schema_restore != "not-required-backward-compatible":
            errors.append(
                "rollback.schema_restore must be not-required-backward-compatible "
                "when schema is verified backward-compatible"
            )
    if rollback_compatibility == "verified-forward-only-with-restore":
        if schema_restore != "verified-restore-rehearsal":
            errors.append(
                "rollback.schema_restore must be verified-restore-rehearsal "
                "for forward-only schema compatibility"
            )

    return errors


def apply_json_pointer(document: Any, operation: dict[str, Any]) -> None:
    parts = [
        part.replace("~1", "/").replace("~0", "~")
        for part in operation["path"].removeprefix("/").split("/")
        if part != ""
    ]
    if not parts:
        raise ReleaseReceiptError("mutation paths must address a receipt field")
    parent = document
    for part in parts[:-1]:
        parent = parent[int(part)] if isinstance(parent, list) else parent[part]
    leaf = parts[-1]
    op = operation["op"]
    if op == "remove":
        if isinstance(parent, list):
            del parent[int(leaf)]
        else:
            del parent[leaf]
    elif op in {"add", "replace"}:
        value = copy.deepcopy(operation["value"])
        if isinstance(parent, list):
            if leaf == "-":
                parent.append(value)
            elif op == "add":
                parent.insert(int(leaf), value)
            else:
                parent[int(leaf)] = value
        else:
            parent[leaf] = value
    else:
        raise ReleaseReceiptError(f"unsupported fixture mutation operation: {op}")


def validate_fixture_suite(repo_root: Path) -> tuple[int, int]:
    manifest = load_json(repo_root / MANIFEST_PATH)
    release_root = repo_root / RELEASE_ROOT
    receipt_count = 0
    for fixture in manifest["receipt_fixtures"]:
        receipt = load_json(release_root / fixture["path"])
        errors = validate_release_receipt(
            receipt,
            repo_root,
            required_promotion_mode=fixture["promotion_mode"],
        )
        eligible = not errors
        if eligible != fixture["expected_eligible"]:
            details = "; ".join(errors) if errors else "receipt unexpectedly eligible"
            raise ReleaseReceiptError(f"{fixture['path']}: {details}")
        receipt_count += 1

    mutation_fixture = load_json(release_root / manifest["mutation_cases"])
    base = load_json(release_root / mutation_fixture["base"])
    case_count = 0
    for case in mutation_fixture["cases"]:
        receipt = copy.deepcopy(base)
        for mutation in case["mutations"]:
            apply_json_pointer(receipt, mutation)
        errors = validate_release_receipt(receipt, repo_root)
        eligible = not errors
        if eligible != case["expected_eligible"]:
            details = "; ".join(errors) if errors else "receipt unexpectedly eligible"
            raise ReleaseReceiptError(f"mutation case {case['id']}: {details}")
        expected_error = case.get("expected_error")
        if expected_error and not any(expected_error in error for error in errors):
            raise ReleaseReceiptError(
                f"mutation case {case['id']}: missing expected error {expected_error!r}; "
                f"actual={errors}"
            )
        case_count += 1
    return receipt_count, case_count


def parse_role_values(values: list[str], label: str) -> dict[str, str]:
    parsed: dict[str, str] = {}
    for raw in values:
        role, separator, value = raw.partition("=")
        if not separator or not role or not value:
            raise ReleaseReceiptError(f"{label} must use ROLE=VALUE: {raw!r}")
        if role in parsed:
            raise ReleaseReceiptError(f"duplicate {label} role: {role}")
        if not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", role):
            raise ReleaseReceiptError(f"invalid {label} role: {role!r}")
        if label == "expected artifact digest" and not re.fullmatch(
            r"sha256:[0-9a-f]{64}", value
        ):
            raise ReleaseReceiptError(f"invalid {label} for {role}: {value!r}")
        parsed[role] = value
    return parsed


def emit_github_outputs(
    output_path: Path,
    receipt: dict[str, Any],
    repo_root: Path,
    target_profile: str | None,
) -> None:
    """Emit the manifest-bound deployment selector only after validation passes."""

    manifest = load_json(repo_root / MANIFEST_PATH)
    profile = manifest["workflow_targets"].get(target_profile or "")
    if profile is None or "deployment_service_role" not in profile:
        raise ReleaseReceiptError(
            "GitHub target output requires a workflow profile with deployment_service_role"
        )
    role = profile["deployment_service_role"]
    service = next(
        (row for row in receipt["service_roles"] if row["role"] == role), None
    )
    if service is None or service["provider"] != "railway":
        raise ReleaseReceiptError(
            "workflow deployment_service_role must resolve to a validated Railway service"
        )
    values = {
        "railway_project_id": service["project_id"]["value"],
        "railway_environment_id": service["environment_id"]["value"],
        "railway_service_id": service["service_id"]["value"],
    }
    try:
        with output_path.open("a", encoding="utf-8") as output:
            for key, value in values.items():
                output.write(f"{key}={value}\n")
    except OSError as error:
        raise ReleaseReceiptError(f"{output_path}: {error}") from error


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("receipt", nargs="?", type=Path)
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--expected-source")
    parser.add_argument(
        "--expected-release-tag",
        help="exact canonical vMAJOR.MINOR.PATCH tag authorized by the receipt",
    )
    parser.add_argument(
        "--expected-operation-id",
        help="short-lived GitHub run/attempt identity authorized by the receipt",
    )
    parser.add_argument(
        "--expected-workflow",
        choices=(".github/workflows/deploy.yaml", ".github/workflows/release.yml"),
    )
    parser.add_argument(
        "--now",
        help="explicit UTC RFC3339 validation clock used for freshness checks",
    )
    parser.add_argument(
        "--target-profile",
        help="manifest workflow target profile that the receipt must match exactly",
    )
    parser.add_argument(
        "--expected-artifact-digest",
        action="append",
        default=[],
        metavar="ROLE=SHA256",
        help="actual prebuilt or registry-read artifact digest; repeat for each role",
    )
    parser.add_argument(
        "--operational",
        action="store_true",
        help="reject test fixtures and require a target profile plus every artifact digest",
    )
    parser.add_argument(
        "--github-output",
        type=Path,
        help="append manifest-bound deployment target IDs after successful validation",
    )
    parser.add_argument(
        "--required-promotion-mode",
        choices=("source-redeploy", "immutable-image"),
    )
    parser.add_argument(
        "--validate-fixtures",
        action="store_true",
        help="validate all positive, incomplete, and mutation fixtures declared by the manifest",
    )
    args = parser.parse_args()
    if args.validate_fixtures == (args.receipt is not None):
        parser.error("provide exactly one receipt path or --validate-fixtures")
    if args.github_output is not None and not args.operational:
        parser.error("--github-output requires --operational")
    return args


def main() -> int:
    args = parse_args()
    repo_root = args.repo_root.resolve()
    try:
        if args.validate_fixtures:
            receipt_count, case_count = validate_fixture_suite(repo_root)
            print(
                "release receipt fixture suite valid: "
                f"receipts={receipt_count} mutation_cases={case_count}"
            )
            return 0

        receipt = load_json(args.receipt)
        if not isinstance(receipt, dict):
            raise ReleaseReceiptError(f"{args.receipt}: receipt must be a JSON object")
        expected_artifact_digests = parse_role_values(
            args.expected_artifact_digest, "expected artifact digest"
        )
        now = None
        if args.now is not None:
            timestamp_errors: list[str] = []
            now = validate_timestamp(args.now, "--now", timestamp_errors)
            if timestamp_errors or now is None:
                raise ReleaseReceiptError("; ".join(timestamp_errors))
        errors = validate_release_receipt(
            receipt,
            repo_root,
            expected_source=args.expected_source,
            required_promotion_mode=args.required_promotion_mode,
            target_profile=args.target_profile,
            expected_artifact_digests=expected_artifact_digests,
            expected_release_tag=args.expected_release_tag,
            expected_operation_id=args.expected_operation_id,
            expected_workflow=args.expected_workflow,
            now=now,
            operational=args.operational,
        )
    except ReleaseReceiptError as error:
        print(f"release receipt validation error: {error}", file=sys.stderr)
        return 2

    if errors:
        print("release receipt ineligible:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    if args.github_output is not None:
        try:
            emit_github_outputs(
                args.github_output,
                receipt,
                repo_root,
                args.target_profile,
            )
        except ReleaseReceiptError as error:
            print(f"release receipt validation error: {error}", file=sys.stderr)
            return 2

    source_revision = receipt["source"]["revision"]["value"]
    print(
        "release receipt eligible: "
        f"id={receipt['receipt_id']} source={source_revision} "
        f"mode={receipt['promotion_mode']} artifacts={len(receipt['artifacts'])} "
        f"assets={len(receipt['assets'])}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
