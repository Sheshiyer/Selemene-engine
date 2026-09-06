from __future__ import annotations

import base64
import copy
import json
import stat
import subprocess
import sys
from pathlib import Path

import yaml

from .conftest import REPO_ROOT, merged_env


SYNTHETIC_SOURCE = "1111111111111111111111111111111111111111"
RELEASE_FIXTURE_ROOT = REPO_ROOT / "contracts/release/v1/fixtures"


def test_gate_scripts_runs_contract_validator() -> None:
    scripts = json.loads((REPO_ROOT / "package.json").read_text(encoding="utf-8"))["scripts"]

    assert "python3 scripts/validate_contracts.py" in scripts["gate:scripts"]
    assert "python3 scripts/validate_release_receipt.py --validate-fixtures" in scripts["gate:scripts"]


def test_root_gate_runs_cross_language_contract_parity() -> None:
    scripts = json.loads((REPO_ROOT / "package.json").read_text(encoding="utf-8"))["scripts"]

    assert "gate:contracts" in scripts["gate"]
    assert "contract_v1_authority" in scripts["gate:contracts"]
    assert "openapi_schema_tests" in scripts["gate:contracts"]
    assert "integration_tests test_calculate_" in scripts["gate:contracts"]
    assert "@selemene/engine-sdk" in scripts["gate:contracts"]
    assert "@noesis/sdk" in scripts["gate:contracts"]


def bash_function(source: str, name: str) -> str:
    start = source.index(f"{name}() {{")
    end = source.index("\n}\n", start) + len("\n}\n")
    return source[start:end]


def workflow_step(source: str, name: str) -> str:
    start = source.index(f"- name: {name}")
    next_step = source.find("\n      - name:", start + 1)
    return source[start:] if next_step < 0 else source[start:next_step]


def test_suno_bridge_migration_step_fails_closed_and_uses_canonical_runner() -> None:
    source = (REPO_ROOT / "infra/suno-bridge/run-all.sh").read_text(encoding="utf-8")
    migrate = bash_function(source, "step_migrate")

    assert "DATABASE_URL not set" in migrate
    assert "return 1" in migrate
    assert "has_cmd psql" in migrate
    assert "psql CLI not installed" in migrate
    assert '"$REPO_ROOT/scripts/apply-migrations.sh"' in migrate
    assert '"$DATABASE_URL"' in migrate
    assert "sqlx migrate" not in migrate


def test_suno_bridge_run_fails_preflight_without_database_url(tmp_path: Path) -> None:
    fixture_script = tmp_path / "repo/infra/suno-bridge/run-all.sh"
    fixture_script.parent.mkdir(parents=True)
    fixture_script.write_text(
        (REPO_ROOT / "infra/suno-bridge/run-all.sh").read_text(encoding="utf-8"),
        encoding="utf-8",
    )
    (fixture_script.parent / ".env").write_text(
        "SUNO_COOKIE=offline-test-cookie\n", encoding="utf-8"
    )

    fake_bin = tmp_path / "bin"
    fake_bin.mkdir()
    for command in ("git", "gh", "bun", "curl", "psql"):
        executable = fake_bin / command
        executable.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
        executable.chmod(executable.stat().st_mode | stat.S_IEXEC)
    fake_timeout = fake_bin / "timeout"
    fake_timeout.write_text(
        '#!/usr/bin/env bash\nshift\nexec "$@"\n', encoding="utf-8"
    )
    fake_timeout.chmod(fake_timeout.stat().st_mode | stat.S_IEXEC)

    env = merged_env({"PATH": f"{fake_bin}:/usr/bin:/bin", "NO_COLOR": "1"})
    env.pop("DATABASE_URL", None)
    result = subprocess.run(
        ["bash", str(fixture_script), "--skip-bulk"],
        cwd=tmp_path / "repo",
        capture_output=True,
        text=True,
        check=False,
        env=env,
    )

    assert result.returncode != 0
    assert "DATABASE_URL not set" in f"{result.stdout}{result.stderr}"
    assert not (fixture_script.parent / ".run-state.json").exists()


