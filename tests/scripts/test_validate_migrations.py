from __future__ import annotations

import hashlib
from pathlib import Path

from .conftest import copy_real_migrations_fixture, run_python_script


def run_validator(migrations_root: Path) -> tuple[int, str]:
    history_file = migrations_root / "history.sha256"
    result = run_python_script(
        "scripts/validate_migrations.py",
        "--migrations-dir",
        migrations_root,
        "--history-file",
        history_file,
    )
    return result.returncode, f"{result.stdout}{result.stderr}"


def add_ledgered_migration(migrations_root: Path, name: str, content: str) -> None:
    migration_file = migrations_root / name
    migration_file.write_text(content, encoding="utf-8")
    digest = hashlib.sha256(content.encode("utf-8")).hexdigest()
    history_file = migrations_root / "history.sha256"
    with history_file.open("a", encoding="utf-8") as history:
        history.write(f"{digest}  {name}\n")


def test_current_repository_migrations_validate() -> None:
    code, output = run_validator(Path("migrations"))
    assert code == 0, output


def test_rejects_deleted_historical_migration(tmp_path: Path) -> None:
    fixture_root = copy_real_migrations_fixture(tmp_path)
    missing_file = fixture_root / "migrations/021_processed_webhook_events.sql"
    missing_file.unlink()

    code, output = run_validator(fixture_root / "migrations")

    assert code != 0
    assert "021_processed_webhook_events.sql" in output
    assert "missing from disk" in output


def test_rejects_renamed_historical_migration(tmp_path: Path) -> None:
    fixture_root = copy_real_migrations_fixture(tmp_path)
    source = fixture_root / "migrations/034_admin_readings_index.sql"
    source.rename(fixture_root / "migrations/034_admin_readings_index_renamed.sql")

    code, output = run_validator(fixture_root / "migrations")

    assert code != 0
    assert "034_admin_readings_index.sql" in output
    assert "034_admin_readings_index_renamed.sql" in output


def test_rejects_checksum_drift(tmp_path: Path) -> None:
    fixture_root = copy_real_migrations_fixture(tmp_path)
    target = fixture_root / "migrations/037_living_reading_invitations.sql"
    target.write_text(target.read_text(encoding="utf-8") + "\n-- drift\n", encoding="utf-8")

    code, output = run_validator(fixture_root / "migrations")

    assert code != 0
    assert "037_living_reading_invitations.sql" in output
    assert "checksum" in output


def test_rejects_coordinated_historical_sql_and_ledger_digest_edit(tmp_path: Path) -> None:
    fixture_root = copy_real_migrations_fixture(tmp_path)
    migrations_root = fixture_root / "migrations"
    target = migrations_root / "021_processed_webhook_events.sql"
    changed_content = target.read_text(encoding="utf-8") + "\n-- coordinated drift\n"
    target.write_text(changed_content, encoding="utf-8")

    history_file = migrations_root / "history.sha256"
    old_history = history_file.read_text(encoding="utf-8")
    changed_digest = hashlib.sha256(changed_content.encode("utf-8")).hexdigest()
    history_file.write_text(
        "\n".join(
            f"{changed_digest}  021_processed_webhook_events.sql"
            if line.endswith("  021_processed_webhook_events.sql")
            else line
            for line in old_history.splitlines()
        )
        + "\n",
        encoding="utf-8",
    )

    code, output = run_validator(migrations_root)

    assert code != 0
    assert "canonical historical baseline" in output


def test_rejects_ledgered_historical_gap_version(tmp_path: Path) -> None:
    fixture_root = copy_real_migrations_fixture(tmp_path)
    migrations_root = fixture_root / "migrations"
    add_ledgered_migration(migrations_root, "015_illegal_backfill.sql", "-- illegal 015\n")

    code, output = run_validator(migrations_root)

    assert code != 0
    assert "015" in output
    assert "historical gap" in output


def test_rejects_untracked_historical_insertion(tmp_path: Path) -> None:
    fixture_root = copy_real_migrations_fixture(tmp_path)
    inserted = fixture_root / "migrations/015_backfilled_late.sql"
    inserted.write_text("-- inserted late\n", encoding="utf-8")

    code, output = run_validator(fixture_root / "migrations")

    assert code != 0
    assert "015_backfilled_late.sql" in output
    assert "historical insertion" in output


def test_rejects_untracked_next_migration(tmp_path: Path) -> None:
    fixture_root = copy_real_migrations_fixture(tmp_path)
    migrations_root = fixture_root / "migrations"
    (migrations_root / "038_first_extension.sql").write_text("-- 038 first\n", encoding="utf-8")

    code, output = run_validator(migrations_root)

    assert code != 0
    assert "038_first_extension.sql" in output
    assert "untracked migration" in output


def test_accepts_checksum_ledgered_next_migration(tmp_path: Path) -> None:
    fixture_root = copy_real_migrations_fixture(tmp_path)
    migrations_root = fixture_root / "migrations"
    add_ledgered_migration(migrations_root, "038_first_extension.sql", "-- 038 first\n")

    code, output = run_validator(migrations_root)

    assert code == 0, output


def test_rejects_new_duplicate_version(tmp_path: Path) -> None:
    fixture_root = copy_real_migrations_fixture(tmp_path)
    migrations_root = fixture_root / "migrations"
    add_ledgered_migration(migrations_root, "038_first_extension.sql", "-- 038 first\n")
    add_ledgered_migration(
        migrations_root, "038_duplicate_extension.sql", "-- 038 duplicate\n"
    )

    code, output = run_validator(migrations_root)

    assert code != 0
    assert "038" in output
    assert "duplicate" in output


def test_rejects_new_gap_after_ledger(tmp_path: Path) -> None:
    fixture_root = copy_real_migrations_fixture(tmp_path)
    migrations_root = fixture_root / "migrations"
    add_ledgered_migration(migrations_root, "039_skips_038.sql", "-- skipped 038\n")

    code, output = run_validator(migrations_root)

    assert code != 0
    assert "039_skips_038.sql" in output
    assert "038" in output


def test_rejects_malformed_filename(tmp_path: Path) -> None:
    fixture_root = copy_real_migrations_fixture(tmp_path)
    malformed = fixture_root / "migrations/38_missing_zeroes.sql"
    malformed.write_text("-- malformed\n", encoding="utf-8")

    code, output = run_validator(fixture_root / "migrations")

    assert code != 0
    assert "38_missing_zeroes.sql" in output
    assert "malformed" in output
