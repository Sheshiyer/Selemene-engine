/**
 * Kimi Image Provider Adapter (T-061, honesty pass in lane S4)
 *
 * Implements ImageProvider for Kimi (Moonshot AI) yantra / runic / vedic sigil generation.
 * Config-driven. Supports generate + edit.
 *
 * Truth table (lane S4, 2026-10-26):
 *   - `isAvailable()` is true ONLY when a key is configured (constructor `apiKey` or KIMI_API_KEY).
 *   - The real Kimi image endpoint is still unresolved, so `generate`/`edit` NEVER
 *     fabricate a placeholder PNG. They return `{ error, mock: true }` with a
 *     `finish_reason` the engine maps to `fallback_used: true`.
 *   - Use `MockImageProvider` (image-provider.ts) when a test needs an image.
 *
 * Prompt templates specialized for yantras (sacred geometry interlocking triangles, sri-yantra precision).
 */

import type {
  GeneratedImage,
  ImageEditOptions,
  ImageGenOptions,
  ImageProvider,
  ImageProviderConfig,
} from './image-provider'

const KIMI_BASE = 'https://api.moonshot.cn/v1' // placeholder; real yantra image endpoint unresolved

export const KIMI_FINISH_NO_KEY = 'MOCK_NO_KEY'
export const KIMI_FINISH_UNIMPLEMENTED = 'KIMI_IMAGE_ENDPOINT_UNAVAILABLE'

function getKimiKey(): string | undefined {
  return Bun.env.KIMI_API_KEY
}

export function getKimiEndpoint(): string {
  return Bun.env.KIMI_ENDPOINT || KIMI_BASE
}

/** True only when a Kimi key is configured (env or explicit). */
export function isKimiAvailable(apiKey?: string): boolean {
  return Boolean(apiKey || getKimiKey())
}

function kimiFailure(
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
      provider: 'kimi',
      style: opts.style,
      seed: opts.seed,
      finish_reason: finishReason,
    },
  }
}

export class KimiImageProvider implements ImageProvider {
  readonly name = 'kimi'
  private config: ImageProviderConfig

  constructor(config: ImageProviderConfig = { provider: 'kimi' }) {
    this.config = config
  }

  isAvailable(): boolean {
    return isKimiAvailable(this.config.apiKey)
  }

  async generate(opts: ImageGenOptions): Promise<GeneratedImage> {
    const model = opts.model ?? this.config.defaultModel ?? 'kimi-yantra-v1'
    if (!this.isAvailable()) {
      return kimiFailure(
        opts,
        model,
        KIMI_FINISH_NO_KEY,
        'kimi not configured. Set KIMI_API_KEY to enable image generation.',
      )
    }
    // The Kimi image endpoint is not confirmed; refusing honestly beats a fake PNG.
    return kimiFailure(
      opts,
      model,
      KIMI_FINISH_UNIMPLEMENTED,
      `kimi image generation is not implemented (endpoint ${getKimiEndpoint()} unverified).`,
    )
  }

  async edit(opts: ImageEditOptions): Promise<GeneratedImage> {
    return this.generate({
      ...opts,
      prompt: opts.instruction ? `${opts.prompt} [edit: ${opts.instruction}]` : opts.prompt,
    })
  }
}

/** Yantra-specialized prompt templates (for use by sigil prompt-builder or direct kimi calls) */
export const YANTRA_PROMPT_TEMPLATES = {
  sri_yantra: (intention: string) =>
    `Precise traditional Sri Yantra: nine interlocking triangles (four upward Shiva, five downward Shakti) forming 43 triangles total, centered bindu point, exact sacred geometry proportions, concentric circles and lotus petals, minimalistic red and black ink lines on aged parchment, high symmetry, vedic tantric yantra, no text, no labels, isolated symbol, spiritually charged, masterwork precision linework, gold accents, ${intention} encoded in proportions`,

  general_yantra: (intention: string, complexity = 'medium') =>
    `Vedic yantra for ${intention}: interlocking triangles, circles, squares and lotus forms in precise sacred geometry, single unified mandala glyph, traditional indian temple art style, red black gold on cream parchment, fine compass straightedge lines, no figures, no text, no words, high contrast, symmetrical, esoteric spiritual diagram, ${complexity} complexity`,

  runic_yantra: (intention: string) =>
    `Hybrid yantra-rune: sri yantra geometry fused with angular Elder Futhark bind-runes, interlocking triangles containing stave forms, sacred proportions, carved stone + parchment texture, dark charcoal and crimson, unified glyph, no text labels, occult vedic-norse synthesis, intention: ${intention}`,

  // negative for yantra styles
  negative_yantra:
    'text, letters, words, alphabet, human, face, body, 3d, realistic, blurry, low contrast, asymmetric, extra lines, watermark, photograph, illustration of deity, multiple disconnected symbols',
}

export function buildYantraPrompt(
  intention: string,
  variant: keyof typeof YANTRA_PROMPT_TEMPLATES = 'general_yantra',
): { prompt: string; negative_prompt: string; style: string } {
  const promptFn = YANTRA_PROMPT_TEMPLATES[variant] ?? YANTRA_PROMPT_TEMPLATES.general_yantra
  const prompt = typeof promptFn === 'function' ? promptFn(intention) : promptFn
  return {
    prompt,
    negative_prompt: YANTRA_PROMPT_TEMPLATES.negative_yantra,
    style: 'yantra',
  }
}
