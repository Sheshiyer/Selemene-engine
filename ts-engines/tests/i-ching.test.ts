import { describe, expect, it } from 'bun:test'
import {
  IChingEngine,
  castDerivation,
  castThreeCoinLines,
  resolveCast,
  threeCoinLineValue,
} from '../src/engines/i-ching/engine'
import {
  HEXAGRAMS,
  getHexagramByNumber,
  linesToHexagramNumber,
} from '../src/engines/i-ching/wisdom'
import { generateWitnessPrompts } from '../src/engines/i-ching/witness'
import type { EngineInput } from '../src/types'

// Independent fixture: six glyph bars read bottom-to-top from the official
// Unicode chart U+4DC0..U+4DFF, in King Wen order. No chart assets are bundled.
// https://www.unicode.org/charts/PDF/U4DC0.pdf
const KING_WEN_PATTERNS = [
  '111111',
  '000000',
  '100010',
  '010001',
  '111010',
  '010111',
  '010000',
  '000010',
  '111011',
  '110111',
  '111000',
  '000111',
  '101111',
  '111101',
  '001000',
  '000100',
  '100110',
  '011001',
  '110000',
  '000011',
  '100101',
  '101001',
  '000001',
  '100000',
  '100111',
  '111001',
  '100001',
  '011110',
  '010010',
  '101101',
  '001110',
  '011100',
  '001111',
  '111100',
  '000101',
  '101000',
  '101011',
  '110101',
  '001010',
  '010100',
  '110001',
  '100011',
  '111110',
  '011111',
  '000110',
  '011000',
  '010110',
  '011010',
  '101110',
  '011101',
  '100100',
  '001001',
  '001011',
  '110100',
  '101100',
  '001101',
  '011011',
  '110110',
  '010011',
  '110010',
  '110011',
  '001100',
  '101010',
  '010101',
]

interface ReadingResult {
  primary_hexagram: {
    number: number
    lines: boolean[]
    wisdom_status: string
    field_status: Record<string, string>
    judgment: string | null
    image: string | null
  }
  changing_lines: number[] | null
  relating_hexagram: { number: number; lines: boolean[] } | null
  casting: { method: string; line_values: number[]; line_order: string; algorithm_version: string }
  seed: number
}

const engine = new IChingEngine()
function input(overrides: Record<string, unknown> = {}): EngineInput {
  return { consciousness_level: 0, parameters: {}, seed: 0, ...overrides } as EngineInput
}

