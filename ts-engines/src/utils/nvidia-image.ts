/**
 * NVIDIA NIM Image Generation Client
 * OpenAI-compatible API at https://integrate.api.nvidia.com/v1
 *
 * Supports:
 *  - Text-to-image generation (POST /images/generations)
 *  - Image editing/variation (POST /images/edits)
 */

const NIM_BASE_URL = 'https://integrate.api.nvidia.com/v1'

/** Available NIM image models ranked by quality for symbolic/esoteric imagery */
export const NVIDIA_IMAGE_MODELS = {
  FLUX_DEV: 'black-forest-labs/flux-dev',
  FLUX_SCHNELL: 'black-forest-labs/flux-schnell',
  SANA_4B: 'nvidia/sana-1.5-4b-1024px',
  SDXL: 'stabilityai/stable-diffusion-xl-base-1.0',
} as const

export type NvidiaImageModel = (typeof NVIDIA_IMAGE_MODELS)[keyof typeof NVIDIA_IMAGE_MODELS]

export interface GenerateImageOptions {
  prompt: string
  negative_prompt?: string
  model?: NvidiaImageModel
  width?: number
  height?: number
  num_inference_steps?: number
  guidance_scale?: number
  seed?: number
  /** 'b64_json' (default) or 'url' */
  response_format?: 'b64_json' | 'url'
}

export interface EditImageOptions {
  /** Base-64 encoded source image (PNG) */
  image: string
  prompt: string
  negative_prompt?: string
  model?: NvidiaImageModel
  /** Optional base-64 encoded mask PNG (white = edit area) */
  mask?: string
  width?: number
  height?: number
  num_inference_steps?: number
  guidance_scale?: number
  seed?: number
  response_format?: 'b64_json' | 'url'
}

export interface GeneratedImageResult {
  b64_json?: string
  url?: string
  /** The model-revised or passthrough prompt */
  revised_prompt?: string
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
 * Generate a new image from a text prompt using NVIDIA NIM.
 */
export async function generateImage(
  opts: GenerateImageOptions,
): Promise<GeneratedImageResult> {
  const apiKey = getApiKey()
  const model = opts.model ?? NVIDIA_IMAGE_MODELS.FLUX_DEV

  const body: Record<string, unknown> = {
    model,
    prompt: opts.prompt,
    n: 1,
    response_format: opts.response_format ?? 'b64_json',
  }
  if (opts.negative_prompt) body.negative_prompt = opts.negative_prompt
  if (opts.width) body.width = opts.width
  if (opts.height) body.height = opts.height
  if (opts.num_inference_steps) body.num_inference_steps = opts.num_inference_steps
  if (opts.guidance_scale !== undefined) body.guidance_scale = opts.guidance_scale
  if (opts.seed !== undefined) body.seed = opts.seed

  const res = await fetch(`${NIM_BASE_URL}/images/generations`, {
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
    throw new Error(`NVIDIA NIM image generation failed (${res.status}): ${text}`)
  }

  const json = (await res.json()) as { data?: Array<{ b64_json?: string; url?: string; revised_prompt?: string }> }
  const item = json?.data?.[0]
  if (!item) {
    throw new Error('NVIDIA NIM returned empty data array')
  }

  return {
    b64_json: item.b64_json,
    url: item.url,
    revised_prompt: item.revised_prompt,
  }
}

/**
 * Edit an existing image using NVIDIA NIM inpainting/variation.
 */
export async function editImage(opts: EditImageOptions): Promise<GeneratedImageResult> {
  const apiKey = getApiKey()
  const model = opts.model ?? NVIDIA_IMAGE_MODELS.FLUX_DEV

  // NIM uses multipart form data for image edits
  const form = new FormData()
  form.append('model', model)
  form.append('prompt', opts.prompt)
  form.append('n', '1')
  form.append('response_format', opts.response_format ?? 'b64_json')

  // Convert base64 to Blob for FormData
  const imageBytes = Buffer.from(opts.image, 'base64')
  form.append('image', new Blob([imageBytes], { type: 'image/png' }), 'image.png')

  if (opts.mask) {
    const maskBytes = Buffer.from(opts.mask, 'base64')
    form.append('mask', new Blob([maskBytes], { type: 'image/png' }), 'mask.png')
  }
  if (opts.negative_prompt) form.append('negative_prompt', opts.negative_prompt)
  if (opts.width) form.append('width', String(opts.width))
  if (opts.height) form.append('height', String(opts.height))
  if (opts.num_inference_steps) form.append('num_inference_steps', String(opts.num_inference_steps))
  if (opts.guidance_scale !== undefined) form.append('guidance_scale', String(opts.guidance_scale))
  if (opts.seed !== undefined) form.append('seed', String(opts.seed))

  const res = await fetch(`${NIM_BASE_URL}/images/edits`, {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${apiKey}`,
      Accept: 'application/json',
    },
    body: form,
  })

  if (!res.ok) {
    const text = await res.text()
    throw new Error(`NVIDIA NIM image edit failed (${res.status}): ${text}`)
  }

  const json = (await res.json()) as { data?: Array<{ b64_json?: string; url?: string }> }
  const item = json?.data?.[0]
  if (!item) {
    throw new Error('NVIDIA NIM image edit returned empty data array')
  }

  return { b64_json: item.b64_json, url: item.url }
}

/** Check whether image generation is available (key is configured) */
export function isImageGenAvailable(): boolean {
  return Boolean(Bun.env.NVIDIA_API_KEY)
}
