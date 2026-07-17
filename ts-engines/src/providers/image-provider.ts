/**
 * Image Generation Provider Abstraction (P1 W1 T-003 + P2 T-035)
 *
 * Config-only switch. SigilForgeEngine accepts injected provider or config.
 * Supports generate + edit paths.
 * Output shape uses GeneratedImage (b64_json | url + metadata)
 *
 * Current impl: nvidia (via low-level utils)
 * T-060: nano-banana (real via runcomfy when RUNCOMFY_TOKEN; graceful mock else) — implemented in src/providers/nano-banana.ts
 * Kimi stub remains.
 *
 * Cites (MANDATORY per task): p1-w1-worker-bootstrap-packet.md, resources-and-assets.md,
 * gaps-and-improvements.md (provider abstraction gap), goal-understanding.md (T-003,T-060),
 * EXECUTION-STATUS.md, P1W2-HANDOFF.md, .worktrees/T-002-copilot/docs/plans/engine-integration/P1W1-CONTRACTS-FROZEN.md,
 * detailed-task-list.md (T-035,T-060), .worktrees/T-024-codex/scripts/ext-contract-harness.ts,
 * ts-engines/src/engines/sigil-forge/* , providers contract in FROZEN.
 * Tags: phase:integration-p1 wave:integration-w2 area:engine-integration engine-sigil
 * External rail unavailable; Codex subagent. No push/merge.
 */

import {
  generateImage as nvidiaGenerate,
  editImage as nvidiaEdit,
  isImageGenAvailable as nvidiaAvailable,
  NVIDIA_IMAGE_MODELS,
  type NvidiaImageModel,
} from '../utils/nvidia-image'

// T-060: real nano-banana impl (not stub). Separate module per task spec + FROZEN style.
import {
  NanoBananaImageProvider,
  isNanoBananaAvailable,
  createNanoBananaProvider,
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

/** NanoBananaImageProvider — re-exported from T-060 impl (src/providers/nano-banana.ts). Real runcomfy when token, mock graceful else. */
export { NanoBananaImageProvider } from './nano-banana'

/** Kimi stub (for yantra/runic; T-061) */
export class KimiImageProvider implements ImageProvider {
  readonly name = 'kimi'
  constructor(private config: ImageProviderConfig) {}

  isAvailable(): boolean {
    return !!this.config.apiKey || !!this.config.endpoint
  }

  async generate(opts: ImageGenOptions): Promise<GeneratedImage> {
    return {
      b64_json: 'kimi-mock-b64',
      metadata: {
        model: opts.model ?? 'kimi-yantra',
        prompt: opts.prompt,
        provider: this.name,
        style: opts.style,
      },
    }
  }

  async edit(opts: ImageEditOptions): Promise<GeneratedImage> {
    return this.generate(opts as any)
  }
}

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
