/**
 * Image Generation Provider Abstraction (P1 W1 T-003 + P2 T-035 + P3 T-061)
 *
 * Config-only switch. SigilForgeEngine accepts injected provider or config.
 * Supports generate + edit paths.
 * Output shape uses GeneratedImage (b64_json | url + metadata)
 *
 * T-060: nano-banana (real via runcomfy when RUNCOMFY_TOKEN; graceful mock else) — src/providers/nano-banana.ts
 * T-061: kimi (full adapter + yantra prompts) — src/providers/kimi.ts
 *
 * Cites (MANDATORY): p1-w1-worker-bootstrap-packet.md, resources-and-assets.md,
 * gaps-and-improvements.md (provider abstraction gap), goal-understanding.md (T-003,T-060,T-061),
 * EXECUTION-STATUS.md, P1W2-HANDOFF.md, .worktrees/T-002-copilot/docs/plans/engine-integration/P1W1-CONTRACTS-FROZEN.md,
 * detailed-task-list.md (T-035,T-060,T-061), ts-engines/src/engines/sigil-forge/*, kimi.ts, nano-banana.ts.
 * Tags: phase:integration-p1 wave:integration-w2 area:engine-integration engine-sigil
 */

import {
  NVIDIA_IMAGE_MODELS,
  type NvidiaImageModel,
  isImageGenAvailable as nvidiaAvailable,
  editImage as nvidiaEdit,
  generateImage as nvidiaGenerate,
} from '../utils/nvidia-image'
import { KimiImageProvider } from './kimi'

// T-060: real nano-banana impl (not stub). Separate module per task spec + FROZEN style.
import {
  NanoBananaImageProvider,
  createNanoBananaProvider,
  isNanoBananaAvailable,
} from './nano-banana'

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
}

export interface ImageProviderConfig {
  provider: 'nvidia' | 'nano-banana' | 'kimi'
  apiKey?: string
  endpoint?: string
  defaultModel?: string
}

/** Mock provider for tests (no network, deterministic) */
export class MockImageProvider implements ImageProvider {
  readonly name = 'mock'
  private callCount = 0

  isAvailable(): boolean {
    return true
  }

  async generate(opts: ImageGenOptions): Promise<GeneratedImage> {
    this.callCount++
    return {
      b64_json:
        'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==', // tiny png
      metadata: {
        model: opts.model ?? 'mock-model',
        prompt: opts.prompt,
        provider: this.name,
        style: opts.style,
        seed: opts.seed,
        finish_reason: 'SUCCESS_MOCK',
      },
    }
  }

  async edit(opts: ImageEditOptions): Promise<GeneratedImage> {
    this.callCount++
    return {
      b64_json:
        'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==',
      metadata: {
        model: opts.model ?? 'mock-edit',
        prompt: opts.prompt,
        provider: this.name,
        style: opts.style,
        seed: opts.seed,
        finish_reason: 'SUCCESS_MOCK_EDIT',
      },
    }
  }
}

/** NVIDIA provider wrapping the low-level utils (config only) */
export class NvidiaImageProvider implements ImageProvider {
  readonly name = 'nvidia'
  private config: ImageProviderConfig

  constructor(config: ImageProviderConfig = { provider: 'nvidia' }) {
    this.config = config
  }

  isAvailable(): boolean {
    return nvidiaAvailable()
  }

  async generate(opts: ImageGenOptions): Promise<GeneratedImage> {
    const res = await nvidiaGenerate({
      prompt: opts.prompt,
      model: (opts.model as NvidiaImageModel) ?? NVIDIA_IMAGE_MODELS.FLUX_DEV,
      width: opts.width,
      height: opts.height,
      seed: opts.seed,
    })
    return {
      b64_json: res.b64_json,
      metadata: {
        model: opts.model ?? NVIDIA_IMAGE_MODELS.FLUX_DEV,
        prompt: opts.prompt,
        provider: this.name,
        style: opts.style,
        seed: opts.seed,
        finish_reason: res.finish_reason,
      },
    }
  }

  async edit(opts: ImageEditOptions): Promise<GeneratedImage> {
    const res = await nvidiaEdit({
      image: opts.image,
      prompt: opts.prompt,
      model: (opts.model as NvidiaImageModel) ?? NVIDIA_IMAGE_MODELS.FLUX_DEV,
      width: opts.width,
      height: opts.height,
      seed: opts.seed,
    })
    return {
      b64_json: res.b64_json,
      metadata: {
        model: opts.model ?? NVIDIA_IMAGE_MODELS.FLUX_DEV,
        prompt: opts.prompt,
        provider: this.name,
        style: opts.style,
        seed: opts.seed,
      },
    }
  }
}

/** NanoBananaImageProvider — re-exported from T-060 impl (src/providers/nano-banana.ts). Real runcomfy when token, mock graceful else. */
export { NanoBananaImageProvider } from './nano-banana'

// KimiImageProvider now implemented in ./kimi.ts (T-061) — imported above for factory + re-export for back-compat
export { KimiImageProvider } from './kimi'
export { YANTRA_PROMPT_TEMPLATES, buildYantraPrompt } from './kimi'

/** Config-only factory. Defaults to nvidia. Supports override for tests. T-060: nano integrated. */
export function createImageProvider(config: Partial<ImageProviderConfig> = {}): ImageProvider {
  const full: ImageProviderConfig = {
    provider: 'nvidia',
    ...config,
  }
  switch (full.provider) {
    case 'nvidia':
      return new NvidiaImageProvider(full)
    case 'nano-banana':
      return new NanoBananaImageProvider(full)
    case 'kimi':
      return new KimiImageProvider(full)
    default:
      return new NvidiaImageProvider(full)
  }
}

/** Default export for back-compat in registry etc. T-060: integrate nano — use nano-banana if RUNCOMFY_TOKEN present else nvidia (config-only). */
export function createDefaultImageProvider(): ImageProvider {
  if (isNanoBananaAvailable()) {
    return createImageProvider({ provider: 'nano-banana' })
  }
  return createImageProvider({ provider: 'nvidia' })
}
