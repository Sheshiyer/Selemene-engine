/**
 * Typed client SDK for the 4 focus engines with FROZEN media contracts.
 *
 * Surfaces (two-prong per goal-understanding.md):
 * - ts-engines server (default http://localhost:3001): raaga + sigil-forge live today
 *   (registry ts-engines/src/index.ts:23-28); face-reading + biofield engine ids route
 *   here until the P4 noesis-api exposure lands (detailed-task-list.md Phase 4 T-080..T-094).
 * - python biofield sidecar (default http://localhost:8002): biofield-capture /analyze
 *   (multipart; python-services/biofield_cv_service/analyze.py:316+; contract biofield-cv/v1).
 * - noesis-api (optional apiUrl): when configured, all engine calculate calls route to
 *   `${apiUrl}/api/v1/engines/{id}/calculate` (P4 surface; p5-p4-next-batch.json p4-sdk-client).
 *
 * Local-first consent enforcement: any call carrying image_data / audio_ref, requesting
 * sigil image generation, or hitting biofield /analyze throws ConsentError BEFORE the
 * network when consent(granted + scope) is missing. No silent fallback. No auto-upload.
 *
 * Payload shapes match scripts/ext-contract-harness.ts:81-115 (FROZEN samples).
 *
 * Cites (mandatory): p1-w1-worker-bootstrap-packet.md, resources-and-assets.md,
 * gaps-and-improvements.md, goal-understanding.md, EXECUTION-STATUS.md, P1W2-HANDOFF.md,
 * P1W1-CONTRACTS-FROZEN.md (.worktrees/T-002-copilot), detailed-task-list.md Phase 4,
 * ts-engines/src/types/engine.ts, engine-media-contracts.ts (sankalpa), ext-contract-harness.ts.
 *
 * Tags: phase:integration-p1 wave:integration-w2 area:engine-integration
 */

import { CONSENT_SCOPES, requireConsent, resolveConsent } from './consent.js'
import { EngineSdkError } from './errors.js'
import type {
  BiofieldAnalyzeInput,
  BiofieldAnalyzeResponse,
  BiofieldCalculateInput,
  BiofieldCapture,
  BiofieldCreateCaptureInput,
  BiofieldCreateSessionInput,
  BiofieldSession,
  Consent,
  EngineInput,
  EngineOutput,
  FaceReadingCalculateInput,
  FaceReadingResult,
  HealthResponse,
  PythonSidecarHealthResponse,
  RaagaCalculateInput,
  RaagaResult,
  SigilForgeCalculateInput,
  SigilForgeResult,
} from './types.js'

export interface EngineClientConfig {
  /** ts-engines server base URL. Default http://localhost:3001 (TS_ENGINES_URL). */
  tsEnginesUrl?: string
  /** python biofield CV sidecar base URL. Default http://localhost:8002 (PYTHON_BIOFIELD_URL). */
  pythonBiofieldUrl?: string
  /** noesis-api base URL (P4). When set, engine calculate calls route here instead of ts-engines. */
  apiUrl?: string
  /** Injectable fetch (tests / Electron main proxy). Defaults to global fetch. */
  fetchImpl?: typeof fetch
  /** Extra headers on every request (e.g. auth once P4 api lands). */
  defaultHeaders?: Record<string, string>
}

const DEFAULT_TS_URL = 'http://localhost:3001'
const DEFAULT_PY_URL = 'http://localhost:8002'

interface ErrorPayloadShape {
  error?: string
  message?: string
  error_message?: string
  detail?: string
  error_code?: string
}

function stripTrailingSlash(url: string): string {
  return url.endsWith('/') ? url.slice(0, -1) : url
}

function b64ToArrayBuffer(b64: string): ArrayBuffer {
  const bin = atob(b64)
  const buffer = new ArrayBuffer(bin.length)
  const bytes = new Uint8Array(buffer)
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i)
  return buffer
}

function parsePayload(text: string): unknown {
  if (!text) return null
  try {
    return JSON.parse(text)
  } catch {
    return { raw: text }
  }
}

