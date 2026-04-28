#!/usr/bin/env python3
"""
Validate all TS engine fixture files against the bridge-envelope.json schema.

Usage:
    python3 tests/fixtures/validate_ts_fixtures.py

Exit codes:
    0 - All fixtures valid
    1 - One or more fixtures failed validation
"""

import json
import os
import sys
from pathlib import Path

try:
    import jsonschema
except ImportError:
    print("ERROR: jsonschema package required. Install with: pip install jsonschema")
    sys.exit(1)

REPO_ROOT = Path(__file__).parent.parent.parent
FIXTURES_DIR = REPO_ROOT / "tests" / "fixtures" / "expected_outputs"
SCHEMA_FILE = FIXTURES_DIR / "bridge-envelope.json"

TS_ENGINES = ["tarot", "i-ching", "enneagram", "sacred-geometry", "sigil-forge"]
REFERENCE_USERS = ["user_nyc_1990", "user_london_1985", "user_tokyo_1995"]


def load_schema() -> dict:
    with open(SCHEMA_FILE) as f:
        return json.load(f)


def validate_fixture(path: Path, schema: dict) -> list[str]:
    """Return a list of validation errors, or an empty list if valid."""
    errors = []
    try:
        with open(path) as f:
            data = json.load(f)
    except json.JSONDecodeError as e:
        return [f"Invalid JSON: {e}"]
    except FileNotFoundError:
        return [f"File not found: {path}"]

    validator = jsonschema.Draft7Validator(schema)
    for error in sorted(validator.iter_errors(data), key=lambda e: list(e.path)):
        errors.append(f"  {'.'.join(str(p) for p in error.path) or '<root>'}: {error.message}")
    return errors


def main() -> int:
    if not SCHEMA_FILE.exists():
        print(f"ERROR: Schema file not found: {SCHEMA_FILE}")
        return 1

    schema = load_schema()
    print(f"Loaded schema: {SCHEMA_FILE}")
    print()

    total = 0
    passed = 0
    failed = 0
    missing = 0

    for engine in TS_ENGINES:
        engine_dir = FIXTURES_DIR / engine
        for user in REFERENCE_USERS:
            fixture_path = engine_dir / f"{user}.json"
            total += 1

            if not fixture_path.exists():
                print(f"  MISSING  {engine}/{user}.json")
                missing += 1
                failed += 1
                continue

            errors = validate_fixture(fixture_path, schema)
            rel = fixture_path.relative_to(REPO_ROOT)
            if errors:
                print(f"  FAIL     {rel}")
                for e in errors:
                    print(e)
                failed += 1
            else:
                print(f"  PASS     {rel}")
                passed += 1

    print()
    print(f"Results: {passed}/{total} passed, {failed} failed ({missing} missing)")

    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
