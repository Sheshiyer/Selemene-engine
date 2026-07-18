# @selemene/engine-sdk

Typed TypeScript client SDK for the **4 focus engines** — biofield, face-reading, raaga,
sigil-forge — against the **FROZEN media contracts** (`image_data`, `audio_ref`, `consent`,
`quality`, `generated_image`, `generated_audio`).

P4 SDK task (`p5-p4-next-batch.json` → `p4-sdk-client`; `detailed-task-list.md` Phase 4
T-080..T-094). Used by Sankalpa (Prong 2 surfaces) and contract harnesses.

- **Local-first consent enforcement** (`goal-understanding.md` invariant): any call carrying
  media, requesting sigil image generation, or hitting biofield `/analyze` throws
  `ConsentError` **before any network call** when a valid grant (+scope) is missing.
- **Fail-closed**: the harness guard skips; the SDK throws. No silent fallback, no auto-upload.
- **Configurable base URLs**: ts-engines `@3001`, python biofield sidecar `@8002`,
  noesis-api when P4 lands.

Tags: `phase:integration-p1` `wave:integration-w2` `area:engine-integration`

## Install / layout

Lives at `packages/noesis-engine-sdk` in the Selemene-engine pnpm workspace
(`pnpm-workspace.yaml` covers `packages/*`; same convention as `@selemene/biofield-api-client`).

```bash
cd packages/noesis-engine-sdk
pnpm install
pnpm test       # mocked fetch, no network
pnpm typecheck
pnpm build      # executable ESM + declarations in dist/
npm pack --dry-run
```

## Quick start

```ts
import { EngineClient, createConsent, CONSENT_SCOPES } from '@selemene/engine-sdk'

const client = new EngineClient() // ts @3001, python @8002 by default

// raaga — consent-free when no media is attached
const raaga = await client.raaga.calculate({ parameters: { melakarta: 1, dosha: 'vata' } })
raaga.result.strudel_ratios       // number[] — feed Strudel player directly
raaga.generated_audio?.clip_url   // null until server clip gen (T-031)

// sigil-forge — consent required only when generate_image is requested
const sigil = await client.sigilForge.calculate({
  parameters: { intention: 'I witness my patterns clearly', generate_image: true },
  consent: createConsent([CONSENT_SCOPES.SIGIL_GEN]),
})
sigil.generated_image?.b64_json   // FROZEN generated_image (no vector_path)

// face-reading — consent required when image_data is attached
const face = await client.faceReading.calculate({
  image_data: { b64: pngB64, mime_type: 'image/jpeg' },
  consent: createConsent([CONSENT_SCOPES.FACE_IMAGE]),
})
face.result.constitutional_type

// biofield capture — ALWAYS consent-gated (transmits a frame), multipart to python /analyze
const analysis = await client.biofield.analyze({
  image_data: { b64: pngB64, mime_type: 'image/png' },
  consent: createConsent([CONSENT_SCOPES.BIOFIELD_CAPTURE]),
})
analysis.metrics                  // 11 spatial metrics (biofield-cv/v1)
analysis.quality_assessment

// persisted biofield lifecycle — authenticated noesis-api session then capture
const apiClient = new EngineClient({
  apiUrl: 'https://api.noesis.example',
  defaultHeaders: { Authorization: `Bearer ${token}` },
})
const session = await apiClient.biofield.createSession({
  client_device_id: 'desktop-1',
  viewer_version: 'sankalpa-0.2.0',
})
const persisted = await apiClient.biofield.createCapture(session.id, {
  image_data: { b64: pngB64, mime_type: 'image/png', file_name: 'capture.png' },
  consent: createConsent([CONSENT_SCOPES.BIOFIELD_CAPTURE]),
  capture_metadata: { platform: 'electron' },
})
persisted.reading_id

// biofield engine calculate (Rust engine via ts proxy today, api when P4 lands)
await client.biofield.calculate({ parameters: { /* birth data */ } })

// aggregate health (P4 health surface)
const { ts, python } = await client.health()
```

## Endpoints used

| Surface | Base URL (default) | Routes |
|---|---|---|
| ts-engines server | `http://localhost:3001` (`tsEnginesUrl`) | `POST /engines/{id}/calculate`, `GET /health` |
| python biofield sidecar | `http://localhost:8002` (`pythonBiofieldUrl`) | `POST /analyze` (multipart), `GET /health` |
| noesis-api (P4/P5) | `apiUrl` when configured | `POST /api/v1/engines/{id}/calculate`, `POST /api/v1/biofield/sessions`, `POST /api/v1/biofield/sessions/{id}/captures` |

