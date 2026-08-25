from __future__ import annotations

import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Iterable


REPO_ROOT = Path(__file__).resolve().parents[2]


def run_python_script(script_relative_path: str, *args: Path | str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(REPO_ROOT / script_relative_path), *[str(arg) for arg in args]],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )


def copy_real_migrations_fixture(tmp_path: Path) -> Path:
    fixture_root = tmp_path / "repo"
    migrations_root = fixture_root / "migrations"
    migrations_root.mkdir(parents=True)

    source_root = REPO_ROOT / "migrations"
    for source_file in sorted(source_root.glob("*.sql")):
        shutil.copy2(source_file, migrations_root / source_file.name)

    shutil.copy2(source_root / "history.sha256", migrations_root / "history.sha256")
    return fixture_root


def write_files(root: Path, files: Iterable[tuple[str, str]]) -> None:
    for relative_path, content in files:
        destination = root / relative_path
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(content, encoding="utf-8")


def merged_env(extra: dict[str, str]) -> dict[str, str]:
    env = os.environ.copy()
    env.update(extra)
    return env
