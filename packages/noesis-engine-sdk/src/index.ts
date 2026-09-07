/**
 * @selemene/engine-sdk — typed client for the 4 focus engines with FROZEN media contracts.
 *
 * P4 SDK task (p5-p4-next-batch.json p4-sdk-client; detailed-task-list.md Phase 4 T-080..T-094).
 * Used by sankalpa (Prong2 surfaces) + scripts/ext-contract-harness.ts style roundtrips.
 *
 * Tags: phase:integration-p1 wave:integration-w2 area:engine-integration
 */

export { EngineClient } from './client.js'
export type { EngineClientConfig } from './client.js'
export { BiofieldEngineApi, FaceReadingEngineApi, RaagaEngineApi, SigilForgeEngineApi } from './client.js'
export { EngineSdkError, ConsentError } from './errors.js'
export { CONSENT_SCOPES, consentAgeMs, createConsent, requireConsent, resolveConsent } from './consent.js'
export type { ConsentScope } from './consent.js'
export {
  CONTRACT_VERSION,
  decodeEngineCapability,
  decodeEngineCapabilityList,
  decodeWorkflowOutcome,
  isEngineCapabilityList,
  isWorkflowOutcome,
} from './contract-v1.js'
export type {
  CapabilityOperations,
  CapabilityAvailability,
  CapabilityReasonCode,
  ContractEngineCapability,
  ContractEngineCapabilityList,
  ContractEngineRequest,
  ContractEngineResult,
  ContractError,
  ContractProvenance,
  ContractVersion,
  DependencyKind,
  DependencyObservation,
  DependencyRequirement,
  OperationSupport,
  RuntimeKind,
  WorkflowEngineFailure,
  WorkflowEngineOutput,
  WorkflowErrorCode,
  WorkflowExecutionStatus,
  WorkflowOutcome,
  WorkflowSynthesisStatus,
} from './contract-v1.js'
export type {
  BiofieldAnalyzeInput,
  BiofieldAnalyzeResponse,
  BiofieldCalculateInput,
  BiofieldCapture,
  BiofieldCaptureArtifact,
  BiofieldCaptureImage,
  BiofieldCreateCaptureInput,
  BiofieldCreateSessionInput,
  BiofieldMetrics,
  BiofieldQualityAssessment,
  BiofieldSession,
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
} from './types.js'
