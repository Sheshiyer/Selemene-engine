/**
 * Raaga clip generation — fills generated_audio.clip_url.
 *
 * Renders strudel_ratios (just-intonation swara set) offline into a mono PCM16
 * WAV clip (arohana ascent, sine voices, short fades) and delivers it through
 * one of three modes:
 *
 *   off      — never render; clip_url stays null (default; backward compat).
 *   local    — write the WAV under RAAGA_CLIP_DIR and serve it from this
 *              process at /clips/raaga/<file>. Railway disks are ephemeral, so
 *              these URLs die on redeploy; the URL is absolute only when a
 *              public base for this service is known (see resolveClipBaseUrl).
 *   r2       — upload the WAV to Cloudflare R2 (key raaga/<sha256>.wav, HEAD
 *              before PUT so repeats are free) and return the absolute public
 *              URL. Durable; the production path.
 *
 * Config (env):
 *   RAAGA_CLIP_MODE             — "off" | "local" | "r2"  (default "off")
 *   RAAGA_CLIP_DIR              — local clip store dir   (default <os-tmp>/raaga-clips)
 *   RAAGA_CLIP_BASE_URL         — URL prefix for local clips (default "/clips/raaga",
 *                                 or https://<TS_ENGINES_PUBLIC_URL|RAILWAY_PUBLIC_DOMAIN>/clips/raaga)
 *   RAAGA_CLIP_SECONDS_PER_NOTE — seconds per swara (default 0.6)
 *   R2_*                        — see src/utils/r2.ts (r2 mode only)
 */

