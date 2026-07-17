/**
 * SigilForgeEngine - Consciousness engine for sigil creation + AI-generated sigil images
 *
 * Refactored per T-035 + FROZEN + T-003:
 * - Uses injected ImageProvider (config-only; default nvidia)
 * - Supports generate/edit paths via provider
 * - Output uses top-level generated_image (FROZEN media contract) + inside result for compat
 * - Prompt builder used for provider prompts; styles preserved
 *
 * Modes:
 *   1. Guidance-only (default)
 *   2. Generate image (parameters.generate_image = true)
 *   3. Edit image (parameters.edit_image_b64 + edit_instruction)
 *
 * Cites (ALL MANDATORY): p1-w1-worker-bootstrap-packet.md + resources-and-assets.md + gaps-and-improvements.md
 * + goal-understanding.md + EXECUTION-STATUS.md + P1W2-HANDOFF.md + .worktrees/T-002-copilot/P1W1-CONTRACTS-FROZEN.md
 * + detailed-task-list.md (T-035) + .worktrees/T-024-codex/scripts/ext-contract-harness.ts + T-028 evidence
 * + ts-engines/src/engines/sigil-forge/engine.ts (this) + prompt-builder + wisdom + providers/image-provider.ts
 * Tags: phase:integration-p1 wave:integration-w2 area:engine-integration engine-sigil
 * External rail unavailable; Codex subagent. No push/merge. Tests: mock + real(nvidia if key).
 */

import type { ConsciousnessEngine, EngineInput, EngineMetadata, EngineOutput } from '../../types'
import { EngineValidationError } from '../../utils'
import { SeededRandom, getDefaultSeed } from '../../utils/random'
import { type SigilStyle, buildSigilEditPrompt, buildSigilPrompt } from './prompt-builder'
import {
  CHARGING_METHODS,
  SIGIL_METHODS,
  getMethodById,
  getMethodIds,
  processWordElimination,
} from './wisdom'
import { generateWitnessPrompts } from './witness'
import type { ImageProvider, ImageProviderConfig, GeneratedImage } from '../../providers/image-provider'
import { createImageProvider, createDefaultImageProvider } from '../../providers/image-provider'

export class SigilForgeEngine implements ConsciousnessEngine {
  private imageProvider: ImageProvider

  constructor(imageProviderOrConfig?: ImageProvider | ImageProviderConfig) {
    if (imageProviderOrConfig && 'generate' in imageProviderOrConfig) {
      this.imageProvider = imageProviderOrConfig as ImageProvider
    } else if (imageProviderOrConfig) {
      this.imageProvider = createImageProvider(imageProviderOrConfig as ImageProviderConfig)
    } else {
      this.imageProvider = createDefaultImageProvider()
    }
  }

  private safeSvgPreview(template?: string): {
    status: 'absent' | 'accepted' | 'rejected'
    reason?: string
  } {
    if (!template) {
      return { status: 'absent' }
    }

    const trimmed = template.trim().toLowerCase()
    if (!trimmed.startsWith('<svg') || !trimmed.includes('</svg>')) {
      return { status: 'rejected', reason: 'Template must be a complete SVG document.' }
    }

    if (trimmed.includes('<script') || trimmed.includes('onload=')) {
      return { status: 'rejected', reason: 'SVG template contains disallowed script content.' }
    }

    return { status: 'accepted' }
  }

