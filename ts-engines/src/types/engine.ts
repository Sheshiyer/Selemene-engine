/**
 * Core types for TypeScript consciousness engines
 * Mirrors the Rust ConsciousnessEngine trait from noesis-core
 */

/** Phase of consciousness required to access an engine */
export type ConsciousnessPhase = 0 | 1 | 2 | 3 | 4 | 5

/** Witness prompt - non-prescriptive inquiry for self-reflection */
export interface WitnessPrompt {
  /** The inquiry question */
  prompt: string
  /** Context for the prompt */
  context?: string
  /** Related themes to explore */
  themes?: string[]
}

/** Input to any consciousness engine */
export interface EngineInput {
  /** User's consciousness phase (0-5) */
  consciousness_level: ConsciousnessPhase
  /** Arbitrary parameters specific to each engine */
  parameters: Record<string, unknown>
  /** Optional seed for reproducible results */
  seed?: number
  /** User's question or intention (optional) */
  question?: string

  // --- P1 W1 / P2 media extensions per FROZEN (T-002/T-005/T-031) + bootstrap-packet + ext-contract-harness.ts + P1W1-CONTRACTS-FROZEN.md + detailed-task-list T-031
  // Cites: resources-and-assets.md (raaga production), gaps-and-improvements.md (no prior media), goal-understanding.md (two-prong local-first consent), EXECUTION-STATUS, P1W2-HANDOFF.md, .worktrees/T-024-codex/scripts/ext-contract-harness.ts, p1-w1-worker-bootstrap-packet.md
  // tags: phase:integration-p1 wave:integration-w2 area:engine-integration engine-raaga
  image_data?: { b64?: string; reference?: string; mime_type?: string; consent?: Consent }
  audio_ref?: { reference?: string; consent?: Consent }
  consent?: Consent
  quality?: QualitySpec
}

/** Consent for media (FROZEN match) */
export interface Consent {
  granted: boolean
  scopes: string[]
  timestamp: string
  token?: string
}

/** Quality (FROZEN) */
export interface QualitySpec {
  sufficient?: boolean
  min_coherence?: number
  scores?: Record<string, number>
}

/** Output from any consciousness engine */
export interface EngineOutput {
  /** Engine identifier */
  engine_id: string
  /** Calculation results (engine-specific) */
  result: Record<string, unknown>
  /** Witness prompts for self-inquiry */
  witness_prompts: WitnessPrompt[]
  /** Calculation timestamp */
  calculated_at: string
  /** Processing time in milliseconds */
  processing_time_ms: number

  // --- P1 W1 media + generated_audio for raaga per FROZEN T-005/T-031
  // generated_audio: {strudel_ratios, clip_url etc} ; result still carries legacy
  generated_image?: { b64_json?: string; url?: string; metadata?: Record<string, unknown> }
  generated_audio?: {
    clip_url?: string | null
    strudel_ratios?: number[]
    root_hz?: number
    metadata?: Record<string, unknown>
  }
}

/** Metadata about an engine */
export interface EngineMetadata {
  /** Unique engine identifier */
  id: string
  /** Human-readable name */
  name: string
  /** Engine description */
  description: string
  /** Version string */
  version: string
  /** Minimum consciousness phase required */
  required_phase: ConsciousnessPhase
  /** Input parameters schema */
  input_schema: Record<string, ParameterSchema>
}

/** Schema for engine input parameters */
export interface ParameterSchema {
  type: 'string' | 'number' | 'boolean' | 'array' | 'object'
  required: boolean
  description: string
  default?: unknown
  enum?: unknown[]
}

/** Base interface for all consciousness engines */
export interface ConsciousnessEngine {
  /** Engine metadata */
  metadata(): EngineMetadata
  /** Calculate and return results */
  calculate(input: EngineInput): Promise<EngineOutput>
  /** Lightweight health probe (optional; defaults to healthy when omitted) */
  selfCheck?(): Promise<EngineHealthStatus>
}

/** Error response format */
export interface ErrorResponse {
  error: string
  error_code: string
  details?: Record<string, unknown>
}

/** Health check response */
export interface HealthResponse {
  status: 'healthy' | 'degraded' | 'unhealthy'
  engines: string[]
  uptime_ms: number
  version: string
}

/** Per-engine health status */
export interface EngineHealthStatus {
  engine_id: string
  healthy: boolean
  detail: string
  latency_ms: number
}

/** Liveness endpoint response */
export interface LivenessResponse {
  status: 'alive'
  uptime_ms: number
  version: string
}

/** Readiness endpoint response */
export interface ReadinessResponse {
  status: 'ready' | 'degraded'
  engines: EngineHealthStatus[]
  failed_engines: string[]
}

/** Per-engine health endpoint response */
export interface EnginesHealthResponse {
  status: 'healthy' | 'degraded'
  engines: EngineHealthStatus[]
}
