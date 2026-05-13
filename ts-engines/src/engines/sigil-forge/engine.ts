/**
 * SigilForgeEngine - Consciousness engine for sigil creation + AI-generated sigil images
 *
 * Modes:
 *   1. Guidance-only (default): Returns method steps, process, and witness prompts.
 *   2. Generate image (parameters.generate_image = true): Calls NVIDIA NIM to generate
 *      an actual sigil image. Returns base64 PNG + the guidance steps.
 *   3. Edit image (parameters.edit_image_b64 provided): Uses NVIDIA NIM image editing
 *      to refine an existing sigil with a new instruction.
 */

import type { ConsciousnessEngine, EngineInput, EngineMetadata, EngineOutput } from '../../types'
import { EngineValidationError } from '../../utils'
import { SeededRandom, getDefaultSeed } from '../../utils/random'
import {
  editImage,
  generateImage,
  isImageGenAvailable,
  NVIDIA_IMAGE_MODELS,
  type NvidiaImageModel,
} from '../../utils/nvidia-image'
import {
  CHARGING_METHODS,
  SIGIL_METHODS,
  getMethodById,
  getMethodIds,
  processWordElimination,
} from './wisdom'
import { generateWitnessPrompts } from './witness'
import {
  buildSigilPrompt,
  buildSigilEditPrompt,
  type SigilStyle,
} from './prompt-builder'

export class SigilForgeEngine implements ConsciousnessEngine {
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
    const imageGenNote = isImageGenAvailable()
      ? 'Set parameters.generate_image=true to receive an AI-generated sigil image via NVIDIA NIM.'
      : 'Image generation is not configured (NVIDIA_API_KEY missing). Guidance-only mode active.'

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
          description: 'The intention or desire to encode into a sigil. Write as a present-tense statement.',
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
          description: 'When true, generates an AI sigil image via NVIDIA NIM. Requires NVIDIA_API_KEY.',
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
          description: 'NVIDIA NIM model. flux.1-dev = best quality; flux.1-schnell = faster.',
          enum: Object.values(NVIDIA_IMAGE_MODELS),
        },
        edit_image_b64: {
          type: 'string',
          required: false,
          description: 'Base64-encoded PNG of an existing sigil to refine. Triggers edit mode.',
        },
        edit_instruction: {
          type: 'string',
          required: false,
          description: 'Edit instruction for refining an existing sigil (used with edit_image_b64).',
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
    const imageModel = input.parameters.image_model as NvidiaImageModel | undefined
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

    // --- Image generation (optional) ---
    let generatedImage: {
      b64_json?: string
      url?: string
      prompt_used?: string
      style?: string
      model?: string
      error?: string
    } | null = null

    if (editImageB64) {
      // Edit mode: refine an existing sigil
      if (!isImageGenAvailable()) {
        generatedImage = { error: 'NVIDIA_API_KEY not configured. Cannot edit image.' }
      } else {
        try {
          const builtPrompt = buildSigilEditPrompt(cleanIntention, editInstruction, imageStyle)
          const result = await editImage({
            image: editImageB64,
            prompt: builtPrompt.prompt,
            model: imageModel,
            seed,
          })
          generatedImage = {
            b64_json: result.b64_json,
            prompt_used: builtPrompt.prompt,
            model: imageModel ?? 'default',
          }
        } catch (err) {
          generatedImage = {
            error: err instanceof Error ? err.message : 'Image edit failed',
          }
        }
      }
    } else if (generateImageFlag) {
      // Generation mode: create new sigil image
      if (!isImageGenAvailable()) {
        generatedImage = {
          error: 'NVIDIA_API_KEY not configured. Set it in Railway env vars to enable image generation.',
        }
      } else {
        try {
          const builtPrompt = buildSigilPrompt(
            cleanIntention,
            method,
            processedLetters ?? undefined,
            imageStyle,
          )
          const result = await generateImage({
            prompt: builtPrompt.prompt,
            model: imageModel ?? NVIDIA_IMAGE_MODELS.FLUX_DEV,
            width: 1024,
            height: 1024,
            seed,
          })
          generatedImage = {
            b64_json: result.b64_json,
            prompt_used: builtPrompt.prompt,
            style: builtPrompt.style,
            model: imageModel ?? NVIDIA_IMAGE_MODELS.FLUX_DEV,
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
        note: generateImageFlag || editImageB64
          ? 'AI-generated sigil image included. The symbol is a starting point — refine it intuitively or use it as-is in your practice.'
          : 'This engine provides the process for sigil creation. The actual visual sigil must be created by you — this personal investment is essential to the magic.',
        next_steps: generateImageFlag || generatedImage?.b64_json
          ? [
              'Contemplate the generated image — does it resonate with your intention?',
              'You may print, trace, or redraw the sigil by hand to further charge it',
              `Choose a charging method: ${chargingSuggestions.map(c => c.name).join(' or ')}`,
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
      image_gen_available: isImageGenAvailable(),
      seed,
    }

    return {
      engine_id: 'sigil-forge',
      result,
      witness_prompts: witnessPrompts,
      calculated_at: new Date().toISOString(),
      processing_time_ms: Math.round(endTime - startTime),
    }
  }
}