describe('I Ching King Wen computation', () => {
  it('matches all 64 independent glyph patterns in both directions', () => {
    expect(HEXAGRAMS).toHaveLength(64)
    expect(new Set(KING_WEN_PATTERNS).size).toBe(64)
    for (let index = 0; index < 64; index++) {
      const lines = [...KING_WEN_PATTERNS[index]].map((value) => value === '1')
      expect(linesToHexagramNumber(lines)).toBe(index + 1)
      expect([...(getHexagramByNumber(index + 1)?.lines ?? [])]).toEqual(lines)
    }
  })

  it('has the exact 1:3:3:1 three-coin distribution across all eight outcomes', () => {
    const counts = [0, 0, 0, 0]
    for (const first of [2, 3]) {
      for (const second of [2, 3]) {
        for (const third of [2, 3]) counts[threeCoinLineValue([first, second, third]) - 6]++
      }
    }
    expect(counts).toEqual([1, 3, 3, 1])
  })

  it('avoids initial-seed bias for every line across 8192 seed simulations', () => {
    const counts = Array.from({ length: 6 }, () => [0, 0, 0, 0])
    for (let seed = 0; seed < 8192; seed++) {
      castThreeCoinLines(seed).forEach((value, line) => counts[line][value - 6]++)
    }
    for (const line of counts) {
      for (let value = 0; value < 4; value++) {
        expect(Math.abs(line[value] - [1024, 3072, 3072, 1024][value])).toBeLessThan(100)
      }
    }
  })

  it('repairs the actual observed inconsistent cast without choosing random IDs', () => {
    const { primary, relating, changingLines } = resolveCast([6, 7, 6, 6, 6, 6])
    expect(primary.number).toBe(7)
    expect(primary.lines).toEqual([false, true, false, false, false, false])
    expect(changingLines).toEqual([1, 3, 4, 5, 6])
    expect(relating?.number).toBe(1)
  })

  it('derives zero, all, and each single changed line accurately', () => {
    const cases: [number, number, number | undefined][] = [
      [7, 1, undefined],
      [8, 2, undefined],
      [6, 2, 1],
      [9, 1, 2],
    ]
    for (const [value, primary, related] of cases) {
      const cast = resolveCast(Array.from({ length: 6 }, () => value))
      expect(cast.primary.number).toBe(primary)
      expect(cast.relating?.number).toBe(related)
      expect(cast.changingLines).toHaveLength(related === undefined ? 0 : 6)
    }
    for (let changed = 0; changed < 6; changed++) {
      const values = Array.from({ length: 6 }, () => 7)
      values[changed] = 6
      const cast = resolveCast(values)
      expect([...cast.primary.lines]).toEqual(values.map((value) => value === 7))
      expect(cast.changingLines).toEqual([changed + 1])
      expect(cast.relating?.number).toBe(1)
      expect([...(cast.relating?.lines ?? [])]).toEqual(Array.from({ length: 6 }, () => true))
    }
  })

  it('retains seeded reproducibility for facts and witness prompts', async () => {
    for (const seed of [0, 1, 42, Number.MAX_SAFE_INTEGER]) {
      const first = await engine.calculate(input({ seed }))
      const second = await engine.calculate(input({ seed }))
      expect(first.result).toEqual(second.result)
      expect(first.witness_prompts).toEqual(second.witness_prompts)
      const reading = first.result as unknown as ReadingResult
      expect(reading.primary_hexagram.lines).toEqual(
        reading.casting.line_values.map((v) => v % 2 === 1),
      )
      expect(reading.casting.method).toBe('three_coins')
      expect(reading.casting.algorithm_version).toBe('1.1.0')
      expect(reading.casting.line_order).toBe('bottom_to_top')
    }
  })

  it('manual selection of every ID uses stable mapped lines without random changes', async () => {
    for (let number = 1; number <= 64; number++) {
      const output = await engine.calculate(input({ parameters: { hexagram: number } }))
      const reading = output.result as unknown as ReadingResult
      expect(reading.primary_hexagram.number).toBe(number)
      expect(reading.casting.method).toBe('manual_selection')
      expect(reading.casting.line_values).toEqual(
        [...KING_WEN_PATTERNS[number - 1]].map((v) => (v === '1' ? 7 : 8)),
      )
      expect(reading.changing_lines).toBeNull()
      expect(reading.relating_hexagram).toBeNull()
    }
  })

  it('rejects invalid seeds before BigInt initialization', async () => {
    for (const seed of [
      -1,
      0.5,
      Number.NaN,
      Number.POSITIVE_INFINITY,
      Number.MAX_SAFE_INTEGER + 1,
      '0',
      null,
    ]) {
      await expect(engine.calculate(input({ seed }))).rejects.toThrow('Seed must be')
    }
  })

  it('rejects unknown methods and explicitly marks yarrow unsupported', async () => {
    for (const method of ['yarrow_stalks', 'unknown', 42, null]) {
      await expect(engine.calculate(input({ parameters: { method } }))).rejects.toThrow(
        'Only three_coins',
      )
    }
    expect(engine.metadata().input_schema.method.enum).toEqual(['three_coins'])
  })

  it('rejects malformed questions, manual IDs, and internal cast inputs', async () => {
    for (const question of ['', '  ', 42, null]) {
      await expect(engine.calculate(input({ question }))).rejects.toThrow('Question must be')
      await expect(engine.calculate(input({ parameters: { question } }))).rejects.toThrow(
        'Question must be',
      )
    }
    for (const hexagram of [0, 65, 1.5, '1', null, Number.NaN]) {
      await expect(engine.calculate(input({ parameters: { hexagram } }))).rejects.toThrow(
        'Hexagram must be',
      )
    }
    expect(() => threeCoinLineValue([2, 2])).toThrow()
    expect(() => threeCoinLineValue([2, 2, 4])).toThrow()
    expect(() => resolveCast([6, 7, 8])).toThrow()
    expect(() => resolveCast([6, 7, 8, 9, 10, 7])).toThrow()
    expect(() => linesToHexagramNumber([true])).toThrow()
  })

  it('marks text gaps honestly and never emits generated stub meanings or quotations', () => {
    expect(new Set(HEXAGRAMS.map((hexagram) => hexagram.judgment)).size).toBe(64)
    expect(new Set(HEXAGRAMS.map((hexagram) => hexagram.image)).size).toBe(64)
    for (const hexagram of HEXAGRAMS) {
      expect(JSON.stringify(hexagram)).not.toContain('stub data')
      expect(hexagram.fieldStatus.lines).toBe('verified_mapping')
      expect(hexagram.wisdomStatus).toBe('source_transcribed')
      expect(hexagram.fieldStatus.judgment).toBe('source_transcribed')
      expect(hexagram.fieldStatus.image).toBe('source_transcribed')
      expect(hexagram.judgment?.length).toBeGreaterThan(10)
      expect(hexagram.image?.length).toBeGreaterThan(10)
      expect(hexagram.provenance.changing_line_texts).toBe('unavailable')
      expect(hexagram.provenance.text).toMatchObject({
        edition: { translator: 'James Legge', year: 1882 },
        transcription_status: 'scan_reviewed_normalized_transcription',
      })
      if (hexagram.number > 8) {
        expect(hexagram.chineseName).toBeNull()
        expect(hexagram.meaning).toBeNull()
        expect(hexagram.fieldStatus.meaning).toBe('unavailable')
        expect(hexagram.fieldStatus.name).toBe('generic_identifier')
      } else {
        expect(hexagram.fieldStatus.meaning).toBe('legacy_unverified')
        expect(hexagram.fieldStatus.name).toBe('legacy_unverified')
      }
      const prompts = generateWitnessPrompts(hexagram, undefined, undefined, 0)
      expect(prompts.length).toBeGreaterThan(0)
      expect(prompts.length).toBeLessThanOrEqual(3)
      for (const prompt of prompts) {
        expect(prompt.prompt).not.toContain('stub data')
        if (hexagram.judgment) expect(prompt.prompt).not.toContain(hexagram.judgment)
        if (hexagram.image) expect(prompt.prompt).not.toContain(hexagram.image)
      }
    }
  })
})

