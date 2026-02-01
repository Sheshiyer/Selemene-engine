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
