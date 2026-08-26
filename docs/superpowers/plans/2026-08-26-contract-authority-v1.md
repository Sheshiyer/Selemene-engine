# Selemene Contract Authority v1 Implementation Plan

> **For the implementation agent:** REQUIRED SKILL: Use superpowers:executing-plans to execute this plan task-by-task with strict RED-GREEN-REFACTOR evidence.

**Goal:** Establish one language-neutral v1 contract authority for engine requests, results, errors, consent, provenance, and capability discovery, then prove additive parity across Rust, OpenAPI, and maintained TypeScript surfaces without changing engine semantics.

**Architecture:** `contracts/v1/` is the canonical source of truth. Draft 2020-12 JSON Schemas and hand-checked golden fixtures define wire behavior. A repository-owned Python validator meta-validates every schema, resolves local references, validates fixtures, and fails on manifest drift. Rust and TypeScript packages remain compatibility adapters in this slice; focused tests load the same fixtures and prove their public types/OpenAPI schemas can represent the canonical fields.

**Tech Stack:** JSON Schema Draft 2020-12, Python 3.12 + `jsonschema`, Rust/Serde/Utoipa, TypeScript/Bun/Vitest, pnpm repository gates.

**Boundary:** Work only in `.worktrees/selemene-contract-convergence` on `codex/selemene-contract-convergence`. Do not modify engine algorithms, providers, routes, auth behavior, persistence, external repositories, live services, or deployment state. Do not merge or push this branch.

---

## Task 1: Create the canonical schema and fixture authority

**Files:**
- Create: `contracts/v1/README.md`
- Create: `contracts/v1/manifest.json`
- Create: `contracts/v1/schemas/engine-request.schema.json`
- Create: `contracts/v1/schemas/engine-result.schema.json`
- Create: `contracts/v1/schemas/error.schema.json`
- Create: `contracts/v1/schemas/consent.schema.json`
- Create: `contracts/v1/schemas/provenance.schema.json`
- Create: `contracts/v1/schemas/engine-capability.schema.json`
- Create: `contracts/v1/fixtures/engine-request.json`
- Create: `contracts/v1/fixtures/engine-request-legacy.json`
- Create: `contracts/v1/fixtures/engine-result.json`
- Create: `contracts/v1/fixtures/error.json`
- Create: `contracts/v1/fixtures/engine-capability.json`
- Create: `tests/scripts/test_validate_contracts.py`

**Step 1: Write the failing authority tests**

Create tests that invoke `scripts/validate_contracts.py` as a subprocess against the repository authority and temporary mutated copies. Each test must name the break it catches:

```python
def test_repository_contract_authority_is_valid():
    result = run_validator(REPO_ROOT / "contracts" / "v1")
    assert result.returncode == 0, result.stderr

def test_missing_manifest_schema_fails_closed(tmp_path):
    authority = copy_authority(tmp_path)
    (authority / "schemas" / "error.schema.json").unlink()
    result = run_validator(authority)
    assert result.returncode != 0
    assert "error.schema.json" in result.stderr

def test_invalid_fixture_fails_closed(tmp_path):
    authority = copy_authority(tmp_path)
    write_json(authority / "fixtures" / "engine-result.json", {"contract_version": "v1"})
    result = run_validator(authority)
    assert result.returncode != 0
    assert "engine-result.json" in result.stderr
```

Run: `python3 -m pytest tests/scripts/test_validate_contracts.py -q`

Expected RED: failures because the validator and authority do not exist.

**Step 2: Add the six schemas and golden fixtures**

Use `$schema: "https://json-schema.org/draft/2020-12/schema"`, stable `$id` values under `https://noesis.tryambakam.org/contracts/v1/`, `additionalProperties: false` for bounded shared objects, and `contract_version: {"const": "v1"}`. Keep engine-specific values inside `parameters`/`result`; model legacy birth/time/location/precision/options fields as optional additive compatibility properties.

The request must reference `consent.schema.json` for media consent. The result must preserve singular and plural witness prompts plus optional generated media. Error keys are `status`, `error_code`, `message`, `error`, `details`, `trace_id`, and `contract_version`. Provenance must use runtime-kind enums and exclude endpoints/secrets. Capability discovery must include availability, runtime kind, dependencies, and contract version.

**Step 3: Re-run the test to confirm the intended remaining failure**

Run: `python3 -m pytest tests/scripts/test_validate_contracts.py -q`

Expected RED: validator still missing; schema/fixture files now exist.

**Step 4: Commit the authority artifacts and RED tests**

```bash
git add contracts/v1 tests/scripts/test_validate_contracts.py
git commit -m "test(contracts): define v1 authority fixtures"
```

## Task 2: Implement fail-closed contract validation and gate wiring

**Files:**
- Create: `scripts/validate_contracts.py`
- Modify: `requirements-gates.txt`
- Modify: `package.json`
- Modify: `tests/scripts/test_gate_wiring.py`

