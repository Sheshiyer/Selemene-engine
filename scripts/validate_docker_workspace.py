#!/usr/bin/env python3
"""Validate that Docker dependency caching represents every Cargo workspace member."""

from __future__ import annotations

import argparse
import re
import shlex
import sys
import tomllib
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cargo-toml", type=Path, default=Path("Cargo.toml"))
    parser.add_argument("--dockerfile", type=Path, default=Path("Dockerfile.prod"))
    return parser.parse_args()


def copied_manifest_sources(dockerfile_text: str) -> set[str]:
    sources: set[str] = set()
    for raw_line in dockerfile_text.splitlines():
        line = raw_line.strip()
        if not line.startswith("COPY "):
            continue
        try:
            tokens = shlex.split(line)
        except ValueError:
            continue
        positional = [token for token in tokens[1:] if not token.startswith("--")]
        if len(positional) < 2:
            continue
        for source in positional[:-1]:
            if source.endswith("/Cargo.toml"):
                sources.add(source.removeprefix("./"))
    return sources


def logical_docker_lines(dockerfile_text: str) -> list[str]:
    return [
        re.sub(r"\\\s*\n", " ", block)
        for block in re.split(r"(?m)(?=^[A-Z]+\s)", dockerfile_text)
        if block.strip()
    ]


def cache_run_commands(stub_section: str) -> str:
    commands: list[str] = []
    for instruction in logical_docker_lines(stub_section):
        uncommented = "\n".join(
            line for line in instruction.splitlines() if not line.lstrip().startswith("#")
        )
        normalized = re.sub(r"\\\s*\n", " ", uncommented).strip()
        if normalized.startswith("RUN "):
            commands.append(normalized.removeprefix("RUN "))
    return "\n".join(commands)


def created_stub_paths(stub_section: str) -> set[str]:
    commands = cache_run_commands(stub_section)
    created = set(
        re.findall(r">\s*[\"']?(crates/[a-zA-Z0-9_./-]+\.rs)\b", commands)
    )

    for loop in re.finditer(
        r"for\s+crate\s+in\s+(?P<names>.*?)\s*;\s*do(?P<body>.*?)\s*;\s*done",
        commands,
        flags=re.DOTALL,
    ):
        body = loop.group("body")
        if "crates/$crate/src/lib.rs" not in body:
            continue
        names = shlex.split(loop.group("names").replace("\\", " "))
        created.update(f"crates/{name}/src/lib.rs" for name in names)

    return created


def explicit_target_paths(
    member: str, manifest: dict[str, object], target_kind: str
) -> set[str]:
    targets = manifest.get(target_kind, [])
    if not isinstance(targets, list):
        raise ValueError(f"{member}/Cargo.toml has invalid [[{target_kind}]] targets")

    default_directories = {"bin": "src/bin", "bench": "benches", "example": "examples"}
    paths: set[str] = set()
    for target in targets:
        if not isinstance(target, dict):
            raise ValueError(f"{member}/Cargo.toml has invalid [[{target_kind}]] target")
        configured_path = target.get("path")
        if configured_path is not None:
            if not isinstance(configured_path, str):
                raise ValueError(
                    f"{member}/Cargo.toml has non-string path in [[{target_kind}]]"
                )
            paths.add(f"{member}/{configured_path}")
            continue
        name = target.get("name")
        if not isinstance(name, str):
            raise ValueError(
                f"{member}/Cargo.toml [[{target_kind}]] target needs a name or path"
            )
        paths.add(f"{member}/{default_directories[target_kind]}/{name}.rs")
    return paths


def required_stub_paths(
    member: str, member_root: Path, manifest: dict[str, object]
) -> set[str]:
    paths: set[str] = set()
    library = manifest.get("lib")
    if library is not None:
        if not isinstance(library, dict):
            raise ValueError(f"{member}/Cargo.toml has invalid [lib] target")
        configured_path = library.get("path", "src/lib.rs")
        if not isinstance(configured_path, str):
            raise ValueError(f"{member}/Cargo.toml has non-string [lib].path")
        paths.add(f"{member}/{configured_path}")
    elif (member_root / "src/lib.rs").is_file():
        paths.add(f"{member}/src/lib.rs")

    paths.update(explicit_target_paths(member, manifest, "bin"))
    paths.update(explicit_target_paths(member, manifest, "bench"))
    paths.update(explicit_target_paths(member, manifest, "example"))
    if not manifest.get("bin") and (member_root / "src/main.rs").is_file():
        paths.add(f"{member}/src/main.rs")
    return paths


def validate(cargo_toml: Path, dockerfile: Path) -> list[str]:
    errors: list[str] = []
    if not cargo_toml.is_file():
        return [f"workspace manifest is missing: {cargo_toml}"]
    if not dockerfile.is_file():
        return [f"Dockerfile is missing: {dockerfile}"]

    cargo_data = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
    workspace = cargo_data.get("workspace")
    if not isinstance(workspace, dict) or not isinstance(workspace.get("members"), list):
        return [f"workspace.members is missing or invalid in {cargo_toml}"]

    members = workspace["members"]
    if not all(isinstance(member, str) for member in members):
        return [f"workspace.members contains a non-string entry in {cargo_toml}"]

    dockerfile_text = dockerfile.read_text(encoding="utf-8")
    copied_manifests = copied_manifest_sources(dockerfile_text)
    cargo_root = cargo_toml.parent

    cache_build_index = dockerfile_text.find("cargo build --release --bin noesis-server")
    if cache_build_index < 0:
        errors.append("Docker dependency-cache stage has no noesis-server cargo build")
        stub_section = dockerfile_text
    else:
        layer_two_index = dockerfile_text.find("Layer 2")
        stub_section = dockerfile_text[max(layer_two_index, 0) : cache_build_index]
    stub_paths = created_stub_paths(stub_section)

    for member_value in members:
        member = str(member_value)
        manifest_relative = f"{member}/Cargo.toml"
        if manifest_relative not in copied_manifests:
            errors.append(f"missing Docker COPY for {manifest_relative}")

        member_root = cargo_root / member
        member_manifest_path = member_root / "Cargo.toml"
        if not member_manifest_path.is_file():
            errors.append(f"workspace member manifest is missing: {member_manifest_path}")
            continue
        member_manifest = tomllib.loads(member_manifest_path.read_text(encoding="utf-8"))
        for target_path in sorted(required_stub_paths(member, member_root, member_manifest)):
            if target_path not in stub_paths:
                errors.append(f"dependency-cache stub is missing target {target_path}")

    for logical_line in logical_docker_lines(dockerfile_text):
        if "cargo build" in logical_line and re.search(r"\|\|\s*true\b", logical_line):
            errors.append("cargo build is suppressed with || true in Dockerfile.prod")

    return errors


def main() -> int:
    args = parse_args()
    try:
        errors = validate(args.cargo_toml, args.dockerfile)
    except (OSError, tomllib.TOMLDecodeError) as error:
        print(f"Docker workspace validation error: {error}", file=sys.stderr)
        return 1

    if errors:
        for error in errors:
            print(f"Docker workspace validation error: {error}", file=sys.stderr)
        return 1
    print("Docker dependency-cache workspace coverage is complete and fail-closed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
