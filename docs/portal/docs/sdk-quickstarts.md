---
title: SDK Quickstarts
sidebar_position: 4
---

## Rust SDK

```rust
use noesis_sdk::{Config, LocalProfile, NoesisClient, TarotSpread};

# async fn run() -> Result<(), noesis_sdk::Error> {
let config = Config::load()?;
let client = NoesisClient::new(&config)?;
let profile = LocalProfile::load()?.unwrap_or_default();
let input = profile.to_engine_input();
let output = client.calculate("numerology", input).await?;
println!("{}", output.witness_prompt);

let tarot = client
  .calculate_tarot(
    profile.to_engine_input(),
    "What does this decision ask of me?",
    TarotSpread::YesNo,
  )
  .await?;
println!("{}", tarot.witness_prompt);
# Ok(())
# }
```

## TypeScript SDK

```ts
import { createClient } from "@selemene/noesis-sdk-ts";

const client = createClient({
  baseUrl: "https://selemene-engine-production.up.railway.app",
  apiKey: process.env.NOESIS_API_KEY,
});

const result = await client.calculate("tarot", {
  current_time: new Date().toISOString(),
  precision: "standard",
  options: {}
});
```
