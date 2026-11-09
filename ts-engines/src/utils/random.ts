/**
 * Seeded pseudo-random number generator
 * Uses xorshift128+ algorithm for reproducible randomness
 */
const MASK_64 = (1n << 64n) - 1n

/**
 * One SplitMix64 step: the next state and its scrambled output. Used to spread
 * a seed across both xorshift words, so nearby small seeds (0, 1, 42, ...)
 * start from unrelated states instead of nearly identical ones.
 */
function splitMix64(state: bigint): [bigint, bigint] {
  const next = (state + 0x9e3779b97f4a7c15n) & MASK_64
  let z = next
  z = ((z ^ (z >> 30n)) * 0xbf58476d1ce4e5b9n) & MASK_64
  z = ((z ^ (z >> 27n)) * 0x94d049bb133111ebn) & MASK_64
  return [next, z ^ (z >> 31n)]
}

export class SeededRandom {
  private state: [bigint, bigint]

  constructor(seed: number) {
    // Initialize state from seed via SplitMix64 (two outputs, never both zero
    // in practice; guarded anyway since an all-zero xorshift state is stuck).
    const [mixed, s0] = splitMix64(BigInt.asUintN(64, BigInt(seed)))
    const [, s1] = splitMix64(mixed)
    this.state = s0 === 0n && s1 === 0n ? [1n, 0n] : [s0, s1]
  }

  /** Generate a random number in [0, 1) */
  next(): number {
    let s1 = this.state[0]
    const s0 = this.state[1]
    this.state[0] = s0
    s1 ^= (s1 << 23n) & MASK_64
    s1 ^= s1 >> 18n
    s1 ^= s0
    s1 ^= s0 >> 5n
    this.state[1] = s1
    const result = (s0 + s1) & MASK_64
    // Top 53 bits divided by 2^53: exactly representable, never reaches 1.
    return Number(result >> 11n) / 2 ** 53
  }

  /** Generate a random integer in [min, max] inclusive */
  nextInt(min: number, max: number): number {
    return Math.floor(this.next() * (max - min + 1)) + min
  }

  /** Generate a random boolean with given probability of true */
  nextBool(probability = 0.5): boolean {
    return this.next() < probability
  }

  /** Shuffle an array in place using Fisher-Yates algorithm */
  shuffle<T>(array: T[]): T[] {
    for (let i = array.length - 1; i > 0; i--) {
      const j = this.nextInt(0, i)
      ;[array[i], array[j]] = [array[j], array[i]]
    }
    return array
  }

  /** Pick a random element from array */
  pick<T>(array: T[]): T {
    return array[this.nextInt(0, array.length - 1)]
  }

  /** Pick n random elements from array without replacement */
  sample<T>(array: T[], n: number): T[] {
    const copy = [...array]
    this.shuffle(copy)
    return copy.slice(0, n)
  }
}

/** Get current timestamp seed if no seed provided (always returns non-negative uint32) */
export function getDefaultSeed(): number {
  return (Date.now() ^ Math.floor(Math.random() * 0x100000000)) >>> 0
}