**Step 1: Extend the gate-wiring test first**

Add a test that executes the root script contract rather than grepping prose:

```python
def test_gate_scripts_runs_contract_validator():
    scripts = json.loads((REPO_ROOT / "package.json").read_text())["scripts"]
    assert "python3 scripts/validate_contracts.py" in scripts["gate:scripts"]
```

Run: `python3 -m pytest tests/scripts/test_validate_contracts.py tests/scripts/test_gate_wiring.py -q`

Expected RED: authority tests fail because the validator is absent; wiring test fails because `gate:scripts` omits it.

**Step 2: Implement the minimal validator**

Pin `jsonschema==4.26.0` in `requirements-gates.txt`. The script accepts optional `--root`; default is `<repo>/contracts/v1`. It must:

1. Parse the manifest and require exactly the six named schemas.
2. Reject missing, extra, or duplicate manifest schema entries.
3. Parse each schema and call `Draft202012Validator.check_schema`.
4. Register all schema `$id` values for local reference resolution.
5. Validate each manifest fixture against its declared schema.
6. Scan error/provenance fixtures recursively for credential, token, secret, stack, and endpoint-like keys.
7. Print a bounded success receipt and exit nonzero with path-qualified diagnostics on any error.

No network access and no remote reference resolution are permitted.

**Step 3: Wire validation into the root gate**

Prepend `python3 scripts/validate_contracts.py &&` within `gate:scripts` so contract drift fails before later gates.

**Step 4: Verify GREEN**

Run:

```bash
python3 -m pytest tests/scripts/test_validate_contracts.py tests/scripts/test_gate_wiring.py -q
python3 scripts/validate_contracts.py
```

Expected GREEN: all focused tests pass and the validator reports six schemas plus five fixtures.

**Step 5: Commit**

```bash
git add scripts/validate_contracts.py requirements-gates.txt package.json tests/scripts
git commit -m "feat(contracts): validate v1 authority fail closed"
```

## Task 3: Add Rust canonical contract types and fixture round trips

**Files:**
- Create: `crates/noesis-core/tests/contract_v1_authority.rs`
- Create: `crates/noesis-core/src/contract.rs`
- Modify: `crates/noesis-core/src/lib.rs`

**Step 1: Write failing fixture tests**

Tests must `include_str!` the canonical fixtures and deserialize them into public `noesis_core::contract` types. Assertions use literal values for `contract_version`, engine ID, runtime kind, availability, witness prompts, and error code. A round-trip test must prove canonical required fields survive serialization.

Run: `cargo test -p noesis-core --test contract_v1_authority --locked`

Expected RED: `noesis_core::contract` does not exist.

**Step 2: Add minimal additive Serde types**

Implement `ContractVersion`, `ContractEngineRequest`, `ContractEngineResult`, `ContractError`, `Consent`, `Quality`, `Provenance`, `EngineCapability`, `CapabilityAvailability`, and `RuntimeKind`. Use `#[serde(deny_unknown_fields)]` only for bounded shared subobjects; retain `serde_json::Value` for engine-specific `parameters`, `result`, and error `details`. Do not replace existing `EngineInput` or `EngineOutput`.

**Step 3: Verify GREEN and regressions**

Run:

```bash
cargo test -p noesis-core --test contract_v1_authority --locked
cargo test -p noesis-core --lib --locked
```

Expected GREEN: canonical fixtures deserialize and round-trip; existing core tests remain green.

**Step 4: Commit**

```bash
git add crates/noesis-core/src crates/noesis-core/tests/contract_v1_authority.rs
git commit -m "feat(core): add additive v1 contract types"
```

## Task 4: Prove API/OpenAPI compatibility with canonical v1

**Files:**
- Modify: `crates/noesis-api/tests/openapi_schema_tests.rs`
- Modify only if the failing parity test requires additive exposure: `crates/noesis-api/src/lib.rs`

**Step 1: Add a failing OpenAPI parity test**

Load the canonical request/result/error schemas as JSON. Generate the live Utoipa document. Assert that current API request/output/error components contain the canonical compatibility properties actually exposed at runtime, including `consciousness_level`, `parameters`, legacy fields, media, witness prompt forms, generated media, status/code/message/error/details/trace, and `envelope_version` compatibility with v1.

Run: `cargo test -p noesis-api --test openapi_schema_tests contract_v1 --locked`

Expected RED: identify the first real schema parity gap; do not predict or fabricate it.

**Step 2: Make only additive compatibility changes**

If the test exposes a missing canonical version field, add a serialized/defaulted or OpenAPI-only additive field without changing existing endpoint paths, auth, status codes, routing, or existing response keys. If current runtime behavior is already compatible, correct the test mapping rather than rewriting DTOs.

**Step 3: Verify GREEN and API regressions**

Run:

```bash
cargo test -p noesis-api --test openapi_schema_tests --locked
cargo test -p noesis-api --test integration_tests --test error_response_snapshot_tests --locked
```

