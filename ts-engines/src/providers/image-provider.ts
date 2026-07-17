/**
 * Image Generation Provider Abstraction (P1 W1 T-003 contract)
 *
 * Current: NVIDIA NIM (flux dev/schnell via utils/nvidia-image.ts)
 * Targets: nano-banana (Google via runcomfy or direct), kimi (yantra/runic styles)
 *
 * Contract: switch providers via config only. No engine changes for new providers.
 * SigilForgeEngine updated to accept/inject provider (back-compat default NVIDIA).
 */

export interface ImageGenOptions {
  prompt: string
  model?: string
  width?: number
  height?: number
  seed?: number
  style?: string
}

export interface ImageEditOptions extends ImageGenOptions {
  image: string // b64 source
  instruction?: string
}

export interface GeneratedImage {
  b64_json?: string
  url?: string
  metadata: {
    model: string
    prompt: string
    provider: string
    style?: string
    seed?: number
    finish_reason?: string
  }
}

export interface ImageProvider {
  readonly name: string
  generate(opts: ImageGenOptions): Promise<GeneratedImage>
  edit?(opts: ImageEditOptions): Promise<GeneratedImage>
  isAvailable(): boolean
  // future: listModels(), health()
}

/** Config for selecting provider (env or passed) */
export interface ImageProviderConfig {
  provider: 'nvidia' | 'nano-banana' | 'kimi'
  apiKey?: string
  endpoint?: string // for runcomfy / custom
  defaultModel?: string
}
