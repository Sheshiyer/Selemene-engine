import { TarotEngine } from './src/engines/tarot/index.ts'
import { IChingEngine } from './src/engines/i-ching/index.ts'
import { EnneagramEngine } from './src/engines/enneagram/index.ts'
import { SacredGeometryEngine } from './src/engines/sacred-geometry/index.ts'
import { SigilForgeEngine } from './src/engines/sigil-forge/index.ts'
import type { EngineInput } from './src/types/engine.ts'

const engines: Record<string, any> = {
  tarot: new TarotEngine(),
  'i-ching': new IChingEngine(),
  enneagram: new EnneagramEngine(),
  'sacred-geometry': new SacredGeometryEngine(),
  'sigil-forge': new SigilForgeEngine(),
}

const referenceUsers = [
  { id: 'user_nyc_1990', seed: 42 },
  { id: 'user_london_1985', seed: 1337 },
  { id: 'user_tokyo_1995', seed: 9999 },
]

const engineInputs: Record<string, (seed: number) => EngineInput> = {
  tarot: (seed) => ({
    consciousness_level: 1,
    parameters: { spread: 'three_card' },
    seed,
  }),
  'i-ching': (seed) => ({
    consciousness_level: 1,
    parameters: { method: 'three_coins' },
    seed,
  }),
  enneagram: (seed) => ({
    consciousness_level: 1,
    parameters: { type: ((seed % 9) + 1) },
    seed,
  }),
  'sacred-geometry': (seed) => ({
    consciousness_level: 2,
    parameters: {},
    seed,
  }),
  'sigil-forge': (seed) => ({
    consciousness_level: 2,
    parameters: { intention: 'I am aligned with my highest purpose' },
    seed,
  }),
}

const results: Record<string, Record<string, unknown>> = {}

for (const [engineId, engine] of Object.entries(engines)) {
  results[engineId] = {}
  for (const user of referenceUsers) {
    const input = engineInputs[engineId](user.seed)
    const output = await engine.calculate(input)
    output.calculated_at = '2024-01-15T14:30:00.000Z'
    results[engineId][user.id] = output
  }
}

console.log(JSON.stringify(results, null, 2))
