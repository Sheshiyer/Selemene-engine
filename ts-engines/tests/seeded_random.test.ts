/**
 * SeededRandom quality probes.
 *
 * Every TypeScript engine draws from SeededRandom (tarot shuffles, I Ching
 * casts, sacred-geometry selection, witness templates), and callers routinely
 * pass small explicit seeds such as 42 or 1401. Those seeds must give the same
 * spread of outcomes as large ones, a fixed seed must keep giving a fixed
 * sequence, and next() must stay inside [0, 1).
 */
import { describe, expect, it } from 'bun:test'
import { SeededRandom } from '../src/utils/random'

const SMALL_SEEDS = 10_000

/** Share of seeds 0..SMALL_SEEDS whose `draw`-th next() value is below 0.5. */
function lowShare(draw: number, seeds = SMALL_SEEDS): number {
  let low = 0
  for (let seed = 0; seed < seeds; seed++) {
    const rng = new SeededRandom(seed)
    let value = 0
    for (let i = 0; i <= draw; i++) value = rng.next()
    if (value < 0.5) low++
  }
  return low / seeds
}

describe('SeededRandom', () => {
  it('gives small consecutive seeds an even spread on each of the first draws', () => {
    for (let draw = 0; draw < 6; draw++) {
      const share = lowShare(draw)
      expect(share).toBeGreaterThan(0.47)
      expect(share).toBeLessThan(0.53)
    }
  })

  it('casts I Ching coins for small seeds in binomial proportions', () => {
    // Three coins per line: an old line (6 or 9) needs all three to agree,
    // so each should appear about 1/8 of the time.
    const counts: Record<number, number> = { 6: 0, 7: 0, 8: 0, 9: 0 }
    for (let seed = 0; seed < SMALL_SEEDS; seed++) {
      const rng = new SeededRandom(seed)
      const heads = [rng.nextBool(), rng.nextBool(), rng.nextBool()].filter(Boolean).length
      counts[6 + heads] += 1
    }
    expect(counts[6] / SMALL_SEEDS).toBeGreaterThan(0.1)
    expect(counts[6] / SMALL_SEEDS).toBeLessThan(0.15)
    expect(counts[9] / SMALL_SEEDS).toBeGreaterThan(0.1)
    expect(counts[9] / SMALL_SEEDS).toBeLessThan(0.15)
  })

  it('keeps a fixed sequence per seed and different sequences across seeds', () => {
    const sequence = (seed: number) => {
      const rng = new SeededRandom(seed)
      return Array.from({ length: 8 }, () => rng.next())
    }
    expect(sequence(42)).toEqual(sequence(42))
    expect(sequence(0)).not.toEqual(sequence(1))
    expect(sequence(1401)).not.toEqual(sequence(1402))
  })

  it('stays in [0, 1) with a bounded state over long runs and edge seeds', () => {
    for (const seed of [0, 1, 0xffffffff, Number.MAX_SAFE_INTEGER, -1]) {
      const rng = new SeededRandom(seed)
      for (let i = 0; i < 20_000; i++) {
        const value = rng.next()
        expect(value >= 0 && value < 1).toBe(true)
      }
      // The xorshift128+ state is two 64-bit words; it must not grow.
      const state = (rng as unknown as { state: [bigint, bigint] }).state
      for (const word of state) {
        expect(word >= 0n && word < 1n << 64n).toBe(true)
      }
    }
  })

  it('keeps nextInt and pick inside their ranges', () => {
    const rng = new SeededRandom(7)
    const items = ['a', 'b', 'c']
    for (let i = 0; i < 5_000; i++) {
      const n = rng.nextInt(3, 5)
      expect(n >= 3 && n <= 5).toBe(true)
      expect(items).toContain(rng.pick(items))
    }
  })
})
