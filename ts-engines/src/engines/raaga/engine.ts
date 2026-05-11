/**
 * RaagaEngine — Carnatic melakarta sound therapy engine
 *
 * Distinct from the Rust NadaBrahman engine (which *recommends* a raga based on
 * dosha/time/chakra). This engine takes a melakarta number or name as input and
 * returns the full just-intonation data, shruti indices, Ayurvedic context, and
 * Strudel-compatible ratio arrays so the front-end RaagaPlayer can render sound.
 *
 * engine_id: "raaga"
 */

import type { ConsciousnessEngine, EngineInput, EngineMetadata, EngineOutput } from '../../types'
import { EngineValidationError } from '../../utils'
import {
  type Dosha,
  MELAKARTAS,
  findMelakartaByName,
  getMelakarta,
  getPraharForHour,
  getRaagasForDosha,
} from './wisdom'
import { generateWitnessPrompts } from './witness'

const VALID_DOSHAS = ['vata', 'pitta', 'kapha'] as const

export class RaagaEngine implements ConsciousnessEngine {
  metadata(): EngineMetadata {
    return {
      id: 'raaga',
      name: 'Raaga Consciousness Engine',
      description:
        'Carnatic melakarta sound therapy. Returns just-intonation swaras, chakra mappings, ' +
        'Ayurvedic affinity, and Strudel-compatible ratio arrays for all 72 Melakartas. ' +
        'Works alongside the NadaBrahman engine: NadaBrahman recommends a raga; ' +
        'Raaga provides its complete musical and therapeutic profile.',
      version: '1.0.0',
      required_phase: 0,
      input_schema: {
        melakarta: {
          type: 'number',
          required: false,
          description: 'Melakarta number (1–72). If omitted, selected by dosha + time-of-day.',
        },
        name: {
          type: 'string',
          required: false,
          description:
            'Melakarta name (e.g. "Mayamalavagaula"). Resolved by partial match. Overrides melakarta number.',
        },
        dosha: {
          type: 'string',
          required: false,
          description:
            'Ayurvedic constitution: "vata" | "pitta" | "kapha". Used for auto-selection and affinity scoring.',
          enum: [...VALID_DOSHAS],
        },
        root_hz: {
          type: 'number',
          required: false,
          description:
            'Root frequency for Sa in Hz. Defaults to 220 (A3). Used in ratio annotations.',
          default: 220,
        },
        include_alternates: {
          type: 'boolean',
          required: false,
          description: 'When true, also returns 2–3 alternate ragas of the same dosha affinity.',
          default: false,
        },
      },
    }
  }