function resolveErrorMessage(payload: unknown, status: number): { message: string; code?: string } {
  if (payload && typeof payload === 'object') {
    const candidate = payload as ErrorPayloadShape
    const message =
      (typeof candidate.error === 'string' && candidate.error) ||
      (typeof candidate.message === 'string' && candidate.message) ||
      (typeof candidate.error_message === 'string' && candidate.error_message) ||
      (typeof candidate.detail === 'string' && candidate.detail) ||
      undefined
    if (message) return { message, code: candidate.error_code }
  }
  return { message: `Request failed: ${status}` }
}

export class EngineClient {
  readonly biofield: BiofieldEngineApi
  readonly faceReading: FaceReadingEngineApi
  readonly raaga: RaagaEngineApi
  readonly sigilForge: SigilForgeEngineApi

  private readonly tsUrl: string
  private readonly pyUrl: string
  private readonly apiUrl?: string
  private readonly fetchImpl: typeof fetch
  private readonly defaultHeaders: Record<string, string>

  constructor(config: EngineClientConfig = {}) {
    this.tsUrl = stripTrailingSlash(config.tsEnginesUrl ?? DEFAULT_TS_URL)
    this.pyUrl = stripTrailingSlash(config.pythonBiofieldUrl ?? DEFAULT_PY_URL)
    this.apiUrl = config.apiUrl ? stripTrailingSlash(config.apiUrl) : undefined
    this.fetchImpl = config.fetchImpl ?? (globalThis.fetch.bind(globalThis) as typeof fetch)
    this.defaultHeaders = config.defaultHeaders ?? {}

    this.biofield = new BiofieldEngineApi(this)
    this.faceReading = new FaceReadingEngineApi(this)
    this.raaga = new RaagaEngineApi(this)
    this.sigilForge = new SigilForgeEngineApi(this)
  }

  /** Aggregate health: ts-engines /health + python sidecar /health (P4 health surface). */
  async health(): Promise<{ ts: HealthResponse; python: PythonSidecarHealthResponse }> {
    const [ts, python] = await Promise.all([
      this.request<HealthResponse>('GET', `${this.tsUrl}/health`),
      this.request<PythonSidecarHealthResponse>('GET', `${this.pyUrl}/health`),
    ])
    return { ts, python }
  }

  /** @internal POST an EngineInput to the calculate route for `engineId` (api when P4 configured, else ts server). */
  async engineCalculate<TResult>(
    engineId: string,
    input: EngineInput,
  ): Promise<EngineOutput<TResult>> {
    const base = this.apiUrl ?? this.tsUrl
    const path = this.apiUrl
      ? `/api/v1/engines/${encodeURIComponent(engineId)}/calculate`
      : `/engines/${encodeURIComponent(engineId)}/calculate`
    return this.request<EngineOutput<TResult>>('POST', `${base}${path}`, {
      body: JSON.stringify(input),
    })
  }

  /** @internal POST multipart to python biofield /analyze (contract biofield-cv/v1). */
  async biofieldAnalyze(input: BiofieldAnalyzeInput): Promise<BiofieldAnalyzeResponse> {
    const form = new FormData()
    const b64 = input.image_data.b64
    if (!b64) {
      throw new EngineSdkError(
        '[biofield-capture] analyze requires image_data.b64 (inline capture frame). ' +
          'Upload-by-reference is not supported by the python sidecar contract.',
        0,
        'MEDIA_REQUIRED',
      )
    }
    const mime = input.image_data.mime_type ?? 'image/png'
    const ext = mime.split('/')[1] ?? 'png'
    const blob = new Blob([b64ToArrayBuffer(b64)], { type: mime })
    form.append('image', blob, `capture.${ext}`)
    if (input.algorithms && input.algorithms.length > 0) {
      form.append('algorithms', JSON.stringify(input.algorithms))
    }
    if (input.options) form.append('options', JSON.stringify(input.options))
    if (input.capture_metadata) {
      form.append('capture_metadata', JSON.stringify(input.capture_metadata))
    }
    return this.request<BiofieldAnalyzeResponse>('POST', `${this.pyUrl}/analyze`, {
      body: form,
    })
  }

