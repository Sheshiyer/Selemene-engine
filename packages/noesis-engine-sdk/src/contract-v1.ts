import type {
  ConsciousnessPhase,
  Consent,
  GeneratedAudioRef,
  GeneratedImageRef,
  MediaRef,
  QualitySpec,
  WitnessPrompt,
} from './types.js'

export const CONTRACT_VERSION = 'v1' as const
export type ContractVersion = typeof CONTRACT_VERSION
export type RuntimeKind = 'native' | 'typescript' | 'python' | 'database-conditional' | 'composed'
export type CapabilityAvailability = 'declared' | 'available' | 'degraded' | 'unavailable'

export interface ContractProvenance {
  runtime_kind: RuntimeKind
  implementation_version: string
  cached: boolean
  fallback_used: boolean
  backend_id?: string
  provider_id?: string
}

export interface ContractEngineRequest {
  contract_version: ContractVersion
  consciousness_level: ConsciousnessPhase
  parameters: Record<string, unknown>
  seed?: number
  question?: string
  birth_data?: Record<string, unknown>
  current_time?: string
  location?: Record<string, unknown>
  precision?: 'standard' | 'high' | 'extreme' | 'Standard' | 'High' | 'Extreme'
  options?: Record<string, unknown>
  image_data?: MediaRef
  audio_ref?: MediaRef
  consent?: Consent
  quality?: QualitySpec & { score?: number; diagnostics?: string[] }
}

export interface ContractEngineResult<TResult = Record<string, unknown>> {
  contract_version: ContractVersion
  engine_id: string
  result: TResult
  consciousness_level: ConsciousnessPhase
  witness_prompt?: string
  witness_prompts?: WitnessPrompt[]
  calculated_at: string
  processing_time_ms: number
  generated_image?: GeneratedImageRef
  generated_audio?: GeneratedAudioRef
  provenance?: ContractProvenance
}

export interface ContractError {
  contract_version: ContractVersion
  status: number
  error_code: string
  message: string
  error: string
  details?: Record<string, unknown>
  trace_id: string
}

export interface ContractEngineCapability {
  contract_version: ContractVersion
  engine_id: string
  display_name: string
  availability: CapabilityAvailability
  runtime_kind: RuntimeKind
  dependencies: string[]
  required_phase?: ConsciousnessPhase
  implementation_version?: string
}
