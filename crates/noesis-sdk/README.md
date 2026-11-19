# noesis-sdk

Rust SDK for Selemene Engine.

## Features
- typed client for engine and workflow APIs
- configurable retry/backoff and HTTP connection pooling
- profile/config helpers and offline-first profile sync
- optional keychain-backed credential storage
- report rendering helpers

## Install

```bash
cargo add noesis-sdk
```

## Usage

```rust
use noesis_sdk::{Config, LocalProfile, NoesisClient};

# async fn example() -> Result<(), noesis_sdk::Error> {
let config = Config::load()?;
let client = NoesisClient::new(&config)?;
let mut profile = LocalProfile::load()?.unwrap_or_default();
let _ = profile.sync(&client).await?;
let output = client.calculate("numerology", profile.to_engine_input()).await?;
println!("{}", output.witness_prompt);
# Ok(())
# }
```


## Contract provenance in engine results

`NoesisClient::calculate` retains the API's optional contract-v1 provenance.
Use `output.contract_provenance()` for its typed runtime, implementation version,
cache/fallback state, optional backend/provider and optional confidence. This
block stays distinct from engine-owned `result.provenance` or
`result.provenance_detail`.

JSON serialization and JSON report exports put the block back at top-level
`provenance` and omit internal transport scaffolding from `result`. Markdown
(including the TUI result screen) and text exports show supplied provenance
when metadata is enabled. Existing `EngineOutput` constructors remain valid.
Legacy saved outputs without a contract block remain readable and are not
assigned inferred provenance. Malformed or conflicting envelope blocks fail
parsing instead of disappearing silently. No saved result is migrated or
recalculated by this change; SDK/TUI registry publication remains a separate gate.