  /** @internal Create an authenticated noesis-api biofield session. */
  async biofieldCreateSession(
    input: BiofieldCreateSessionInput,
  ): Promise<BiofieldSession> {
    return this.request<BiofieldSession>(
      'POST',
      `${this.requireApiUrl('biofield session creation')}/api/v1/biofield/sessions`,
      { body: JSON.stringify(input) },
    )
  }

  /** @internal Upload a consent-approved capture to an active noesis-api session. */
  async biofieldCreateCapture(
    sessionId: string,
    input: BiofieldCreateCaptureInput,
    consent: Consent,
  ): Promise<BiofieldCapture> {
    if (!sessionId.trim()) {
      throw new EngineSdkError(
        'biofield.createCapture requires a non-empty session id.',
        0,
        'SESSION_ID_REQUIRED',
      )
    }

    const b64 = input.image_data.b64
    if (!b64) {
      throw new EngineSdkError(
        'biofield.createCapture requires image_data.b64 (inline capture frame).',
        0,
        'MEDIA_REQUIRED',
      )
    }

    const form = new FormData()
    const mime = input.image_data.mime_type ?? 'image/png'
    const ext = mime.split('/')[1] ?? 'png'
    const blob = new Blob([b64ToArrayBuffer(b64)], { type: mime })

    // The authenticated API contract requires this field name exactly. Keep it first so
    // multipart consumers can start streaming the payload without buffering metadata.
    form.append('image', blob, input.image_data.file_name ?? `capture.${ext}`)
    if (input.algorithms && input.algorithms.length > 0) {
      form.append('algorithms', JSON.stringify(input.algorithms))
    }
    if (input.options) form.append('options', JSON.stringify(input.options))
    form.append(
      'capture_metadata',
      JSON.stringify({ ...input.capture_metadata, consent }),
    )

    return this.request<BiofieldCapture>(
      'POST',
      `${this.requireApiUrl('biofield capture upload')}/api/v1/biofield/sessions/${encodeURIComponent(sessionId)}/captures`,
      { body: form },
    )
  }

  private requireApiUrl(surface: string): string {
    if (this.apiUrl) return this.apiUrl
    throw new EngineSdkError(
      `${surface} requires EngineClient({ apiUrl }).`,
      0,
      'API_URL_REQUIRED',
    )
  }

  /** @internal shared fetch with JSON/error handling. */
  async request<T>(method: string, url: string, init: RequestInit = {}): Promise<T> {
    const headers: Record<string, string> = { ...this.defaultHeaders }
    const isForm = typeof FormData !== 'undefined' && init.body instanceof FormData
    if (init.body && !isForm) headers['Content-Type'] = 'application/json'

    let response: Response
    try {
      response = await this.fetchImpl(url, { ...init, method, headers })
    } catch (err) {
      if (err instanceof EngineSdkError) throw err
      throw new EngineSdkError(
        `Network error calling ${url}: ${err instanceof Error ? err.message : String(err)}`,
        -1,
        'NETWORK_ERROR',
      )
    }

    const text = await response.text()
    const payload = parsePayload(text)

    if (!response.ok) {
      const { message, code } = resolveErrorMessage(payload, response.status)
      throw new EngineSdkError(message, response.status, code, payload)
    }
    return payload as T
  }
}

// ---------------------------------------------------------------------------
// Per-engine APIs (typed, consent-gated per FROZEN + local-first)
// ---------------------------------------------------------------------------

function buildEngineInput(base: {
  consciousness_level?: EngineInput['consciousness_level']
  parameters?: Record<string, unknown>
  seed?: number
  question?: string
  quality?: EngineInput['quality']
  image_data?: EngineInput['image_data']
  audio_ref?: EngineInput['image_data']
  consent?: EngineInput['consent']
}): EngineInput {
  return {
    consciousness_level: base.consciousness_level ?? 2,
    parameters: base.parameters ?? {},
    ...(base.seed !== undefined ? { seed: base.seed } : {}),
    ...(base.question !== undefined ? { question: base.question } : {}),
    ...(base.quality !== undefined ? { quality: base.quality } : {}),
    ...(base.image_data !== undefined ? { image_data: base.image_data } : {}),
    ...(base.audio_ref !== undefined ? { audio_ref: base.audio_ref } : {}),
    ...(base.consent !== undefined ? { consent: base.consent } : {}),
  }
}

