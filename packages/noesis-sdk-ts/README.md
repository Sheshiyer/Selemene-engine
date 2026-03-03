# @selemene/sdk

TypeScript SDK for Selemene Engine.

## Installation

```bash
npm i @selemene/sdk
```

## Quickstart (Node.js)

```ts
import { NoesisClient } from "@selemene/sdk";

const client = new NoesisClient("https://selemene-engine-production.up.railway.app", {
  authToken: process.env.NOESIS_API_KEY,
  maxRetries: 2,
  backoffMs: 200,
});

const health = await client.health();
console.log(health.version);
```

## Quickstart (Browser)

```ts
import { NoesisClient } from "@selemene/sdk";

const client = new NoesisClient("https://selemene-engine-production.up.railway.app", {
  authToken: "nk_...",
});

const result = await client.calculate("numerology", {
  birth_data: {
    date: "1991-08-13",
    time: "13:31",
    latitude: 12.9716,
    longitude: 77.5946,
    timezone: "Asia/Kolkata",
    name: "Witness Alchemist",
  },
});

console.log(result.engine_id);
```

## Engine coverage (16)

`biofield`, `biorhythm`, `enneagram`, `face-reading`, `gene-keys`, `human-design`, `i-ching`, `nadabrahman`, `numerology`, `panchanga`, `sacred-geometry`, `sigil-forge`, `tarot`, `transits`, `vedic-clock`, `vimshottari`

## Workflow coverage (6)

`birth-blueprint`, `creative-expression`, `daily-practice`, `decision-support`, `full-spectrum`, `self-inquiry`

## Error handling

```ts
import { NoesisClient, SelemeneError } from "@selemene/sdk";

try {
  await client.workflow("daily-practice", { birth_data: { date: "1991-08-13" } });
} catch (error) {
  if (error instanceof SelemeneError) {
    console.error(error.status, error.details);
  }
}
```

## AbortController support

```ts
const controller = new AbortController();
setTimeout(() => controller.abort(), 1000);
await client.health({ signal: controller.signal });
```

## API Reference

| Method | Description |
|---|---|
| `health(options?)` | Fetches `/health/live` |
| `calculate(engineId, input, options?)` | Runs `/api/v1/engines/{engineId}/calculate` |
| `workflow(workflowId, input, options?)` | Runs `/api/v1/workflows/{workflowId}/execute` |
| `rateLimitInfo` | Last parsed `X-RateLimit-*` and daily quota headers |
