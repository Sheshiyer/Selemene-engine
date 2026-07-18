/**
 * @selemene/engine-sdk tests (bun) — mocked fetch, no network.
 *
 * Covers all 4 focus engines (biofield analyze/calculate, face-reading, raaga, sigil-forge),
 * the local-first consent guard (throws BEFORE network; fetch never invoked), error paths
 * (404/422/500/network), and configurable base URLs (ts @3001, python @8002, api for P4).
 *
 * Payload samples mirror scripts/ext-contract-harness.ts:75-115 (FROZEN).
 *
 * Cites: p1-w1-worker-bootstrap-packet.md, goal-understanding.md (local-first + consent),
 * gaps-and-improvements.md (media contracts missing → this SDK), P1W2-HANDOFF.md,
 * P1W1-CONTRACTS-FROZEN.md, detailed-task-list.md Phase 4, engine-media-contracts.ts.
 *
 * Tags: phase:integration-p1 wave:integration-w2 area:engine-integration
 */

import { describe, expect, it } from 'bun:test'
import {
  CONSENT_SCOPES,
  ConsentError,
  EngineClient,
  EngineSdkError,
  consentAgeMs,
  createConsent,
  requireConsent,
} from '../src/index'
import type {
  BiofieldAnalyzeResponse,
  EngineOutput,
  RaagaResult,
  SigilForgeResult,
} from '../src/index'

// tiny valid 1x1 png b64 (FROZEN sample, same as ext-contract-harness.ts:29)
const TINY_PNG_B64 =
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=='

interface RecordedCall {
  url: string
  init?: RequestInit
}

function mockFetch(handler: (url: string, init?: RequestInit) => Response | Promise<Response>) {
  const calls: RecordedCall[] = []
  const fetchImpl = (async (url: unknown, init?: RequestInit) => {
    calls.push({ url: String(url), init })
    return handler(String(url), init)
  }) as unknown as typeof fetch
  return { fetchImpl, calls }
}

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  })
}

const NOW = new Date().toISOString()
const consentBio = createConsent([CONSENT_SCOPES.BIOFIELD_CAPTURE])
const consentFace = createConsent([CONSENT_SCOPES.FACE_IMAGE])
const consentRaaga = createConsent([CONSENT_SCOPES.RAAGA_AUDIO])
const consentSigil = createConsent([CONSENT_SCOPES.SIGIL_GEN])

// ---------------------------------------------------------------------------
// raaga
// ---------------------------------------------------------------------------

describe('raaga.calculate', () => {
  const raagaOutput: EngineOutput<RaagaResult> = {
    engine_id: 'raaga',
    result: {
      melakarta: { num: 1, name: 'Kanakangi', chakra: 'muladhara', ma_type: 'shuddha' },
      swaras: [{ swara: 'Sa', ratio: 1 }],
      strudel_ratios: [1, 1.0535, 1.125],
      root_hz: 256,
      arohana_indices: [0, 1, 2],
      avarohana_indices: [2, 1, 0],
      prahar: { num: 3, label: 'afternoon', is_recommended_time: true },
      dosha_affinities: { vata: true },
      total_melakartas: 72,
    },
    witness_prompts: [{ prompt: 'What does this sound open?', context: 'raaga' }],
    calculated_at: NOW,
    processing_time_ms: 3,
    generated_audio: { clip_url: null, strudel_ratios: [1, 1.0535, 1.125], root_hz: 256 },
  }

  it('posts melakarta input to ts server without consent (no media)', async () => {
    const { fetchImpl, calls } = mockFetch(() => jsonResponse(raagaOutput))
    const client = new EngineClient({ fetchImpl })

    const out = await client.raaga.calculate({ parameters: { melakarta: 1, dosha: 'vata' } })

    expect(calls).toHaveLength(1)
    expect(calls[0].url).toBe('http://localhost:3001/engines/raaga/calculate')
    expect(calls[0].init?.method).toBe('POST')
    const body = JSON.parse(String(calls[0].init?.body))
    expect(body.parameters.melakarta).toBe(1)
    expect(body.consciousness_level).toBe(2)
    expect(out.engine_id).toBe('raaga')
    expect(out.result.melakarta.name).toBe('Kanakangi')
    expect(out.generated_audio?.strudel_ratios?.length).toBe(3)
    expect(out.generated_audio?.clip_url).toBeNull()
  })

  it('throws ConsentError before network when audio_ref present without consent', async () => {
    const { fetchImpl, calls } = mockFetch(() => jsonResponse(raagaOutput))
    const client = new EngineClient({ fetchImpl })

    await expect(
      client.raaga.calculate({
        parameters: { melakarta: 1 },
        audio_ref: { reference: 'file:local.m4a' },
      }),
    ).rejects.toBeInstanceOf(ConsentError)
    expect(calls).toHaveLength(0)
  })

  it('accepts consent carried on the audio_ref (FROZEN harness pattern)', async () => {
    const { fetchImpl, calls } = mockFetch(() => jsonResponse(raagaOutput))
    const client = new EngineClient({ fetchImpl })

    const out = await client.raaga.calculate({
      parameters: { melakarta: 1 },
      audio_ref: { reference: 'file:local.m4a', consent: consentRaaga },
    })

    expect(calls).toHaveLength(1)
    const body = JSON.parse(String(calls[0].init?.body))
    expect(body.audio_ref.reference).toBe('file:local.m4a')
    expect(body.audio_ref.consent.scopes).toContain('raaga-audio')
    expect(out.engine_id).toBe('raaga')
  })

  it('rejects consent granted for the wrong scope', async () => {
    const { fetchImpl, calls } = mockFetch(() => jsonResponse(raagaOutput))
    const client = new EngineClient({ fetchImpl })

    await expect(
      client.raaga.calculate({
        audio_ref: { reference: 'file:local.m4a' },
        consent: consentSigil,
      }),
    ).rejects.toBeInstanceOf(ConsentError)
    expect(calls).toHaveLength(0)
  })
})