def test_suno_bridge_reruns_migration_despite_persisted_done_state(tmp_path: Path) -> None:
    fixture_root = tmp_path / "repo"
    fixture_script = fixture_root / "infra/suno-bridge/run-all.sh"
    fixture_script.parent.mkdir(parents=True)
    fixture_script.write_text(
        (REPO_ROOT / "infra/suno-bridge/run-all.sh").read_text(encoding="utf-8"),
        encoding="utf-8",
    )
    (fixture_script.parent / ".env").write_text(
        "SUNO_COOKIE=offline-test-cookie\n", encoding="utf-8"
    )
    (fixture_script.parent / ".run-state.json").write_text(
        '{"step_1_deploy_bridge":"done","step_2_wire_env":"done",'
        '"step_3_migrate":"done","step_4_smoke_test":"done"}\n',
        encoding="utf-8",
    )

    migration_log = tmp_path / "migration-run.log"
    canonical_runner = fixture_root / "scripts/apply-migrations.sh"
    canonical_runner.parent.mkdir(parents=True)
    canonical_runner.write_text(
        '#!/usr/bin/env bash\nprintf "%s\\n" "$*" >> "$MIGRATION_RUN_LOG"\n',
        encoding="utf-8",
    )
    canonical_runner.chmod(canonical_runner.stat().st_mode | stat.S_IEXEC)

    fake_bin = tmp_path / "bin"
    fake_bin.mkdir()
    for command in ("git", "gh", "bun", "curl", "psql"):
        executable = fake_bin / command
        executable.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
        executable.chmod(executable.stat().st_mode | stat.S_IEXEC)

    fake_timeout = fake_bin / "timeout"
    fake_timeout.write_text(
        '#!/usr/bin/env bash\nshift\nexec "$@"\n', encoding="utf-8"
    )
    fake_timeout.chmod(fake_timeout.stat().st_mode | stat.S_IEXEC)

    result = subprocess.run(
        ["bash", str(fixture_script), "--skip-bulk"],
        cwd=fixture_root,
        capture_output=True,
        text=True,
        check=False,
        env=merged_env(
            {
                "PATH": f"{fake_bin}:/usr/bin:/bin",
                "NO_COLOR": "1",
                "DATABASE_URL": "postgresql://offline-test",
                "MIGRATION_RUN_LOG": str(migration_log),
            }
        ),
    )

    assert result.returncode == 0, f"{result.stdout}{result.stderr}"
    assert migration_log.is_file()
    assert "--migrations-dir" in migration_log.read_text(encoding="utf-8")


def test_ts_media_smoke_fails_on_health_or_contract_drift() -> None:
    source = (REPO_ROOT / ".github/workflows/test.yml").read_text(encoding="utf-8")
    smoke = workflow_step(source, "P1 W2 / T-020 media contract smoke")

    assert "set -euo pipefail" in smoke
    assert "trap cleanup EXIT" in smoke
    assert "for attempt in" in smoke
    assert "curl --fail-with-body --show-error" in smoke
    assert "jq -e" in smoke
    assert '.engine_id == "raaga"' in smoke
    assert ".generated_audio.strudel_ratios" in smoke
    assert '.engine_id == "sigil-forge"' in smoke
    assert ".result.method.id" in smoke
    assert '.result.svg_preview.status == "absent"' in smoke
    assert ".result.generated_image == null" in smoke
    assert ".result.svg_preview | type" not in smoke
    assert "|| echo" not in smoke
    assert "pkill" not in smoke


def test_python_biofield_smoke_fails_on_http_or_contract_drift() -> None:
    source = (REPO_ROOT / ".github/workflows/test.yml").read_text(encoding="utf-8")
    smoke = workflow_step(source, "Biofield sidecar smoke")

    assert "set -euo pipefail" in smoke
    assert "trap cleanup EXIT" in smoke
    assert "for attempt in" in smoke
    assert "curl --fail-with-body --show-error" in smoke
    assert "-F \"image=@" in smoke
    assert "jq -e" in smoke
    assert '.contract_version == "biofield-cv/v1"' in smoke
    assert '.analysis_version == "real-cv/v1"' in smoke
    assert ".metrics | keys | length == 11" in smoke
    assert ".algorithms_run | length == 11" in smoke
    assert ".quality_assessment.sufficient_quality | type == \"boolean\"" in smoke
    assert "pkill" not in smoke
    assert smoke.count("|| true") == 2


