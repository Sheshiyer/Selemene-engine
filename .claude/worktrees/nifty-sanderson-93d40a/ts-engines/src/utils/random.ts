/**
 * Seeded pseudo-random number generator
 * Uses xorshift128+ algorithm for reproducible randomness
 */
export class SeededRandom {
  private state: [bigint, bigint]

  constructor(seed: number) {
    // Initialize state from seed
    const s = BigInt(seed)
    this.state = [s ^ 0x5deece66dn, (s >> 32n) ^ 0xbb67ae85n]
  }

  /** Generate a random number in [0, 1) */
  next(): number {
    let s1 = this.state[0]
    const s0 = this.state[1]
    this.state[0] = s0
    s1 ^= s1 << 23n
    s1 ^= s1 >> 18n
    s1 ^= s0
    s1 ^= s0 >> 5n
    this.state[1] = s1
    const result = (s0 + s1) & 0xffffffffffffffffn
    return Number(result) / Number(0xffffffffffffffffn)
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

/** Get current timestamp seed if no seed provided */
export function getDefaultSeed(): number {
  return Date.now() ^ (Math.random() * 0x100000000)
}