  metadata(): EngineMetadata {
    const imageGenNote = this.imageProvider.isAvailable()
      ? `Set parameters.generate_image=true to receive an AI-generated sigil image via ${this.imageProvider.name}.`
      : `Image generation is not configured (${this.imageProvider.name}). Guidance-only mode active.`

    return {
      id: 'sigil-forge',
      name: 'Sigil Forge Consciousness Engine',
      description: `Guided sigil creation with multiple methods and optional AI-generated sigil imagery. ${imageGenNote}`,
      version: '2.0.0',
      required_phase: 1,
      input_schema: {
        intention: {
          type: 'string',
          required: true,
          description:
            'The intention or desire to encode into a sigil. Write as a present-tense statement.',
        },
        method: {
          type: 'string',
          required: false,
          description: 'Sigil creation method. Auto-selected if omitted.',
          enum: getMethodIds(),
        },
        generate_image: {
          type: 'boolean',
          required: false,
          description:
            'When true, generates an AI sigil image via the configured provider (config-only).',
          default: false,
        },
        image_style: {
          type: 'string',
          required: false,
          description: 'Visual style for generated sigil. Default is method-dependent.',
          enum: ['ceremonial', 'chaos', 'organic', 'geometric', 'runic', 'ethereal'],
        },
        image_model: {
          type: 'string',
          required: false,
          description: 'Provider model (e.g. flux for nvidia).',
          // Note: enum relaxed for config-only providers; validated at runtime
        },
        edit_image_b64: {
          type: 'string',
          required: false,
          description: 'Base64-encoded PNG of an existing sigil to refine. Triggers edit mode.',
        },
        edit_instruction: {
          type: 'string',
          required: false,
          description:
            'Edit instruction for refining an existing sigil (used with edit_image_b64).',
        },
      },
    }
  }