def test_deployment_and_release_require_both_image_builds() -> None:
    source = (REPO_ROOT / ".github/workflows/deploy.yaml").read_text(encoding="utf-8")
    railway = source[source.index("  deploy-railway:") : source.index("  smoke-test:")]
    release = source[source.index("  release:") :]

    assert "needs: [validate-source, validate-release-receipt, build-api, build-ts-engines]" in railway
    assert "build-ts-engines" in release.split("if: |", maxsplit=1)[0]
    assert "needs.build-ts-engines.result == 'success'" in release
    assert "sha-${{ needs.validate-source.outputs.source-sha }}" in release
    assert "github.ref_name" not in release


def load_workflow(name: str) -> dict:
    path = REPO_ROOT / ".github/workflows" / name
    document = yaml.safe_load(path.read_text(encoding="utf-8"))
    assert isinstance(document, dict)
    return document


def job_needs(job: dict) -> set[str]:
    needs = job.get("needs", [])
    if isinstance(needs, str):
        return {needs}
    return set(needs)


def transitively_needs(jobs: dict, job_name: str, required_job: str) -> bool:
    pending = list(job_needs(jobs[job_name]))
    visited: set[str] = set()
    while pending:
        dependency = pending.pop()
        if dependency == required_job:
            return True
        if dependency in visited or dependency not in jobs:
            continue
        visited.add(dependency)
        pending.extend(job_needs(jobs[dependency]))
    return False


def workflow_mutation_jobs(document: dict) -> set[str]:
    action_markers = (
        "docker/build-push-action@",
        "softprops/action-gh-release@",
        "stefanzweifel/git-auto-commit-action@",
    )
    run_markers = ("railway up ", "kubectl apply -f -")
    mutations: set[str] = set()
    for job_name, job in document["jobs"].items():
        for step in job.get("steps", []):
            uses = step.get("uses", "")
            run = step.get("run", "")
            if any(marker in uses for marker in action_markers) or any(
                marker in run for marker in run_markers
            ):
                mutations.add(job_name)
    return mutations


def workflow_step_by_name(document: dict, job_name: str, step_name: str) -> dict:
    return next(
        step
        for step in document["jobs"][job_name]["steps"]
        if step.get("name") == step_name
    )


def eligible_receipt(mode: str) -> dict:
    receipt = json.loads(
        (RELEASE_FIXTURE_ROOT / "eligible-source-redeploy.json").read_text(
            encoding="utf-8"
        )
    )
    if mode == "immutable-image":
        receipt["receipt_id"] = "fixture.immutable-image.workflow"
        receipt["promotion_mode"] = "immutable-image"
        receipt["target"]["provider_scope"] = ["github", "ghcr"]
        for artifact in receipt["artifacts"]:
            artifact["deployed"]["image_digest"] = copy.deepcopy(
                artifact["built"]["image_digest"]
            )
            artifact["deployed"]["image_digest"]["source"] = (
                "synthetic mocked workflow readback"
            )
    return receipt


