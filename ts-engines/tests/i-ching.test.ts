import { describe, expect, it } from 'bun:test'
import {
  IChingEngine,
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