  async calculate(input: EngineInput): Promise<EngineOutput> {
    const startTime = performance.now()

    const numParam = input.parameters.melakarta as number | undefined
    const nameParam = input.parameters.name as string | undefined
    const doshaParam = input.parameters.dosha as string | undefined
    const rootHz = (input.parameters.root_hz as number | undefined) ?? 220
    const includeAlternates = Boolean(input.parameters.include_alternates ?? false)

    // Validate dosha
    let dosha: Dosha | undefined
    if (doshaParam !== undefined) {
      if (!VALID_DOSHAS.includes(doshaParam as Dosha)) {
        throw new EngineValidationError(
          `Invalid dosha "${doshaParam}". Must be one of: ${VALID_DOSHAS.join(', ')}.`,
          'INVALID_DOSHA',
          { provided: doshaParam, valid: VALID_DOSHAS },
        )
      }
      dosha = doshaParam as Dosha
    }

    // Resolve melakarta
    let melakartaNum: number

    if (nameParam !== undefined && nameParam.trim() !== '') {
      const found = findMelakartaByName(nameParam)
      if (!found) {
        throw new EngineValidationError(
          `No melakarta found matching "${nameParam}".`,
          'UNKNOWN_MELAKARTA_NAME',
          { provided: nameParam, example: 'Mayamalavagaula' },
        )
      }
      melakartaNum = found.num
    } else if (numParam !== undefined) {
      if (!Number.isInteger(numParam) || numParam < 1 || numParam > 72) {
        throw new EngineValidationError(
          'Melakarta number must be an integer between 1 and 72.',
          'INVALID_MELAKARTA_NUMBER',
          { provided: numParam },
        )
      }
      melakartaNum = numParam
    } else if (dosha) {
      // Auto-select by dosha × time-of-day
      const hour = new Date().getHours()
      const prahar = getPraharForHour(hour)
      // Prefer ragas in the prahar recommendation that also match dosha affinity
      const doshaRagas = getRaagasForDosha(dosha).map((m) => m.num)
      const intersection = prahar.recommended.filter((n) => doshaRagas.includes(n))
      melakartaNum = intersection.length > 0 ? intersection[0] : doshaRagas[0]
    } else {
      // Random selection weighted toward the current prahar
      const hour = new Date().getHours()
      const prahar = getPraharForHour(hour)
      melakartaNum = prahar.recommended[0]
    }

    const m = getMelakarta(melakartaNum)
    if (!m) {
      throw new EngineValidationError(
        `No melakarta found for number ${melakartaNum}.`,
        'MELAKARTA_NOT_FOUND',
        { melakartaNum },
      )
    }

    // Build swara annotation (name + ratio)
    const swara_labels = ['Sa', 'Re', 'Ga', 'Ma', 'Pa', 'Dha', 'Ni', "Sa'"]
    const swaras = m.arohana.map((shruti_idx, i) => ({
      swara: swara_labels[i],
      shruti_index: shruti_idx,
      ratio_num: [
        1, 256, 16, 10, 9, 32, 6, 5, 81, 4, 27, 45, 729, 3, 128, 8, 5, 27, 16, 9, 15, 243, 2,
      ][shruti_idx],
      ratio_den: [1, 243, 15, 9, 8, 27, 5, 4, 64, 3, 20, 32, 512, 2, 81, 5, 3, 16, 9, 5, 8, 128, 1][
        shruti_idx
      ],
      ratio_decimal: m.ratios[i],
      hz: +(rootHz * m.ratios[i]).toFixed(3),
    }))

    // Current prahar context
    const hour = new Date().getHours()
    const prahar = getPraharForHour(hour)
    const prahar_match = prahar.recommended.includes(melakartaNum)

    // Dosha affinity for the selected raga
    const dosha_affinities: Record<string, boolean> = {
      vata: getRaagasForDosha('vata').some((x) => x.num === melakartaNum),
      pitta: getRaagasForDosha('pitta').some((x) => x.num === melakartaNum),
      kapha: getRaagasForDosha('kapha').some((x) => x.num === melakartaNum),
    }

    // Alternate ragas for same dosha
    const alternates =
      includeAlternates && dosha
        ? getRaagasForDosha(dosha)
            .filter((x) => x.num !== melakartaNum)
            .slice(0, 3)
            .map((x) => ({ num: x.num, name: x.name, chakra: x.chakra, ma_type: x.ma_type }))
        : []

    const result: Record<string, unknown> = {
      melakarta: {
        num: m.num,
        name: m.name,
        chakra: m.chakra,
        ma_type: m.ma_type,
      },
      swaras,
      // Strudel-compatible: pass directly to RaagaPlayer.play(num, { rootHz })
      strudel_ratios: m.ratios,
      root_hz: rootHz,
      arohana_indices: [...m.arohana],
      avarohana_indices: [...m.avarohana],
      prahar: {
        num: prahar.prahar,
        label: prahar.label,
        is_recommended_time: prahar_match,
      },
      dosha_affinities,
      ...(alternates.length > 0 ? { alternate_ragas: alternates } : {}),
      total_melakartas: MELAKARTAS.length,
    }

    return {
      engine_id: 'raaga',
      result,
      witness_prompts: generateWitnessPrompts(m, dosha),
      calculated_at: new Date().toISOString(),
      processing_time_ms: Math.round(performance.now() - startTime),
    }
  }
}