// #1341: provenance, confidence and degradation. The contract block must stay
// within the seven contracts/v1 fields (the Rust parser denies unknown keys).
const SEVEN_FIELDS = [
  'runtime_kind',
  'implementation_version',
  'cached',
  'fallback_used',
  'backend_id',
  'provider_id',
  'confidence',
]

interface Derivation {
  line_values: { origin: string; method: string; seed_source?: string }
  primary_hexagram: { origin: string; derived_from: string[] }
  changing_lines: { origin: string; derived_from: string[]; positions: number[] }
  relating_hexagram: {
    origin: string
    derived_from?: string[]
    flipped_positions?: number[]
    reason?: string
  }
  changing_line_texts: { origin: string; reason: string }
}

describe('I Ching provenance and degradation (#1341)', () => {
  function provenanceOf(output: Awaited<ReturnType<IChingEngine['calculate']>>) {
    expect(output.provenance).toBeDefined()
    const p = output.provenance as unknown as Record<string, unknown>
    for (const key of Object.keys(p)) expect(SEVEN_FIELDS).toContain(key)
    return p
  }

  it('happy path: a seeded cast labels lines as cast and everything else as derived', async () => {
    const output = await engine.calculate(input({ seed: 42 }))
    expect(provenanceOf(output)).toEqual({
      runtime_kind: 'typescript',
      implementation_version: '1.1.0',
      cached: false,
      fallback_used: false,
      confidence: 1,
    })
    const reading = output.result as unknown as ReadingResult & { derivation: Derivation }
    const d = reading.derivation
    expect(d.line_values).toEqual({
      origin: 'cast',
      method: 'seeded_three_coin_simulation',
      seed_source: 'request',
    })
    expect(d.primary_hexagram).toEqual({ origin: 'derived', derived_from: ['line_values'] })
    expect(d.changing_lines.origin).toBe('derived')
    expect(d.changing_lines.derived_from).toEqual(['line_values'])
    expect(d.changing_lines.positions).toEqual(reading.changing_lines ?? [])
    if (reading.relating_hexagram) {
      expect(d.relating_hexagram).toEqual({
        origin: 'derived',
        derived_from: ['primary_hexagram', 'changing_lines'],
        flipped_positions: reading.changing_lines as number[],
      })
    } else {
      expect(d.relating_hexagram.origin).toBe('not_applicable')
    }
  })

  it('happy path: an omitted seed is reported as generated, and the seed is echoed', async () => {
    const output = await engine.calculate({ consciousness_level: 0, parameters: {} })
    const reading = output.result as unknown as ReadingResult & { derivation: Derivation }
    expect(reading.derivation.line_values.seed_source).toBe('generated')
    expect(Number.isSafeInteger(reading.seed)).toBe(true)
    provenanceOf(output)
  })

  it('boundary: zero changing lines marks the relating hexagram not applicable', () => {
    const d = castDerivation('three_coins', 'request', resolveCast([7, 8, 7, 8, 7, 8]))
    expect(d.changing_lines.positions).toEqual([])
    expect(d.relating_hexagram.origin).toBe('not_applicable')
    expect(d.relating_hexagram.reason).toContain('No changing lines')
  })

  it('boundary: all six changing lines flip every position', () => {
    const d = castDerivation('three_coins', 'request', resolveCast([9, 6, 9, 6, 9, 6]))
    expect(d.changing_lines.positions).toEqual([1, 2, 3, 4, 5, 6])
    expect(d.relating_hexagram).toEqual({
      origin: 'derived',
      derived_from: ['primary_hexagram', 'changing_lines'],
      flipped_positions: [1, 2, 3, 4, 5, 6],
    })
  })

  it('boundary: manual selection is labelled user-selected, not cast, and has no relating hexagram', async () => {
    const output = await engine.calculate(input({ parameters: { hexagram: 64 } }))
    const reading = output.result as unknown as ReadingResult & { derivation: Derivation }
    expect(reading.derivation.line_values).toEqual({
      origin: 'user_selected',
      method: 'manual_king_wen_selection',
    })
    expect(reading.derivation.relating_hexagram.origin).toBe('not_applicable')
    expect(provenanceOf(output).fallback_used).toBe(false)
  })

  it('unavailable dependency: changing-line texts are marked unavailable, never substituted', async () => {
    const output = await engine.calculate(input({ seed: 42 }))
    const reading = output.result as unknown as ReadingResult & { derivation: Derivation }
    expect(reading.derivation.changing_line_texts.origin).toBe('unavailable')
    expect(reading.derivation.changing_line_texts.reason).toContain('not bundled')
    expect(provenanceOf(output).fallback_used).toBe(false)
    expect(JSON.stringify(reading)).not.toContain('line_texts":[')
  })

  it('unavailable dependency: yarrow stalks fail with UNSUPPORTED_METHOD instead of falling back to coins', async () => {
    const error = await engine
      .calculate(input({ parameters: { method: 'yarrow_stalks' } }))
      .catch((caught: unknown) => caught)
    expect((error as { code?: string }).code).toBe('UNSUPPORTED_METHOD')
  })

  it('invalid input: malformed requests fail before any provenance is emitted', async () => {
    for (const bad of [
      input({ seed: -1 }),
      input({ parameters: { hexagram: 65 } }),
      input({ parameters: { method: 'unknown' } }),
    ]) {
      const error = await engine.calculate(bad).catch((caught: unknown) => caught)
      expect((error as { code?: string }).code).toMatch(/^INVALID_/)
    }
  })
})
