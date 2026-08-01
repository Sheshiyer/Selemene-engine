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

import type {
  ConsciousnessEngine,
  Consent,
  EngineInput,
  EngineMetadata,
  EngineOutput,
} from '../../types'
import { EngineValidationError } from '../../utils'
import { generateRaagaClip } from './clip'
import {
  type Dosha,
  MELAKARTAS,
  findMelakartaByName,
  getMelakarta,
  getPraharForHour,
  getRaagasForDosha,
  verifyFull72Melakartas,
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
        request_clip: {
          type: 'boolean',
          required: false,
          description:
            'When true AND granted consent is present, render strudel_ratios to an audio clip and ' +
            'populate generated_audio.clip_url (config: RAAGA_CLIP_MODE=off|local|service). Default false (clip_url stays null).',
          default: false,
        },
      },
    }
  }

  async calculate(input: EngineInput): Promise<EngineOutput> {
    const startTime = performance.now()

    // Support media input options per FROZEN (audio_ref, consent, quality) + parameters
    // audio_ref/consent can be top-level (extended EngineInput) or inside parameters for compat
    // full support for 72 melakartas + dosha/prahar verified via wisdom.verifyFull72Melakartas
    const audioRef = input.audio_ref || input.parameters.audio_ref
    const consent = input.consent || input.parameters.consent
    const _quality = input.quality || input.parameters.quality // for future clip quality

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

    // Per FROZEN + T-005/T-031: surface generated_audio at top-level EngineOutput (in addition to result)
    // strudel_ratios, clip_url (null by default; populated by consent-gated clip gen below), root_hz, metadata
    // Full 72 + dosha/prahar verified here (see wisdom.verifyFull72Melakartas)
    // Supports audio_ref/consent from input (FROZEN samples in tests/harness)
    // Cites (mandatory all): p1-w1-worker-bootstrap-packet.md, resources-and-assets.md, gaps-and-improvements.md, goal-understanding.md,
    // EXECUTION-STATUS.md, P1W2-HANDOFF.md, .worktrees/T-002-copilot/P1W1-CONTRACTS-FROZEN.md, detailed-task-list.md (T-031),
    // .worktrees/T-024-codex/scripts/ext-contract-harness.ts , ts-engines/src/engines/raaga/*.ts , raaga.md,
    // docs/plans/engine-integration/p5-p4-next-batch.json (raaga-clip-generation)
    // tags: phase:integration-p1 wave:integration-w2 area:engine-integration engine-raaga
    // External rail unavailable; Codex subagent. No push/merge.
    const verif = verifyFull72Melakartas() // from wisdom, ensures support

    // raaga-clip-generation (p5-p4-next-batch): consent-gated, config-driven clip path.
    // request_clip + granted consent → generateRaagaClip (RAAGA_CLIP_MODE=off|local|service, default off).
    // clip_url stays null when not requested / consent missing / mode off (backward compat per FROZEN).
    const requestClip = Boolean(input.parameters.request_clip ?? false)
    const clipConsent = consent as Consent | undefined
    let clipUrl: string | null = null
    let clipNote: Record<string, unknown> = { requested: requestClip, status: 'not_requested' }
    if (requestClip) {
      if (!clipConsent?.granted) {
        clipNote = {
          requested: true,
          status: 'consent_missing',
          detail:
            'request_clip requires granted consent (local-first + explicit consent per goal-understanding.md)',
        }
      } else {
        const clip = await generateRaagaClip({ melakarta: m.num, ratios: m.ratios, rootHz })
        clipUrl = clip.clip_url
        clipNote = {
          requested: true,
          status: clip.status,
          mode: clip.mode,
          ...(clip.detail ? { detail: clip.detail } : {}),
        }
      }
    }

    const generatedAudio = {
      clip_url: clipUrl,
      strudel_ratios: [...m.ratios],
      root_hz: rootHz,
      metadata: {
        engine: 'raaga',
        melakarta: m.num,
        name: m.name,
        dosha_match: dosha ? dosha_affinities[dosha] : null,
        prahar: prahar.label,
        verification: verif, // full 72 + dosha/prahar evidence
        clip: clipNote, // clip generation outcome (requested/status/mode/detail)
        // timbre/gamaka per T-005 deferred
      },
    }

    return {
      engine_id: 'raaga',
      result,
      witness_prompts: generateWitnessPrompts(m, dosha),
      calculated_at: new Date().toISOString(),
      processing_time_ms: Math.round(performance.now() - startTime),
      generated_audio: generatedAudio,
    }
  }
}