// ---------------------------------------------------------------------------
// sigil-forge
// ---------------------------------------------------------------------------

describe('sigilForge.calculate', () => {
  const sigilOutput: EngineOutput<SigilForgeResult> = {
    engine_id: 'sigil-forge',
    result: {
      intention: 'I witness my patterns clearly',
      method: { id: 'word-elimination', name: 'Word Elimination', description: 'd', steps: ['s1'] },
      processing: { type: 'word_elimination', original: 'x', remaining_letters: 'x', letter_count: 1 },
      charging_suggestions: [{ name: 'moonlight', description: 'd' }],
    },
    witness_prompts: [],
    calculated_at: NOW,
    processing_time_ms: 12,
    generated_image: {
      b64_json: TINY_PNG_B64,
      metadata: { provider: 'nvidia', model: 'sdxl', style: 'runic' },
    },
  }

  it('guidance-only call (generate_image absent) needs no consent', async () => {
    const { fetchImpl, calls } = mockFetch(() => jsonResponse(sigilOutput))
    const client = new EngineClient({ fetchImpl })

    const out = await client.sigilForge.calculate({
      parameters: { intention: 'I witness my patterns clearly', method: 'word-elimination' },
    })

    expect(calls).toHaveLength(1)
    expect(calls[0].url).toBe('http://localhost:3001/engines/sigil-forge/calculate')
    expect(out.result.method.id).toBe('word-elimination')
  })

  it('throws ConsentError before network when generate_image requested without consent', async () => {
    const { fetchImpl, calls } = mockFetch(() => jsonResponse(sigilOutput))
    const client = new EngineClient({ fetchImpl })

    await expect(
      client.sigilForge.calculate({
        parameters: { intention: 'x', generate_image: true },
      }),
    ).rejects.toBeInstanceOf(ConsentError)
    expect(calls).toHaveLength(0)
  })

  it('generates image with sigil-gen consent and returns generated_image (no vector_path)', async () => {
    const { fetchImpl, calls } = mockFetch(() => jsonResponse(sigilOutput))
    const client = new EngineClient({ fetchImpl })

    const out = await client.sigilForge.calculate({
      parameters: { intention: 'x', generate_image: true, provider: 'nvidia' },
      consent: consentSigil,
    })

    expect(calls).toHaveLength(1)
    const body = JSON.parse(String(calls[0].init?.body))
    expect(body.consent.scopes).toContain('sigil-gen')
    expect(out.generated_image?.b64_json).toBe(TINY_PNG_B64)
    expect(out.generated_image?.metadata?.provider).toBe('nvidia')
    expect('vector_path' in (out.result as Record<string, unknown>)).toBe(false)
  })
})

