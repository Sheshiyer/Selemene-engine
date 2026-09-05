#!/usr/bin/env python3
"""Validate workflow action refs are pinned to commit SHAs."""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import Iterable, List, Tuple

import yaml

WORKFLOW_EXTENSIONS = {".yml", ".yaml"}
GITHUB_ACTION_SHA = re.compile(r"^[^@\s]+@[0-9a-fA-F]{40}$")
DOCKER_DIGEST = re.compile(r"^docker://[^@\s]+@sha256:[0-9a-fA-F]{64}$")


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate GitHub Action and reusable workflow references."
    )
    parser.add_argument(
        "--path",
        default=".github/workflows",
        help="Workflow file or directory to validate (default: .github/workflows).",
    )
    return parser.parse_args(argv)


def iter_uses_values(document: object) -> Iterable[str]:
    if isinstance(document, dict):
        for key, value in document.items():
            if key == "uses" and isinstance(value, str):
                yield value
            yield from iter_uses_values(value)
    elif isinstance(document, list):
        for item in document:
            yield from iter_uses_values(item)


def is_expression(ref: str) -> bool:
    return "${{" in ref


def is_local_ref(ref: str) -> bool:
    return ref.startswith("./") or ref.startswith("../")


def is_valid_ref(ref: str) -> bool:
    if not ref:
        return False
    if is_expression(ref):
        return False
    if is_local_ref(ref):
        return "@" not in ref
    if ref.startswith("docker://"):
        return bool(DOCKER_DIGEST.fullmatch(ref))
    return bool(GITHUB_ACTION_SHA.fullmatch(ref))


def find_invalid_refs(root: Path) -> List[Tuple[Path, str]]:
    if not root.exists():
        raise FileNotFoundError(f"Workflow path does not exist: {root}")

    if root.is_file():
        workflow_files = [root]
    elif root.is_dir():
        workflow_files = sorted(
            path
            for path in root.rglob("*")
            if path.is_file() and path.suffix in WORKFLOW_EXTENSIONS
        )
    else:
        raise NotADirectoryError(f"Workflow path is not a file or directory: {root}")

    violations: List[Tuple[Path, str]] = []
    for workflow_file in workflow_files:
        try:
            parsed = yaml.safe_load(workflow_file.read_text(encoding="utf-8"))
        except yaml.YAMLError as exc:
            raise ValueError(f"Invalid YAML in {workflow_file}: {exc}") from exc

        if parsed is None:
            continue

        for uses_ref in iter_uses_values(parsed):
            if not is_valid_ref(uses_ref):
                violations.append((workflow_file, uses_ref))

    return violations


def _reason(uses_ref: str) -> str:
    if is_expression(uses_ref):
        return "contains a GitHub expression instead of an explicit SHA"
    if uses_ref.startswith("docker://"):
        return "docker refs must be pinned with @sha256:..."
    if is_local_ref(uses_ref):
        return "local action/workflow ref"
    return "non-SHA action ref (branch, tag, or PR ref)"


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        violations = find_invalid_refs(Path(args.path))
    except (FileNotFoundError, NotADirectoryError, ValueError) as exc:
        print(f"::error::{exc}", file=sys.stderr)
        return 1

    if not violations:
        return 0

    print("::error::Found unpinned workflow references.", file=sys.stderr)
    for file_path, uses_ref in sorted(violations, key=lambda item: (str(item[0]), item[1])):
        print(f"  {file_path}: {uses_ref} ({_reason(uses_ref)})", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
