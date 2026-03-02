# @selemene/sdk

TypeScript SDK for Selemene Engine.

## Install

```bash
npm i @selemene/sdk
```

## Quickstart

```ts
import { NoesisClient } from "@selemene/sdk";

const client = new NoesisClient("https://selemene-engine-production.up.railway.app");

const health = await client.health();
console.log(health.version);
```

## API

- `health()`
- `calculate(engineId, input)`
- `workflow(workflowId, input)`
