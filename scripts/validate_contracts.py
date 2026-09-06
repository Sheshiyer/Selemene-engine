#!/usr/bin/env python3
"""Validate the canonical Selemene v1 contract authority without network access."""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import Counter
from pathlib import Path
from typing import Any
from urllib.parse import urldefrag, urljoin

from jsonschema import Draft202012Validator, FormatChecker
from jsonschema.exceptions import SchemaError
from referencing import Registry, Resource


EXPECTED_SCHEMAS = {
    "schemas/engine-request.schema.json",
    "schemas/engine-result.schema.json",
    "schemas/error.schema.json",
    "schemas/consent.schema.json",
    "schemas/provenance.schema.json",
    "schemas/engine-capability.schema.json",
}
EXPECTED_FIXTURES = {
    "fixtures/engine-request.json": "schemas/engine-request.schema.json",
    "fixtures/engine-request-legacy.json": "schemas/engine-request.schema.json",
    "fixtures/engine-result.json": "schemas/engine-result.schema.json",
    "fixtures/error.json": "schemas/error.schema.json",
    "fixtures/engine-capability.json": "schemas/engine-capability.schema.json",
}
EXPECTED_REGISTRIES = {"registries/engines.json"}
EXPECTED_RUNTIME_CLASSES = {
    "native": 12,
    "database-conditional": 1,
    "typescript": 6,
}
EXPECTED_DATABASE_CONDITIONAL_ID = "biofield-capture"
EXPECTED_PUBLIC_EXCLUSIONS = {"biofield-capture", "financial-biosensor"}
EXPECTED_EVIDENCE_AXES = (
    "declared",
    "implemented",
    "executable",
    "integrated",
    "deployed",
    "operational",
)
EXPECTED_EVIDENCE_STATUSES = {
    "evidenced",
    "partial",
    "absent",
    "unknown",
    "not-applicable",
}
EXPECTED_ISSUE_ROLES = {
    "authority_baseline",
    "runtime_registration",
    "golden_fixtures",
    "release_gate",
    "deployment_recovery",
}
RUNTIME_ID = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
SENSITIVE_KEY = re.compile(
    r"(^|_)(api_)?(token|secret|password|credential|stack|endpoint)(_|$)", re.IGNORECASE
)


class ContractValidationError(ValueError):
    """A bounded, operator-readable contract validation failure."""


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ContractValidationError(f"{path}: {error}") from error


def iter_refs(value: Any) -> list[str]:
    refs: list[str] = []
    if isinstance(value, dict):
        ref = value.get("$ref")
        if isinstance(ref, str):
            refs.append(ref)
        for nested in value.values():
            refs.extend(iter_refs(nested))
    elif isinstance(value, list):
        for nested in value:
            refs.extend(iter_refs(nested))
    return refs


def sensitive_keys(value: Any, prefix: str = "$") -> list[str]:
    findings: list[str] = []
    if isinstance(value, dict):
        for key, nested in value.items():
            child = f"{prefix}.{key}"
            if SENSITIVE_KEY.search(key):
                findings.append(child)
            findings.extend(sensitive_keys(nested, child))
    elif isinstance(value, list):
        for index, nested in enumerate(value):
            findings.extend(sensitive_keys(nested, f"{prefix}[{index}]"))
    return findings


def authority_path(root: Path, relative: str) -> Path:
    if Path(relative).is_absolute():
        raise ContractValidationError(f"{relative}: authority path must be relative")
    resolved_root = root.resolve()
    resolved = (resolved_root / relative).resolve()
    try:
        resolved.relative_to(resolved_root)
    except ValueError as error:
        raise ContractValidationError(
            f"{relative}: path escapes contract authority root"
        ) from error
    return resolved