export class BiofieldEngineApi {
  constructor(private readonly client: EngineClient) {}

  /**
   * Birth/engine calculate (Rust engine-biofield; routes via api when P4 configured).
   * Consent scope biofield-capture required when image_data is attached.
   */
  async calculate(input: BiofieldCalculateInput = {}): Promise<EngineOutput<Record<string, unknown>>> {
    if (input.image_data) {
      requireConsent(
        resolveConsent(input.consent, input.image_data.consent),
        CONSENT_SCOPES.BIOFIELD_CAPTURE,
        'biofield',
      )
    }
    return this.client.engineCalculate('biofield', buildEngineInput(input))
  }

  /**
   * Live capture analysis via python sidecar /analyze (11 metrics + quality; contract biofield-cv/v1).
   * ALWAYS consent-gated (scope biofield-capture) — it transmits a capture frame.
   */
  async analyze(input: BiofieldAnalyzeInput): Promise<BiofieldAnalyzeResponse> {
    requireConsent(
      resolveConsent(input.consent, input.image_data?.consent),
      CONSENT_SCOPES.BIOFIELD_CAPTURE,
      'biofield-capture',
    )
    return this.client.biofieldAnalyze(input)
  }

  /** Create an authenticated biofield lifecycle session through noesis-api. */
  async createSession(input: BiofieldCreateSessionInput = {}): Promise<BiofieldSession> {
    return this.client.biofieldCreateSession(input)
  }

  /**
   * Upload an image to an active biofield session using multipart field `image`.
   * Consent is verified locally before URL validation, FormData construction, or fetch.
   */
  async createCapture(
    sessionId: string,
    input: BiofieldCreateCaptureInput,
  ): Promise<BiofieldCapture> {
    const consent = resolveConsent(input.consent, input.image_data?.consent)
    requireConsent(consent, CONSENT_SCOPES.BIOFIELD_CAPTURE, 'biofield-capture')
    return this.client.biofieldCreateCapture(sessionId, input, consent)
  }
}

export class FaceReadingEngineApi {
  constructor(private readonly client: EngineClient) {}

  /**
   * Face-reading calculate (image_data + consent per FROZEN T-004/T-027; heuristic + landmark hook).
   * Consent scope face-image required when image_data is attached.
   */
  async calculate(input: FaceReadingCalculateInput = {}): Promise<EngineOutput<FaceReadingResult>> {
    if (input.image_data) {
      requireConsent(
        resolveConsent(input.consent, input.image_data.consent),
        CONSENT_SCOPES.FACE_IMAGE,
        'face-reading',
      )
    }
    return this.client.engineCalculate<FaceReadingResult>('face-reading', buildEngineInput(input))
  }
}

export class RaagaEngineApi {
  constructor(private readonly client: EngineClient) {}

  /**
   * Raaga calculate (melakarta → swaras + strudel_ratios; generated_audio per FROZEN T-005/T-031).
   * Consent scope raaga-audio required when audio_ref is attached. Melakarta-only calls are consent-free.
   */
  async calculate(input: RaagaCalculateInput = {}): Promise<EngineOutput<RaagaResult>> {
    if (input.audio_ref) {
      requireConsent(
        resolveConsent(input.consent, input.audio_ref.consent),
        CONSENT_SCOPES.RAAGA_AUDIO,
        'raaga',
      )
    }
    return this.client.engineCalculate<RaagaResult>('raaga', buildEngineInput(input))
  }
}

export class SigilForgeEngineApi {
  constructor(private readonly client: EngineClient) {}

  /**
   * Sigil-forge calculate (intention → method + optional generated_image via provider T-003/T-060/T-061).
   * Consent scope sigil-gen required when parameters.generate_image is true. Guidance-only calls are consent-free.
   */
  async calculate(input: SigilForgeCalculateInput = {}): Promise<EngineOutput<SigilForgeResult>> {
    if (input.parameters?.generate_image === true) {
      requireConsent(resolveConsent(input.consent), CONSENT_SCOPES.SIGIL_GEN, 'sigil-forge')
    }
    return this.client.engineCalculate<SigilForgeResult>('sigil-forge', buildEngineInput(input))
  }
}
