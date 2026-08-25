from __future__ import annotations

from pathlib import Path

from .conftest import run_python_script, write_files


def run_validator(cargo_toml: Path, dockerfile: Path) -> tuple[int, str]:
    result = run_python_script(
        "scripts/validate_docker_workspace.py",
        "--cargo-toml",
        cargo_toml,
        "--dockerfile",
        dockerfile,
    )
    return result.returncode, f"{result.stdout}{result.stderr}"


def test_current_repository_docker_workspace_validates() -> None:
    code, output = run_validator(Path("Cargo.toml"), Path("Dockerfile.prod"))
    assert code == 0, output


def test_rejects_synthetic_missing_workspace_member(tmp_path: Path) -> None:
    fixture_root = tmp_path / "repo"
    write_files(
        fixture_root,
        [
            (
                "Cargo.toml",
                """[workspace]
members = ["crates/present", "crates/missing"]
""",
            ),
            (
                "crates/present/Cargo.toml",
                """[package]
name = "present"
version = "0.1.0"
edition = "2021"
""",
            ),
            (
                "crates/missing/Cargo.toml",
                """[package]
name = "missing"
version = "0.1.0"
edition = "2021"
""",
            ),
            (
                "Dockerfile.prod",
                """FROM rust:1.89-slim-bookworm AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates/present/Cargo.toml ./crates/present/Cargo.toml
RUN mkdir -p crates/present/src && echo "" > crates/present/src/lib.rs
RUN cargo build --release --bin demo
""",
            ),
        ],
    )

    code, output = run_validator(fixture_root / "Cargo.toml", fixture_root / "Dockerfile.prod")

    assert code != 0
    assert "crates/missing/Cargo.toml" in output
    assert "COPY" in output


def test_rejects_cargo_build_suppression(tmp_path: Path) -> None:
    fixture_root = tmp_path / "repo"
    write_files(
        fixture_root,
        [
            (
                "Cargo.toml",
                """[workspace]
members = ["crates/demo"]
""",
            ),
            (
                "crates/demo/Cargo.toml",
                """[package]
name = "demo"
version = "0.1.0"
edition = "2021"
""",
            ),
            (
                "Dockerfile.prod",
                """FROM rust:1.89-slim-bookworm AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates/demo/Cargo.toml ./crates/demo/Cargo.toml
RUN mkdir -p crates/demo/src && echo "" > crates/demo/src/lib.rs
RUN cargo build --release --bin demo || true
""",
            ),
        ],
    )

    code, output = run_validator(fixture_root / "Cargo.toml", fixture_root / "Dockerfile.prod")

    assert code != 0
    assert "|| true" in output


def test_rejects_library_stub_named_only_in_comment(tmp_path: Path) -> None:
    fixture_root = tmp_path / "repo"
    write_files(
        fixture_root,
        [
            (
                "Cargo.toml",
                """[workspace]
members = ["crates/comment-only"]
""",
            ),
            (
                "crates/comment-only/Cargo.toml",
                """[package]
name = "comment-only"
version = "0.1.0"
edition = "2021"

[lib]
path = "src/lib.rs"
""",
            ),
            (
                "Dockerfile.prod",
                """FROM rust:1.89-slim-bookworm AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates/comment-only/Cargo.toml ./crates/comment-only/Cargo.toml
# comment-only would normally get crates/comment-only/src/lib.rs here
RUN cargo build --release --bin noesis-server
""",
            ),
        ],
    )

    code, output = run_validator(fixture_root / "Cargo.toml", fixture_root / "Dockerfile.prod")

    assert code != 0
    assert "crates/comment-only/src/lib.rs" in output


def test_rejects_missing_explicit_cargo_target_stub(tmp_path: Path) -> None:
    fixture_root = tmp_path / "repo"
    write_files(
        fixture_root,
        [
            (
                "Cargo.toml",
                """[workspace]
members = ["crates/demo"]
""",
            ),
            (
                "crates/demo/Cargo.toml",
                """[package]
name = "demo"
version = "0.1.0"
edition = "2021"

[[bench]]
name = "declared_bench"
harness = false
""",
            ),
            (
                "Dockerfile.prod",
                """FROM rust:1.89-slim-bookworm AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates/demo/Cargo.toml ./crates/demo/Cargo.toml
RUN mkdir -p crates/demo/src && echo "" > crates/demo/src/lib.rs
# benches/declared_bench.rs is intentionally absent
RUN cargo build --release --bin noesis-server
""",
            ),
        ],
    )

    code, output = run_validator(fixture_root / "Cargo.toml", fixture_root / "Dockerfile.prod")

    assert code != 0
    assert "crates/demo/benches/declared_bench.rs" in output