def resolve_fragment(document: Any, fragment: str, context: str) -> None:
    if not fragment:
        return
    if not fragment.startswith("/"):
        raise ContractValidationError(f"{context}: unsupported fragment #{fragment}")
    current = document
    for raw_part in fragment.removeprefix("/").split("/"):
        part = raw_part.replace("~1", "/").replace("~0", "~")
        if isinstance(current, dict) and part in current:
            current = current[part]
        elif isinstance(current, list) and part.isdigit() and int(part) < len(current):
            current = current[int(part)]
        else:
            raise ContractValidationError(f"{context}: unresolved fragment #{fragment}")


def validate_engine_registry(path: Path) -> int:
    registry = load_json(path)
    if not isinstance(registry, dict):
        raise ContractValidationError(f"{path}: engine registry must be an object")
    if registry.get("registry_version") != "v1":
        raise ContractValidationError(f"{path}: registry_version must be v1")
    if registry.get("contract_version") != "v1":
        raise ContractValidationError(f"{path}: contract_version must be v1")

    counts = registry.get("counts")
    if not isinstance(counts, dict):
        raise ContractValidationError(f"{path}: counts must be an object")
    expected_count_keys = {"runtime_ids", "public_mirror_groups", "runtime_classes"}
    if set(counts) != expected_count_keys:
        raise ContractValidationError(
            f"{path}: counts must contain exactly {sorted(expected_count_keys)}"
        )
    if counts.get("runtime_ids") != 19:
        raise ContractValidationError(f"{path}: declared runtime_ids count must be 19")
    if counts.get("public_mirror_groups") != 17:
        raise ContractValidationError(
            f"{path}: declared public_mirror_groups count must be 17"
        )
    if counts.get("runtime_classes") != EXPECTED_RUNTIME_CLASSES:
        raise ContractValidationError(
            f"{path}: declared runtime_classes must equal {EXPECTED_RUNTIME_CLASSES}"
        )

    axes = registry.get("evidence_axes")
    if axes != list(EXPECTED_EVIDENCE_AXES):
        raise ContractValidationError(
            f"{path}: evidence_axes must equal {list(EXPECTED_EVIDENCE_AXES)}"
        )
    statuses = registry.get("evidence_statuses")
    if not isinstance(statuses, list) or set(statuses) != EXPECTED_EVIDENCE_STATUSES:
        raise ContractValidationError(
            f"{path}: evidence_statuses must equal {sorted(EXPECTED_EVIDENCE_STATUSES)}"
        )

    issue_source = registry.get("issue_source")
    if (
        not isinstance(issue_source, str)
        or not issue_source
        or Path(issue_source).is_absolute()
        or ".." in Path(issue_source).parts
    ):
        raise ContractValidationError(
            f"{path}: issue_source must be a safe repository-relative path"
        )

    rows = registry.get("engines")
    if not isinstance(rows, list):
        raise ContractValidationError(f"{path}: engines must be an array")
    if len(rows) != counts["runtime_ids"]:
        raise ContractValidationError(
            f"{path}: runtime ID count mismatch; expected={counts['runtime_ids']} actual={len(rows)}"
        )

    row_keys = {
        "id",
        "display_name",
        "runtime_class",
        "owner",
        "public_mirror_group",
        "public_mirror_exclusion",
        "issue_ids",
        "evidence",
    }
    seen_ids: set[str] = set()
    seen_groups: set[str] = set()
    excluded_ids: set[str] = set()
    all_issue_ids: set[int] = set()
    class_counts: Counter[str] = Counter()

    for index, row in enumerate(rows):
        context = f"{path}: engines[{index}]"
        if not isinstance(row, dict) or set(row) != row_keys:
            raise ContractValidationError(
                f"{context}: row must contain exactly {sorted(row_keys)}"
            )

        engine_id = row["id"]
        if not isinstance(engine_id, str) or not RUNTIME_ID.fullmatch(engine_id):
            raise ContractValidationError(
                f"{context}: id must be a lowercase kebab-case runtime ID"
            )
        if engine_id in seen_ids:
            raise ContractValidationError(
                f"{context}: duplicate runtime ID {engine_id}"
            )
        seen_ids.add(engine_id)

        display_name = row["display_name"]
        if not isinstance(display_name, str) or not display_name.strip():
            raise ContractValidationError(f"{context}: display_name must be nonempty")

        runtime_class = row["runtime_class"]
        if runtime_class not in EXPECTED_RUNTIME_CLASSES:
            raise ContractValidationError(
                f"{context}: unsupported runtime class {runtime_class!r}"
            )
        class_counts[runtime_class] += 1
        if (
            runtime_class == "database-conditional"
            and engine_id != EXPECTED_DATABASE_CONDITIONAL_ID
        ):
            raise ContractValidationError(
                f"{context}: database-conditional runtime must be {EXPECTED_DATABASE_CONDITIONAL_ID}"
            )

        owner = row["owner"]
        if (
            not isinstance(owner, str)
            or not owner
            or Path(owner).is_absolute()
            or ".." in Path(owner).parts
        ):
            raise ContractValidationError(
                f"{context}: owner must be a safe repository-relative path"
            )

        public_group = row["public_mirror_group"]
        exclusion = row["public_mirror_exclusion"]
        if public_group is None:
            if not isinstance(exclusion, str) or not exclusion.strip():
                raise ContractValidationError(
                    f"{context}: excluded runtime needs public_mirror_exclusion"
                )
            excluded_ids.add(engine_id)
        else:
            if not isinstance(public_group, str) or not RUNTIME_ID.fullmatch(
                public_group
            ):
                raise ContractValidationError(
                    f"{context}: public_mirror_group must be null or lowercase kebab-case"
                )
            if exclusion is not None:
                raise ContractValidationError(
                    f"{context}: included public mirror must not have an exclusion reason"
                )
            if public_group in seen_groups:
                raise ContractValidationError(
                    f"{context}: duplicate public mirror group {public_group}"
                )
            seen_groups.add(public_group)

        issues = row["issue_ids"]
        if not isinstance(issues, dict) or set(issues) != EXPECTED_ISSUE_ROLES:
            raise ContractValidationError(
                f"{context}: issue_ids must contain exactly {sorted(EXPECTED_ISSUE_ROLES)}"
            )
        issue_ids = list(issues.values())
        if not all(
            isinstance(issue_id, int) and issue_id > 0 for issue_id in issue_ids
        ):
            raise ContractValidationError(
                f"{context}: issue IDs must be positive integers"
            )
        if len(issue_ids) != len(set(issue_ids)):
            raise ContractValidationError(
                f"{context}: duplicate issue ID within runtime row"
            )
        duplicates = sorted(set(issue_ids) & all_issue_ids)
        if duplicates:
            raise ContractValidationError(
                f"{context}: issue IDs reused across runtime rows: {duplicates}"
            )
        all_issue_ids.update(issue_ids)

        evidence = row["evidence"]
        if not isinstance(evidence, dict) or set(evidence) != set(
            EXPECTED_EVIDENCE_AXES
        ):
            raise ContractValidationError(
                f"{context}: evidence must contain all six axes {list(EXPECTED_EVIDENCE_AXES)}"
            )
        for axis in EXPECTED_EVIDENCE_AXES:
            cell = evidence[axis]
            axis_context = f"{context}.evidence.{axis}"
            if not isinstance(cell, dict) or set(cell) != {"status", "references"}:
                raise ContractValidationError(
                    f"{axis_context}: evidence cell needs status and references"
                )
            status = cell["status"]
            references = cell["references"]
            if status not in EXPECTED_EVIDENCE_STATUSES:
                raise ContractValidationError(
                    f"{axis_context}: unsupported evidence status {status!r}"
                )
            if (
                not isinstance(references, list)
                or not all(isinstance(reference, str) for reference in references)
                or len(references) != len(set(references))
            ):
                raise ContractValidationError(
                    f"{axis_context}: references must be a unique string array"
                )
            if status not in {"unknown", "not-applicable"} and not references:
                raise ContractValidationError(
                    f"{axis_context}: {status} status requires at least one reference"
                )
            if any(not reference.startswith("repo://") for reference in references):
                raise ContractValidationError(
                    f"{axis_context}: references must use repo:// provenance"
                )

    ids_in_order = [row["id"] for row in rows]
    if ids_in_order != sorted(ids_in_order):
        raise ContractValidationError(
            f"{path}: engine rows must be sorted by runtime ID"
        )
    if class_counts != Counter(EXPECTED_RUNTIME_CLASSES):
        raise ContractValidationError(
            f"{path}: runtime class count mismatch; expected={EXPECTED_RUNTIME_CLASSES} actual={dict(class_counts)}"
        )
    if len(seen_groups) != counts["public_mirror_groups"]:
        raise ContractValidationError(
            f"{path}: public mirror group count mismatch; expected={counts['public_mirror_groups']} actual={len(seen_groups)}"
        )
    if excluded_ids != EXPECTED_PUBLIC_EXCLUSIONS:
        raise ContractValidationError(
            f"{path}: public mirror exclusions must be {sorted(EXPECTED_PUBLIC_EXCLUSIONS)}; actual={sorted(excluded_ids)}"
        )

    return len(rows)


