/**
 * Raaga clip generation — fills generated_audio.clip_url (T-031 left it null).
 *
 * Offline, local-first render of strudel_ratios (just-intonation swara set) into a
 * mono PCM16 WAV clip (arohana ascent, sine voices, short fades). No network, no
 * secrets — matches the goal-understanding.md local-first + explicit-consent model.
 * A config-driven `service` mode exists for a future external render service
 * (e.g. the Suno bridge already proxied in src/server/app.ts), and degrades
 * gracefully to null when unreachable.
 *
 * Config (env):
 *   RAAGA_CLIP_MODE        — "off" | "local" | "service"   (default "off")
 *   RAAGA_CLIP_DIR         — clip store dir                (default <os-tmp>/raaga-clips)
 *   RAAGA_CLIP_BASE_URL    — public URL prefix for clips   (default "/clips/raaga")
 *   RAAGA_CLIP_SERVICE_URL — external render endpoint      (service mode only)
 *   RAAGA_CLIP_SECONDS_PER_NOTE — seconds per swara        (default 0.6)
 *
 * Cites (mandatory): p1-w1-worker-bootstrap-packet.md, resources-and-assets.md,
 * gaps-and-improvements.md (§4 audio clip path missing), goal-understanding.md
 * (local-first + consent; unresolved prod URLs), EXECUTION-STATUS.md,
 * P1W1-W2-HANDOFF.md, .worktrees/T-002-copilot/.../P1W1-CONTRACTS-FROZEN.md
 * (generated_audio: strudel_ratios + clip_url), detailed-task-list.md,
 * docs/plans/engine-integration/p5-p4-next-batch.json (raaga-clip-generation).
 * tags: phase:integration-p1 wave:integration-w2 area:engine-integration engine-raaga
 */

