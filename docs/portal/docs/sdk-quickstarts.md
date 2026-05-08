---
title: SDK Quickstarts
sidebar_position: 4
---

> Base URL: `https://selemene.tryambakam.space` · Version: 3.3.0

## Rust SDK

```rust
use noesis_sdk::{Config, LocalProfile, NoesisClient};

# async fn run() -> Result<(), noesis_sdk::Error> {
let config = Config::load()?;
let client = NoesisClient::new(&config)?;
let profile = LocalProfile::load()?.unwrap_or_default();
let input = profile.to_engine_input();
let output = client.calculate("numerology", input).await?;
println!("{}", output.witness_prompt);
# Ok(())
# }
```

## TypeScript SDK

```ts
import { createClient } from "@selemene/noesis-sdk-ts";

const client = createClient({
  baseUrl: "https://selemene.tryambakam.space",
  apiKey: process.env.NOESIS_API_KEY,
});

// Single engine
const result = await client.calculate("tarot", {
  current_time: new Date().toISOString(),
  precision: "standard",
  options: {}
});

// Workflow (returns reading-object in v3.3.0+)
const reading = await client.workflow("daily-practice", {
  birth_data: {
    date: "1991-08-13",
    time: "13:19",
    timezone: "Asia/Kolkata",
    latitude: 12.9716,
    longitude: 77.5946,
  }
});
// reading.reading_id, reading.witness_layer.question, etc.
```

## Python (requests)

```python
import os, requests

NOESIS_URL = "https://selemene.tryambakam.space"
headers = {"X-API-Key": os.environ["NOESIS_API_KEY"]}

# Health check
r = requests.get(f"{NOESIS_URL}/health/live")
print(r.json())  # {"status": "ok", "version": "3.3.0", ...}

# Workflow
r = requests.post(
    f"{NOESIS_URL}/api/v1/workflows/birth-blueprint/execute",
    headers=headers,
    json={"birth_data": {"date": "1991-08-13", "time": "13:19",
                         "timezone": "Asia/Kolkata", "latitude": 12.9716, "longitude": 77.5946}}
)
data = r.json()
print(data["witness_layer"]["question"])
```
