/**
 * NanoBananaImageProvider — Google Nano Banana 2 provider (via RunComfy) for sigil/yantras.
 *
 * Implements ImageProvider iface per T-003 / FROZEN / T-060.
 * - generate: text-to-image via runcomfy google/nano-banana-2/text-to-image → b64_json or url
 * - edit: no native b64 edit path; routes through generate with the instruction prefixed
 * - config driven (RUNCOMFY_TOKEN for real)
 *
 * Truth table (lane S4, 2026-10-26):
 *   - `isAvailable()` is true ONLY when RUNCOMFY_TOKEN (or constructor apiKey) is set.
 *   - No token, CLI failure, or empty output → `{ error, mock: true }` with a
 *     finish_reason. A placeholder PNG is never returned.
 */

import type {
  GeneratedImage,
  ImageEditOptions,
  ImageGenOptions,
  ImageProvider,
  ImageProviderConfig,
} from './image-provider'

const NANO_MODEL = 'google/nano-banana-2'
const NANO_T2I = `${NANO_MODEL}/text-to-image`

export const NANO_FINISH_NO_TOKEN = 'MOCK_NO_TOKEN'
export const NANO_FINISH_ERROR = 'ERROR'

interface NanoResult {
  b64_json?: string
  url?: string
  finish_reason?: string
}

function getRuncomfyToken(): string | undefined {
  return Bun.env.RUNCOMFY_TOKEN
}

/** True only when a RunComfy token is configured (env or explicit). */
export function isNanoBananaAvailable(apiKey?: string): boolean {
  return Boolean(apiKey || getRuncomfyToken())
}

async function execRuncomfy(
  input: Record<string, unknown>,
  endpoint: string,
  token: string,
): Promise<NanoResult> {
  // Use temp dir for output to control
  const tmpDir = `/tmp/nano-banana-${Date.now()}`
  try {
    await Bun.spawn(['mkdir', '-p', tmpDir]).exited
  } catch {}

  const args = [
    'runcomfy',
    '--output',
    'json',
    'run',
    endpoint,
    '--input',
    JSON.stringify(input),
    '--output-dir',
    tmpDir,
  ]

  const proc = Bun.spawn(args, {
    env: { ...Bun.env, RUNCOMFY_TOKEN: token },
    stdout: 'pipe',
    stderr: 'pipe',
  })

  const exitCode = await proc.exited
  const outText = await new Response(proc.stdout).text()
  const errText = await new Response(proc.stderr).text()

  if (exitCode !== 0) {
    throw new Error(
      `runcomfy nano-banana failed (${exitCode}): ${errText.slice(0, 400) || outText.slice(0, 400)}`,
    )
  }

  let json: { images?: unknown }
  try {
    json = JSON.parse(outText.trim().split('\n').pop() || outText)
  } catch {
    json = {}
  }

  const images: string[] = Array.isArray(json?.images) ? (json.images as string[]) : []
  if (images.length === 0) {
    try {
      const files = await Array.fromAsync(new Bun.Glob('**/*.{png,jpg,jpeg,webp}').scan(tmpDir))
      if (files.length > 0) {
        const filePath = `${tmpDir}/${files[0]}`
        const buf = await Bun.file(filePath).arrayBuffer()
        return { b64_json: Buffer.from(buf).toString('base64'), finish_reason: 'SUCCESS' }
      }
    } catch {}
    throw new Error('nano-banana returned no image path')
  }

  const first = images[0]
  if (first.startsWith('http')) {
    return { url: first, finish_reason: 'SUCCESS' }
  }

  try {
    const buf = await Bun.file(first).arrayBuffer()
    return { b64_json: Buffer.from(buf).toString('base64'), finish_reason: 'SUCCESS' }
  } catch {
    return { url: first, finish_reason: 'SUCCESS' }
  }
}

/** Nano banana provider impl — real via runcomfy when token present, honest error otherwise. */
export class NanoBananaImageProvider implements ImageProvider {
  readonly name = 'nano-banana'
  private config: ImageProviderConfig

  constructor(config: ImageProviderConfig = { provider: 'nano-banana' }) {
    this.config = config
  }

  isAvailable(): boolean {
    return isNanoBananaAvailable(this.config.apiKey)
  }

  private failure(
    opts: ImageGenOptions,
    model: string,
    finishReason: string,
    error: string,
  ): GeneratedImage {
    return {
      error,
      mock: true,
      metadata: {
        model,
        prompt: opts.prompt,
        provider: this.name,
        style: opts.style,
        seed: opts.seed,
        finish_reason: finishReason,
      },
    }
  }

  async generate(opts: ImageGenOptions): Promise<GeneratedImage> {
    const model = opts.model ?? this.config.defaultModel ?? 'nano-banana-2'
    const prompt = opts.prompt
    const token = this.config.apiKey || getRuncomfyToken()

    if (!token) {
      return this.failure(
        opts,
        model,
        NANO_FINISH_NO_TOKEN,
        'nano-banana not configured. Set RUNCOMFY_TOKEN to enable image generation.',
      )
    }

    try {
      const input: Record<string, unknown> = {
        prompt,
        resolution: '1K',
        aspect_ratio: '1:1',
        output_format: 'png',
        seed: opts.seed ?? 0,
        num_images: 1,
        limit_generations: true,
      }

      const res = await execRuncomfy(input, NANO_T2I, token)
      return {
        b64_json: res.b64_json,
        url: res.url,
        metadata: {
          model,
          prompt,
          provider: this.name,
          style: opts.style,
          seed: opts.seed,
          finish_reason: res.finish_reason ?? 'SUCCESS',
        },
      }
    } catch (err) {
      return this.failure(
        opts,
        model,
        NANO_FINISH_ERROR,
        err instanceof Error ? err.message.slice(0, 300) : 'nano-banana generation failed',
      )
    }
  }

  async edit(opts: ImageEditOptions): Promise<GeneratedImage> {
    // Edit via nano requires public image_urls; b64 input is not supported directly,
    // so edit routes through generate with the instruction as a prompt prefix.
    const prompt = `Edit instruction: ${opts.instruction || 'refine'}. ${opts.prompt}`
    return this.generate({ ...opts, prompt })
  }
}

/** Factory helper for direct use */
export function createNanoBananaProvider(
  config?: Partial<ImageProviderConfig>,
): NanoBananaImageProvider {
  return new NanoBananaImageProvider({ provider: 'nano-banana', ...config })
}

export default NanoBananaImageProvider
