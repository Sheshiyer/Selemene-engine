/**
 * @selemene/engine-sdk — typed client for the 4 focus engines with FROZEN media contracts.
 *
 * P4 SDK task (p5-p4-next-batch.json p4-sdk-client; detailed-task-list.md Phase 4 T-080..T-094).
 * Used by sankalpa (Prong2 surfaces) + scripts/ext-contract-harness.ts style roundtrips.
 *
 * Tags: phase:integration-p1 wave:integration-w2 area:engine-integration
 */

export { EngineClient } from './client'
export type { EngineClientConfig } from './client'
export { BiofieldEngineApi, FaceReadingEngineApi, RaagaEngineApi, SigilForgeEngineApi } from './client'
export { EngineSdkError, ConsentError } from './errors'
export { CONSENT_SCOPES, consentAgeMs, createConsent, requireConsent, resolveConsent } from './consent'
export type { ConsentScope } from './consent'
export type {
  BiofieldAnalyzeInput,
  BiofieldAnalyzeResponse,
  BiofieldCalculateInput,
  BiofieldMetrics,
  BiofieldQualityAssessment,
  ConsciousnessPhase,
  Consent,
  EngineInput,
  EngineOutput,
  ErrorResponse,
  FaceReadingCalculateInput,
  FaceReadingResult,
  GeneratedAudioRef,
  GeneratedImageRef,
  HealthResponse,
  MediaRef,
  PythonSidecarHealthResponse,
  QualitySpec,
  RaagaCalculateInput,
  RaagaResult,
  SigilForgeCalculateInput,
  SigilForgeResult,
  WitnessPrompt,
} from './types'