// ---------------------------------------------------------------------------
// face-reading
// ---------------------------------------------------------------------------

describe('faceReading.calculate', () => {
  const faceOutput: EngineOutput = {
    engine_id: 'face-reading',
    result: {
      constitutional_type: 'vata-pitta',
      confidence: 0.62,
      key_observation: 'symmetric brow',
      zones: [{ name: 'brow', score: 0.7, observation: 'balanced', elemental: 'air' }],
      is_mock_data: true,
      backend: 'heuristic-image-landmark-hook',
    },
    witness_prompts: [],
    calculated_at: NOW,
    processing_time_ms: 5,
  }

  it('posts image_data + consent to face-reading calculate (FROZEN)', async () => {
    const { fetchImpl, calls } = mockFetch(() => jsonResponse(faceOutput))
    const client = new EngineClient({ fetchImpl })

    const out = await client.faceReading.calculate({
      image_data: { b64: TINY_PNG_B64, mime_type: 'image/jpeg', consent: consentFace },
    })

    expect(calls).toHaveLength(1)
    expect(calls[0].url).toBe('http://localhost:3001/engines/face-reading/calculate')
    const body = JSON.parse(String(calls[0].init?.body))
    expect(body.image_data.b64).toBe(TINY_PNG_B64)
    expect(body.image_data.consent.scopes).toContain('face-image')
    expect(out.result.constitutional_type).toBe('vata-pitta')
  })

  it('throws ConsentError before network when image_data lacks consent', async () => {
    const { fetchImpl, calls } = mockFetch(() => jsonResponse(faceOutput))
    const client = new EngineClient({ fetchImpl })

    await expect(
      client.faceReading.calculate({
        image_data: { b64: TINY_PNG_B64, mime_type: 'image/jpeg' },
      }),
    ).rejects.toBeInstanceOf(ConsentError)
    expect(calls).toHaveLength(0)
  })
})

// ---------------------------------------------------------------------------
// biofield
// ---------------------------------------------------------------------------

describe('biofield.analyze (python sidecar)', () => {
  const analyzeResponse: BiofieldAnalyzeResponse = {
    contract_version: 'biofield-cv/v1',
    analysis_version: 'real-cv/v1',
    metrics: {
      light_quanta_density: 1.2,
      normalized_area: 0.4,
      average_intensity: 0.5,
      inner_noise: 0.1,
      energy_analysis: { low: 1, medium: 2, high: 3, total: 6 },
      entropy_form_coefficient: 4.2,
      fractal_dimension: 1.5,
      correlation_dimension: 1.1,
      body_symmetry: 0.9,
      contour_complexity: 0.3,
      pattern_regularity: 0.8,
    },
    quality_assessment: {
      sharpness: 0.82,
      contrast: 0.7,
      noise_level: 0.1,
      exposure: 0.5,
      sufficient_quality: true,
    },
    algorithms_run: ['light_quanta_density'],
    processing_time_ms: 42,
  }

  it('posts multipart form to python /analyze with consent (biofield-capture scope)', async () => {
    const { fetchImpl, calls } = mockFetch(() => jsonResponse(analyzeResponse))
    const client = new EngineClient({ fetchImpl })

    const out = await client.biofield.analyze({
      image_data: { b64: TINY_PNG_B64, mime_type: 'image/png', consent: consentBio },
      algorithms: ['light_quanta_density'],
      capture_metadata: { session: 's1' },
    })

    expect(calls).toHaveLength(1)
    expect(calls[0].url).toBe('http://localhost:8002/analyze')
    expect(calls[0].init?.method).toBe('POST')
    const form = calls[0].init?.body as FormData
    expect(form).toBeInstanceOf(FormData)
    const image = form.get('image')
    expect(image).toBeTruthy()
    expect((image as File).type).toBe('image/png')
    expect(form.get('algorithms')).toBe('["light_quanta_density"]')
    expect(form.get('capture_metadata')).toBe('{"session":"s1"}')
    expect(out.contract_version).toBe('biofield-cv/v1')
    expect(out.metrics.energy_analysis.total).toBe(6)
    expect(out.quality_assessment.sufficient_quality).toBe(true)
  })

  it('always throws ConsentError without consent (transmits a capture frame)', async () => {
    const { fetchImpl, calls } = mockFetch(() => jsonResponse(analyzeResponse))
    const client = new EngineClient({ fetchImpl })

    await expect(
      client.biofield.analyze({ image_data: { b64: TINY_PNG_B64, mime_type: 'image/png' } }),
    ).rejects.toBeInstanceOf(ConsentError)
    expect(calls).toHaveLength(0)
  })

  it('throws MEDIA_REQUIRED when image_data.b64 is missing', async () => {
    const { fetchImpl, calls } = mockFetch(() => jsonResponse(analyzeResponse))
    const client = new EngineClient({ fetchImpl })

    await expect(
      client.biofield.analyze({
        image_data: { reference: 'upload:123', consent: consentBio },
        consent: consentBio,
      }),
    ).rejects.toMatchObject({ code: 'MEDIA_REQUIRED', status: 0 })
    expect(calls).toHaveLength(0)
  })
})

