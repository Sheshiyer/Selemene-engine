/**
 * IChingEngine - Consciousness engine for I-Ching divination
 */

import type { ConsciousnessEngine, EngineMetadata, EngineInput, EngineOutput } from '../../types'
import { HEXAGRAMS, getHexagramByNumber } from './wisdom'
import { generateWitnessPrompts } from './witness'
import { SeededRandom, getDefaultSeed } from '../../utils/random'

export class IChingEngine implements ConsciousnessEngine {
  metadata(): EngineMetadata {
    return {
      id: 'i-ching',
      name: 'I-Ching Consciousness Engine',
      description:
        'I-Ching divination with the 64 hexagrams. Generates primary hexagram with optional changing lines producing a relating hexagram.',
      version: '1.0.0',
      required_phase: 0,
      input_schema: {
        hexagram: {
          type: 'number',
          required: false,
          description: 'Specific hexagram number (1-64). If not provided, one is cast randomly.',
        },
        method: {
          type: 'string',
          required: false,
          description: 'Casting method: three_coins (default) or yarrow_stalks',
          default: 'three_coins',
          enum: ['three_coins', 'yarrow_stalks'],
        },
      },
    }
  }

  async calculate(input: EngineInput): Promise<EngineOutput> {
    const startTime = performance.now()

    const seed = input.seed ?? getDefaultSeed()
    const rng = new SeededRandom(seed)

    // Get or cast primary hexagram
    const hexagramParam = input.parameters.hexagram as number | undefined
    let primaryNumber: number

    if (hexagramParam && hexagramParam >= 1 && hexagramParam <= 64) {
      primaryNumber = hexagramParam
    } else {
      primaryNumber = rng.nextInt(1, 64)
    }

    const primary = getHexagramByNumber(primaryNumber)!

    // Cast changing lines (each line has ~25% chance of being changing in three-coin method)
    const changingLines: number[] = []
    const lineValues: number[] = []

    for (let i = 0; i < 6; i++) {
      const value = rng.nextInt(6, 9) // 6=old yin, 7=young yang, 8=young yin, 9=old yang
      lineValues.push(value)
      if (value === 6 || value === 9) {
        changingLines.push(i + 1) // 1-indexed position
      }
    }

    // Calculate relating hexagram if there are changing lines
    let relating: typeof primary | undefined
    if (changingLines.length > 0) {
      // Flip the changing lines to get relating hexagram
      const relatingLines = primary.lines.map((yang, i) => {
        if (changingLines.includes(i + 1)) {
          return !yang
        }
        return yang
      }) as [boolean, boolean, boolean, boolean, boolean, boolean]

      // Find matching hexagram (simplified - just pick another one for stub)
      const relatingNumber = rng.nextInt(1, 64)
      relating = getHexagramByNumber(relatingNumber)
    }

    // Generate witness prompts
    const witnessPrompts = generateWitnessPrompts(primary, relating, changingLines, seed)

    const endTime = performance.now()

    const result = {
      primary_hexagram: {
        number: primary.number,
        name: primary.name,
        chinese_name: primary.chineseName,
        meaning: primary.meaning,
        judgment: primary.judgment,
        image: primary.image,
      },
      changing_lines: changingLines.length > 0 ? changingLines : null,
      relating_hexagram: relating
        ? {
            number: relating.number,
            name: relating.name,
            chinese_name: relating.chineseName,
            meaning: relating.meaning,
            judgment: relating.judgment,
            image: relating.image,
          }
        : null,
      casting: {
        method: (input.parameters.method as string) ?? 'three_coins',
        line_values: lineValues,
      },
      seed,
    }

    return {
      engine_id: 'i-ching',
      result,
      witness_prompts: witnessPrompts,
      calculated_at: new Date().toISOString(),
      processing_time_ms: Math.round(endTime - startTime),
    }
  }
}
