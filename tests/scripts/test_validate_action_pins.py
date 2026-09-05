from pathlib import Path
import textwrap
import importlib.util

MODULE_PATH = Path(__file__).resolve().parents[2] / "scripts" / "validate_action_pins.py"
SPEC = importlib.util.spec_from_file_location("validate_action_pins", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
validate_action_pins = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(validate_action_pins)

PINNED_SHA = "a" * 40
DOCKER_DIGEST = "b" * 64


def write_workflow(tmp_path: Path, name: str, body: str) -> Path:
    workflows = tmp_path / ".github" / "workflows"
    workflows.mkdir(parents=True, exist_ok=True)
    path = workflows / name
    path.write_text(textwrap.dedent(body).strip() + "\n", encoding="utf-8")
    return workflows


def test_accepts_local_and_pinned_refs(tmp_path: Path):
    workflow_dir = write_workflow(
        tmp_path,
        "valid.yml",
        f"""
        name: valid
        jobs:
          reusable:
            uses: ./.github/workflows/reusable.yml
          test:
            runs-on: ubuntu-latest
            steps:
              - uses: owner/example/action@{PINNED_SHA}
              - uses: ./local-action
              - uses: docker://ghcr.io/library/example@sha256:{DOCKER_DIGEST}
              - nested:
                  uses: owner/repo/action/path@{PINNED_SHA}
        """,
    )
    assert validate_action_pins.find_invalid_refs(workflow_dir) == []


def test_scans_nested_job_structures(tmp_path: Path):
    workflow_dir = write_workflow(
        tmp_path,
        "nested.yml",
        f"""
        jobs:
          valid:
            runs-on: ubuntu-latest
            steps:
              - uses: actions/checkout@{PINNED_SHA}
          nested:
            strategy:
              matrix:
                include:
                  - metadata:
                      uses: actions/setup-node@{PINNED_SHA}
        """,
    )
    assert validate_action_pins.find_invalid_refs(workflow_dir) == []


def test_rejects_branch_and_tag_refs(tmp_path: Path):
    workflow_dir = write_workflow(
        tmp_path,
        "bad.yml",
        """
        jobs:
          invalid:
            runs-on: ubuntu-latest
            steps:
              - uses: actions/checkout@v4
              - uses: owner/example/action@main
        """,
    )
    invalid_refs = {uses_ref for _, uses_ref in validate_action_pins.find_invalid_refs(workflow_dir)}
    assert "actions/checkout@v4" in invalid_refs
    assert "owner/example/action@main" in invalid_refs


def test_rejects_expression_based_pin(tmp_path: Path):
    workflow_dir = write_workflow(
        tmp_path,
        "expression.yml",
        """
        jobs:
          invalid:
            runs-on: ubuntu-latest
            steps:
              - uses: actions/checkout@${{ github.sha }}
        """,
    )
    invalid_refs = {uses_ref for _, uses_ref in validate_action_pins.find_invalid_refs(workflow_dir)}
    assert "actions/checkout@${{ github.sha }}" in invalid_refs


def test_rejects_docker_images_without_digest(tmp_path: Path):
    workflow_dir = write_workflow(
        tmp_path,
        "docker.yml",
        f"""
        jobs:
          invalid:
            runs-on: ubuntu-latest
            steps:
              - uses: docker://alpine:3.20
              - uses: docker://ghcr.io/library/example@sha256:{DOCKER_DIGEST}
        """,
    )
    invalid_refs = [uses_ref for _, uses_ref in validate_action_pins.find_invalid_refs(workflow_dir)]
    assert invalid_refs == ["docker://alpine:3.20"]


def test_missing_workflow_directory_is_rejected(tmp_path: Path):
    missing = tmp_path / "missing-workflows"
    assert validate_action_pins.main(["--path", str(missing)]) == 1
