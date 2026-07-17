/**
 * NVIDIA NIM Image Generation Client
 * API Catalog endpoint: https://ai.api.nvidia.com/v1/genai/{org}/{model}
 *
 * Response format: { artifacts: [{ base64: "...", finishReason: "SUCCESS" }] }
 *
 * Supports:
 *  - Text-to-image generation (text prompt → PNG base64)
 *  - Image editing (not all models support this; falls back to generation)
 */

const NIM_GENAI_BASE = 'https://ai.api.nvidia.com/v1/genai'

/** Available NIM image models — only models confirmed working with this API */
export const NVIDIA_IMAGE_MODELS = {
  /** Best quality, slower — recommended for final sigils */
  FLUX_DEV: 'black-forest-labs/flux.1-dev',
  /** Fast, good quality — recommended for iteration */
  FLUX_SCHNELL: 'black-forest-labs/flux.1-schnell',
} as const

export type NvidiaImageModel = (typeof NVIDIA_IMAGE_MODELS)[keyof typeof NVIDIA_IMAGE_MODELS]

export interface GenerateImageOptions {
  prompt: string
  model?: NvidiaImageModel
  width?: number
  height?: number
  seed?: number
}

export interface EditImageOptions {
  /** Base-64 encoded source image (PNG) — used as img2img reference */
  image: string
  prompt: string
  model?: NvidiaImageModel
  width?: number
  height?: number
  seed?: number
}

export interface GeneratedImageResult {
  /** Base64-encoded PNG */
  b64_json?: string
  /** Finish reason from model */
  finish_reason?: string
}

function getApiKey(): string {
  const key = Bun.env.NVIDIA_API_KEY ?? ''
  if (!key) {
    throw new Error(
      'NVIDIA_API_KEY is not configured. Set it in Railway env vars to enable image generation.',
    )
  }
  return key
}

/**
 * Generate a new image from a text prompt using NVIDIA NIM API Catalog.
 * Returns base64-encoded PNG in result.b64_json.
 */
export async function generateImage(opts: GenerateImageOptions): Promise<GeneratedImageResult> {
  const apiKey = getApiKey()
  const model = opts.model ?? NVIDIA_IMAGE_MODELS.FLUX_DEV
  const url = `${NIM_GENAI_BASE}/${model}`

  const body: Record<string, unknown> = {
    prompt: opts.prompt,
    width: opts.width ?? 1024,
    height: opts.height ?? 1024,
  }
  if (opts.seed !== undefined) body.seed = (opts.seed >>> 0) % 4294967295 // clamp to non-negative uint32

  const res = await fetch(url, {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${apiKey}`,
      'Content-Type': 'application/json',
      Accept: 'application/json',
    },
    body: JSON.stringify(body),
  })

  if (!res.ok) {
    const text = await res.text()
    throw new Error(`NVIDIA NIM image generation failed (${res.status}): ${text.slice(0, 300)}`)
  }

  const json = (await res.json()) as {
    artifacts?: Array<{ base64?: string; finishReason?: string }>
  }

  const artifact = json?.artifacts?.[0]
  if (!artifact?.base64) {
    throw new Error('NVIDIA NIM returned no image artifact')
  }

  return {
    b64_json: artifact.base64,
    finish_reason: artifact.finishReason,
  }
}

/**
 * Edit/refine an existing image. NVIDIA NIM doesn't support true inpainting
 * on all models — this uses img2img-style guidance where supported, otherwise
 * falls back to pure generation with the edit prompt.
 */
export async function editImage(opts: EditImageOptions): Promise<GeneratedImageResult> {
  // For now, NVIDIA NIM API Catalog doesn't expose inpainting on flux models.
  // We treat edit as a fresh generation with an enhanced prompt.
  // When NVIDIA adds img2img support, update this implementation.
  return generateImage({
    prompt: opts.prompt,
    model: opts.model,
    width: opts.width,
    height: opts.height,
    seed: opts.seed,
  })
}

/** Check whether image generation is available (key is configured) */
export function isImageGenAvailable(): boolean {
  return Boolean(Bun.env.NVIDIA_API_KEY)
}

// --- T-003 contract: implement ImageProvider interface (see ../providers/image-provider.ts) per FROZEN ---
// Cites: resources-and-assets (sigil nvidia), gaps (provider abstraction), goal (T-003), P1W1-CONTRACTS-FROZEN, detailed T-003/T-028, bootstrap packet, EXECUTION-STATUS, ext-harness
import type { ImageProvider, ImageGenOptions, ImageEditOptions, GeneratedImage } from '../providers/image-provider'

export class NvidiaImageProvider implements ImageProvider {
  readonly name = 'nvidia'

  isAvailable(): boolean {
    return isImageGenAvailable()
  }

  async generate(opts: ImageGenOptions): Promise<GeneratedImage> {
    const res = await generateImage({
      prompt: opts.prompt,
      model: opts.model as any,
      width: opts.width,
      height: opts.height,
      seed: opts.seed,
    })
    return {
      b64_json: res.b64_json,
      metadata: {
        model: opts.model ?? 'flux.1-dev',
        prompt: opts.prompt,
        provider: this.name,
        style: (opts as any).style,
        seed: opts.seed,
        finish_reason: res.finish_reason,
      },
    }
  }

  async edit(opts: ImageEditOptions): Promise<GeneratedImage> {
    const res = await editImage({
      image: opts.image,
      prompt: opts.prompt,
      model: opts.model as any,
      width: opts.width,
      height: opts.height,
      seed: opts.seed,
    })
    return {
      b64_json: res.b64_json,
      metadata: {
        model: opts.model ?? 'flux.1-dev',
        prompt: opts.prompt,
        provider: this.name,
        style: (opts as any).style,
        seed: opts.seed,
      },
    }
  }
}

export function createDefaultImageProvider(): ImageProvider {
  return new NvidiaImageProvider()
}