When `apiUrl` is set, **all** engine `calculate` calls route to the P4 api surface instead
of the ts server. Today the ts registry hosts raaga + sigil-forge live
(`ts-engines/src/index.ts:23-28`); face-reading + biofield engine ids route the same way and
resolve once the P4 api/bridge exposure lands (`p5-p4-next-batch.json` → `p4-api-bridge-health`).

## Consent scopes (FROZEN, `scripts/ext-contract-harness.ts:75-78`)

| Engine call | Scope | Required when |
|---|---|---|
| `biofield.analyze` | `biofield-capture` | always (transmits a capture frame) |
| `biofield.createCapture` | `biofield-capture` | always (uploads and persists a capture frame) |
| `biofield.calculate` | `biofield-capture` | `image_data` attached |
| `faceReading.calculate` | `face-image` | `image_data` attached |
| `raaga.calculate` | `raaga-audio` | `audio_ref` attached |
| `sigilForge.calculate` | `sigil-gen` | `parameters.generate_image === true` |

Consent resolution order: `input.consent` → `mediaRef.consent` (harness pattern).
Missing/wrong scope → `ConsentError` (`status: 0`, `code: 'CONSENT_REQUIRED'`) and **zero
network calls** — local preview stays in Sankalpa until explicit opt-in.

## Usage from Sankalpa (Prong 2)

Sankalpa's `src/renderer/data/engine-media-contracts.ts` owns the local-first UX
(`ImageMediaRef.localDataUrl`, `ConsentState`, consent gates). Bridge to the SDK with its
serializers:

```ts
import { toBackendMediaRef } from '../data/engine-media-contracts'
import { EngineClient } from '@selemene/engine-sdk'

// inside a consent-gated submit handler (assertConsentForBackend(ref) === true):
const out = await engineClient.faceReading.calculate({
  image_data: toBackendMediaRef(imageRef, consent), // localDataUrl → b64, consent mapped
  consent: {
    granted: consent.granted,
    scopes: [consent.engineScope],
    timestamp: consent.grantedAt,
    token: consent.token,
  },
})
```

Renderer holds **no secrets**; `defaultHeaders` is the only auth hook (wire tokens from the
Electron main process when P4 api auth lands). Never construct the client with credentials
in renderer-bundled code.

## Usage from the contract harness

`scripts/ext-contract-harness.ts` (T-024) exercises the same FROZEN shapes with raw fetch.
To type the roundtrips, swap its `roundtrip(...)` bodies for SDK calls — same URLs, same
payloads (`EngineClient({ tsEnginesUrl: FROZEN_TS_URL, pythonBiofieldUrl: FROZEN_PY_URL })`).
The harness stays fail-open (logs + skips); the SDK is fail-closed (throws), so harness
wrappers should catch `ConsentError`/`EngineSdkError` and record `SKIPPED_GUARD`/`FAIL-OPEN`
as it does today.

## Errors

```ts
try {
  await client.raaga.calculate({ parameters: { melakarta: 1 } })
} catch (err) {
  if (err instanceof ConsentError) { /* status 0 — nothing was sent */ }
  if (err instanceof EngineSdkError) {
    err.status   // HTTP status, 0 for client-side guards, -1 for network failures
    err.code     // server error_code (ENGINE_NOT_FOUND, VALIDATION_ERROR, ...) or SDK code
    err.details  // raw server payload
  }
}
```

## References (contract-first; read before editing)

- `docs/plans/engine-integration/p1-w1-worker-bootstrap-packet.md` — P1 agent packet
- `docs/plans/engine-integration/{resources-and-assets,gaps-and-improvements,goal-understanding}.md` — extraction pack
- `.worktrees/T-002-copilot/docs/plans/engine-integration/P1W1-CONTRACTS-FROZEN.md` — FROZEN contracts
- `docs/plans/engine-integration/{EXECUTION-STATUS,P1W2-HANDOFF,detailed-task-list}.md`
- `scripts/ext-contract-harness.ts` — FROZEN payloads + scopes
- `ts-engines/src/types/engine.ts` — TS mirror of the frozen EngineInput/Output
- `sankalpa/src/renderer/data/engine-media-contracts.ts` — Prong 2 mirror
- `python-services/biofield_cv_service/analyze.py` — `/analyze` multipart contract
- `packages/biofield-api-client/src/biofield-client.ts` — client conventions followed here
