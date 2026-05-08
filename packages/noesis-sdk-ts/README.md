# @noesis/sdk

TypeScript SDK for the Noesis Consciousness Engine — 16 engines covering Vedic astrology, Human Design, Gene Keys, numerology, biorhythm, Tarot, I-Ching, and more.

**API**: `https://selemene.tryambakam.space` | **Web viewer**: `https://noesis.tryambakam.space`

## Installation

```bash
npm i @noesis/sdk
```

## Quickstart (Node.js)

```ts
import { NoesisClient } from "@noesis/sdk";

const client = new NoesisClient("https://selemene.tryambakam.space", {
  authToken: process.env.NOESIS_API_KEY,
  maxRetries: 2,
  backoffMs: 200,
});

const health = await client.health();
console.log(health.version); // "3.3.0"
```

## Quickstart (Browser)

```ts
import { NoesisClient } from "@noesis/sdk";

const client = new NoesisClient("https://selemene.tryambakam.space", {
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
import { NoesisClient, SelemeneError } from "@noesis/sdk";

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

## v3.3.0 reading-object contract

Workflow responses now include reading persistence fields:

```ts
const result = await client.workflow("full-spectrum", { birth_data: { date: "1991-08-13" } });
console.log(result.reading_id);         // "r_abc123"
console.log(result.reading_url);        // "https://noesis.tryambakam.space/readings/r_abc123"
console.log(result.witness_layer?.question); // "What are you not saying?"
```

## API Reference

| Method | Description |
|---|---|
| `health(options?)` | `/health/live` — server health + version |
| `calculate(engineId, input, options?)` | `/api/v1/engines/{id}/calculate` |
| `workflow(workflowId, input, options?)` | `/api/v1/workflows/{id}/execute` |
| `listEngines(options?)` | `/api/v1/engines` — engine catalogue |
| `listWorkflows(options?)` | `/api/v1/workflows` — workflow catalogue |
| `getEngineInfo(engineId, options?)` | `/api/v1/engines/{id}` |
| `getWorkflowInfo(workflowId, options?)` | `/api/v1/workflows/{id}` |
| `getMe(options?)` | `/api/v1/auth/me` — user profile |
| `getMyUsage(options?)` | `/api/v1/usage/me` — credit usage |
| `getBillingBalance(options?)` | `/api/v1/billing/balance` |
| `getBillingSubscription(options?)` | `/api/v1/billing/subscription` |
| `createCheckout(request, options?)` | `/api/v1/billing/checkout` — Dodo checkout URL |
| `getBillingPortal(options?)` | `/api/v1/billing/portal` — Dodo portal URL |
| `listReadings(opts?, options?)` | `/api/v1/readings` — reading history |
| `getReading(readingId, options?)` | `/api/v1/readings/{id}` |
| `interpretWitness(input, options?)` | `/api/v1/witness/interpret` |
| `validateEngine(engineId, options?)` | Returns `{ valid: boolean }` |
| `rateLimitInfo` | Last parsed `X-RateLimit-*` headers |