Expected GREEN: OpenAPI parity and existing envelope/error snapshots pass.

**Step 4: Commit**

```bash
git add crates/noesis-api
git commit -m "test(api): enforce canonical v1 openapi parity"
```

## Task 5: Converge maintained TypeScript contract surfaces

**Files:**
- Create: `packages/noesis-engine-sdk/src/contract-v1.ts`
- Create: `packages/noesis-engine-sdk/tests/contract-v1.test.ts`
- Modify: `packages/noesis-engine-sdk/src/index.ts`
- Create: `ts-engines/tests/contract-v1.test.ts`
- Modify: `ts-engines/src/types/engine.ts`
- Modify: `ts-engines/src/types/index.ts`
- Modify: `packages/noesis-sdk-ts/src/index.test.ts`
- Modify: `packages/noesis-sdk-ts/src/index.ts`

**Step 1: Write failing shared-fixture tests**

Load canonical fixtures from `contracts/v1`. Use `satisfies`/explicit assignments so the compiler proves the fixtures match exported request, result, error, provenance, consent, and capability types. Runtime assertions use literal `v1`, engine ID, runtime kind, prompt count, and error code. In the general SDK, add a test that the 17 public mirror IDs and envelope field names cannot drift from the canonical public capability fixture.

Run:

```bash
cd packages/noesis-engine-sdk && bun test tests/contract-v1.test.ts && npm run typecheck
cd ../../../ts-engines && bun test tests/contract-v1.test.ts && bun run typecheck
cd ../packages/noesis-sdk-ts && npm test -- --run src/index.test.ts && npm run typecheck
```

Expected RED: canonical v1 exports and parity assignments are missing.

**Step 2: Add minimal additive exports**

Export `CONTRACT_VERSION = "v1"` and matching contract interfaces from the engine SDK. Add optional compatibility fields to TS engine types without removing current fields. Reconcile the general SDK's request/output/info types and engine identifiers additively; retain existing public aliases where removal would break consumers. Do not change fetch routes, auth headers, or HTTP behavior.

**Step 3: Verify GREEN and package regressions**

Run:

```bash
cd packages/noesis-engine-sdk && bun test && npm run typecheck && npm run build
cd ../../../ts-engines && bun test && bun run typecheck
cd ../packages/noesis-sdk-ts && npm test && npm run typecheck && npm run build
```

Expected GREEN: all packages accept the same fixtures and existing suites remain green.

**Step 4: Commit**

```bash
git add packages/noesis-engine-sdk packages/noesis-sdk-ts ts-engines
git commit -m "feat(ts): converge v1 contract surfaces"
```

## Task 6: Document authority, audit scope, and run the full gate

**Files:**
- Modify: `contracts/v1/README.md`
- Modify: `docs/plans/selemene-engine/ROADMAP.md` only if the isolated branch already tracks this file; otherwise leave the primary untracked planning document untouched
- Create: `docs/plans/selemene-engine/CONTRACT-V1-EVIDENCE.md`

**Step 1: Document the operational boundary**

Explain authority precedence, schema/fixture update procedure, compatibility-adapter status, validation commands, and the explicit deferral of engine semantic migration. Record actual RED/GREEN commands and counts only after executing them.

**Step 2: Run focused and full verification from the worktree**

```bash
python3 scripts/validate_contracts.py
python3 -m pytest tests/scripts -q
cargo test -p noesis-core --test contract_v1_authority --locked
cargo test -p noesis-api --test openapi_schema_tests --test integration_tests --test error_response_snapshot_tests --locked
cd packages/noesis-engine-sdk && bun test && npm run typecheck && npm run build
cd ../noesis-sdk-ts && npm test && npm run typecheck && npm run build
cd ../../ts-engines && bun test && bun run typecheck
cd .. && pnpm run gate
git diff --check
git status --short --branch
```

Expected GREEN: every focused command and the complete root gate exit zero. The status contains only intentional contract-branch changes.

**Step 3: Perform mutation and anti-scope review**

Mentally mutate: remove a manifest entry, remove a required fixture field, change `v1`, add a credential key, remove a Rust/TS field, or drift an OpenAPI property. Confirm at least one named test catches each realistic mutation. Confirm no changed file implements engine calculations, routing, providers, persistence, external platforms, or live operations.

**Step 4: Commit evidence**

```bash
git add contracts/v1/README.md docs/plans/selemene-engine/CONTRACT-V1-EVIDENCE.md
git commit -m "docs(contracts): record v1 convergence evidence"
```

**Step 5: Independent review handoff**

Request a read-only review for P0/P1 contract drift, compatibility breaks, false-green validation, and engine-scope leakage. Remediate valid findings with a fresh RED-GREEN cycle, rerun the affected focused suites, then rerun `pnpm run gate` before declaring the branch ready. Do not merge, push, publish, deploy, or begin engine work.
