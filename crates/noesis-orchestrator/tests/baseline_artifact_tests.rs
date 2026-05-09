use noesis_orchestrator::{SynthesisType, WorkflowRegistry};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize)]
struct EngineMatrix {
    rust_engines: Vec<RustEngineEntry>,
    ts_engines: Vec<TsEngineEntry>,
    counts: EngineMatrixCounts,
}

#[derive(Debug, Deserialize)]
struct RustEngineEntry {
    engine_id: String,
    crate_name: String,
    version: String,
    required_phase: u8,
}

#[derive(Debug, Deserialize)]
struct TsEngineEntry {
    engine_id: String,
    engine_version: String,
}

#[derive(Debug, Deserialize)]
struct EngineMatrixCounts {
    rust_engines: usize,
    ts_engines: usize,
    total_engines: usize,
}

#[derive(Debug, Deserialize)]
struct WorkflowParity {
    workflows: Vec<WorkflowParityEntry>,
}

#[derive(Debug, Deserialize)]
struct WorkflowParityEntry {
    id: String,
    required_phase: u8,
    engine_ids: Vec<String>,
    synthesis_type: SynthesisType,
}

#[derive(Debug, Deserialize)]
struct DependencyGraph {
    workspace_crate_count: usize,
    packages: Vec<DependencyPackage>,
}

#[derive(Debug, Deserialize)]
struct DependencyPackage {
    name: String,
    version: String,
    dependencies: Vec<String>,
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root should resolve")
}

fn parse_json_file<T: for<'de> Deserialize<'de>>(path: &Path) -> T {
    let raw = fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("failed to read {}: {}", path.display(), e);
    });
    serde_json::from_str(&raw).unwrap_or_else(|e| {
        panic!("failed to parse {}: {}", path.display(), e);
    })
}

fn cargo_version_for(crate_name: &str) -> String {
    let manifest_path = workspace_root()
        .join("crates")
        .join(crate_name)
        .join("Cargo.toml");
    let raw = fs::read_to_string(&manifest_path).unwrap_or_else(|e| {
        panic!("failed to read {}: {}", manifest_path.display(), e);
    });

    if raw
        .lines()
        .any(|line| line.trim() == "version.workspace = true")
    {
        let workspace_manifest = workspace_root().join("Cargo.toml");
        let workspace_raw =
            fs::read_to_string(&workspace_manifest).expect("workspace Cargo.toml should read");
        return workspace_raw
            .lines()
            .find_map(|line| line.strip_prefix("version = ").map(str::trim))
            .and_then(|value| value.strip_prefix('"'))
            .and_then(|value| value.strip_suffix('"'))
            .map(str::to_string)
            .unwrap_or_else(|| {
                panic!(
                    "failed to find workspace version in {}",
                    workspace_manifest.display()
                )
            });
    }

    raw.lines()
        .find_map(|line| line.strip_prefix("version = ").map(str::trim))
        .and_then(|value| value.strip_prefix('"'))
        .and_then(|value| value.strip_suffix('"'))
        .map(str::to_string)
        .unwrap_or_else(|| panic!("failed to find version in {}", manifest_path.display()))
}

fn current_ts_package_version() -> String {
    let package_json = workspace_root().join("ts-engines/package.json");
    let raw = fs::read_to_string(&package_json).expect("ts-engines package.json should exist");
    let json: Value = serde_json::from_str(&raw).expect("package.json should parse");
    json["version"]
        .as_str()
        .expect("package version should be a string")
        .to_string()
}

fn cargo_internal_dependency_map() -> BTreeMap<String, Vec<String>> {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--no-deps")
        .arg("--format-version")
        .arg("1")
        .current_dir(workspace_root())
        .output()
        .expect("cargo metadata should run");

    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let metadata: Value =
        serde_json::from_slice(&output.stdout).expect("cargo metadata output should parse");
    let packages = metadata["packages"]
        .as_array()
        .expect("packages should be an array");

    let mut map = BTreeMap::new();
    for package in packages {
        let manifest_path = package["manifest_path"]
            .as_str()
            .expect("manifest_path should be a string");
        if !manifest_path.contains("/crates/") {
            continue;
        }

        let name = package["name"]
            .as_str()
            .expect("package name should be a string")
            .to_string();

        let mut deps: Vec<String> = package["dependencies"]
            .as_array()
            .expect("dependencies should be an array")
            .iter()
            .filter(|dep| dep["path"].is_string())
            .filter(|dep| dep["kind"].is_null())
            .filter_map(|dep| dep["name"].as_str())
            .filter(|dep_name| *dep_name != name)
            .map(str::to_string)
            .collect();
        deps.sort();
        deps.dedup();

        map.insert(name, deps);
    }

    map
}

#[test]
fn engine_matrix_matches_workspace_versions() {
    let path = workspace_root().join("docs/baseline/engine-matrix.json");
    let matrix: EngineMatrix = parse_json_file(&path);

    assert_eq!(matrix.rust_engines.len(), 11);
    assert_eq!(matrix.ts_engines.len(), 6);
    assert_eq!(matrix.counts.rust_engines, 11);
    assert_eq!(matrix.counts.ts_engines, 6);
    assert_eq!(matrix.counts.total_engines, 17);

    for entry in &matrix.rust_engines {
        assert_eq!(entry.version, cargo_version_for(&entry.crate_name));
        assert!(
            entry.required_phase <= 3,
            "{} has invalid phase",
            entry.engine_id
        );
        assert!(!entry.engine_id.trim().is_empty());
    }

    let ts_package_version = current_ts_package_version();
    for entry in &matrix.ts_engines {
        assert_eq!(entry.engine_version, ts_package_version);
        assert!(!entry.engine_id.trim().is_empty());
    }
}

#[test]
fn workflow_parity_matches_registry() {
    let path = workspace_root().join("docs/baseline/workflow-parity.json");
    let parity: WorkflowParity = parse_json_file(&path);
    let registry = WorkflowRegistry::new();

    assert_eq!(parity.workflows.len(), 6);
    assert_eq!(registry.len(), 6);

    for documented in &parity.workflows {
        let actual = registry
            .get(&documented.id)
            .unwrap_or_else(|| panic!("missing workflow {}", documented.id));
        assert_eq!(documented.required_phase, actual.required_phase);
        assert_eq!(documented.engine_ids, actual.engine_ids);
        assert_eq!(documented.synthesis_type, actual.synthesis_type);
    }
}

#[test]
fn dependency_graph_matches_cargo_metadata() {
    let path = workspace_root().join("docs/baseline/dependency-graph.json");
    let graph: DependencyGraph = parse_json_file(&path);
    let metadata_map = cargo_internal_dependency_map();

    assert_eq!(graph.workspace_crate_count, 26);
    assert_eq!(graph.packages.len(), metadata_map.len());

    let documented: BTreeMap<String, Vec<String>> = graph
        .packages
        .iter()
        .map(|package| {
            assert_eq!(package.version, cargo_version_for(&package.name));
            (package.name.clone(), package.dependencies.clone())
        })
        .collect();

    assert_eq!(documented, metadata_map);
}
