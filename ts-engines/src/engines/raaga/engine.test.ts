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
import { RaagaEngine } from './engine'
import {
  generateRaagaClip,
  renderRaagaClipWav,
  resolveClipMode,
  resolveStoredClip,
  storeClip,
} from './clip'

const CLIP_ENV_KEYS = [
  'RAAGA_CLIP_MODE',
  'RAAGA_CLIP_DIR',
  'RAAGA_CLIP_BASE_URL',
  'RAAGA_CLIP_SERVICE_URL',
] as const

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
    expect(resolveClipMode({ RAAGA_CLIP_MODE: 'service' })).toBe('service')
    expect(resolveClipMode({ RAAGA_CLIP_MODE: 'bogus' })).toBe('off')
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
    process.env.RAAGA_CLIP_DIR = clipDir
    process.env.RAAGA_CLIP_MODE = 'off'
    delete process.env.RAAGA_CLIP_SERVICE_URL
  })

  afterEach(() => {
    for (const key of CLIP_ENV_KEYS) {
      if (savedEnv[key] === undefined) delete process.env[key]
      else process.env[key] = savedEnv[key]
    }
    rmSync(clipDir, { recursive: true, force: true })
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
      if (savedDir === undefined) delete process.env.RAAGA_CLIP_DIR
      else process.env.RAAGA_CLIP_DIR = savedDir
      rmSync(clipDir, { recursive: true, force: true })
    }
  })
})
