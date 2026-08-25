from __future__ import annotations

import stat
import subprocess
import sys
from pathlib import Path

from .conftest import REPO_ROOT, copy_real_migrations_fixture, merged_env


def install_fake_psql(tmp_path: Path) -> tuple[Path, Path, Path]:
    bin_root = tmp_path / "bin"
    bin_root.mkdir()
    log_file = tmp_path / "psql.log"
    journal_file = tmp_path / "journal.tsv"
    fake_psql = bin_root / "psql"
    fake_psql.write_text(
        r'''#!/usr/bin/env bash
set -euo pipefail
input=$(cat)
operation=""; filename=""; checksum=""; mode=""
args=("$@")
for ((index=0; index<${#args[@]}; index++)); do
  if [[ "${args[$index]}" == "-v" ]]; then
    assignment="${args[$((index + 1))]}"
    case "$assignment" in
      gate_operation=*) operation="${assignment#*=}" ;;
      migration_filename=*) filename="${assignment#*=}" ;;
      migration_checksum=*) checksum="${assignment#*=}" ;;
      migration_mode=*) mode="${assignment#*=}" ;;
    esac
  fi
done
printf 'ARGS:%s\n' "$*" >> "$PSQL_LOG"

case "$operation" in
  probe)
    journal_value=f
    [[ -f "$PSQL_JOURNAL" ]] && journal_value=t
    schema_value="${PSQL_SCHEMA_NONEMPTY:-f}"
    if [[ "${PSQL_BOOLEAN_STYLE:-short}" == "words" ]]; then
      [[ "$journal_value" == "t" ]] && journal_value=true || journal_value=false
      [[ "$schema_value" == "t" ]] && schema_value=true || schema_value=false
    fi
    printf '%s|%s\n' "$journal_value" "$schema_value"
    ;;
  init)
    touch "$PSQL_JOURNAL"
    printf 'INIT\n' >> "$PSQL_LOG"
    ;;
  dirty)
    [[ -f "$PSQL_JOURNAL" ]] && awk -F '\t' '$3 == "applying" { print $1 "|" $2 }' "$PSQL_JOURNAL"
    ;;
  journal_count)
    if [[ -f "$PSQL_JOURNAL" ]]; then wc -l < "$PSQL_JOURNAL" | tr -d ' ';
    else printf '0\n'; fi
    ;;
  lookup)
    [[ -f "$PSQL_JOURNAL" ]] && awk -F '\t' -v wanted="$filename" '$1 == wanted { print $2 "|" $3 }' "$PSQL_JOURNAL"
    ;;
  apply)
    printf 'APPLY:%s:%s\n' "$filename" "$mode" >> "$PSQL_LOG"
    printf '%s\n' "$input" > "$PSQL_LAST_INPUT"
    if [[ "$mode" == "nontransactional" ]]; then
      printf '%s\t%s\tapplying\n' "$filename" "$checksum" >> "$PSQL_JOURNAL"
    fi
    if [[ "$filename" == "${FAIL_MIGRATION:-}" ]]; then
      printf 'FAIL:%s\n' "$filename" >> "$PSQL_LOG"
      exit 1
    fi
    if [[ "$mode" == "nontransactional" ]]; then
      awk -F '\t' -v OFS='\t' -v wanted="$filename" '$1 == wanted {$3="applied"} {print}' "$PSQL_JOURNAL" > "$PSQL_JOURNAL.tmp"
      mv "$PSQL_JOURNAL.tmp" "$PSQL_JOURNAL"
    else
      printf '%s\t%s\tapplied\n' "$filename" "$checksum" >> "$PSQL_JOURNAL"
    fi
    printf 'RECORDED:%s\n' "$filename" >> "$PSQL_LOG"
    ;;
  adopt)
    touch "$PSQL_JOURNAL"
    printf '%s\n' "$input" > "$PSQL_LAST_INPUT"
    printf '%s\n' "$input" | awk -v journal="$PSQL_JOURNAL" -v log_path="$PSQL_LOG" '
      substr($1, 2) == "set" && $2 == "migration_filename" {
        name=$3; gsub(/^'\''|'\''$/, "", name)
      }
      substr($1, 2) == "set" && $2 == "migration_checksum" {
        checksum=$3; gsub(/^'\''|'\''$/, "", checksum)
        print name "\t" checksum "\tapplied" >> journal
        print "ADOPT:" name >> log_path
      }
    '
    ;;
  *)
    echo "unrecognized fake psql operation: $operation" >&2
    exit 9
    ;;
esac
''',
        encoding="utf-8",
    )
    fake_psql.chmod(fake_psql.stat().st_mode | stat.S_IEXEC)
    fake_python = bin_root / "python3"
    fake_python.write_text(
        r'''#!/usr/bin/env bash
set -euo pipefail
if [[ "${RUN_REAL_MIGRATION_VALIDATOR:-0}" == "1" ]]; then
  exec "$REAL_PYTHON" "$@"
fi
exit 0
''',
        encoding="utf-8",
    )
    fake_python.chmod(fake_python.stat().st_mode | stat.S_IEXEC)
    return bin_root, log_file, journal_file