import { createHash } from 'node:crypto'
import { existsSync, mkdirSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { missingR2Vars, putIfAbsent } from '../../utils/r2'

export type RaagaClipMode = 'off' | 'local' | 'r2'

/** Who produced the clip bytes (surfaces as provenance.provider_id). Always the in-process synth. */
export type RaagaClipProvider = 'selemene-wav-synth'

export interface RaagaClipRequest {
  melakarta: number
  ratios: readonly number[]
  rootHz: number
  env?: Record<string, string | undefined>
}

export type RaagaClipStatus =
  /** local: clip rendered and delivered */
  | 'generated'
  /** r2: HEAD 404 → PUT succeeded */
  | 'uploaded'
  /** r2: HEAD 200 → existing object reused, no PUT */
  | 'reused'
  /** r2: HEAD/PUT failed or R2 not configured */
  | 'upload_failed'
  | 'skipped_off'
  | 'error'

export interface RaagaClipResult {
  clip_url: string | null
  mode: RaagaClipMode
  status: RaagaClipStatus
  /** Set whenever clip_url is non-null. */
  provider?: RaagaClipProvider
  detail?: string
}

type Env = Record<string, string | undefined>

export const R2_CLIP_CACHE_CONTROL = 'public, max-age=31536000, immutable'
export const R2_CLIP_CONTENT_TYPE = 'audio/wav'

/** Resolve the configured clip mode; defaults to "off" (backward compat per FROZEN). */
export function resolveClipMode(env: Env = process.env): RaagaClipMode {
  const raw = (env.RAAGA_CLIP_MODE ?? 'off').trim().toLowerCase()
  if (raw === 'local' || raw === 'r2' || raw === 'off') return raw
  return 'off'
}

export function resolveClipDir(env: Env = process.env): string {
  return env.RAAGA_CLIP_DIR ?? join(tmpdir(), 'raaga-clips')
}

/**
 * Public base for this service, when known: TS_ENGINES_PUBLIC_URL (explicit)
 * or Railway's auto-injected RAILWAY_PUBLIC_DOMAIN. null otherwise.
 */
export function resolvePublicServiceBase(env: Env = process.env): string | null {
  const explicit = (env.TS_ENGINES_PUBLIC_URL ?? '').trim()
  if (explicit) return explicit.replace(/\/+$/, '')
  const domain = (env.RAILWAY_PUBLIC_DOMAIN ?? '').trim()
  if (domain) return `https://${domain.replace(/^https?:\/\//, '').replace(/\/+$/, '')}`
  return null
}

/**
 * URL prefix for local-mode clips. RAAGA_CLIP_BASE_URL wins; otherwise an
 * absolute prefix when a public base is known, else the relative
 * "/clips/raaga" (resolved by whoever is talking to this process).
 */
export function resolveClipBaseUrl(env: Env = process.env): string {
  const explicit = (env.RAAGA_CLIP_BASE_URL ?? '').trim()
  if (explicit) return explicit.replace(/\/+$/, '')
  const base = resolvePublicServiceBase(env)
  return base ? `${base}/clips/raaga` : '/clips/raaga'
}

// ---------------------------------------------------------------------------
// WAV (PCM16 mono) synthesis — pure, deterministic
// ---------------------------------------------------------------------------

function secondsPerNote(env: Env): number {
  const raw = Number(env.RAAGA_CLIP_SECONDS_PER_NOTE)
  return Number.isFinite(raw) && raw > 0.05 && raw <= 10 ? raw : 0.6
}

export interface RenderWavOptions {
  sampleRate?: number
  secondsPerNote?: number
}

/**
 * Render just-intonation ratios as an ascending sine-tone sequence.
 * Each swara gets a short raised-cosine fade to avoid clicks; a brief gap
 * separates notes so the arohana ladder is audible. Pure function of its
 * arguments: same ratios/root/options → byte-identical WAV.
 */
export function renderWav(
  ratios: readonly number[],
  rootHz: number,
  opts: RenderWavOptions = {},
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

/** @deprecated alias kept for existing imports; use renderWav. */
export const renderRaagaClipWav = renderWav

/** Content-addressed R2 key for a rendered clip: raaga/<sha256 of bytes>.wav */
export function r2ClipKey(wav: Uint8Array): string {
  return `raaga/${createHash('sha256').update(wav).digest('hex')}.wav`
}

// ---------------------------------------------------------------------------
// Local clip store
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

function errorDetail(err: unknown): string {
  return err instanceof Error ? err.message : String(err)
}

export async function generateRaagaClip(request: RaagaClipRequest): Promise<RaagaClipResult> {
  const env = request.env ?? process.env
  const mode = resolveClipMode(env)

  if (mode === 'off') {
    return {
      clip_url: null,
      mode,
      status: 'skipped_off',
      detail:
        'RAAGA_CLIP_MODE=off (default); set to local or r2 to enable clip generation',
    }
  }

  if (mode === 'local') {
    try {
      const wav = renderWav(request.ratios, request.rootHz, { secondsPerNote: secondsPerNote(env) })
      const filename = storeClip(wav, request.melakarta, request.rootHz, resolveClipDir(env))
      const base = resolveClipBaseUrl(env)
      return {
        clip_url: `${base}/${filename}`,
        mode,
        status: 'generated',
        provider: 'selemene-wav-synth',
        ...(base.startsWith('/')
          ? { detail: 'relative URL: local clip store is ephemeral (lost on redeploy)' }
          : {}),
      }
    } catch (err) {
      return { clip_url: null, mode, status: 'error', detail: errorDetail(err) }
    }
  }

  // r2 mode — content-addressed upload to Cloudflare R2 (the durable, production path)
  const missing = missingR2Vars(env)
  if (missing.length > 0) {
    return {
      clip_url: null,
      mode,
      status: 'upload_failed',
      detail: `R2 not configured: missing ${missing.join(', ')}`,
    }
  }
  try {
    const wav = renderWav(request.ratios, request.rootHz, { secondsPerNote: secondsPerNote(env) })
    const key = r2ClipKey(wav)
    const { url, existed } = await putIfAbsent(key, wav, R2_CLIP_CONTENT_TYPE, {
      cacheControl: R2_CLIP_CACHE_CONTROL,
      env,
    })
    return {
      clip_url: url,
      mode,
      status: existed ? 'reused' : 'uploaded',
      provider: 'selemene-wav-synth',
    }
  } catch (err) {
    return { clip_url: null, mode, status: 'upload_failed', detail: errorDetail(err) }
  }
}
