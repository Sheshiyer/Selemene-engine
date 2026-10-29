/**
 * SacredGeometryEngine - Consciousness engine for sacred geometric forms
 */

import type { ConsciousnessEngine, EngineInput, EngineMetadata, EngineOutput } from '../../types'
import { EngineValidationError } from '../../utils'
import { SeededRandom, getDefaultSeed } from '../../utils/random'
import { buildGeometry } from './geometry'
import { SACRED_FORMS, getFormById, getFormIds } from './wisdom'
import { generateWitnessPrompts } from './witness'

export class SacredGeometryEngine implements ConsciousnessEngine {
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
    return {
      id: 'sacred-geometry',
      name: 'Sacred Geometry Consciousness Engine',
      description:
        'Exploration of sacred geometric forms with symbolism, meditation guidance, and witness prompts. Includes Flower of Life, Platonic Solids, Sri Yantra, and more.',
      version: '1.0.0',
      required_phase: 0,
      input_schema: {
        form: {
          type: 'string',
          required: false,
          description: 'Specific form to contemplate. If not provided, a random form is selected.',
          enum: getFormIds(),
        },
        intention: {
          type: 'string',
          required: false,
          description: 'Optional intention or question to hold while contemplating the form.',
        },
      },
    }
  }

  async calculate(input: EngineInput): Promise<EngineOutput> {
    const startTime = performance.now()

    // Extract parameters
    const formParam = input.parameters.form as string | undefined
    const intention = input.question ?? (input.parameters.intention as string | undefined)
    const svgTemplate = input.parameters.svg_template as string | undefined
    const seed = input.seed ?? getDefaultSeed()

    // Select form
    let form = formParam ? getFormById(formParam) : undefined

    if (formParam && !form) {
      throw new EngineValidationError('Unknown sacred geometry form.', 'INVALID_SACRED_FORM', {
        provided: formParam,
        supported: getFormIds(),
      })
    }

    const formSelection = form
      ? ({ kind: 'requested' } as const)
      : ({ kind: 'seeded_random', seed } as const)

    if (!form) {
      // Random selection if no form specified
      const rng = new SeededRandom(seed)
      form = rng.pick(SACRED_FORMS)
    }

    // Generate witness prompts
    const witnessPrompts = generateWitnessPrompts(form, intention, seed)

    const svgPreview = this.safeSvgPreview(svgTemplate)

    // Construction data is computed from the form's geometric definition
    // (GH #1524). Forms without a faithful construction get null geometry and
    // stay not_implemented with the reason.
    const built = buildGeometry(form.id)
    const geometry = 'geometry' in built ? built.geometry : null
    const calculatedGeometry =
      'geometry' in built
        ? {
            status: 'calculated',
            kind: 'deterministic_construction',
            source: 'geometry.buildGeometry',
            construction: built.geometry.construction,
            computed: {
              circles: built.geometry.circles.length,
              polygons: built.geometry.polygons.length,
              polylines: built.geometry.polylines.length,
              lines: built.geometry.lines.length,
              points: built.geometry.points.length,
              solid: built.geometry.solid !== undefined,
            },
          }
        : { status: 'not_implemented', reason: built.unsupported.reason }

    const endTime = performance.now()

    // Build result
    const result = {
      form: {
        id: form.id,
        name: form.name,
        description: form.description,
        symbolism: form.symbolism,
        elements: form.elements,
        numerology: form.numerology,
      },
      meditation: {
        prompt: form.meditationPrompt,
        duration_suggestion: '5-15 minutes',
      },
      intention: intention ?? null,
      svg_preview: svgPreview,
      seed,
      geometry,
      // Per-part origin of this result (GH #1401). Form text is a static table
      // lookup; geometry is computed per form (GH #1524). Nothing is rendered:
      // clients draw from result.geometry, so rendered_artifact stays
      // not_implemented rather than left implied.
      provenance_detail: {
        form: { kind: 'symbolic_lookup', source: 'wisdom.SACRED_FORMS' },
        form_selection: formSelection,
        meditation: { kind: 'symbolic_lookup', source: 'wisdom.SACRED_FORMS' },
        calculated_geometry: calculatedGeometry,
        rendered_artifact: {
          status: 'not_implemented',
          reason:
            'The engine renders no SVG or image; clients draw result.geometry from its numbers, and svg_preview only validates a caller-supplied svg_template.',
        },
        svg_preview: { kind: 'caller_template_validation', status: svgPreview.status },
        witness_prompts: { kind: 'templated_narrative' },
      },
    }

    return {
      engine_id: 'sacred-geometry',
      result,
      witness_prompts: witnessPrompts,
      calculated_at: new Date().toISOString(),
      processing_time_ms: Math.round(endTime - startTime),
      // Everything above comes from the in-process SACRED_FORMS table and
      // witness templates: there is no external dependency, fallback or mock
      // path, so fallback_used is always false and confidence is 1.
      provenance: {
        runtime_kind: 'typescript',
        implementation_version: this.metadata().version,
        cached: false,
        fallback_used: false,
        confidence: 1,
      },
    }
  }
}
