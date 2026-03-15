# Engine Onboarding Guide

This guide covers the current repo-supported path for adding a new **native
Rust** engine to Selemene.

It is grounded in the live runtime surfaces:

- trait contract in [crates/noesis-core/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-core/src/lib.rs)
- shared types in [crates/noesis-core/src/types.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-core/src/types.rs)
- orchestrator registration in [crates/noesis-orchestrator/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-orchestrator/src/lib.rs)
- workflow registry in [crates/noesis-orchestrator/src/workflow/registry.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-orchestrator/src/workflow/registry.rs)
- API runtime registration in [crates/noesis-api/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/src/lib.rs)

## Scope

This guide is for **Rust-native engines** that implement the
`ConsciousnessEngine` trait directly.

TypeScript engines follow the HTTP bridge path instead:

- [crates/noesis-bridge/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-bridge/src/lib.rs)
- `BridgeEngine`
- `BridgeManager`

## Current Scaffold Path

The repository does **not** currently ship a `cargo-generate` template for new
engines. The current, proven scaffold path is:

```bash
cargo new crates/engine-your-engine --lib
```

If a template is added later, it should produce the same file shape and trait
surface described below.

## Step 1: Create the Crate

```bash
cargo new crates/engine-example --lib
```

Add the crate to the workspace and declare the minimum dependencies used by
existing engines:

```toml
[package]
name = "engine-example"
version = "0.1.0"
edition = "2021"

[dependencies]
noesis-core = { path = "../noesis-core" }
async-trait = "0.1"
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Step 2: Implement the Trait

Every engine must implement:

- `engine_id()`
- `engine_name()`
- `required_phase()`
- `calculate()`
- `validate()`
- `cache_key()`

Minimal example:

```rust
use async_trait::async_trait;
use chrono::Utc;
use noesis_core::{
    CalculationMetadata, ConsciousnessEngine, EngineError, EngineInput, EngineOutput,
    ValidationResult,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExampleResult {
    pub summary: String,
    pub confidence: f64,
}

pub struct ExampleEngine;

impl ExampleEngine {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ConsciousnessEngine for ExampleEngine {
    fn engine_id(&self) -> &str {
        "example"
    }

    fn engine_name(&self) -> &str {
        "Example"
    }

    fn required_phase(&self) -> u8 {
        0
    }

    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        let result = ExampleResult {
            summary: format!("observed precision: {:?}", input.precision),
            confidence: 1.0,
        };

        Ok(EngineOutput {
            engine_id: self.engine_id().to_string(),
            result: serde_json::to_value(result)
                .map_err(|e| EngineError::InternalError(e.to_string()))?,
            witness_prompt: "What changes when you treat this output as a mirror instead of a verdict?".to_string(),
            consciousness_level: 0,
            metadata: CalculationMetadata {
                calculation_time_ms: 0.0,
                backend: "native-rust".to_string(),
                precision_achieved: "Standard".to_string(),
                cached: false,
                timestamp: Utc::now(),
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        })
    }

    async fn validate(&self, output: &EngineOutput) -> Result<ValidationResult, EngineError> {
        let valid = output.engine_id == self.engine_id();
        Ok(ValidationResult {
            valid,
            confidence: if valid { 1.0 } else { 0.0 },
            messages: if valid {
                vec!["engine_id matched example output".to_string()]
            } else {
                vec!["engine_id did not match example output".to_string()]
            },
        })
    }

    fn cache_key(&self, input: &EngineInput) -> String {
        format!(
            "{}:{}:{}",
            self.engine_id(),
            input.birth_data.as_ref().map(|b| b.date.as_str()).unwrap_or("no-date"),
            input.current_time.timestamp()
        )
    }
}
```

## Step 3: Use the Shared Input and Output Types Correctly

The contract is defined in [crates/noesis-core/src/types.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-core/src/types.rs).

Important constraints:

- `EngineInput.birth_data` is optional at the type level; your engine must
  enforce it if required.
- `EngineOutput.result` is engine-specific JSON.
- `metadata.backend` should state the real backend used.
- `metadata.engine_version` should be populated from `env!("CARGO_PKG_VERSION")`.

## Step 4: Witness Prompt Guidelines

Witness prompts should be reflection-oriented, not deterministic instructions.

Good patterns:

- open-ended question
- points attention back to the user
- invites observation, synthesis, or inquiry

Avoid:

- certainty language
- prescriptive fortune-telling
- generic filler unrelated to the engine result

Existing examples:

- [crates/engine-numerology/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/engine-numerology/src/lib.rs)
- [crates/engine-panchanga/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/engine-panchanga/src/lib.rs)
- [crates/engine-nadabrahman/src/engine.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/engine-nadabrahman/src/engine.rs)

## Step 5: Register the Engine

### Orchestrator Registration

Register the new engine in the API app-state builder:

```rust
orchestrator.register_engine(Arc::new(engine_example::ExampleEngine::new()));
```

The live registration pattern is in:

- [crates/noesis-api/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/src/lib.rs)

You must also add the crate dependency to:

- [crates/noesis-api/Cargo.toml](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/Cargo.toml)

### Workflow Integration

Only add the new engine to workflows if you have a real synthesis reason.

Workflow definitions live in:

- [crates/noesis-orchestrator/src/workflow/registry.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-orchestrator/src/workflow/registry.rs)

If you add an engine to a workflow, update the corresponding workflow tests and
baseline artifacts.

## Step 6: Validation and CI Checklist

Minimum checklist before opening a PR:

- [ ] `cargo check -p engine-your-engine`
- [ ] `cargo test -p engine-your-engine`
- [ ] engine returns deterministic `cache_key()` for identical input
- [ ] `validate()` rejects malformed or mismatched output
- [ ] API runtime can register the engine without compile errors
- [ ] workflow membership changes, if any, are reflected in registry tests and API workflow docs

Helpful existing verification surfaces:

- [crates/noesis-orchestrator/tests/trait_conformance_tests.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-orchestrator/tests/trait_conformance_tests.rs)
- [crates/noesis-api/tests/workflow_tests.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/tests/workflow_tests.rs)
- [crates/noesis-api/tests/routing_enforcement_tests.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/tests/routing_enforcement_tests.rs)

## Step 7: Prove the Guide Works

The acceptance standard for this guide is stronger than “the docs look right.”

The recommended proof loop is:

1. scaffold a temporary engine crate outside the repo
2. implement the minimal trait surface
3. compile it against local path dependencies
4. run a small test that registers the engine with `WorkflowOrchestrator`

That is the validation used for this guide.

## Temporary Validation Example

The validation crate only needs to prove three things:

- the trait implementation compiles
- the engine produces an `EngineOutput`
- the orchestrator can register and execute it

If you follow the current scaffold path and those three checks pass, the guide is
working against the live repository structure.
