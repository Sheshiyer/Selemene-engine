#!/usr/bin/env python3
"""Validate the canonical Selemene v1 contract authority without network access."""

from __future__ import annotations

import argparse
import json
import re
import sys
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


def validate_authority(root: Path) -> tuple[int, int]:
    manifest_path = root / "manifest.json"
    manifest = load_json(manifest_path)
    if not isinstance(manifest, dict):
        raise ContractValidationError(f"{manifest_path}: manifest must be an object")
    if manifest.get("contract_version") != "v1":
        raise ContractValidationError(f"{manifest_path}: contract_version must be v1")

    schema_entries = manifest.get("schemas")
    if not isinstance(schema_entries, list) or not all(isinstance(item, str) for item in schema_entries):
        raise ContractValidationError(f"{manifest_path}: schemas must be a string array")
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
        path = root / relative
        schema = load_json(path)
        if not isinstance(schema, dict):
            raise ContractValidationError(f"{path}: schema must be an object")
        try:
            Draft202012Validator.check_schema(schema)
        except SchemaError as error:
            raise ContractValidationError(f"{path}: invalid Draft 2020-12 schema: {error.message}") from error
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
            if reference.startswith("#"):
                continue
            target, _ = urldefrag(urljoin(schema_id, reference))
            if target not in known_ids:
                raise ContractValidationError(
                    f"{schema_paths[schema_id]}: unresolved local $ref {reference}"
                )

    registry = Registry().with_resources(
        (schema_id, Resource.from_contents(schema)) for schema_id, schema in schemas.items()
    )
    fixtures = manifest.get("fixtures")
    if not isinstance(fixtures, list):
        raise ContractValidationError(f"{manifest_path}: fixtures must be an array")

    for entry in fixtures:
        if not isinstance(entry, dict) or set(entry) != {"path", "schema"}:
            raise ContractValidationError(f"{manifest_path}: each fixture needs path and schema")
        fixture_relative = entry["path"]
        schema_relative = entry["schema"]
        if not isinstance(fixture_relative, str) or not isinstance(schema_relative, str):
            raise ContractValidationError(f"{manifest_path}: fixture path and schema must be strings")
        if schema_relative not in entry_set:
            raise ContractValidationError(f"{manifest_path}: unknown fixture schema {schema_relative}")
        fixture_path = root / fixture_relative
        fixture = load_json(fixture_path)
        schema_path = root / schema_relative
        schema = next(value for key, value in schemas.items() if schema_paths[key] == schema_path)
        validator = Draft202012Validator(schema, registry=registry, format_checker=FormatChecker())
        errors = sorted(validator.iter_errors(fixture), key=lambda error: list(error.absolute_path))
        if errors:
            first = errors[0]
            location = ".".join(str(part) for part in first.absolute_path) or "$"
            raise ContractValidationError(f"{fixture_path}: {location}: {first.message}")

        diagnostic_value: Any | None = None
        if schema_relative == "schemas/error.schema.json":
            diagnostic_value = fixture
        elif schema_relative == "schemas/engine-result.schema.json" and isinstance(fixture, dict):
            diagnostic_value = fixture.get("provenance")
        if diagnostic_value is not None:
            findings = sensitive_keys(diagnostic_value)
            if findings:
                raise ContractValidationError(
                    f"{fixture_path}: sensitive diagnostic key(s): {', '.join(findings)}"
                )

    return len(schemas), len(fixtures)


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
        schema_count, fixture_count = validate_authority(args.root.resolve())
    except ContractValidationError as error:
        print(f"contract validation failed: {error}", file=sys.stderr)
        return 1
    print(f"contract authority v1 valid: schemas={schema_count} fixtures={fixture_count}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
