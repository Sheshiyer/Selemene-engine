/**
 * Enneagram Engine - Public API exports
 * The Enneagram describes PATTERNS, not fixed identities.
 */

// Core types
export type {
  EnneagramNumber,
  EnneagramType,
  EnneagramConnection,
  CenterOfIntelligence,
  HornevianGroup,
  HarmonicGroup,
} from './wisdom'

// Wisdom data
export {
  getEnneagramType,
  getAllEnneagramTypes,
  getEnneagramConnections,
  getIntegrationConnection,
  getDisintegrationConnection,
  getWingTypes,
  getTypesByCenter,
  getTypesByHornevianGroup,
  getTypesByHarmonicGroup,
  ENNEAGRAM_TYPES,
  ENNEAGRAM_CONNECTIONS,
} from './wisdom'

// Assessment
export type {
  AssessmentQuestion,
  AssessmentAnswer,
  AssessmentResult,
  TypeScore,
} from './assessment'
export {
  getAssessmentQuestions,
  getQuestionsByType,
  calculateAssessment,
  calculateFromAnswerArray,
} from './assessment'

// Witness prompts
export {
  generateTypeWitnessPrompts,
  generateAssessmentWitnessPrompts,
  generateMovementPrompts,
} from './witness'

// Engine
export { EnneagramEngine } from './engine'