describe('biofield.calculate', () => {
  it('routes to ts server with consent when image_data attached', async () => {
    const { fetchImpl, calls } = mockFetch(() =>
      jsonResponse({
        engine_id: 'biofield',
        result: { dominant_element: 'air', is_mock_data: true },
        witness_prompts: [],
        calculated_at: NOW,
        processing_time_ms: 2,
      }),
    )
    const client = new EngineClient({ fetchImpl })

    await client.biofield.calculate({
      image_data: { b64: TINY_PNG_B64, mime_type: 'image/png' },
      consent: consentBio,
    })

    expect(calls).toHaveLength(1)
    expect(calls[0].url).toBe('http://localhost:3001/engines/biofield/calculate')
  })

  it('throws ConsentError when image_data attached without consent', async () => {
    const { fetchImpl, calls } = mockFetch(() => jsonResponse({}))
    const client = new EngineClient({ fetchImpl })

    await expect(
      client.biofield.calculate({ image_data: { b64: TINY_PNG_B64 } }),
    ).rejects.toBeInstanceOf(ConsentError)
    expect(calls).toHaveLength(0)
  })
})

// ---------------------------------------------------------------------------
// configurable base URLs + P4 api routing
// ---------------------------------------------------------------------------

describe('configurable base URLs', () => {
  it('honors custom tsEnginesUrl + pythonBiofieldUrl', async () => {
    const { fetchImpl, calls } = mockFetch(() =>
      jsonResponse({ status: 'healthy', engines: [], uptime_ms: 1, version: 'x' }),
    )
    const client = new EngineClient({
      tsEnginesUrl: 'http://ts.local:3999/',
      pythonBiofieldUrl: 'http://py.local:8999/',
      fetchImpl,
    })

    await client.health()

    expect(calls.map((c) => c.url).sort()).toEqual([
      'http://py.local:8999/health',
      'http://ts.local:3999/health',
    ])
  })

  it('routes calculate via apiUrl when P4 noesis-api is configured', async () => {
    const { fetchImpl, calls } = mockFetch(() =>
      jsonResponse({
        engine_id: 'raaga',
        result: {},
        witness_prompts: [],
        calculated_at: NOW,
        processing_time_ms: 1,
      }),
    )
    const client = new EngineClient({ apiUrl: 'https://api.noesis.example', fetchImpl })

    await client.raaga.calculate({ parameters: { melakarta: 1 } })

    expect(calls[0].url).toBe('https://api.noesis.example/api/v1/engines/raaga/calculate')
  })

  it('sends defaultHeaders on every request', async () => {
    const { fetchImpl, calls } = mockFetch(() =>
      jsonResponse({
        engine_id: 'raaga',
        result: {},
        witness_prompts: [],
        calculated_at: NOW,
        processing_time_ms: 1,
      }),
    )
    const client = new EngineClient({
      defaultHeaders: { Authorization: 'Bearer test-token' },
      fetchImpl,
    })

    await client.raaga.calculate({ parameters: { melakarta: 1 } })

    const headers = calls[0].init?.headers as Record<string, string>
    expect(headers.Authorization).toBe('Bearer test-token')
    expect(headers['Content-Type']).toBe('application/json')
  })
})

// ---------------------------------------------------------------------------
// error paths
// ---------------------------------------------------------------------------

