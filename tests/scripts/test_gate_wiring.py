from __future__ import annotations

import base64
import copy
import json
import stat
import subprocess
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

import yaml

from .conftest import REPO_ROOT, merged_env


SYNTHETIC_SOURCE = "1111111111111111111111111111111111111111"
API_DIGEST = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
TS_DIGEST = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
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


def test_deployment_requires_both_prebuilt_image_digests() -> None:
    source = (REPO_ROOT / ".github/workflows/deploy.yaml").read_text(encoding="utf-8")
    railway = source[source.index("  deploy-railway:") : source.index("  smoke-test:")]

    assert "needs: [validate-source, validate-release-receipt, build-api, build-ts-engines]" in railway
    assert "needs: [validate-source, prebuild-api, prebuild-ts-engines]" in source
    assert '--expected-artifact-digest "api=${EXPECTED_API_DIGEST}"' in source
    assert (
        '--expected-artifact-digest "typescript-engines=${EXPECTED_TS_DIGEST}"'
        in source
    )


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
    action_markers = ("softprops/action-gh-release@",)
    run_markers = (
        "railway up ",
        "kubectl apply -f -",
        "docker push ",
        "docker buildx imagetools create",
    )
    mutations: set[str] = set()
    for job_name, job in document["jobs"].items():
        for step in job.get("steps", []):
            uses = step.get("uses", "")
            run = step.get("run", "")
            pushes_image = (
                "docker/build-push-action@" in uses
                and step.get("with", {}).get("push") is True
            )
            if pushes_image or any(marker in uses for marker in action_markers) or any(
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


def eligible_receipt(workflow_name: str) -> dict:
    receipt = json.loads(
        (RELEASE_FIXTURE_ROOT / "eligible-source-redeploy.json").read_text(
            encoding="utf-8"
        )
    )
    receipt.pop("test_fixture")
    receipt["receipt_id"] = f"mock.{workflow_name.replace('.', '-')}.operational"
    now = datetime.now(timezone.utc).replace(microsecond=0)
    receipt["issued_at"] = (now - timedelta(seconds=30)).strftime("%Y-%m-%dT%H:%M:%SZ")
    receipt["expires_at"] = (now + timedelta(minutes=10)).strftime("%Y-%m-%dT%H:%M:%SZ")
    receipt["operation"] = {
        "id": "github-run-90000000001-attempt-1",
        "workflow": f".github/workflows/{workflow_name}",
        "run_id": "90000000001",
        "run_attempt": 1,
    }
    receipt["target"]["environment"] = "production"
    expected_digests = {"api": API_DIGEST, "typescript-engines": TS_DIGEST}
    for artifact in receipt["artifacts"]:
        artifact["built"]["image_digest"]["value"] = expected_digests[artifact["role"]]
    if workflow_name == "release.yml":
        receipt["promotion_mode"] = "immutable-image"
        receipt["release"]["tag"] = "v1.2.3"
        receipt["target"]["provider_scope"] = ["github", "ghcr"]
    else:
        receipt["promotion_mode"] = "source-redeploy"
        receipt["target"]["provider_scope"] = ["ghcr", "railway"]
    return receipt


class MutationSpy:
    def __init__(self) -> None:
        self.calls: list[str] = []

    def call(self, job_name: str) -> None:
        self.calls.append(job_name)


def execute_mocked_mutation_graph(document: dict, spy: MutationSpy) -> None:
    jobs = document["jobs"]
    mutation_jobs = workflow_mutation_jobs(document)
    completed = set(jobs) - mutation_jobs
    remaining = set(mutation_jobs)
    while remaining:
        ready = sorted(
            job_name
            for job_name in remaining
            if job_needs(jobs[job_name]) <= completed
        )
        if not ready:
            raise AssertionError(f"mocked mutation graph is blocked: {sorted(remaining)}")
        for job_name in ready:
            spy.call(job_name)
            completed.add(job_name)
            remaining.remove(job_name)


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
            "GITHUB_REF_NAME": "v1.2.3",
            "GITHUB_RUN_ID": "90000000001",
            "GITHUB_RUN_ATTEMPT": "1",
            "REQUESTED_ENVIRONMENT": "production",
            "ENABLE_K8S_DEPLOY": "false",
            "EXPECTED_API_DIGEST": API_DIGEST,
            "EXPECTED_TS_DIGEST": TS_DIGEST,
            "GITHUB_OUTPUT": str(runner_temp / "github-output"),
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
    spy = MutationSpy()
    execute_mocked_mutation_graph(document, spy)
    return set(spy.calls), output


def test_every_release_mutation_depends_on_receipt_validation() -> None:
    expected = {
        "deploy.yaml": {
            "build-api",
            "build-ts-engines",
            "deploy",
            "deploy-railway",
        },
        "release.yml": set(),
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
    assert set(receipt_job["needs"]) == {
        "validate-source",
        "prebuild-api",
        "prebuild-ts-engines",
    }
    assert (
        receipt_job["env"]["RELEASE_RECEIPT_B64"]
        == "${{ vars.DEPLOY_RELEASE_RECEIPT_B64 }}"
    )
    assert "--operational" in json.dumps(receipt_job)
    assert "--target-profile deploy-production" in json.dumps(receipt_job)
    assert "fixtures/" not in json.dumps(receipt_job)


def test_semver_tags_have_one_authoritative_publication_workflow() -> None:
    deploy_source = (REPO_ROOT / ".github/workflows/deploy.yaml").read_text(
        encoding="utf-8"
    )
    release_source = (REPO_ROOT / ".github/workflows/release.yml").read_text(
        encoding="utf-8"
    )

    deploy_trigger = deploy_source[: deploy_source.index("\nenv:")]
    release_trigger = release_source[: release_source.index("\npermissions:")]
    assert "tags:" not in deploy_trigger
    assert "tags:" in release_trigger
    assert "softprops/action-gh-release@" not in deploy_source
    assert "name: Create GitHub Release" not in release_source


def test_release_mutation_is_absent_until_atomic_authority_exists() -> None:
    release = load_workflow("release.yml")
    source = (REPO_ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")

    assert release["permissions"] == {"contents": "read", "packages": "read"}
    assert release["concurrency"] == {
        "group": "production-release",
        "cancel-in-progress": False,
    }
    assert workflow_mutation_jobs(release) == set()
    assert "docker buildx imagetools create" not in source
    assert "softprops/action-gh-release@" not in source
    result_job = release["jobs"]["release-result"]
    assert result_job["if"] == "always()"
    assert set(job_needs(result_job)) == {
        "validate-release-receipt",
        "resolve-release-artifacts",
    }
    hold_step = workflow_step_by_name(
        release, "release-result", "Enforce atomic promotion hold"
    )["run"]
    assert "atomic multi-registry alias promotion with compensation" in hold_step
    assert "exit 1" in hold_step


def test_prepublication_builds_and_registry_reads_feed_exact_digest_gate() -> None:
    deploy = load_workflow("deploy.yaml")
    release = load_workflow("release.yml")

    for job_name in ("prebuild-api", "prebuild-ts-engines"):
        build_step = next(
            step
            for step in deploy["jobs"][job_name]["steps"]
            if "docker/build-push-action@" in step.get("uses", "")
        )
        assert build_step["with"]["push"] is False
        assert "cache-to" not in build_step["with"]
        assert build_step["env"]["DOCKER_BUILD_RECORD_UPLOAD"] == "false"

    deploy_validation = json.dumps(deploy["jobs"]["validate-release-receipt"])
    assert "prebuild-api.outputs.image-digest" in deploy_validation
    assert "prebuild-ts-engines.outputs.image-digest" in deploy_validation
    assert "--expected-artifact-digest" in deploy_validation

    for job_name, verify_name, publish_name in (
        ("build-api", "Verify local API digest matches receipt", "Publish validated API image"),
        (
            "build-ts-engines",
            "Verify local TypeScript digest matches receipt",
            "Publish validated TypeScript image",
        ),
    ):
        steps = deploy["jobs"][job_name]["steps"]
        build_step = next(
            step for step in steps if "docker/build-push-action@" in step.get("uses", "")
        )
        assert build_step["with"]["push"] is False
        assert build_step["with"]["load"] is True
        assert "cache-to" not in build_step["with"]
        verify_index = next(i for i, step in enumerate(steps) if step.get("name") == verify_name)
        publish_index = next(i for i, step in enumerate(steps) if step.get("name") == publish_name)
        assert verify_index < publish_index
        assert "docker push" in steps[publish_index]["run"]

    release_source = (REPO_ROOT / ".github/workflows/release.yml").read_text(
        encoding="utf-8"
    )
    assert "docker/build-push-action@" not in release_source
    assert "docker buildx imagetools inspect" in release_source
    assert "--target-profile release-production" in release_source
    assert '--expected-release-tag "$GITHUB_REF_NAME"' in release_source
    assert "--expected-operation-id" in release_source
    assert "--expected-workflow .github/workflows/release.yml" in release_source
    assert "update-changelog:" not in release_source
    assert "ref: main" not in release_source
    assert "build-binaries:" not in release_source
    manifest = json.loads(
        (REPO_ROOT / "contracts/release/v1/manifest.json").read_text(encoding="utf-8")
    )
    assert manifest["artifact_scope"] == {
        "kind": "container-images-only",
        "roles": ["api", "typescript-engines"],
        "excluded_publications": ["native-binaries"],
    }


def test_railway_mutation_uses_only_manifest_bound_selector() -> None:
    deploy = load_workflow("deploy.yaml")
    receipt_job = deploy["jobs"]["validate-release-receipt"]
    railway_job = deploy["jobs"]["deploy-railway"]
    railway_source = json.dumps(railway_job)

    for role in ("api", "typescript_engines"):
        for field in ("project_id", "environment_id", "service_id", "source_root", "config_path"):
            key = f"railway_{role}_{field}"
            assert receipt_job["outputs"][key] == f"${{{{ steps.receipt.outputs.{key} }}}}"
    assert railway_job["env"]["RAILWAY_API_SERVICE_ID"] == (
        "${{ needs.validate-release-receipt.outputs.railway_api_service_id }}"
    )
    assert railway_job["env"]["RAILWAY_TS_SERVICE_ID"] == (
        "${{ needs.validate-release-receipt.outputs.railway_typescript_engines_service_id }}"
    )
    assert "vars.RAILWAY_SERVICE" not in railway_source
    assert "vars.RAILWAY_ENVIRONMENT" not in railway_source
    assert "secrets.RAILWAY_PROJECT_ID" not in railway_source
    assert "projectToken { projectId environmentId }" in railway_source
    install_step = workflow_step_by_name(
        deploy, "deploy-railway", "Install Railway CLI"
    )["run"]
    assert install_step == "npm install -g @railway/cli@5.41.0"
    deploy_step = workflow_step_by_name(
        deploy, "deploy-railway", "Deploy to Railway services"
    )["run"]
    assert deploy_step.count("railway up") == 2
    assert 'cd "${RAILWAY_API_SOURCE_ROOT}"' in deploy_step
    assert '--service "${RAILWAY_API_SERVICE_ID}"' in deploy_step
    assert 'cd "${RAILWAY_TS_SOURCE_ROOT}"' in deploy_step
    assert '--service "${RAILWAY_TS_SERVICE_ID}"' in deploy_step


def test_deploy_publishes_only_source_immutable_candidate_tags() -> None:
    deploy = load_workflow("deploy.yaml")
    expected = "type=raw,value=sha-${{ needs.validate-source.outputs.source-sha }}"

    for job_name in (
        "prebuild-api",
        "prebuild-ts-engines",
        "build-api",
        "build-ts-engines",
    ):
        metadata = next(
            step
            for step in deploy["jobs"][job_name]["steps"]
            if "docker/metadata-action@" in step.get("uses", "")
        )
        assert metadata["with"]["tags"].strip() == expected

    workflow_source = (REPO_ROOT / ".github/workflows/deploy.yaml").read_text(
        encoding="utf-8"
    )
    assert "type=ref,event=" not in workflow_source
    assert "type=semver" not in workflow_source
    assert "value=latest" not in workflow_source


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
    for workflow_name in ("deploy.yaml", "release.yml"):
        mutations, output = mocked_workflow_mutations(
            workflow_name, tmp_path, None
        )
        assert mutations == set()
        assert "Release receipt missing" in output

        mismatched = eligible_receipt(workflow_name)
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


def test_mocked_workflows_reject_synthetic_and_wrong_target_receipts(
    tmp_path: Path,
) -> None:
    synthetic = json.loads(
        (RELEASE_FIXTURE_ROOT / "eligible-source-redeploy.json").read_text(
            encoding="utf-8"
        )
    )
    for workflow_name in ("deploy.yaml", "release.yml"):
        mutations, output = mocked_workflow_mutations(
            workflow_name, tmp_path, synthetic
        )
        assert mutations == set()
        assert "rejects receipts containing test_fixture metadata" in output

        wrong_target = eligible_receipt(workflow_name)
        wrong_target["target"]["environment"] = "staging"
        mutations, output = mocked_workflow_mutations(
            workflow_name, tmp_path, wrong_target
        )
        assert mutations == set()
        assert "target.environment does not match workflow authority" in output


def test_mocked_workflows_reject_wrong_service_and_artifact_identity(
    tmp_path: Path,
) -> None:
    for workflow_name in ("deploy.yaml", "release.yml"):
        wrong_service = eligible_receipt(workflow_name)
        api = next(
            row for row in wrong_service["service_roles"] if row["role"] == "api"
        )
        api["provider"] = "vercel"
        api["project_id"]["value"] = "fabricated-project"
        api["service_id"]["value"] = "fabricated-api-service"
        mutations, output = mocked_workflow_mutations(
            workflow_name, tmp_path, wrong_service
        )
        assert mutations == set()
        assert "service_roles.api.provider does not match release authority" in output
        assert "service_roles.api.project_id does not match release authority" in output
        assert "service_roles.api.service_id does not match release authority" in output

        wrong_digest = eligible_receipt(workflow_name)
        api = next(
            row for row in wrong_digest["artifacts"] if row["role"] == "api"
        )
        api["built"]["image_digest"]["value"] = (
            "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        )
        mutations, output = mocked_workflow_mutations(
            workflow_name, tmp_path, wrong_digest
        )
        assert mutations == set()
        assert "does not match workflow artifact" in output


def test_production_profiles_hold_all_mocked_mutations(tmp_path: Path) -> None:
    for workflow_name in ("deploy.yaml", "release.yml"):
        mutations, output = mocked_workflow_mutations(
            workflow_name, tmp_path, eligible_receipt(workflow_name)
        )
        assert mutations == set()
        assert "production mutation is disabled" in output
