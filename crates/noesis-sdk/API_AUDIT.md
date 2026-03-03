# noesis-sdk Public API Audit (Wave W2 #440)

This audit covers the primary public API surface requested in issue #440:
- `NoesisClient`
- `LocalProfile`
- `Config`
- `KeychainStore`
- `MarkdownRenderer`
- `ConsciousnessLevel`
- `Error`

## Verification commands

```bash
cargo doc -p noesis-sdk --no-deps
cargo check -p noesis-sdk
cargo test -p noesis-sdk
```

## API Surface Coverage Snapshot

### NoesisClient
- `new`
- `builder`
- `with_api_key`
- `calculate`
- `workflow`
- `list_engines`
- `list_workflows`
- `list_readings`
- `get_reading`
- `health`
- `get_me`
- `update_me`

### Config
- `load`, `load_from_file`, `load_from_path`
- `dir`, `path`, `save`
- `engine_config`, `builder`
- builder methods for URL/auth/timeout/cache/retry/backoff/pool

### LocalProfile
- profile lifecycle (`new`, `load`, `save`, `delete`)
- conversion (`to_engine_input`, `to_engine_input_with_options`)
- mutation helpers (`update_birth_data`, `set_preference`, `validate`)
- sync API (`sync`) with deterministic conflict policy

### KeychainStore
- `new`, `with_service`
- credential CRUD helpers

### MarkdownRenderer
- `new`, `minimal`
- `render_engine_output`, `render_workflow_result`
- `render`, `render_workflow`

### ConsciousnessLevel
- level constants and display utility helpers

## Example snippets

### Rust client quickstart

```rust
use noesis_sdk::{Config, LocalProfile, NoesisClient};

# async fn run() -> Result<(), noesis_sdk::Error> {
let config = Config::load()?;
let client = NoesisClient::new(&config)?;
let profile = LocalProfile::load()?.unwrap_or_default();
let output = client.calculate("numerology", profile.to_engine_input()).await?;
println!("{}", output.witness_prompt);
# Ok(())
# }
```

### Profile sync

```rust
use noesis_sdk::{Config, LocalProfile, NoesisClient};

# async fn run() -> Result<(), noesis_sdk::Error> {
let config = Config::load()?;
let client = NoesisClient::new(&config)?;
let mut profile = LocalProfile::load()?.unwrap_or_default();
let sync = profile.sync(&client).await?;
println!("synced at {}", sync.synced_at);
# Ok(())
# }
```

### Retry/pooling configuration

```rust
use noesis_sdk::Config;

let cfg = Config::builder()
    .max_retries(3)
    .backoff_ms(200)
    .pool_max_idle_per_host(16)
    .build();
```
