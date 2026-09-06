from __future__ import annotations

import importlib.util
import json
import shutil
import subprocess
from pathlib import Path

import pytest

from .conftest import REPO_ROOT, run_python_script


AUTHORITY_ROOT = REPO_ROOT / "contracts" / "v1"

SPEC = importlib.util.spec_from_file_location(
    "validate_contracts",
    REPO_ROOT / "scripts/validate_contracts.py",
)
assert SPEC is not None and SPEC.loader is not None
validate_contracts_module = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(validate_contracts_module)
ContractValidationError = validate_contracts_module.ContractValidationError
validate_repo_reference = validate_contracts_module.validate_repo_reference


def run_validator(root: Path) -> subprocess.CompletedProcess[str]:
    return run_python_script(
        "scripts/validate_contracts.py",
        "--root",
        root,
        "--repo-root",
        REPO_ROOT,
    )


def copy_authority(tmp_path: Path) -> Path:
    destination = tmp_path / "contracts" / "v1"
    shutil.copytree(AUTHORITY_ROOT, destination)
    return destination


def read_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: object) -> None:
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def assert_rust_compiles(source_path: Path, output_path: Path) -> None:
    result = subprocess.run(
        [
            "rustc",
            "--edition=2021",
            "--crate-name",
            "anchor_probe",
            "--crate-type=lib",
            "--emit=metadata",
            "-o",
            str(output_path),
            str(source_path),
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, result.stderr


def engine_registry(authority: Path) -> tuple[Path, dict[str, object]]:
    registry_path = authority / "registries" / "engines.json"
    return registry_path, read_json(registry_path)


def engine_row(registry: dict[str, object], engine_id: str) -> dict[str, object]:
    rows = registry["engines"]
    assert isinstance(rows, list)
    row = next(
        item for item in rows if isinstance(item, dict) and item.get("id") == engine_id
    )
    return row


def test_repository_contract_authority_is_valid() -> None:
    result = run_validator(AUTHORITY_ROOT)
    assert result.returncode == 0, result.stderr


def test_missing_manifest_schema_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    (authority / "schemas" / "error.schema.json").unlink()

    result = run_validator(authority)

    assert result.returncode != 0
    assert "error.schema.json" in result.stderr


def test_extra_manifest_schema_entry_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    manifest_path = authority / "manifest.json"
    manifest = read_json(manifest_path)
    schema_files = manifest["schemas"]
    assert isinstance(schema_files, list)
    manifest["schemas"] = [*schema_files, "schemas/bonus.schema.json"]
    write_json(manifest_path, manifest)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "bonus.schema.json" in result.stderr


def test_duplicate_manifest_schema_entry_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    manifest_path = authority / "manifest.json"
    manifest = read_json(manifest_path)
    schema_files = manifest["schemas"]
    assert isinstance(schema_files, list)
    manifest["schemas"] = [schema_files[0], *schema_files]
    write_json(manifest_path, manifest)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "duplicate" in result.stderr.lower()


def test_invalid_schema_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    schema_path = authority / "schemas" / "error.schema.json"
    schema = read_json(schema_path)
    schema["type"] = 7
    write_json(schema_path, schema)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "error.schema.json" in result.stderr


def test_invalid_fixture_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    write_json(
        authority / "fixtures" / "engine-result.json", {"contract_version": "v1"}
    )

    result = run_validator(authority)

    assert result.returncode != 0
    assert "engine-result.json" in result.stderr


def test_unresolvable_local_reference_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    schema_path = authority / "schemas" / "engine-request.schema.json"
    schema = read_json(schema_path)
    properties = schema["properties"]
    assert isinstance(properties, dict)
    consent = properties["consent"]
    assert isinstance(consent, dict)
    consent["$ref"] = "missing-consent.schema.json"
    write_json(schema_path, schema)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "engine-request.schema.json" in result.stderr


def test_sensitive_diagnostic_key_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    fixture_path = authority / "fixtures" / "error.json"
    fixture = read_json(fixture_path)
    details = fixture["details"]
    assert isinstance(details, dict)
    details["api_token"] = "sensitive"
    write_json(fixture_path, fixture)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "error.json" in result.stderr
    assert "api_token" in result.stderr


def test_contract_version_drift_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    fixture_path = authority / "fixtures" / "engine-capability.json"
    fixture = read_json(fixture_path)
    fixture["contract_version"] = "v2"
    write_json(fixture_path, fixture)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "engine-capability.json" in result.stderr
    assert "contract_version" in result.stderr


def test_empty_fixture_manifest_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    manifest_path = authority / "manifest.json"
    manifest = read_json(manifest_path)
    manifest["fixtures"] = []
    write_json(manifest_path, manifest)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "fixtures" in result.stderr


def test_duplicate_fixture_entry_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    manifest_path = authority / "manifest.json"
    manifest = read_json(manifest_path)
    fixtures = manifest["fixtures"]
    assert isinstance(fixtures, list)
    manifest["fixtures"] = [fixtures[0], *fixtures]
    write_json(manifest_path, manifest)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "duplicate" in result.stderr.lower()


def test_fixture_path_cannot_escape_authority_root(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    outside = authority.parent / "outside.json"
    outside.write_text(
        (authority / "fixtures" / "engine-request.json").read_text(encoding="utf-8"),
        encoding="utf-8",
    )
    manifest_path = authority / "manifest.json"
    manifest = read_json(manifest_path)
    fixtures = manifest["fixtures"]
    assert isinstance(fixtures, list)
    assert isinstance(fixtures[0], dict)
    fixtures[0]["path"] = "../outside.json"
    write_json(manifest_path, manifest)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "../outside.json" in result.stderr


def test_broken_internal_fragment_reference_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    schema_path = authority / "schemas" / "engine-result.schema.json"
    schema = read_json(schema_path)
    properties = schema["properties"]
    assert isinstance(properties, dict)
    properties["latent_broken_field"] = {"$ref": "#/$defs/missingPrompt"}
    write_json(schema_path, schema)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "missingPrompt" in result.stderr


def test_negative_seed_is_not_a_canonical_v1_request(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    fixture_path = authority / "fixtures" / "engine-request.json"
    fixture = read_json(fixture_path)
    fixture["seed"] = -1
    write_json(fixture_path, fixture)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "seed" in result.stderr


def test_missing_registry_manifest_entry_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    manifest_path = authority / "manifest.json"
    manifest = read_json(manifest_path)
    manifest["registries"] = []
    write_json(manifest_path, manifest)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "registry manifest drift" in result.stderr
    assert "engines.json" in result.stderr


def test_missing_runtime_id_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    registry_path, registry = engine_registry(authority)
    rows = registry["engines"]
    assert isinstance(rows, list)
    rows.pop()
    write_json(registry_path, registry)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "runtime ID count mismatch" in result.stderr
    assert "actual=18" in result.stderr


def test_duplicate_runtime_id_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    registry_path, registry = engine_registry(authority)
    rows = registry["engines"]
    assert isinstance(rows, list)
    assert isinstance(rows[0], dict)
    assert isinstance(rows[-1], dict)
    rows[-1]["id"] = rows[0]["id"]
    write_json(registry_path, registry)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "duplicate runtime ID biofield" in result.stderr


def test_duplicate_public_mirror_group_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    registry_path, registry = engine_registry(authority)
    engine_row(registry, "biorhythm")["public_mirror_group"] = "biofield"
    write_json(registry_path, registry)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "duplicate public mirror group biofield" in result.stderr


def test_wrong_database_conditional_class_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    registry_path, registry = engine_registry(authority)
    engine_row(registry, "biofield-capture")["runtime_class"] = "native"
    write_json(registry_path, registry)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "runtime class count mismatch" in result.stderr
    assert "database-conditional" in result.stderr


def test_wrong_typescript_runtime_class_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    registry_path, registry = engine_registry(authority)
    engine_row(registry, "tarot")["runtime_class"] = "native"
    write_json(registry_path, registry)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "runtime class count mismatch" in result.stderr
    assert "typescript" in result.stderr


def test_missing_registry_owner_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    registry_path, registry = engine_registry(authority)
    engine_row(registry, "panchanga")["owner"] = ""
    write_json(registry_path, registry)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "owner must be a safe repository-relative path" in result.stderr


def test_missing_evidence_axis_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    registry_path, registry = engine_registry(authority)
    evidence = engine_row(registry, "raaga")["evidence"]
    assert isinstance(evidence, dict)
    del evidence["operational"]
    write_json(registry_path, registry)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "evidence must contain all six axes" in result.stderr


def test_evidenced_axis_without_reference_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    registry_path, registry = engine_registry(authority)
    evidence = engine_row(registry, "numerology")["evidence"]
    assert isinstance(evidence, dict)
    declared = evidence["declared"]
    assert isinstance(declared, dict)
    declared["references"] = []
    write_json(registry_path, registry)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "evidenced status requires at least one reference" in result.stderr


def test_registry_evidence_missing_repo_path_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    registry_path, registry = engine_registry(authority)
    evidence = engine_row(registry, "biofield")["evidence"]
    assert isinstance(evidence, dict)
    evidence["declared"]["references"][0] = (
        "repo://definitely/missing/file.rs#nonexistent"
    )
    write_json(registry_path, registry)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "repo:// path does not exist" in result.stderr


def test_registry_evidence_missing_source_anchor_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    registry_path, registry = engine_registry(authority)
    evidence = engine_row(registry, "biofield")["evidence"]
    assert isinstance(evidence, dict)
    evidence["declared"]["references"][0] = (
        "repo://crates/noesis-orchestrator/src/lib.rs#DefinitelyMissingSymbol"
    )
    write_json(registry_path, registry)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "repo:// source anchor does not exist" in result.stderr


def test_registry_evidence_traversal_path_fails_closed(tmp_path: Path) -> None:
    authority = copy_authority(tmp_path)
    registry_path, registry = engine_registry(authority)
    evidence = engine_row(registry, "biofield")["evidence"]
    assert isinstance(evidence, dict)
    evidence["declared"]["references"][0] = "repo://../outside.rs#symbol"
    write_json(registry_path, registry)

    result = run_validator(authority)

    assert result.returncode != 0
    assert "unsafe repo:// path" in result.stderr


@pytest.mark.parametrize(
    ("suffix", "source"),
    [
        (".rs", "// fn comment_only_anchor() {}\n"),
        (".ts", "// export function comment_only_anchor() {}\n"),
        (".js", "/* function comment_only_anchor() {} */\n"),
        (".py", "# def comment_only_anchor(): pass\n"),
    ],
)
def test_source_anchor_rejects_comment_only_declaration(
    tmp_path: Path,
    suffix: str,
    source: str,
) -> None:
    source_path = tmp_path / f"comment-only{suffix}"
    source_path.write_text(source, encoding="utf-8")

    with pytest.raises(ContractValidationError, match="source anchor does not exist"):
        validate_repo_reference(
            f"repo://{source_path.name}#comment_only_anchor",
            tmp_path,
            "test evidence",
        )


@pytest.mark.parametrize(
    ("suffix", "source"),
    [
        (".rs", 'const NOTE: &str = "fn string_only_anchor() {}";\n'),
        (".ts", 'const note = "function string_only_anchor() {}";\n'),
        (".js", 'const note = "function string_only_anchor() {}";\n'),
        (".py", 'note = "def string_only_anchor(): pass"\n'),
    ],
)
def test_source_anchor_rejects_string_literal_only_declaration(
    tmp_path: Path,
    suffix: str,
    source: str,
) -> None:
    source_path = tmp_path / f"string-only{suffix}"
    source_path.write_text(source, encoding="utf-8")

    with pytest.raises(ContractValidationError, match="source anchor does not exist"):
        validate_repo_reference(
            f"repo://{source_path.name}#string_only_anchor",
            tmp_path,
            "test evidence",
        )


def test_qualified_rust_anchor_rejects_method_under_wrong_type(
    tmp_path: Path,
) -> None:
    source_path = tmp_path / "wrong-qualifier.rs"
    source_path.write_text(
        "struct Expected;\n"
        "struct Wrong;\n"
        "impl Expected {\n"
        "    fn terminal_method() {}\n"
        "}\n",
        encoding="utf-8",
    )

    with pytest.raises(ContractValidationError, match="source anchor does not exist"):
        validate_repo_reference(
            "repo://wrong-qualifier.rs#Wrong::terminal_method",
            tmp_path,
            "test evidence",
        )


@pytest.mark.parametrize(
    "reference",
    [
        "repo://crates/noesis-orchestrator/src/lib.rs#SUPPORTED_ENGINE_IDS",
        "repo://crates/noesis-orchestrator/src/lib.rs#WorkflowOrchestrator::register_native_runtime_engines",
        "repo://crates/noesis-api/src/lib.rs#register_database_conditional_engines",
        "repo://crates/noesis-bridge/src/lib.rs#BridgeManager::new",
        "repo://ts-engines/src/server/registry.ts#registerTypeScriptRuntimeEngines",
    ],
)
def test_existing_source_anchor_declarations_remain_valid(reference: str) -> None:
    validate_repo_reference(reference, REPO_ROOT, "test evidence")


@pytest.mark.parametrize(
    ("suffix", "source", "fragment"),
    [
        (".rs", "pub fn rust_anchor() {}\n", "rust_anchor"),
        (".ts", "export function typescriptAnchor() {}\n", "typescriptAnchor"),
        (".js", "export const javascriptAnchor = () => {};\n", "javascriptAnchor"),
        (
            ".ts",
            "export class TypeScriptOwner {\n    method(): void {}\n}\n",
            "TypeScriptOwner::method",
        ),
        (
            ".js",
            "export class JavaScriptOwner {\n    method() {}\n}\n",
            "JavaScriptOwner::method",
        ),
        (".py", "def python_anchor():\n    pass\n", "python_anchor"),
        (
            ".py",
            "class PythonOwner:\n    def method(self):\n        pass\n",
            "PythonOwner::method",
        ),
    ],
)
def test_supported_source_declarations_resolve(
    tmp_path: Path,
    suffix: str,
    source: str,
    fragment: str,
) -> None:
    source_path = tmp_path / f"valid{suffix}"
    source_path.write_text(source, encoding="utf-8")

    validate_repo_reference(
        f"repo://{source_path.name}#{fragment}",
        tmp_path,
        "test evidence",
    )


@pytest.mark.parametrize(
    ("suffix", "source", "fragment"),
    [
        (
            ".rs",
            "fn outer() {\n    fn nested_rust_anchor() {}\n}\n",
            "nested_rust_anchor",
        ),
        (
            ".rs",
            "struct RustOwner;\nimpl RustOwner {\n    fn rust_member_anchor() {}\n}\n",
            "rust_member_anchor",
        ),
        (
            ".ts",
            "function outer() {\n    function nestedTypescriptAnchor() {}\n}\n",
            "nestedTypescriptAnchor",
        ),
        (
            ".js",
            "function outer() {\n    const nestedJavascriptAnchor = () => {};\n}\n",
            "nestedJavascriptAnchor",
        ),
    ],
)
def test_unqualified_source_anchor_rejects_non_file_scope_declaration(
    tmp_path: Path,
    suffix: str,
    source: str,
    fragment: str,
) -> None:
    source_path = tmp_path / f"nested{suffix}"
    source_path.write_text(source, encoding="utf-8")

    with pytest.raises(ContractValidationError, match="source anchor does not exist"):
        validate_repo_reference(
            f"repo://{source_path.name}#{fragment}",
            tmp_path,
            "test evidence",
        )


@pytest.mark.parametrize(
    ("suffix", "source", "fragment"),
    [
        (
            ".rs",
            "fn outer() {\n"
            "    struct Local;\n"
            "    impl Local { fn local_method() {} }\n"
            "}\n",
            "Local::local_method",
        ),
        (
            ".ts",
            "function outer() {\n"
            "    class Local { localMethod(): void {} }\n"
            "}\n",
            "Local::localMethod",
        ),
        (
            ".js",
            "function outer() {\n"
            "    class Local { localMethod() {} }\n"
            "}\n",
            "Local::localMethod",
        ),
        (
            ".py",
            "def outer():\n"
            "    class Local:\n"
            "        def local_method(self):\n"
            "            pass\n",
            "Local::local_method",
        ),
    ],
)
def test_qualified_source_anchor_rejects_function_local_owner(
    tmp_path: Path,
    suffix: str,
    source: str,
    fragment: str,
) -> None:
    source_path = tmp_path / f"local-owner{suffix}"
    source_path.write_text(source, encoding="utf-8")

    with pytest.raises(ContractValidationError, match="source anchor does not exist"):
        validate_repo_reference(
            f"repo://{source_path.name}#{fragment}",
            tmp_path,
            "test evidence",
        )


def test_unqualified_python_anchor_rejects_class_method_but_qualified_resolves(
    tmp_path: Path,
) -> None:
    source_path = tmp_path / "python-class.py"
    source_path.write_text(
        "class PythonOwner:\n"
        "    def class_method_anchor(self):\n"
        "        pass\n",
        encoding="utf-8",
    )

    with pytest.raises(ContractValidationError, match="source anchor does not exist"):
        validate_repo_reference(
            "repo://python-class.py#class_method_anchor",
            tmp_path,
            "test evidence",
        )
    validate_repo_reference(
        "repo://python-class.py#PythonOwner::class_method_anchor",
        tmp_path,
        "test evidence",
    )


def test_qualified_rust_anchor_accepts_lifetime_and_generic_impl_target(
    tmp_path: Path,
) -> None:
    source_path = tmp_path / "generic-impl.rs"
    source_path.write_text(
        "struct GenericOwner<'a, T>(&'a T);\n"
        "impl<'a, T> GenericOwner<'a, T>\n"
        "where\n"
        "    T: 'a,\n"
        "{\n"
        "    pub fn generic_method_anchor(&self) {}\n"
        "}\n",
        encoding="utf-8",
    )
    assert_rust_compiles(source_path, tmp_path / "generic-impl.rmeta")

    validate_repo_reference(
        "repo://generic-impl.rs#GenericOwner::generic_method_anchor",
        tmp_path,
        "test evidence",
    )


def test_qualified_rust_anchor_accepts_lifetime_generic_trait_impl(
    tmp_path: Path,
) -> None:
    source_path = tmp_path / "generic-trait-impl.rs"
    source_path.write_text(
        "trait Trait { fn trait_method_anchor(&self); }\n"
        "struct TraitOwner<'a, T>(&'a T);\n"
        "impl<'a, T> Trait for TraitOwner<'a, T>\n"
        "where\n"
        "    T: 'a,\n"
        "{\n"
        "    fn trait_method_anchor(&self) {}\n"
        "}\n",
        encoding="utf-8",
    )
    assert_rust_compiles(source_path, tmp_path / "generic-trait-impl.rmeta")

    validate_repo_reference(
        "repo://generic-trait-impl.rs#TraitOwner::trait_method_anchor",
        tmp_path,
        "test evidence",
    )


@pytest.mark.parametrize("prefix", ["r", "br", "cr"])
def test_rust_raw_string_contents_cannot_satisfy_anchor(
    tmp_path: Path,
    prefix: str,
) -> None:
    source_path = tmp_path / f"raw-{prefix}.rs"
    literal_type = {
        "r": "&str",
        "br": "&[u8]",
        "cr": "&core::ffi::CStr",
    }[prefix]
    source_path.write_text(
        f'const TEXT: {literal_type} = {prefix}#"ignored "\n'
        "fn raw_literal_anchor() {}\n"
        '"#;\n',
        encoding="utf-8",
    )
    assert_rust_compiles(source_path, tmp_path / f"raw-{prefix}.rmeta")

    with pytest.raises(ContractValidationError, match="source anchor does not exist"):
        validate_repo_reference(
            f"repo://{source_path.name}#raw_literal_anchor",
            tmp_path,
            "test evidence",
        )


def test_rust_byte_c_character_lifetime_and_label_syntax_preserves_scope(
    tmp_path: Path,
) -> None:
    source_path = tmp_path / "rust-literals.rs"
    source_path.write_text(
        'const BYTE_TEXT: &[u8] = b"fn byte_literal_anchor() {}";\n'
        'const C_TEXT: &core::ffi::CStr = c"fn c_literal_anchor() {}";\n'
        "fn labels_and_characters() {\n"
        "    let _character = 'x';\n"
        "    'outer: loop { break 'outer; }\n"
        "}\n"
        "struct SyntaxOwner;\n"
        "impl SyntaxOwner { fn method(&self) {} }\n",
        encoding="utf-8",
    )
    assert_rust_compiles(source_path, tmp_path / "rust-literals.rmeta")

    for rejected in ("byte_literal_anchor", "c_literal_anchor"):
        with pytest.raises(ContractValidationError, match="source anchor does not exist"):
            validate_repo_reference(
                f"repo://rust-literals.rs#{rejected}",
                tmp_path,
                "test evidence",
            )
    validate_repo_reference(
        "repo://rust-literals.rs#SyntaxOwner::method",
        tmp_path,
        "test evidence",
    )


@pytest.mark.parametrize(
    ("suffix", "source", "fragment"),
    [
        (
            ".tsx",
            "export const view = <section>function jsxTextAnchor()</section>;\n",
            "jsxTextAnchor",
        ),
        (
            ".tsx",
            'export const view = <section title="function jsxAttributeAnchor()" />;\n',
            "jsxAttributeAnchor",
        ),
        (
            ".tsx",
            "export const view = <section>{() => { function jsxExpressionAnchor() {} }}</section>;\n",
            "jsxExpressionAnchor",
        ),
        (
            ".tsx",
            "export const view = <Outer><Inner>function jsxNestedAnchor()</Inner></Outer>;\n",
            "jsxNestedAnchor",
        ),
        (
            ".tsx",
            "export const view = <>function jsxFragmentAnchor()</>;\n",
            "jsxFragmentAnchor",
        ),
        (
            ".jsx",
            "export const view = <section>function jsxFileAnchor()</section>;\n",
            "jsxFileAnchor",
        ),
    ],
)
def test_tsx_jsx_source_anchors_fail_closed(
    tmp_path: Path,
    suffix: str,
    source: str,
    fragment: str,
) -> None:
    source_path = tmp_path / f"jsx-source{suffix}"
    source_path.write_text(source, encoding="utf-8")

    with pytest.raises(ContractValidationError, match="unsupported repo:// source file type"):
        validate_repo_reference(
            f"repo://{source_path.name}#{fragment}",
            tmp_path,
            "test evidence",
        )


@pytest.mark.parametrize(
    "source",
    [
        "```markdown\n# hidden-markdown-anchor\n```\n",
        "~~~markdown\n# hidden-markdown-anchor\n~~~\n",
        "<!--\n# hidden-markdown-anchor\n-->\n",
    ],
)
def test_markdown_anchor_rejects_heading_in_non_rendered_content(
    tmp_path: Path,
    source: str,
) -> None:
    source_path = tmp_path / "hidden-heading.md"
    source_path.write_text(source, encoding="utf-8")

    with pytest.raises(ContractValidationError, match="Markdown anchor does not exist"):
        validate_repo_reference(
            "repo://hidden-heading.md#hidden-markdown-anchor",
            tmp_path,
            "test evidence",
        )


def test_markdown_anchor_accepts_rendered_heading(tmp_path: Path) -> None:
    source_path = tmp_path / "rendered-heading.md"
    source_path.write_text("# Rendered heading\n", encoding="utf-8")

    validate_repo_reference(
        "repo://rendered-heading.md#rendered-heading",
        tmp_path,
        "test evidence",
    )