def run_runner(
    migrations_root: Path,
    bin_root: Path,
    log_file: Path,
    journal_file: Path,
    *,
    fail_migration: str = "",
    schema_nonempty: bool = False,
    adopt_through: str | None = None,
    boolean_style: str = "short",
    run_real_validator: bool = False,
) -> subprocess.CompletedProcess[str]:
    command = [
        "bash",
        str(REPO_ROOT / "scripts/apply-migrations.sh"),
        "--migrations-dir",
        str(migrations_root),
    ]
    if adopt_through is not None:
        command.extend(["--adopt-through", adopt_through])
    command.extend(["--", "--dbname", "postgresql://unit-test"])
    return subprocess.run(
        command,
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
        env=merged_env(
            {
                "PATH": f"{bin_root}:/usr/bin:/bin",
                "PSQL_LOG": str(log_file),
                "PSQL_JOURNAL": str(journal_file),
                "PSQL_LAST_INPUT": str(migrations_root.parent / "last-input.sql"),
                "PSQL_SCHEMA_NONEMPTY": "t" if schema_nonempty else "f",
                "PSQL_BOOLEAN_STYLE": boolean_style,
                "FAIL_MIGRATION": fail_migration,
                "RUN_REAL_MIGRATION_VALIDATOR": "1" if run_real_validator else "0",
                "REAL_PYTHON": sys.executable,
            }
        ),
    )


def operations(log_file: Path, prefix: str) -> list[str]:
    if not log_file.exists():
        return []
    return [
        line.removeprefix(prefix)
        for line in log_file.read_text(encoding="utf-8").splitlines()
        if line.startswith(prefix)
    ]


def journal_rows(journal_file: Path) -> list[list[str]]:
    if not journal_file.exists():
        return []
    return [line.split("\t") for line in journal_file.read_text(encoding="utf-8").splitlines()]


def test_second_run_skips_exact_files_and_only_applies_new_038(tmp_path: Path) -> None:
    migrations_root = tmp_path / "migrations"
    migrations_root.mkdir()
    (migrations_root / "007_alpha.sql").write_text("SELECT 7;\n", encoding="utf-8")
    (migrations_root / "007_beta.sql").write_text("SELECT 8;\n", encoding="utf-8")
    bin_root, log_file, journal_file = install_fake_psql(tmp_path)

    first = run_runner(migrations_root, bin_root, log_file, journal_file)
    second = run_runner(migrations_root, bin_root, log_file, journal_file)
    (migrations_root / "038_next.sql").write_text("SELECT 38;\n", encoding="utf-8")
    third = run_runner(migrations_root, bin_root, log_file, journal_file)

    assert first.returncode == second.returncode == third.returncode == 0
    assert operations(log_file, "APPLY:") == [
        "007_alpha.sql:transactional",
        "007_beta.sql:transactional",
        "038_next.sql:transactional",
    ]
    assert "Skipping already-applied migration: 007_alpha.sql" in second.stdout
    assert "Skipping already-applied migration: 007_beta.sql" in second.stdout


def test_checksum_drift_fails_before_reapplying_migration(tmp_path: Path) -> None:
    migrations_root = tmp_path / "migrations"
    migrations_root.mkdir()
    migration = migrations_root / "001_first.sql"
    migration.write_text("SELECT 1;\n", encoding="utf-8")
    bin_root, log_file, journal_file = install_fake_psql(tmp_path)
    assert run_runner(migrations_root, bin_root, log_file, journal_file).returncode == 0

    migration.write_text("SELECT 2;\n", encoding="utf-8")
    drift = run_runner(migrations_root, bin_root, log_file, journal_file)

    assert drift.returncode != 0
    assert operations(log_file, "APPLY:") == ["001_first.sql:transactional"]
    assert "checksum mismatch" in f"{drift.stdout}{drift.stderr}"


def test_failed_transactional_sql_stops_and_is_not_recorded(tmp_path: Path) -> None:
    migrations_root = tmp_path / "migrations"
    migrations_root.mkdir()
    (migrations_root / "002_second.sql").write_text("SELECT broken;\n", encoding="utf-8")
    (migrations_root / "001_first.sql").write_text("SELECT 1;\n", encoding="utf-8")
    (migrations_root / "003_third.sql").write_text("SELECT 3;\n", encoding="utf-8")
    bin_root, log_file, journal_file = install_fake_psql(tmp_path)

    failed = run_runner(
        migrations_root, bin_root, log_file, journal_file, fail_migration="002_second.sql"
    )

    assert failed.returncode != 0
    assert operations(log_file, "APPLY:") == [
        "001_first.sql:transactional",
        "002_second.sql:transactional",
    ]
    rows = journal_rows(journal_file)
    assert len(rows) == 1
    assert rows[0][0] == "001_first.sql"
    assert len(rows[0][1]) == 64
    assert rows[0][2] == "applied"
    transaction = (tmp_path / "last-input.sql").read_text(encoding="utf-8")
    assert transaction.startswith("BEGIN;")
    assert transaction.rstrip().endswith("COMMIT;")
    assert "003_third.sql" not in f"{failed.stdout}{failed.stderr}"
    args_lines = operations(log_file, "ARGS:")
    assert all("-X" in line for line in args_lines)
    assert all("ON_ERROR_STOP=1" in line for line in args_lines)


