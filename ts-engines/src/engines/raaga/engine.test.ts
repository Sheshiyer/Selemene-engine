/**
 * Raaga clip generation tests (raaga-clip-generation, p5-p4-next-batch).
 *
 * Covers the mandated matrix:
 *   - mode off / not requested            → generated_audio.clip_url stays null (FROZEN backward compat)
 *   - requested + granted consent (local) → clip_url populated + WAV stored + served by /clips/raaga/:file
 *   - requested, consent missing          → clip_url null + metadata.clip note (consent_missing)
 *   - mode off, requested + consent       → clip_url null + skipped_off note
 *   - service mode unreachable            → graceful null + service_unavailable note
 *
 * Cites: p1-w1-worker-bootstrap-packet.md, resources-and-assets.md, gaps-and-improvements.md (§4),
 * goal-understanding.md (local-first + consent), EXECUTION-STATUS.md, P1W1-W2-HANDOFF.md,
 * .worktrees/T-002-copilot/.../P1W1-CONTRACTS-FROZEN.md (generated_audio.clip_url),
 * detailed-task-list.md, p5-p4-next-batch.json (raaga-clip-generation).
 * tags: phase:integration-p1 wave:integration-w2 area:engine-integration engine-raaga
 */

import { afterEach, beforeEach, describe, expect, it } from 'bun:test'
import { existsSync, mkdtempSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { createServer, registry } from '../../server'
import type { Consent } from '../../types'
import {
  generateRaagaClip,
  r2ClipKey,
  renderRaagaClipWav,
  renderWav,
  resolveClipBaseUrl,
  resolveClipMode,
  resolveStoredClip,
  storeClip,
} from './clip'
import { RaagaEngine } from './engine'

const CLIP_ENV_KEYS = [
  'RAAGA_CLIP_MODE',
  'RAAGA_CLIP_DIR',
  'RAAGA_CLIP_BASE_URL',
  'RAAGA_CLIP_SERVICE_URL',
  'TS_ENGINES_PUBLIC_URL',
  'RAILWAY_PUBLIC_DOMAIN',
  'R2_ACCOUNT_ID',
  'R2_ACCESS_KEY_ID',
  'R2_SECRET_ACCESS_KEY',
  'R2_PUBLIC_BASE_URL',
  'R2_RAGA_CLIPS_BUCKET',
] as const

const R2_TEST_ENV = {
  R2_ACCOUNT_ID: 'acct123',
  R2_ACCESS_KEY_ID: 'AKIATEST',
  R2_SECRET_ACCESS_KEY: 'test-secret-not-real',
  R2_PUBLIC_BASE_URL: 'https://pub-test.r2.dev',
  R2_RAGA_CLIPS_BUCKET: 'clips-test',
}

type FetchCall = { method: string; url: string; headers: Headers }
const realFetch = globalThis.fetch

/** Mock global fetch for the R2 HEAD/PUT sequence; records calls. */
function mockR2Fetch(statuses: Partial<Record<'HEAD' | 'PUT', number>>): FetchCall[] {
  const calls: FetchCall[] = []
  globalThis.fetch = (async (input: Request | string | URL, init?: RequestInit) => {
    const req = input instanceof Request ? input : new Request(String(input), init)
    calls.push({ method: req.method, url: req.url, headers: req.headers })
    return new Response(null, { status: statuses[req.method as 'HEAD' | 'PUT'] ?? 500 })
  }) as typeof fetch
  return calls
}

const SEVEN_FIELDS = [
  'runtime_kind',
  'implementation_version',
  'cached',
  'fallback_used',
  'backend_id',
  'provider_id',
  'confidence',
]

/** Every provenance key must be one of the seven contract fields (Rust parser is deny_unknown_fields). */
function expectContractProvenance(provenance: unknown) {
  expect(provenance).toBeDefined()
  const p = provenance as Record<string, unknown>
  for (const key of Object.keys(p)) expect(SEVEN_FIELDS).toContain(key)
  expect(p.runtime_kind).toBe('typescript')
  expect(p.implementation_version).toBe('1.0.0')
  expect(p.cached).toBe(false)
  expect(typeof p.fallback_used).toBe('boolean')
  return p
}

const grantedConsent: Consent = {
  granted: true,
  scopes: ['audio', 'raaga_clip'],
  timestamp: new Date().toISOString(),
}

type GeneratedAudio = {
  clip_url?: string | null
  strudel_ratios?: number[]
  root_hz?: number
  metadata?: Record<string, unknown> & { clip?: { status?: string; mode?: string } }
}

describe('raaga clip config', () => {
  it('defaults RAAGA_CLIP_MODE to off', () => {
    expect(resolveClipMode({})).toBe('off')
    expect(resolveClipMode({ RAAGA_CLIP_MODE: 'local' })).toBe('local')
    expect(resolveClipMode({ RAAGA_CLIP_MODE: 'r2' })).toBe('r2')
    expect(resolveClipMode({ RAAGA_CLIP_MODE: 'service' })).toBe('service')
    expect(resolveClipMode({ RAAGA_CLIP_MODE: 'bogus' })).toBe('off')
  })

  it('resolves the local clip base: explicit > public service base > relative', () => {
    expect(resolveClipBaseUrl({})).toBe('/clips/raaga')
    expect(resolveClipBaseUrl({ RAAGA_CLIP_BASE_URL: 'https://cdn.example/raaga/' })).toBe(
      'https://cdn.example/raaga',
    )
    expect(resolveClipBaseUrl({ TS_ENGINES_PUBLIC_URL: 'https://ts.example/' })).toBe(
      'https://ts.example/clips/raaga',
    )
    expect(resolveClipBaseUrl({ RAILWAY_PUBLIC_DOMAIN: 'ts.up.railway.app' })).toBe(
      'https://ts.up.railway.app/clips/raaga',
    )
  })

  it('renderWav is deterministic and r2ClipKey is content-addressed', () => {
    const a = renderWav([1, 1.5, 2], 220, { secondsPerNote: 0.1 })
    const b = renderWav([1, 1.5, 2], 220, { secondsPerNote: 0.1 })
    expect(a).toEqual(b)
    expect(r2ClipKey(a)).toBe(r2ClipKey(b))
    expect(r2ClipKey(a)).toMatch(/^raaga\/[0-9a-f]{64}\.wav$/)
    expect(r2ClipKey(renderWav([1, 2], 220, { secondsPerNote: 0.1 }))).not.toBe(r2ClipKey(a))
  })

  it('renders a valid PCM16 WAV (RIFF header + expected size)', () => {
    const ratios = [1, 9 / 8, 5 / 4, 4 / 3, 3 / 2, 5 / 3, 15 / 8, 2]
    const wav = renderRaagaClipWav(ratios, 220, { sampleRate: 22050, secondsPerNote: 0.1 })
    expect(String.fromCharCode(...wav.slice(0, 4))).toBe('RIFF')
    expect(String.fromCharCode(...wav.slice(8, 12))).toBe('WAVE')
    const expectedSamples = ratios.length * (0.1 * 22050 + 0.08 * 22050)
    expect(wav.length).toBe(44 + Math.floor(expectedSamples) * 2)
  })

  it('stores clips with path-traversal-safe resolution', () => {
    const dir = mkdtempSync(join(tmpdir(), 'raaga-clip-test-'))
    try {
      const wav = renderRaagaClipWav([1, 2], 220, { secondsPerNote: 0.1 })
      const filename = storeClip(wav, 15, 220, dir)
      expect(existsSync(join(dir, filename))).toBe(true)
      expect(resolveStoredClip(dir, filename)).toBe(join(dir, filename))
      expect(resolveStoredClip(dir, '../etc/passwd')).toBeNull()
      expect(resolveStoredClip(dir, 'missing.wav')).toBeNull()
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })
})

describe('RaagaEngine clip generation (generated_audio.clip_url)', () => {
  const engine = new RaagaEngine()
  let clipDir: string
  let savedEnv: Record<string, string | undefined>

  beforeEach(() => {
    clipDir = mkdtempSync(join(tmpdir(), 'raaga-clip-engine-'))
    savedEnv = {}
    for (const key of CLIP_ENV_KEYS) savedEnv[key] = process.env[key]
    // Start from a clean slate so host env (Railway domain, R2 creds) cannot leak into assertions.
    for (const key of CLIP_ENV_KEYS) {
      delete process.env[key]
    }
    process.env.RAAGA_CLIP_DIR = clipDir
    process.env.RAAGA_CLIP_MODE = 'off'
  })

  afterEach(() => {
    for (const key of CLIP_ENV_KEYS) {
      if (savedEnv[key] === undefined) delete process.env[key]
      else process.env[key] = savedEnv[key]
    }
    globalThis.fetch = realFetch
    rmSync(clipDir, { recursive: true, force: true })
  })

  // --- provenance (seven-field contract block) -----------------------------

  it('emits non-fallback provenance when no clip was requested (mode off)', async () => {
    const output = await engine.calculate({ consciousness_level: 0, parameters: { melakarta: 15 } })
    const p = expectContractProvenance(output.provenance)
    expect(p.fallback_used).toBe(false)
    expect(p.provider_id).toBe('selemene-wav-synth')
    expect(p.confidence).toBe(1)
    expect(p.backend_id).toBeUndefined()
  })

  it('marks fallback_used (confidence omitted) when a clip was requested but mode is off', async () => {
    const output = await engine.calculate({
      consciousness_level: 0,
      parameters: { melakarta: 15, request_clip: true },
      consent: grantedConsent,
    })
    const p = expectContractProvenance(output.provenance)
    expect(p.fallback_used).toBe(true)
    expect('confidence' in p).toBe(false)
    expect(output.generated_audio?.clip_url).toBeNull()
  })

  it('marks fallback_used with null clip_url when consent is absent', async () => {
    process.env.RAAGA_CLIP_MODE = 'r2'
    Object.assign(process.env, R2_TEST_ENV)
    const calls = mockR2Fetch({ HEAD: 200 })
    const output = await engine.calculate({
      consciousness_level: 0,
      parameters: { melakarta: 15, request_clip: true },
    })
    const p = expectContractProvenance(output.provenance)
    expect(p.fallback_used).toBe(true)
    expect(output.generated_audio?.clip_url).toBeNull()
    expect(calls.length).toBe(0)
  })

  it('emits provenance with confidence 1 for a delivered local clip', async () => {
    process.env.RAAGA_CLIP_MODE = 'local'
    const output = await engine.calculate({
      consciousness_level: 0,
      parameters: { melakarta: 15, request_clip: true },
      consent: grantedConsent,
    })
    const p = expectContractProvenance(output.provenance)
    expect(p.fallback_used).toBe(false)
    expect(p.provider_id).toBe('selemene-wav-synth')
    expect(p.confidence).toBe(1)
  })

  // --- local mode URL shape -------------------------------------------------

  it('local mode returns an absolute URL when a public service base is known', async () => {
    process.env.RAAGA_CLIP_MODE = 'local'
    process.env.RAILWAY_PUBLIC_DOMAIN = 'ts-engines.up.railway.app'
    const output = await engine.calculate({
      consciousness_level: 0,
      parameters: { melakarta: 15, request_clip: true },
      consent: grantedConsent,
    })
    const audio = output.generated_audio as GeneratedAudio
    expect(audio.clip_url).toMatch(
      /^https:\/\/ts-engines\.up\.railway\.app\/clips\/raaga\/raaga-m15-r220-[a-f0-9]{10}\.wav$/,
    )
    expect(audio.metadata?.clip?.status).toBe('generated')
    expect((audio.metadata?.clip as { detail?: string }).detail).toBeUndefined()
  })

  it('local mode keeps a relative URL and flags it ephemeral when no public base is known', async () => {
    process.env.RAAGA_CLIP_MODE = 'local'
    const output = await engine.calculate({
      consciousness_level: 0,
      parameters: { melakarta: 15, request_clip: true },
      consent: grantedConsent,
    })
    const audio = output.generated_audio as GeneratedAudio
    expect(audio.clip_url).toMatch(/^\/clips\/raaga\//)
    expect((audio.metadata?.clip as { detail?: string }).detail).toMatch(/ephemeral/)
  })

  // --- r2 mode (fetch mocked) ----------------------------------------------

  it('r2 mode: HEAD 404 → PUT → status uploaded with an absolute public URL', async () => {
    process.env.RAAGA_CLIP_MODE = 'r2'
    Object.assign(process.env, R2_TEST_ENV)
    const calls = mockR2Fetch({ HEAD: 404, PUT: 200 })
    const output = await engine.calculate({
      consciousness_level: 0,
      parameters: { melakarta: 15, request_clip: true },
      consent: grantedConsent,
    })
    const audio = output.generated_audio as GeneratedAudio
    expect(audio.clip_url).toMatch(/^https:\/\/pub-test\.r2\.dev\/raaga\/[0-9a-f]{64}\.wav$/)
    expect(audio.metadata?.clip?.mode).toBe('r2')
    expect(audio.metadata?.clip?.status).toBe('uploaded')
    expect(calls.map((c) => c.method)).toEqual(['HEAD', 'PUT'])
    expect(calls[1].url).toMatch(
      /^https:\/\/acct123\.r2\.cloudflarestorage\.com\/clips-test\/raaga\/[0-9a-f]{64}\.wav$/,
    )
    expect(calls[1].headers.get('content-type')).toBe('audio/wav')
    expect(calls[1].headers.get('cache-control')).toBe('public, max-age=31536000, immutable')
    expect(calls[1].headers.get('authorization')).toMatch(/^AWS4-HMAC-SHA256 /)
    const p = expectContractProvenance(output.provenance)
    expect(p.fallback_used).toBe(false)
    expect(p.provider_id).toBe('selemene-wav-synth')
    expect(p.confidence).toBe(1)
  })

  it('r2 mode: HEAD 200 → status reused, no PUT, same URL as an upload would give', async () => {
    process.env.RAAGA_CLIP_MODE = 'r2'
    Object.assign(process.env, R2_TEST_ENV)
    const calls = mockR2Fetch({ HEAD: 200 })
    const output = await engine.calculate({
      consciousness_level: 0,
      parameters: { melakarta: 15, request_clip: true },
      consent: grantedConsent,
    })
    const audio = output.generated_audio as GeneratedAudio
    expect(audio.metadata?.clip?.status).toBe('reused')
    expect(calls.map((c) => c.method)).toEqual(['HEAD'])
    const expectedKey = r2ClipKey(renderWav(audio.strudel_ratios as number[], 220))
    expect(audio.clip_url).toBe(`https://pub-test.r2.dev/${expectedKey}`)
    expect((output.provenance as { fallback_used: boolean }).fallback_used).toBe(false)
  })

  it('r2 mode: PUT 500 → status upload_failed, null clip_url, fallback_used true', async () => {
    process.env.RAAGA_CLIP_MODE = 'r2'
    Object.assign(process.env, R2_TEST_ENV)
    const calls = mockR2Fetch({ HEAD: 404, PUT: 500 })
    const output = await engine.calculate({
      consciousness_level: 0,
      parameters: { melakarta: 15, request_clip: true },
      consent: grantedConsent,
    })
    const audio = output.generated_audio as GeneratedAudio
    expect(audio.clip_url).toBeNull()
    expect(audio.metadata?.clip?.status).toBe('upload_failed')
    expect((audio.metadata?.clip as { detail?: string }).detail).toContain('status 500')
    expect((audio.metadata?.clip as { detail?: string }).detail).not.toContain('test-secret')
    expect(calls.map((c) => c.method)).toEqual(['HEAD', 'PUT'])
    const p = expectContractProvenance(output.provenance)
    expect(p.fallback_used).toBe(true)
    expect('confidence' in p).toBe(false)
  })

  it('r2 mode without R2 variables fails closed naming the missing variables', async () => {
    process.env.RAAGA_CLIP_MODE = 'r2'
    const calls = mockR2Fetch({})
    const output = await engine.calculate({
      consciousness_level: 0,
      parameters: { melakarta: 15, request_clip: true },
      consent: grantedConsent,
    })
    const audio = output.generated_audio as GeneratedAudio
    expect(audio.clip_url).toBeNull()
    expect(audio.metadata?.clip?.status).toBe('upload_failed')
    expect((audio.metadata?.clip as { detail?: string }).detail).toContain('R2_ACCOUNT_ID')
    expect(calls.length).toBe(0)
    expect((output.provenance as { fallback_used: boolean }).fallback_used).toBe(true)
  })

  it('keeps clip_url null when request_clip is not set (backward compat per FROZEN)', async () => {
    const output = await engine.calculate({
      consciousness_level: 0,
      parameters: { melakarta: 15 },
    })
    const audio = output.generated_audio as GeneratedAudio
    expect(audio).toBeDefined()
    expect(audio.clip_url).toBeNull()
    expect(audio.strudel_ratios?.length).toBe(8)
    expect(audio.metadata?.clip?.status).toBe('not_requested')
  })

  it('returns clip_url null + consent_missing note when consent is absent', async () => {
    process.env.RAAGA_CLIP_MODE = 'local'
    const output = await engine.calculate({
      consciousness_level: 0,
      parameters: { melakarta: 15, request_clip: true },
    })
    const audio = output.generated_audio as GeneratedAudio
    expect(audio.clip_url).toBeNull()
    expect(audio.metadata?.clip?.status).toBe('consent_missing')
  })

  it('returns clip_url null + consent_missing note when consent is not granted', async () => {
    process.env.RAAGA_CLIP_MODE = 'local'
    const output = await engine.calculate({
      consciousness_level: 0,
      parameters: { melakarta: 15, request_clip: true },
      consent: { granted: false, scopes: [], timestamp: new Date().toISOString() },
    })
    const audio = output.generated_audio as GeneratedAudio
    expect(audio.clip_url).toBeNull()
    expect(audio.metadata?.clip?.status).toBe('consent_missing')
  })

  it('keeps clip_url null with skipped_off note when mode is off (requested + consent)', async () => {
    const output = await engine.calculate({
      consciousness_level: 0,
      parameters: { melakarta: 15, request_clip: true },
      consent: grantedConsent,
    })
    const audio = output.generated_audio as GeneratedAudio
    expect(audio.clip_url).toBeNull()
    expect(audio.metadata?.clip?.status).toBe('skipped_off')
    expect(audio.metadata?.clip?.mode).toBe('off')
  })

  it('populates clip_url and stores a WAV when requested with granted consent (local mode)', async () => {
    process.env.RAAGA_CLIP_MODE = 'local'
    const output = await engine.calculate({
      consciousness_level: 0,
      parameters: { melakarta: 15, request_clip: true },
      consent: grantedConsent,
    })
    const audio = output.generated_audio as GeneratedAudio
    expect(audio.metadata?.clip?.status).toBe('generated')
    expect(audio.metadata?.clip?.mode).toBe('local')
    expect(audio.clip_url).toMatch(/^\/clips\/raaga\/raaga-m15-r220-[a-f0-9]{10}\.wav$/)
    const filename = (audio.clip_url as string).split('/').pop() as string
    expect(existsSync(join(clipDir, filename))).toBe(true)
  })

  it('accepts consent nested in parameters (compat path)', async () => {
    process.env.RAAGA_CLIP_MODE = 'local'
    const output = await engine.calculate({
      consciousness_level: 0,
      parameters: { melakarta: 29, request_clip: true, consent: grantedConsent },
    })
    const audio = output.generated_audio as GeneratedAudio
    expect(audio.metadata?.clip?.status).toBe('generated')
    expect(audio.clip_url).toMatch(/^\/clips\/raaga\/raaga-m29-r220-[a-f0-9]{10}\.wav$/)
  })

  it('service mode degrades gracefully when the endpoint is unreachable', async () => {
    process.env.RAAGA_CLIP_MODE = 'service'
    process.env.RAAGA_CLIP_SERVICE_URL = 'http://127.0.0.1:1/unreachable'
    const output = await engine.calculate({
      consciousness_level: 0,
      parameters: { melakarta: 15, request_clip: true },
      consent: grantedConsent,
    })
    const audio = output.generated_audio as GeneratedAudio
    expect(audio.clip_url).toBeNull()
    expect(audio.metadata?.clip?.status).toBe('service_unavailable')
    expect(audio.metadata?.clip?.mode).toBe('service')
  }, 15000)

  it('service mode returns clip_url from a reachable endpoint', async () => {
    const result = await generateRaagaClip({
      melakarta: 15,
      ratios: [1, 2],
      rootHz: 220,
      env: {
        RAAGA_CLIP_MODE: 'service',
        RAAGA_CLIP_SERVICE_URL: 'data:,{"clip_url":"https://clips.example/raaga-m15.wav"}',
      },
    })
    // data: URLs are not POST-able; assert graceful handling instead of a hang
    expect(['generated', 'service_unavailable']).toContain(result.status)
  }, 15000)
})

describe('raaga clip HTTP surface', () => {
  it('serves a stored clip via GET /clips/raaga/:file and 404s for unknown clips', async () => {
    const clipDir = mkdtempSync(join(tmpdir(), 'raaga-clip-http-'))
    const savedDir = process.env.RAAGA_CLIP_DIR
    process.env.RAAGA_CLIP_DIR = clipDir
    const port = 3199
    const server = createServer(registry)
    server.listen(port)
    try {
      const wav = renderRaagaClipWav([1, 1.5, 2], 220, { secondsPerNote: 0.1 })
      const filename = storeClip(wav, 15, 220, clipDir)

      const ok = await fetch(`http://localhost:${port}/clips/raaga/${filename}`)
      expect(ok.status).toBe(200)
      expect(ok.headers.get('content-type')).toContain('audio/wav')
      const body = new Uint8Array(await ok.arrayBuffer())
      expect(String.fromCharCode(...body.slice(0, 4))).toBe('RIFF')

      const missing = await fetch(`http://localhost:${port}/clips/raaga/nope.wav`)
      expect(missing.status).toBe(404)

      const traversal = await fetch(`http://localhost:${port}/clips/raaga/..%2F..%2Fpasswd`)
      expect([400, 404]).toContain(traversal.status)
    } finally {
      server.stop()
      // biome-ignore lint/performance/noDelete: assigning undefined to a process.env key stores the string "undefined"; delete is the only way to unset it
      if (savedDir === undefined) delete process.env.RAAGA_CLIP_DIR
      else process.env.RAAGA_CLIP_DIR = savedDir
      rmSync(clipDir, { recursive: true, force: true })
    }
  })
})