def mocked_workflow_mutations(
    workflow_name: str,
    tmp_path: Path,
    receipt: dict | None,
) -> tuple[set[str], str]:
    document = load_workflow(workflow_name)
    materialize = workflow_step_by_name(
        document, "validate-release-receipt", "Materialize candidate release receipt"
    )["run"]
    validate = workflow_step_by_name(
        document, "validate-release-receipt", "Validate source-bound release receipt"
    )["run"]
    runner_temp = tmp_path / workflow_name.replace(".", "-")
    runner_temp.mkdir(exist_ok=True)
    env = merged_env(
        {
            "RUNNER_TEMP": str(runner_temp),
            "GITHUB_SHA": SYNTHETIC_SOURCE,
        }
    )
    if receipt is None:
        env.pop("RELEASE_RECEIPT_B64", None)
    else:
        encoded = base64.b64encode(json.dumps(receipt).encode("utf-8")).decode("ascii")
        env["RELEASE_RECEIPT_B64"] = encoded

    materialized = subprocess.run(
        ["bash", "-c", materialize],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
        env=env,
    )
    output = f"{materialized.stdout}{materialized.stderr}"
    if materialized.returncode != 0:
        return set(), output

    validated = subprocess.run(
        ["bash", "-c", validate],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
        env=env,
    )
    output += f"{validated.stdout}{validated.stderr}"
    if validated.returncode != 0:
        return set(), output
    return workflow_mutation_jobs(document), output


def test_every_release_mutation_depends_on_receipt_validation() -> None:
    expected = {
        "deploy.yaml": {
            "build-api",
            "build-ts-engines",
            "deploy",
            "deploy-railway",
            "release",
        },
        "release.yml": {
            "create-release",
            "build-docker",
            "build-binaries",
            "update-changelog",
        },
    }
    for workflow_name, expected_mutations in expected.items():
        document = load_workflow(workflow_name)
        jobs = document["jobs"]
        mutations = workflow_mutation_jobs(document)
        assert mutations == expected_mutations
        assert "if" not in jobs["validate-release-receipt"]
        for mutation_job in mutations:
            assert transitively_needs(jobs, mutation_job, "validate-release-receipt")


def test_workflow_dispatch_cannot_bypass_receipt_validation() -> None:
    source = (REPO_ROOT / ".github/workflows/deploy.yaml").read_text(encoding="utf-8")
    document = load_workflow("deploy.yaml")
    receipt_job = document["jobs"]["validate-release-receipt"]

    assert "workflow_dispatch:" in source
    assert receipt_job["needs"] == "validate-source"
    assert receipt_job["env"]["RELEASE_RECEIPT_B64"] == "${{ vars.RELEASE_RECEIPT_B64 }}"
    assert "fixtures/" not in json.dumps(receipt_job)


def test_release_workflows_retain_immutable_action_pins() -> None:
    for workflow_name in ("deploy.yaml", "release.yml"):
        result = subprocess.run(
            [
                sys.executable,
                str(REPO_ROOT / "scripts/validate_action_pins.py"),
                "--path",
                str(REPO_ROOT / ".github/workflows" / workflow_name),
            ],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        assert result.returncode == 0, f"{result.stdout}{result.stderr}"


def test_mocked_workflows_make_zero_mutations_without_valid_receipt(
    tmp_path: Path,
) -> None:
    workflows = {
        "deploy.yaml": "source-redeploy",
        "release.yml": "immutable-image",
    }
    for workflow_name, mode in workflows.items():
        mutations, output = mocked_workflow_mutations(
            workflow_name, tmp_path, None
        )
        assert mutations == set()
        assert "Release receipt missing" in output

        mismatched = eligible_receipt(mode)
        mismatched["source"]["revision"]["value"] = (
            "2222222222222222222222222222222222222222"
        )
        mismatched["source"]["validated_revision"]["value"] = (
            "2222222222222222222222222222222222222222"
        )
        mutations, output = mocked_workflow_mutations(
            workflow_name, tmp_path, mismatched
        )
        assert mutations == set()
        assert "does not match expected source" in output


def test_correct_receipts_reach_only_mocked_mutation_graph(tmp_path: Path) -> None:
    expected = {
        "deploy.yaml": {
            "build-api",
            "build-ts-engines",
            "deploy",
            "deploy-railway",
            "release",
        },
        "release.yml": {
            "create-release",
            "build-docker",
            "build-binaries",
            "update-changelog",
        },
    }
    modes = {"deploy.yaml": "source-redeploy", "release.yml": "immutable-image"}
    for workflow_name, expected_mutations in expected.items():
        mutations, output = mocked_workflow_mutations(
            workflow_name, tmp_path, eligible_receipt(modes[workflow_name])
        )
        assert mutations == expected_mutations, output
