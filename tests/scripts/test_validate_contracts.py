from __future__ import annotations

import json
import shutil
import subprocess
from pathlib import Path

from .conftest import REPO_ROOT, run_python_script


AUTHORITY_ROOT = REPO_ROOT / "contracts" / "v1"


def run_validator(root: Path) -> subprocess.CompletedProcess[str]:
    return run_python_script("scripts/validate_contracts.py", "--root", root)


def copy_authority(tmp_path: Path) -> Path:
    destination = tmp_path / "contracts" / "v1"
    shutil.copytree(AUTHORITY_ROOT, destination)
    return destination


def read_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: object) -> None:
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def test_repository_contract_authority_is_valid() -> None:
    result = run_validator(AUTHORITY_ROOT)
    assert result.returncode == 0, result.stderr


def test_missing_manifest_schema_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    (authority / "schemas" / "error.schema.json").unlink()

    result = run_validator(authority)

    assert result.returncode != 0
    assert "error.schema.json" in result.stderr


def test_extra_manifest_schema_entry_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    manifest_path = authority / "manifest.json"
    manifest = read_json(manifest_path)
    schema_files = manifest["schemas"]
    assert isinstance(schema_files, list)
    manifest["schemas"] = [*schema_files, "schemas/bonus.schema.json"]
    write_json(manifest_path, manifest)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "bonus.schema.json" in result.stderr


def test_duplicate_manifest_schema_entry_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    manifest_path = authority / "manifest.json"
    manifest = read_json(manifest_path)
    schema_files = manifest["schemas"]
    assert isinstance(schema_files, list)
    manifest["schemas"] = [schema_files[0], *schema_files]
    write_json(manifest_path, manifest)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "duplicate" in result.stderr.lower()


def test_invalid_schema_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    schema_path = authority / "schemas" / "error.schema.json"
    schema = read_json(schema_path)
    schema["type"] = 7
    write_json(schema_path, schema)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "error.schema.json" in result.stderr


def test_invalid_fixture_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    write_json(authority / "fixtures" / "engine-result.json", {"contract_version": "v1"})

    result = run_validator(authority)

    assert result.returncode != 0
    assert "engine-result.json" in result.stderr


def test_unresolvable_local_reference_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    schema_path = authority / "schemas" / "engine-request.schema.json"
    schema = read_json(schema_path)
    properties = schema["properties"]
    assert isinstance(properties, dict)
    consent = properties["consent"]
    assert isinstance(consent, dict)
    consent["$ref"] = "missing-consent.schema.json"
    write_json(schema_path, schema)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "engine-request.schema.json" in result.stderr


def test_sensitive_diagnostic_key_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    fixture_path = authority / "fixtures" / "error.json"
    fixture = read_json(fixture_path)
    details = fixture["details"]
    assert isinstance(details, dict)
    details["api_token"] = "sensitive"
    write_json(fixture_path, fixture)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "error.json" in result.stderr
    assert "api_token" in result.stderr


def test_contract_version_drift_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    fixture_path = authority / "fixtures" / "engine-capability.json"
    fixture = read_json(fixture_path)
    fixture["contract_version"] = "v2"
    write_json(fixture_path, fixture)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "engine-capability.json" in result.stderr
    assert "contract_version" in result.stderr


def test_empty_fixture_manifest_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    manifest_path = authority / "manifest.json"
    manifest = read_json(manifest_path)
    manifest["fixtures"] = []
    write_json(manifest_path, manifest)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "fixtures" in result.stderr


def test_duplicate_fixture_entry_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    manifest_path = authority / "manifest.json"
    manifest = read_json(manifest_path)
    fixtures = manifest["fixtures"]
    assert isinstance(fixtures, list)
    manifest["fixtures"] = [fixtures[0], *fixtures]
    write_json(manifest_path, manifest)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "duplicate" in result.stderr.lower()


def test_fixture_path_cannot_escape_authority_root(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    outside = authority.parent / "outside.json"
    outside.write_text(
        (authority / "fixtures" / "engine-request.json").read_text(encoding="utf-8"),
        encoding="utf-8",
    )
    manifest_path = authority / "manifest.json"
    manifest = read_json(manifest_path)
    fixtures = manifest["fixtures"]
    assert isinstance(fixtures, list)
    assert isinstance(fixtures[0], dict)
    fixtures[0]["path"] = "../outside.json"
    write_json(manifest_path, manifest)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "../outside.json" in result.stderr


def test_broken_internal_fragment_reference_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    schema_path = authority / "schemas" / "engine-result.schema.json"
    schema = read_json(schema_path)
    properties = schema["properties"]
    assert isinstance(properties, dict)
    properties["latent_broken_field"] = {"$ref": "#/$defs/missingPrompt"}
    write_json(schema_path, schema)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "missingPrompt" in result.stderr


def test_negative_seed_is_not_a_canonical_v1_request(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    fixture_path = authority / "fixtures" / "engine-request.json"
    fixture = read_json(fixture_path)
    fixture["seed"] = -1
    write_json(fixture_path, fixture)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "seed" in result.stderr