def validate_authority(root: Path) -> tuple[int, int, int]:
    manifest_path = root / "manifest.json"
    manifest = load_json(manifest_path)
    if not isinstance(manifest, dict):
        raise ContractValidationError(f"{manifest_path}: manifest must be an object")
    if manifest.get("contract_version") != "v1":
        raise ContractValidationError(f"{manifest_path}: contract_version must be v1")

    registry_entries = manifest.get("registries")
    if not isinstance(registry_entries, list) or not all(
        isinstance(item, str) for item in registry_entries
    ):
        raise ContractValidationError(
            f"{manifest_path}: registries must be a string array"
        )
    if len(registry_entries) != len(set(registry_entries)):
        raise ContractValidationError(f"{manifest_path}: duplicate registry entry")
    registry_entry_set = set(registry_entries)
    if registry_entry_set != EXPECTED_REGISTRIES:
        missing = sorted(EXPECTED_REGISTRIES - registry_entry_set)
        unexpected = sorted(registry_entry_set - EXPECTED_REGISTRIES)
        raise ContractValidationError(
            f"{manifest_path}: registry manifest drift; missing={missing}, unexpected={unexpected}"
        )
    registry_row_count = sum(
        validate_engine_registry(authority_path(root, relative))
        for relative in registry_entries
    )

    schema_entries = manifest.get("schemas")
    if not isinstance(schema_entries, list) or not all(
        isinstance(item, str) for item in schema_entries
    ):
        raise ContractValidationError(
            f"{manifest_path}: schemas must be a string array"
        )
    if len(schema_entries) != len(set(schema_entries)):
        raise ContractValidationError(f"{manifest_path}: duplicate schema entry")
    entry_set = set(schema_entries)
    if entry_set != EXPECTED_SCHEMAS:
        missing = sorted(EXPECTED_SCHEMAS - entry_set)
        unexpected = sorted(entry_set - EXPECTED_SCHEMAS)
        raise ContractValidationError(
            f"{manifest_path}: schema manifest drift; missing={missing}, unexpected={unexpected}"
        )

    schemas: dict[str, dict[str, Any]] = {}
    schema_paths: dict[str, Path] = {}
    for relative in schema_entries:
        path = authority_path(root, relative)
        schema = load_json(path)
        if not isinstance(schema, dict):
            raise ContractValidationError(f"{path}: schema must be an object")
        try:
            Draft202012Validator.check_schema(schema)
        except SchemaError as error:
            raise ContractValidationError(
                f"{path}: invalid Draft 2020-12 schema: {error.message}"
            ) from error
        schema_id = schema.get("$id")
        if not isinstance(schema_id, str) or not schema_id:
            raise ContractValidationError(f"{path}: nonempty $id is required")
        if schema_id in schemas:
            raise ContractValidationError(f"{path}: duplicate $id {schema_id}")
        schemas[schema_id] = schema
        schema_paths[schema_id] = path

    known_ids = set(schemas)
    for schema_id, schema in schemas.items():
        for reference in iter_refs(schema):
            target, fragment = urldefrag(urljoin(schema_id, reference))
            if target not in known_ids:
                raise ContractValidationError(
                    f"{schema_paths[schema_id]}: unresolved local $ref {reference}"
                )
            resolve_fragment(
                schemas[target],
                fragment,
                f"{schema_paths[schema_id]}: $ref {reference}",
            )

    registry = Registry().with_resources(
        (schema_id, Resource.from_contents(schema))
        for schema_id, schema in schemas.items()
    )
    fixtures = manifest.get("fixtures")
    if not isinstance(fixtures, list):
        raise ContractValidationError(f"{manifest_path}: fixtures must be an array")

    fixture_bindings: list[tuple[str, str]] = []
    for entry in fixtures:
        if (
            isinstance(entry, dict)
            and isinstance(entry.get("path"), str)
            and isinstance(entry.get("schema"), str)
        ):
            fixture_bindings.append((entry["path"], entry["schema"]))
    if len(fixture_bindings) != len(set(fixture_bindings)):
        raise ContractValidationError(f"{manifest_path}: duplicate fixture entry")
    for fixture_relative, schema_relative in fixture_bindings:
        authority_path(root, fixture_relative)
        authority_path(root, schema_relative)
    if dict(fixture_bindings) != EXPECTED_FIXTURES or len(fixture_bindings) != len(
        EXPECTED_FIXTURES
    ):
        raise ContractValidationError(
            f"{manifest_path}: fixtures must exactly match {sorted(EXPECTED_FIXTURES)}"
        )

    for entry in fixtures:
        if not isinstance(entry, dict) or set(entry) != {"path", "schema"}:
            raise ContractValidationError(
                f"{manifest_path}: each fixture needs path and schema"
            )
        fixture_relative = entry["path"]
        schema_relative = entry["schema"]
        if not isinstance(fixture_relative, str) or not isinstance(
            schema_relative, str
        ):
            raise ContractValidationError(
                f"{manifest_path}: fixture path and schema must be strings"
            )
        if schema_relative not in entry_set:
            raise ContractValidationError(
                f"{manifest_path}: unknown fixture schema {schema_relative}"
            )
        fixture_path = authority_path(root, fixture_relative)
        fixture = load_json(fixture_path)
        schema_path = authority_path(root, schema_relative)
        schema = next(
            value for key, value in schemas.items() if schema_paths[key] == schema_path
        )
        validator = Draft202012Validator(
            schema, registry=registry, format_checker=FormatChecker()
        )
        errors = sorted(
            validator.iter_errors(fixture), key=lambda error: list(error.absolute_path)
        )
        if errors:
            first = errors[0]
            location = ".".join(str(part) for part in first.absolute_path) or "$"
            raise ContractValidationError(
                f"{fixture_path}: {location}: {first.message}"
            )

        diagnostic_value: Any | None = None
        if schema_relative == "schemas/error.schema.json":
            diagnostic_value = fixture
        elif schema_relative == "schemas/engine-result.schema.json" and isinstance(
            fixture, dict
        ):
            diagnostic_value = fixture.get("provenance")
        if diagnostic_value is not None:
            findings = sensitive_keys(diagnostic_value)
            if findings:
                raise ContractValidationError(
                    f"{fixture_path}: sensitive diagnostic key(s): {', '.join(findings)}"
                )

    return len(schemas), len(fixtures), registry_row_count


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parents[1] / "contracts" / "v1",
        help="contract authority root (default: repository contracts/v1)",
    )
    args = parser.parse_args()
    try:
        schema_count, fixture_count, registry_row_count = validate_authority(
            args.root.resolve()
        )
    except ContractValidationError as error:
        print(f"contract validation failed: {error}", file=sys.stderr)
        return 1
    print(
        "contract authority v1 valid: "
        f"schemas={schema_count} fixtures={fixture_count} "
        f"registries={len(EXPECTED_REGISTRIES)} engines={registry_row_count}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