def test_failed_concurrent_migration_leaves_dirty_row_and_next_run_refuses(tmp_path: Path) -> None:
    migrations_root = tmp_path / "migrations"
    migrations_root.mkdir()
    name = "030_concurrent.sql"
    (migrations_root / name).write_text(
        "CREATE INDEX CONCURRENTLY demo_idx ON demo(id);\n", encoding="utf-8"
    )
    bin_root, log_file, journal_file = install_fake_psql(tmp_path)

    failed = run_runner(
        migrations_root, bin_root, log_file, journal_file, fail_migration=name
    )
    retry = run_runner(migrations_root, bin_root, log_file, journal_file)

    assert failed.returncode != 0
    assert retry.returncode != 0
    assert journal_rows(journal_file)[0][2] == "applying"
    assert operations(log_file, "APPLY:") == [f"{name}:nontransactional"]
    assert "dirty migration journal" in f"{retry.stdout}{retry.stderr}"


def test_nonempty_unjournaled_schema_requires_explicit_validated_adoption(tmp_path: Path) -> None:
    fixture_root = copy_real_migrations_fixture(tmp_path)
    migrations_root = fixture_root / "migrations"
    bin_root, log_file, journal_file = install_fake_psql(tmp_path)

    refused = run_runner(
        migrations_root, bin_root, log_file, journal_file, schema_nonempty=True
    )
    adopted = run_runner(
        migrations_root,
        bin_root,
        log_file,
        journal_file,
        schema_nonempty=True,
        adopt_through="037",
    )

    assert refused.returncode != 0
    assert "--adopt-through" in f"{refused.stdout}{refused.stderr}"
    assert adopted.returncode == 0, f"{adopted.stdout}{adopted.stderr}"
    assert len(operations(log_file, "ADOPT:")) == len(list(migrations_root.glob("*.sql")))
    assert operations(log_file, "APPLY:") == []
    assert all(row[2] == "applied" for row in journal_rows(journal_file))
    adoption_sql = (fixture_root / "last-input.sql").read_text(encoding="utf-8")
    assert adoption_sql.startswith("BEGIN;")
    assert adoption_sql.rstrip().endswith("COMMIT;")
    assert adoption_sql.count("\\set migration_filename ") == len(
        list(migrations_root.glob("*.sql"))
    )


def test_nonempty_schema_with_empty_journal_also_refuses_replay(tmp_path: Path) -> None:
    migrations_root = tmp_path / "migrations"
    migrations_root.mkdir()
    (migrations_root / "001_first.sql").write_text("SELECT 1;\n", encoding="utf-8")
    bin_root, log_file, journal_file = install_fake_psql(tmp_path)
    journal_file.touch()

    refused = run_runner(
        migrations_root, bin_root, log_file, journal_file, schema_nonempty=True
    )

    assert refused.returncode != 0
    assert "empty migration journal" in f"{refused.stdout}{refused.stderr}"
    assert operations(log_file, "APPLY:") == []


def test_probe_accepts_postgresql_word_form_booleans(tmp_path: Path) -> None:
    migrations_root = tmp_path / "migrations"
    migrations_root.mkdir()
    (migrations_root / "001_first.sql").write_text("SELECT 1;\n", encoding="utf-8")
    bin_root, log_file, journal_file = install_fake_psql(tmp_path)

    result = run_runner(
        migrations_root,
        bin_root,
        log_file,
        journal_file,
        boolean_style="words",
    )

    assert result.returncode == 0, f"{result.stdout}{result.stderr}"
    assert operations(log_file, "APPLY:") == ["001_first.sql:transactional"]


def test_validator_rejects_unledgered_038_before_any_psql_call(tmp_path: Path) -> None:
    fixture_root = copy_real_migrations_fixture(tmp_path)
    migrations_root = fixture_root / "migrations"
    (migrations_root / "038_unledgered.sql").write_text("SELECT 38;\n", encoding="utf-8")
    bin_root, log_file, journal_file = install_fake_psql(tmp_path)

    result = run_runner(
        migrations_root,
        bin_root,
        log_file,
        journal_file,
        run_real_validator=True,
    )

    assert result.returncode != 0
    assert "untracked migration" in f"{result.stdout}{result.stderr}"
    assert not log_file.exists() or log_file.read_text(encoding="utf-8") == ""
