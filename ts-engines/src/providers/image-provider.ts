/**
 * Image Generation Provider Abstraction (P1 W1 T-003 + P2 T-035 + P3 T-061)
 *
 * Config-only switch. SigilForgeEngine accepts injected provider or config.
 * Supports generate + edit paths.
 * Output shape uses GeneratedImage (b64_json | url + metadata)
 *
 * Current impl: nvidia (via low-level utils)
 * nano-banana (stub), kimi (T-061 full adapter + yantra prompts)
 *
 * Cites (MANDATORY): p1-w1-worker-bootstrap-packet.md, resources-and-assets.md,
 * gaps-and-improvements.md, goal-understanding.md, EXECUTION-STATUS.md, P1W2-HANDOFF.md,
 * FROZEN .worktrees/T-002-copilot/.../P1W1-CONTRACTS-FROZEN.md,
 * detailed-task-list.md (T-061 + T-035), image-provider.ts, kimi.ts, sigil engine.
 * Tags: phase:integration-p1 wave:integration-w2 engine-sigil
 * Worktree edits only. Deliverable: kimi provider integrated + tests.
 */

import {
  generateImage as nvidiaGenerate,
  editImage as nvidiaEdit,
  isImageGenAvailable as nvidiaAvailable,
  NVIDIA_IMAGE_MODELS,
  type NvidiaImageModel,
} from '../utils/nvidia-image'
import { KimiImageProvider } from './kimi'

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
      b64_json: 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==', // tiny png
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
      b64_json: 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==',
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

/** Nano banana stub (config driven, for T-060 later; returns mock for now) */
export class NanoBananaImageProvider implements ImageProvider {
  readonly name = 'nano-banana'
  constructor(private config: ImageProviderConfig) {}

  isAvailable(): boolean {
    return true // assume configured via endpoint
  }

  async generate(opts: ImageGenOptions): Promise<GeneratedImage> {
    // stub: in real would call runcomfy / google nano
    return {
      b64_json: 'nano-banana-mock-b64',
      metadata: {
        model: opts.model ?? this.config.defaultModel ?? 'nano-banana-2',
        prompt: opts.prompt,
        provider: this.name,
        style: opts.style,
        seed: opts.seed,
      },
    }
  }

  async edit(opts: ImageEditOptions): Promise<GeneratedImage> {
    return {
      b64_json: 'nano-banana-edit-mock-b64',
      metadata: {
        model: opts.model ?? this.config.defaultModel ?? 'nano-banana-2',
        prompt: opts.prompt,
        provider: this.name,
        style: opts.style,
      },
    }
  }
}

// KimiImageProvider now implemented in ./kimi.ts (T-061) — imported above for factory + re-export for back-compat
export { KimiImageProvider } from './kimi'
export { YANTRA_PROMPT_TEMPLATES, buildYantraPrompt } from './kimi'

/** Config-only factory. Defaults to nvidia. Supports override for tests. */
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

/** Default export for back-compat in registry etc */
export function createDefaultImageProvider(): ImageProvider {
  return createImageProvider({ provider: 'nvidia' })
}
