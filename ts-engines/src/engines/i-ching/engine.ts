/** Seeded three-coin simulation and explicit manual King Wen selection. */

import type { ConsciousnessEngine, EngineInput, EngineMetadata, EngineOutput } from '../../types'
import { EngineValidationError } from '../../utils'
import { SeededRandom, getDefaultSeed } from '../../utils/random'
import { type Hexagram, getHexagramByNumber, linesToHexagramNumber } from './wisdom'
import { generateWitnessPrompts } from './witness'

export type LineValue = 6 | 7 | 8 | 9
const I_CHING_VERSION = '1.1.0'

function validateSeed(seed: number): void {
  if (!Number.isSafeInteger(seed) || seed < 0) {
    throw new EngineValidationError('Seed must be a nonnegative safe integer.', 'INVALID_SEED', {
      field: 'seed',
    })
  }
}

export function threeCoinLineValue(coins: readonly number[]): LineValue {
  if (coins.length !== 3 || coins.some((coin) => coin !== 2 && coin !== 3)) {
    throw new RangeError('A three-coin line requires three coin faces, each valued 2 or 3.')
  }
  return coins.reduce((sum, coin) => sum + coin, 0) as LineValue
}

export function castThreeCoinLines(seed: number): LineValue[] {
  validateSeed(seed)
  const rng = new SeededRandom(seed)
  // The shared generator's initial draws are correlated with its small seed.
  // Discard the initialization transient locally: without this, seeds 0..8191
  // all produce old yin for the first line. Keep the same reproducible PRNG.
  for (let draw = 0; draw < 16; draw++) rng.next()
  return Array.from({ length: 6 }, () =>
    threeCoinLineValue(Array.from({ length: 3 }, () => (rng.nextBool() ? 3 : 2))),
  )
}

export function resolveCast(lineValues: readonly number[]): {
  primary: Hexagram
  relating: Hexagram | undefined
  changingLines: number[]
} {
  if (
    lineValues.length !== 6 ||
    lineValues.some((value) => !Number.isInteger(value) || value < 6 || value > 9)
  ) {
    throw new RangeError('A cast requires six line values from 6 through 9, bottom to top.')
  }
  const lines = lineValues.map((value) => value === 7 || value === 9)
  const primary = getHexagramByNumber(linesToHexagramNumber(lines)) as Hexagram
  const changingLines = lineValues.flatMap((value, index) =>
    value === 6 || value === 9 ? [index + 1] : [],
  )
  const relatingLines = lines.map((yang, index) =>
    lineValues[index] === 6 || lineValues[index] === 9 ? !yang : yang,
  )
  const relating =
    changingLines.length > 0 ? getHexagramByNumber(linesToHexagramNumber(relatingLines)) : undefined
  return { primary, relating, changingLines }
}

function hexagramResult(hexagram: Hexagram): Record<string, unknown> {
  return {
    number: hexagram.number,
    name: hexagram.name,
    chinese_name: hexagram.chineseName,
    meaning: hexagram.meaning,
    judgment: hexagram.judgment,
    image: hexagram.image,
    lines: [...hexagram.lines],
    wisdom_status: hexagram.wisdomStatus,
    field_status: { ...hexagram.fieldStatus },
    provenance: hexagram.provenance,
  }
}

export class IChingEngine implements ConsciousnessEngine {
  metadata(): EngineMetadata {
    return {
      id: 'i-ching',
      name: 'I-Ching Consciousness Engine',
      description:
        'Seeded three-coin simulation with King Wen line mapping and a relating hexagram derived from changing lines. Text availability and provenance are returned separately.',
      version: I_CHING_VERSION,
      required_phase: 0,
      input_schema: {
        hexagram: {
          type: 'number',
          required: false,
          description:
            'Manual King Wen selection (integer 1-64), with stable lines and no simulated changing lines. Omit to simulate three coins for each line.',
        },
        method: {
          type: 'string',
          required: false,
          description: 'Supported simulated casting method: three_coins. Yarrow is unavailable.',
          default: 'three_coins',
          enum: ['three_coins'],
        },
      },
    }
  }

  async calculate(input: EngineInput): Promise<EngineOutput> {
    const startTime = performance.now()
    const seed = input.seed === undefined ? getDefaultSeed() : input.seed
    validateSeed(seed)

    const question = input.question === undefined ? input.parameters.question : input.question
    if (question !== undefined && (typeof question !== 'string' || question.trim() === '')) {
      throw new EngineValidationError('Question must be a nonempty string.', 'INVALID_QUESTION', {
        field: 'question',
      })
    }
    const method = input.parameters.method === undefined ? 'three_coins' : input.parameters.method
    if (method !== 'three_coins') {
      throw new EngineValidationError(
        'Only three_coins is implemented; yarrow_stalks is unavailable.',
        method === 'yarrow_stalks' ? 'UNSUPPORTED_METHOD' : 'INVALID_METHOD',
        { field: 'method', supported: ['three_coins'] },
      )
    }

    const selected = input.parameters.hexagram
    if (
      selected !== undefined &&
      (typeof selected !== 'number' || !Number.isInteger(selected) || selected < 1 || selected > 64)
    ) {
      throw new EngineValidationError(
        'Hexagram must be an integer between 1 and 64.',
        'INVALID_HEXAGRAM',
        { field: 'hexagram', min: 1, max: 64 },
      )
    }
    const manual = selected !== undefined
    const lineValues = manual
      ? (getHexagramByNumber(selected as number) as Hexagram).lines.map((yang) => (yang ? 7 : 8))
      : castThreeCoinLines(seed)
    const { primary, relating, changingLines } = resolveCast(lineValues)

    return {
      engine_id: 'i-ching',
      result: {
        primary_hexagram: hexagramResult(primary),
        changing_lines: changingLines.length > 0 ? changingLines : null,
        relating_hexagram: relating ? hexagramResult(relating) : null,
        casting: {
          algorithm_version: I_CHING_VERSION,
          method: manual ? 'manual_selection' : 'three_coins',
          selection_mode: manual ? 'manual' : 'seeded_coin_simulation',
          line_order: 'bottom_to_top',
          line_values: lineValues,
        },
        seed,
      },
      witness_prompts: generateWitnessPrompts(primary, relating, changingLines, seed),
      calculated_at: new Date().toISOString(),
      processing_time_ms: Math.round(performance.now() - startTime),
    }
  }
}
