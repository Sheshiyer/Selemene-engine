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
