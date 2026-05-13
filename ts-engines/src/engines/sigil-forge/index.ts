/**
 * Sigil Forge Engine - Public API exports
 */

// Wisdom data
export type { SigilMethod, ChargingMethod } from './wisdom'
export {
  SIGIL_METHODS,
  CHARGING_METHODS,
  getMethodById,
  getMethodIds,
  processWordElimination,
} from './wisdom'

// Witness prompts
export { generateWitnessPrompts } from './witness'

// Prompt builder
export type { SigilStyle, BuiltPrompt } from './prompt-builder'
export { buildSigilPrompt, buildSigilEditPrompt } from './prompt-builder'

// Engine
export { SigilForgeEngine } from './engine'