  async calculate(input: EngineInput): Promise<EngineOutput> {
    const startTime = performance.now()

    // --- Extract parameters ---
    const intention =
      input.question ??
      (input.parameters.intention as string | undefined) ??
      (input.parameters.intent as string | undefined) ??
      (input.parameters.intent_text as string | undefined) ??
      (input.parameters.question as string | undefined)

    const methodParam = input.parameters.method as string | undefined
    const svgTemplate = input.parameters.svg_template as string | undefined
    const seed = input.seed ?? getDefaultSeed()
    const generateImageFlag = Boolean(input.parameters.generate_image)
    const imageStyle = input.parameters.image_style as SigilStyle | undefined
    const imageModel = input.parameters.image_model as string | undefined
    const editImageB64 = input.parameters.edit_image_b64 as string | undefined
    const editInstruction = (input.parameters.edit_instruction as string | undefined) ?? ''

    // --- Validate ---
    if (!intention || typeof intention !== 'string' || intention.trim() === '') {
      throw new EngineValidationError(
        'Intention parameter is required. Please provide a present-tense statement of your desire.',
        'MISSING_INTENTION',
        { field: 'intention' },
      )
    }

    const cleanIntention = intention.trim()

    // --- Select method ---
    let method = methodParam ? getMethodById(methodParam) : undefined

    if (methodParam && !method) {
      throw new EngineValidationError('Unknown sigil method.', 'INVALID_SIGIL_METHOD', {
        provided: methodParam,
        supported: getMethodIds(),
      })
    }

    if (!method) {
      const rng = new SeededRandom(seed)
      if (cleanIntention.length > 50) {
        method = SIGIL_METHODS[0] // word-elimination
      } else if (cleanIntention.split(' ').length <= 3) {
        method = SIGIL_METHODS[2] // pictographic
      } else {
        method = rng.pick(SIGIL_METHODS)
      }
    }

    // --- Process intention for word elimination ---
    let processedLetters: string | null = null
    if (method.id === 'word-elimination') {
      processedLetters = processWordElimination(cleanIntention)
    }

    // --- Charging suggestions ---
    const rng = new SeededRandom(seed + 1)
    const chargingSuggestions = rng.sample(CHARGING_METHODS, 2)

    // --- Witness prompts ---
    const witnessPrompts = generateWitnessPrompts(
      method,
      cleanIntention,
      processedLetters ?? undefined,
      seed,
    )

    const svgPreview = this.safeSvgPreview(svgTemplate)

    // --- Image generation/edit via provider abstraction (T-035) ---
    let generatedImage: {
      b64_json?: string
      url?: string
      prompt_used?: string
      style?: string
      model?: string
      error?: string
    } | null = null

    const provider = this.imageProvider

    if (editImageB64) {
      // Edit mode
      if (!provider.isAvailable() || !provider.edit) {
        generatedImage = { error: `${provider.name} not configured or does not support edit.` }
      } else {
        try {
          const builtPrompt = buildSigilEditPrompt(cleanIntention, editInstruction, imageStyle)
          const result: GeneratedImage = await provider.edit({
            image: editImageB64,
            prompt: builtPrompt.prompt,
            model: imageModel,
            seed,
            style: imageStyle,
          })
          generatedImage = {
            b64_json: result.b64_json,
            url: result.url,
            prompt_used: builtPrompt.prompt,
            model: result.metadata.model,
            style: result.metadata.style,
          }
        } catch (err) {
          generatedImage = {
            error: err instanceof Error ? err.message : 'Image edit failed',
          }
        }
      }
    } else if (generateImageFlag) {
      // Generate mode
      if (!provider.isAvailable()) {
        generatedImage = {
          error: `${provider.name} not configured. Set provider credentials for image generation.`,
        }
      } else {
        try {
          const builtPrompt = buildSigilPrompt(
            cleanIntention,
            method,
            processedLetters ?? undefined,
            imageStyle,
          )
          const result: GeneratedImage = await provider.generate({
            prompt: builtPrompt.prompt,
            model: imageModel,
            width: 1024,
            height: 1024,
            seed,
            style: imageStyle,
          })
          generatedImage = {
            b64_json: result.b64_json,
            url: result.url,
            prompt_used: builtPrompt.prompt,
            style: builtPrompt.style,
            model: result.metadata.model,
          }
        } catch (err) {
          generatedImage = {
            error: err instanceof Error ? err.message : 'Image generation failed',
          }
        }
      }
    }

    const endTime = performance.now()

    const result: Record<string, unknown> = {
      intention: cleanIntention,
      method: {
        id: method.id,
        name: method.name,
        description: method.description,
        steps: method.steps,
      },
      processing: processedLetters
        ? {
            type: 'word_elimination',
            original: cleanIntention,
            remaining_letters: processedLetters,
            letter_count: processedLetters.length,
          }
        : null,
      charging_suggestions: chargingSuggestions.map((c) => ({
        name: c.name,
        description: c.description,
      })),
      guidance: {
        note:
          generateImageFlag || editImageB64
            ? 'AI-generated sigil image included. The symbol is a starting point — refine it intuitively or use it as-is in your practice.'
            : 'This engine provides the process for sigil creation. The actual visual sigil must be created by you — this personal investment is essential to the magic.',
        next_steps:
          generateImageFlag || generatedImage?.b64_json
            ? [
                'Contemplate the generated image — does it resonate with your intention?',
                'You may print, trace, or redraw the sigil by hand to further charge it',
                `Choose a charging method: ${chargingSuggestions.map((c) => c.name).join(' or ')}`,
                'Release attachment to outcome after charging',
              ]
            : [
                'Gather your materials (paper, pen, or digital canvas)',
                `Follow the ${method.name} steps above`,
                'Allow intuition to guide the final form',
                'Choose a charging method that resonates',
                'Release attachment to outcome',
              ],
      },
      svg_preview: svgPreview,
      generated_image: generatedImage,
      image_gen_available: provider.isAvailable(),
      provider: provider.name,
      seed,
    }

    // Per FROZEN (T-002/T-003/T-035): surface generated_image at top-level EngineOutput when present (no error)
    const topGenerated: GeneratedImage | undefined = generatedImage && !generatedImage.error
      ? {
          b64_json: generatedImage.b64_json,
          url: generatedImage.url,
          metadata: {
            model: generatedImage.model ?? 'default',
            prompt: generatedImage.prompt_used ?? cleanIntention,
            provider: provider.name,
            style: generatedImage.style,
            seed,
          },
        }
      : undefined

    const output: EngineOutput = {
      engine_id: 'sigil-forge',
      result,
      witness_prompts: witnessPrompts,
      calculated_at: new Date().toISOString(),
      processing_time_ms: Math.round(endTime - startTime),
    }

    if (topGenerated) {
      ;(output as any).generated_image = topGenerated
    }

    return output
  }
}