describe('error handling', () => {
  it('maps 404 ENGINE_NOT_FOUND to EngineSdkError with status + code', async () => {
    const { fetchImpl } = mockFetch(() =>
      jsonResponse({ error: 'Engine not found: nope', error_code: 'ENGINE_NOT_FOUND' }, 404),
    )
    const client = new EngineClient({ fetchImpl })

    await expect(client.raaga.calculate({ parameters: {} })).rejects.toMatchObject({
      name: 'EngineSdkError',
      status: 404,
      code: 'ENGINE_NOT_FOUND',
      message: 'Engine not found: nope',
    })
  })

  it('maps 422 validation errors with details', async () => {
    const { fetchImpl } = mockFetch(() =>
      jsonResponse({ error: 'intention required', error_code: 'VALIDATION_ERROR', details: { field: 'intention' } }, 422),
    )
    const client = new EngineClient({ fetchImpl })

    await expect(client.sigilForge.calculate({ parameters: {} })).rejects.toMatchObject({
      status: 422,
      code: 'VALIDATION_ERROR',
    })
  })

  it('maps 500 calculation errors', async () => {
    const { fetchImpl } = mockFetch(() =>
      jsonResponse({ error: 'boom', error_code: 'CALCULATION_ERROR' }, 500),
    )
    const client = new EngineClient({ fetchImpl })

    await expect(client.raaga.calculate({ parameters: { melakarta: 1 } })).rejects.toMatchObject({
      status: 500,
      code: 'CALCULATION_ERROR',
      message: 'boom',
    })
  })

  it('handles non-JSON error bodies', async () => {
    const { fetchImpl } = mockFetch(() => new Response('upstream exploded', { status: 502 }))
    const client = new EngineClient({ fetchImpl })

    await expect(client.raaga.calculate({ parameters: { melakarta: 1 } })).rejects.toMatchObject({
      status: 502,
      message: 'Request failed: 502',
    })
  })

  it('wraps fetch rejections as NETWORK_ERROR (status -1)', async () => {
    const { fetchImpl } = mockFetch(() => {
      throw new Error('connection refused')
    })
    const client = new EngineClient({ fetchImpl })

    await expect(client.raaga.calculate({ parameters: { melakarta: 1 } })).rejects.toMatchObject({
      status: -1,
      code: 'NETWORK_ERROR',
    })
  })

  it('health() surfaces service failure', async () => {
    const { fetchImpl } = mockFetch((url) => {
      if (url.includes('8002')) throw new Error('sidecar down')
      return jsonResponse({ status: 'healthy', engines: ['raaga'], uptime_ms: 1, version: '1' })
    })
    const client = new EngineClient({ fetchImpl })

    await expect(client.health()).rejects.toMatchObject({ code: 'NETWORK_ERROR' })
  })
})

// ---------------------------------------------------------------------------
// consent primitives
// ---------------------------------------------------------------------------

describe('consent primitives', () => {
  it('requireConsent passes with granted + scope', () => {
    expect(() => requireConsent(consentFace, CONSENT_SCOPES.FACE_IMAGE, 'face-reading')).not.toThrow()
  })

  it('requireConsent throws ConsentError with engine + scope context', () => {
    try {
      requireConsent(createConsent(['other']), CONSENT_SCOPES.FACE_IMAGE, 'face-reading')
      expect.unreachable('should have thrown')
    } catch (err) {
      expect(err).toBeInstanceOf(ConsentError)
      const ce = err as ConsentError
      expect(ce.engineId).toBe('face-reading')
      expect(ce.requiredScope).toBe('face-image')
      expect(ce.status).toBe(0)
      expect(ce.code).toBe('CONSENT_REQUIRED')
    }
  })

  it('requireConsent rejects granted=false', () => {
    const revoked = { ...consentBio, granted: false }
    expect(() => requireConsent(revoked, CONSENT_SCOPES.BIOFIELD_CAPTURE, 'biofield-capture')).toThrow(
      ConsentError,
    )
  })

  it('createConsent produces granted timestamped token', () => {
    const c = createConsent(['x'])
    expect(c.granted).toBe(true)
    expect(c.scopes).toEqual(['x'])
    expect(c.token).toContain('consent-')
    expect(consentAgeMs(c)).toBeGreaterThanOrEqual(0)
  })
})