import { createHash } from 'node:crypto'
import { existsSync, mkdirSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

export type RaagaClipMode = 'off' | 'local' | 'service'

export interface RaagaClipRequest {
  melakarta: number
  ratios: readonly number[]
  rootHz: number
  env?: Record<string, string | undefined>
}

export interface RaagaClipResult {
  clip_url: string | null
  mode: RaagaClipMode
  status: 'generated' | 'skipped_off' | 'service_unavailable' | 'error'
  detail?: string
}

type Env = Record<string, string | undefined>

/** Resolve the configured clip mode; defaults to "off" (backward compat per FROZEN). */
export function resolveClipMode(env: Env = process.env): RaagaClipMode {
  const raw = (env.RAAGA_CLIP_MODE ?? 'off').trim().toLowerCase()
  if (raw === 'local' || raw === 'service' || raw === 'off') return raw
  return 'off'
}

export function resolveClipDir(env: Env = process.env): string {
  return env.RAAGA_CLIP_DIR ?? join(tmpdir(), 'raaga-clips')
}

export function resolveClipBaseUrl(env: Env = process.env): string {
  return env.RAAGA_CLIP_BASE_URL ?? '/clips/raaga'
}

// ---------------------------------------------------------------------------
// WAV (PCM16 mono) synthesis — WebAudio-compatible offline render
// ---------------------------------------------------------------------------

function secondsPerNote(env: Env): number {
  const raw = Number(env.RAAGA_CLIP_SECONDS_PER_NOTE)
  return Number.isFinite(raw) && raw > 0.05 && raw <= 10 ? raw : 0.6
}

/**
 * Render just-intonation ratios as an ascending sine-tone sequence.
 * Each swara gets a short raised-cosine fade to avoid clicks; a brief gap
 * separates notes so the arohana ladder is audible.
 */
export function renderRaagaClipWav(
  ratios: readonly number[],
  rootHz: number,
  opts: { sampleRate?: number; secondsPerNote?: number } = {},
): Uint8Array {
  const sampleRate = opts.sampleRate ?? 22050
  const perNote = opts.secondsPerNote ?? 0.6
  const gapSec = 0.08
  const noteSamples = Math.floor(perNote * sampleRate)
  const gapSamples = Math.floor(gapSec * sampleRate)
  const totalSamples = ratios.length * (noteSamples + gapSamples)
  const dataBytes = totalSamples * 2
  const buffer = new ArrayBuffer(44 + dataBytes)
  const view = new DataView(buffer)

  const writeStr = (offset: number, s: string) => {
    for (let i = 0; i < s.length; i++) view.setUint8(offset + i, s.charCodeAt(i))
  }

  // RIFF/WAVE header, PCM16 mono
  writeStr(0, 'RIFF')
  view.setUint32(4, 36 + dataBytes, true)
  writeStr(8, 'WAVE')
  writeStr(12, 'fmt ')
  view.setUint32(16, 16, true) // fmt chunk size
  view.setUint16(20, 1, true) // PCM
  view.setUint16(22, 1, true) // mono
  view.setUint32(24, sampleRate, true)
  view.setUint32(28, sampleRate * 2, true) // byte rate
  view.setUint16(32, 2, true) // block align
  view.setUint16(34, 16, true) // bits per sample
  writeStr(36, 'data')
  view.setUint32(40, dataBytes, true)

  const amplitude = 0.3
  const fadeSamples = Math.min(Math.floor(0.02 * sampleRate), Math.floor(noteSamples / 4))
  let cursor = 44

  for (const ratio of ratios) {
    const freq = rootHz * ratio
    const phaseInc = (2 * Math.PI * freq) / sampleRate
    for (let i = 0; i < noteSamples; i++) {
      let envelope = 1
      if (i < fadeSamples) envelope = 0.5 * (1 - Math.cos((Math.PI * i) / fadeSamples))
      else if (i >= noteSamples - fadeSamples)
        envelope = 0.5 * (1 - Math.cos((Math.PI * (noteSamples - i)) / fadeSamples))
      const sample = amplitude * envelope * Math.sin(phaseInc * i)
      view.setInt16(cursor, Math.max(-1, Math.min(1, sample)) * 32767, true)
      cursor += 2
    }
    cursor += gapSamples * 2 // silence gap already zeroed
  }

  return new Uint8Array(buffer)
}

// ---------------------------------------------------------------------------
// Clip store
// ---------------------------------------------------------------------------

/** Write WAV bytes to the clip store; returns the store-relative filename. */
export function storeClip(wav: Uint8Array, melakarta: number, rootHz: number, dir: string): string {
  mkdirSync(dir, { recursive: true })
  const hash = createHash('sha1').update(wav).digest('hex').slice(0, 10)
  const filename = `raaga-m${melakarta}-r${rootHz}-${hash}.wav`
  writeFileSync(join(dir, filename), wav)
  return filename
}

/** Resolve a clip filename inside the store (path-traversal safe). Returns null if invalid/missing. */
export function resolveStoredClip(dir: string, filename: string): string | null {
  if (!/^[a-zA-Z0-9._-]+\.wav$/.test(filename)) return null
  const path = join(dir, filename)
  return existsSync(path) ? path : null
}

// ---------------------------------------------------------------------------
// Generation entry point
// ---------------------------------------------------------------------------

export async function generateRaagaClip(request: RaagaClipRequest): Promise<RaagaClipResult> {
  const env = request.env ?? process.env
  const mode = resolveClipMode(env)

  if (mode === 'off') {
    return {
      clip_url: null,
      mode,
      status: 'skipped_off',
      detail: 'RAAGA_CLIP_MODE=off (default); set to local or service to enable clip generation',
    }
  }

  if (mode === 'local') {
    try {
      const wav = renderRaagaClipWav(request.ratios, request.rootHz, {
        secondsPerNote: secondsPerNote(env),
      })
      const filename = storeClip(wav, request.melakarta, request.rootHz, resolveClipDir(env))
      return {
        clip_url: `${resolveClipBaseUrl(env)}/${filename}`,
        mode,
        status: 'generated',
      }
    } catch (err) {
      return {
        clip_url: null,
        mode,
        status: 'error',
        detail: err instanceof Error ? err.message : String(err),
      }
    }
  }

  // service mode — external render endpoint (e.g. a suno-bridge style service), graceful fallback
  const serviceUrl = env.RAAGA_CLIP_SERVICE_URL
  if (!serviceUrl) {
    return {
      clip_url: null,
      mode,
      status: 'service_unavailable',
      detail: 'RAAGA_CLIP_SERVICE_URL not configured',
    }
  }

  try {
    const controller = new AbortController()
    const timeout = setTimeout(() => controller.abort(), 10_000)
    const res = await fetch(serviceUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Accept: 'application/json' },
      body: JSON.stringify({
        melakarta: request.melakarta,
        ratios: request.ratios,
        root_hz: request.rootHz,
      }),
      signal: controller.signal,
    })
    clearTimeout(timeout)
    if (!res.ok) {
      return {
        clip_url: null,
        mode,
        status: 'service_unavailable',
        detail: `clip service responded ${res.status}`,
      }
    }
    const body = (await res.json().catch(() => ({}))) as { clip_url?: string; url?: string }
    const url = body.clip_url ?? body.url ?? null
    if (!url) {
      return {
        clip_url: null,
        mode,
        status: 'service_unavailable',
        detail: 'clip service response missing clip_url',
      }
    }
    return { clip_url: url, mode, status: 'generated' }
  } catch (err) {
    return {
      clip_url: null,
      mode,
      status: 'service_unavailable',
      detail: err instanceof Error ? err.message : String(err),
    }
  }
}
