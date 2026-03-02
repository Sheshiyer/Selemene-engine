# noesis-sdk

Rust SDK for Selemene Engine.

## Features
- typed client for engine and workflow APIs
- profile/config helpers
- optional keychain-backed credential storage
- report rendering helpers

## Install

```bash
cargo add noesis-sdk
```

## Usage

```rust
use noesis_sdk::NoesisClient;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = NoesisClient::new("https://selemene-engine-production.up.railway.app")?;
let _health = client.health().await?;
# Ok(())
# }
```
