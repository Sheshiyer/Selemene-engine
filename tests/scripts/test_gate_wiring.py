from __future__ import annotations

import json
from .conftest import REPO_ROOT


def test_gate_scripts_runs_contract_validator() -> None:
    scripts = json.loads((REPO_ROOT / "package.json").read_text(encoding="utf-8"))["scripts"]

    assert "python3 scripts/validate_contracts.py" in scripts["gate:scripts"]


def test_root_gate_runs_cross_language_contract_parity() -> None:
    scripts = json.loads((REPO_ROOT / "package.json").read_text(encoding="utf-8"))["scripts"]

    assert "gate:contracts" in scripts["gate"]
    assert "contract_v1_authority" in scripts["gate:contracts"]
    assert "openapi_schema_tests" in scripts["gate:contracts"]
    assert "integration_tests test_calculate_" in scripts["gate:contracts"]
    assert "@selemene/engine-sdk" in scripts["gate:contracts"]
    assert "@noesis/sdk" in scripts["gate:contracts"]
