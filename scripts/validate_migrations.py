#!/usr/bin/env python3
"""Fail closed when immutable SQL migration history is changed or extended unsafely."""

from __future__ import annotations

import argparse
import hashlib
import re
import sys
from collections import Counter
from pathlib import Path


MIGRATION_NAME = re.compile(r"^(?P<version>\d{3})_[a-z0-9][a-z0-9_]*\.sql$")
LEDGER_LINE = re.compile(r"^(?P<digest>[0-9a-f]{64})  (?P<name>[^/]+\.sql)$")
HISTORICAL_MAX_VERSION = 37
ALLOWED_HISTORICAL_GAPS = {15, 16}
ALLOWED_HISTORICAL_DUPLICATES = {7: 2}
CANONICAL_HISTORICAL_LEDGER_SHA256 = (
    "b48320d81cff45a2ddbea0d6eb2abd253ad6000254f5412ffcabf8ee83c4a2c7"
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--migrations-dir", type=Path, default=Path("migrations"))
    parser.add_argument(
        "--history-file", type=Path, default=Path("migrations/history.sha256")
    )
    return parser.parse_args()


def read_history(history_file: Path) -> dict[str, str]:
    if not history_file.is_file():
        raise ValueError(f"migration history ledger is missing: {history_file}")

    history: dict[str, str] = {}
    for line_number, raw_line in enumerate(
        history_file.read_text(encoding="utf-8").splitlines(), start=1
    ):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        match = LEDGER_LINE.fullmatch(line)
        if match is None:
            raise ValueError(
                f"malformed ledger line {line_number} in {history_file}: {raw_line!r}"
            )
        name = match.group("name")
        if name in history:
            raise ValueError(f"duplicate migration in history ledger: {name}")
        history[name] = match.group("digest")
    if not history:
        raise ValueError(f"migration history ledger is empty: {history_file}")
    return history


def version_for(name: str) -> int:
    match = MIGRATION_NAME.fullmatch(name)
    if match is None:
        raise ValueError(f"malformed migration filename: {name}")
    return int(match.group("version"))


def validate_historical_shape(history: dict[str, str]) -> list[str]:
    errors: list[str] = []
    historical_entries = sorted(
        (name, digest)
        for name, digest in history.items()
        if version_for(name) <= HISTORICAL_MAX_VERSION
    )
    counts = Counter(version_for(name) for name, _ in historical_entries)
    expected_versions = set(range(1, HISTORICAL_MAX_VERSION + 1)) - ALLOWED_HISTORICAL_GAPS

    for version in sorted(set(counts) - expected_versions):
        if version in ALLOWED_HISTORICAL_GAPS:
            errors.append(
                f"historical gap version {version:03d} must not have a ledger entry"
            )
        else:
            errors.append(f"unexpected historical version {version:03d} in ledger")

    for version in sorted(expected_versions):
        expected_count = ALLOWED_HISTORICAL_DUPLICATES.get(version, 1)
        actual_count = counts.get(version, 0)
        if actual_count != expected_count:
            errors.append(
                f"historical version {version:03d} has {actual_count} migration(s); "
                f"expected {expected_count}"
            )

    canonical_representation = "".join(
        f"{digest}  {name}\n" for name, digest in historical_entries
    )
    actual_baseline_digest = hashlib.sha256(
        canonical_representation.encode("utf-8")
    ).hexdigest()
    if actual_baseline_digest != CANONICAL_HISTORICAL_LEDGER_SHA256:
        errors.append(
            "migration ledger does not match canonical historical baseline through 037: "
            f"expected {CANONICAL_HISTORICAL_LEDGER_SHA256}, got {actual_baseline_digest}"
        )
    return errors


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as migration_file:
        for chunk in iter(lambda: migration_file.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def validate(migrations_dir: Path, history_file: Path) -> list[str]:
    if not migrations_dir.is_dir():
        return [f"migrations directory is missing: {migrations_dir}"]

    try:
        history = read_history(history_file)
        errors = validate_historical_shape(history)
    except ValueError as error:
        return [str(error)]

    disk_files = sorted(migrations_dir.glob("*.sql"), key=lambda path: path.name)
    disk_names = {path.name for path in disk_files}

    parsed_versions: dict[str, int] = {}
    for migration_file in disk_files:
        try:
            parsed_versions[migration_file.name] = version_for(migration_file.name)
        except ValueError as error:
            errors.append(str(error))

    for name, expected_digest in history.items():
        migration_file = migrations_dir / name
        if name not in disk_names:
            errors.append(f"ledgered migration {name} is missing from disk")
            continue
        actual_digest = sha256(migration_file)
        if actual_digest != expected_digest:
            errors.append(
                f"checksum drift for {name}: expected {expected_digest}, got {actual_digest}"
            )

    untracked_migrations = sorted(name for name in parsed_versions if name not in history)
    for name in untracked_migrations:
        version = parsed_versions[name]
        if version <= HISTORICAL_MAX_VERSION:
            errors.append(f"untracked historical insertion: {name}")
        else:
            errors.append(
                f"untracked migration: {name}; every SQL file must be checksum-ledgered "
                f"and the next legal version is {HISTORICAL_MAX_VERSION + 1:03d}"
            )

    new_by_version: dict[int, list[str]] = {}
    for name in history:
        version = version_for(name)
        if version > HISTORICAL_MAX_VERSION:
            new_by_version.setdefault(version, []).append(name)

    for version, names in sorted(new_by_version.items()):
        if len(names) > 1:
            errors.append(
                f"duplicate new migration version {version:03d}: {', '.join(sorted(names))}"
            )

    if new_by_version:
        highest_new_version = max(new_by_version)
        for version in range(HISTORICAL_MAX_VERSION + 1, highest_new_version + 1):
            names = new_by_version.get(version, [])
            if not names:
                later_names = sorted(
                    name
                    for later_version, version_names in new_by_version.items()
                    if later_version > version
                    for name in version_names
                )
                errors.append(
                    f"new migration sequence has a gap at {version:03d}; "
                    f"next legal version is {HISTORICAL_MAX_VERSION + 1:03d}; "
                    f"found later migration(s): {', '.join(later_names)}"
                )

    return errors


def main() -> int:
    args = parse_args()
    errors = validate(args.migrations_dir, args.history_file)
    if errors:
        for error in errors:
            print(f"migration validation error: {error}", file=sys.stderr)
        return 1
    print(
        f"Migration history valid through {HISTORICAL_MAX_VERSION:03d}; "
        f"next legal version is {HISTORICAL_MAX_VERSION + 1:03d}."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
