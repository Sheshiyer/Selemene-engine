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
export type CapabilityReasonCode =
  | 'NOT_OBSERVED'
  | 'REGISTERED'
  | 'CAPABILITY_AVAILABLE'
  | 'CAPABILITY_DEGRADED'
  | 'CAPABILITY_UNAVAILABLE'
  | 'REQUIRED_DEPENDENCY_UNAVAILABLE'
  | 'OPTIONAL_DEPENDENCY_UNAVAILABLE'
  | 'DEPENDENCY_UNAVAILABLE'
  | 'DATABASE_UNCONFIGURED'
  | 'MODULE_UNAVAILABLE'
  | 'TIMEOUT'
  | 'MALFORMED_OBSERVATION'
  | 'OPERATION_UNSUPPORTED'
export type DependencyKind = 'rust' | 'typescript' | 'python' | 'database' | 'network' | 'filesystem'
export type DependencyRequirement = 'required' | 'optional'
export type OperationSupport = 'supported' | 'unsupported'

export interface DependencyObservation {
  dependency_id: string
  dependency_kind: DependencyKind
  requirement: DependencyRequirement
  availability: CapabilityAvailability
  reason_code: CapabilityReasonCode
}

export interface CapabilityOperations {
  calculate: OperationSupport
  validate: OperationSupport
  witness_eligible: boolean
}

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
  reason_code?: CapabilityReasonCode
  dependency_observations?: DependencyObservation[]
  operations?: CapabilityOperations
}

export interface ContractEngineCapabilityList {
  contract_version: ContractVersion
  capabilities: ContractEngineCapability[]
  count: 19
  public_mirror_count: 17
}

export type WorkflowExecutionStatus = 'complete' | 'partial' | 'failed'
export type WorkflowSynthesisStatus = 'available' | 'failed' | 'unsupported'
export type WorkflowErrorCode =
  | 'ENGINE_NOT_FOUND'
  | 'DEPENDENCY_UNAVAILABLE'
  | 'ENGINE_TIMEOUT'
  | 'UPSTREAM_INVALID_RESPONSE'
  | 'OPERATION_UNSUPPORTED'

export interface WorkflowEngineOutput {
  result: Record<string, unknown>
  provenance?: Record<string, unknown>
}

export interface WorkflowEngineFailure {
  engine_id: string
  error_code: WorkflowErrorCode
  message: string
}

export interface WorkflowOutcome {
  contract_version: ContractVersion
  workflow_id: string
  requested_engine_ids: string[]
  engine_outputs: Record<string, WorkflowEngineOutput>
  engine_failures: WorkflowEngineFailure[]
  execution_status: WorkflowExecutionStatus
  synthesis_status: WorkflowSynthesisStatus
  synthesis?: { text: string }
  engine_results?: Record<string, WorkflowEngineOutput>
}

type UnknownRecord = Record<string, unknown>
const CAPABILITY_KEYS = [
  'contract_version',
  'engine_id',
  'display_name',
  'availability',
  'runtime_kind',
  'dependencies',
  'required_phase',
  'implementation_version',
  'reason_code',
  'dependency_observations',
  'operations',
] as const
const OBSERVATION_KEYS = [
  'dependency_id',
  'dependency_kind',
  'requirement',
  'availability',
  'reason_code',
] as const
const OPERATION_KEYS = ['calculate', 'validate', 'witness_eligible'] as const
const WORKFLOW_KEYS = [
  'contract_version',
  'workflow_id',
  'requested_engine_ids',
  'engine_outputs',
  'engine_failures',
  'execution_status',
  'synthesis_status',
  'synthesis',
  'engine_results',
] as const
const OUTPUT_KEYS = ['result', 'provenance'] as const
const FAILURE_KEYS = ['engine_id', 'error_code', 'message'] as const
const availabilityValues = new Set<CapabilityAvailability>([
  'declared',
  'available',
  'degraded',
  'unavailable',
])
const runtimeValues = new Set<RuntimeKind>([
  'native',
  'typescript',
  'python',
  'database-conditional',
  'composed',
])
const reasonValues = new Set<CapabilityReasonCode>([
  'NOT_OBSERVED',
  'REGISTERED',
  'CAPABILITY_AVAILABLE',
  'CAPABILITY_DEGRADED',
  'CAPABILITY_UNAVAILABLE',
  'REQUIRED_DEPENDENCY_UNAVAILABLE',
  'OPTIONAL_DEPENDENCY_UNAVAILABLE',
  'DEPENDENCY_UNAVAILABLE',
  'DATABASE_UNCONFIGURED',
  'MODULE_UNAVAILABLE',
  'TIMEOUT',
  'MALFORMED_OBSERVATION',
  'OPERATION_UNSUPPORTED',
])
const dependencyKinds = new Set<DependencyKind>([
  'rust',
  'typescript',
  'python',
  'database',
  'network',
  'filesystem',
])
const operationValues = new Set<OperationSupport>(['supported', 'unsupported'])
const workflowErrorValues = new Set<WorkflowErrorCode>([
  'ENGINE_NOT_FOUND',
  'DEPENDENCY_UNAVAILABLE',
  'ENGINE_TIMEOUT',
  'UPSTREAM_INVALID_RESPONSE',
  'OPERATION_UNSUPPORTED',
])
const engineIdPattern = /^[a-z0-9]+(?:-[a-z0-9]+)*$/

function fail(path: string, message: string): never {
  throw new TypeError(`contract v1 ${path}: ${message}`)
}

function record(value: unknown, path: string): UnknownRecord {
  if (value === null || typeof value !== 'object' || Array.isArray(value)) fail(path, 'must be an object')
  return value as UnknownRecord
}

function exactKeys(value: UnknownRecord, allowed: readonly string[], path: string, required: readonly string[] = []): void {
  const allowedSet = new Set(allowed)
  for (const key of Object.keys(value)) if (!allowedSet.has(key)) fail(`${path}.${key}`, 'unknown field')
  for (const key of required) if (!(key in value)) fail(path, `missing required field ${key}`)
}

function stringValue(value: unknown, path: string, max: number, pattern?: RegExp): string {
  if (typeof value !== 'string' || value.length < 1 || value.length > max) fail(path, `must be a non-empty string of at most ${max} characters`)
  if (pattern && !pattern.test(value)) fail(path, 'has an invalid format')
  return value
}

function enumValue<T extends string>(value: unknown, values: Set<T>, path: string): T {
  if (typeof value !== 'string' || !values.has(value as T)) fail(path, 'contains an unsupported value')
  return value as T
}

function stringArray(value: unknown, path: string, max: number, unique: boolean): string[] {
  if (!Array.isArray(value) || value.length > max) fail(path, `must be an array of at most ${max} strings`)
  const values = value.map((item, index) => stringValue(item, `${path}[${index}]`, 80, /^[a-z0-9]+(?::[a-z0-9-]+)*$/))
  if (unique && new Set(values).size !== values.length) fail(path, 'must not contain duplicates')
  return values
}

function decodeDependencyObservation(value: unknown, path: string): DependencyObservation {
  const item = record(value, path)
  exactKeys(item, OBSERVATION_KEYS, path, OBSERVATION_KEYS)
  return {
    dependency_id: stringValue(item.dependency_id, `${path}.dependency_id`, 80, /^[a-z0-9]+(?::[a-z0-9-]+)*$/),
    dependency_kind: enumValue(item.dependency_kind, dependencyKinds, `${path}.dependency_kind`),
    requirement: enumValue(item.requirement, new Set<DependencyRequirement>(['required', 'optional']), `${path}.requirement`),
    availability: enumValue(item.availability, availabilityValues, `${path}.availability`),
    reason_code: enumValue(item.reason_code, reasonValues, `${path}.reason_code`),
  }
}

function decodeOperations(value: unknown, path: string): CapabilityOperations {
  const item = record(value, path)
  exactKeys(item, OPERATION_KEYS, path, OPERATION_KEYS)
  return {
    calculate: enumValue(item.calculate, operationValues, `${path}.calculate`),
    validate: enumValue(item.validate, operationValues, `${path}.validate`),
    witness_eligible: typeof item.witness_eligible === 'boolean' ? item.witness_eligible : fail(`${path}.witness_eligible`, 'must be a boolean'),
  }
}

export function decodeEngineCapability(value: unknown, path = 'capability'): ContractEngineCapability {
  const item = record(value, path)
  exactKeys(item, CAPABILITY_KEYS, path, [
    'contract_version',
    'engine_id',
    'display_name',
    'availability',
    'runtime_kind',
    'dependencies',
  ])
  if (item.contract_version !== CONTRACT_VERSION) fail(`${path}.contract_version`, 'must be v1')
  const capability: ContractEngineCapability = {
    contract_version: CONTRACT_VERSION,
    engine_id: stringValue(item.engine_id, `${path}.engine_id`, 64, engineIdPattern),
    display_name: stringValue(item.display_name, `${path}.display_name`, 120),
    availability: enumValue(item.availability, availabilityValues, `${path}.availability`),
    runtime_kind: enumValue(item.runtime_kind, runtimeValues, `${path}.runtime_kind`),
    dependencies: stringArray(item.dependencies, `${path}.dependencies`, 16, true),
  }
  if ('required_phase' in item) {
    if (!Number.isInteger(item.required_phase) || (item.required_phase as number) < 0 || (item.required_phase as number) > 5) fail(`${path}.required_phase`, 'must be an integer from 0 through 5')
    capability.required_phase = item.required_phase as ConsciousnessPhase
  }
  if ('implementation_version' in item) capability.implementation_version = stringValue(item.implementation_version, `${path}.implementation_version`, 64)
  if ('reason_code' in item) capability.reason_code = enumValue(item.reason_code, reasonValues, `${path}.reason_code`)
  if ('dependency_observations' in item) {
    if (!Array.isArray(item.dependency_observations) || item.dependency_observations.length > 16) fail(`${path}.dependency_observations`, 'must contain at most 16 observations')
    capability.dependency_observations = item.dependency_observations.map((observation, index) => decodeDependencyObservation(observation, `${path}.dependency_observations[${index}]`))
  }
  if ('operations' in item) capability.operations = decodeOperations(item.operations, `${path}.operations`)
  return capability
}

export function decodeEngineCapabilityList(value: unknown): ContractEngineCapabilityList {
  const item = record(value, 'capability_list')
  exactKeys(item, ['contract_version', 'capabilities', 'count', 'public_mirror_count'], 'capability_list', ['contract_version', 'capabilities', 'count', 'public_mirror_count'])
  if (item.contract_version !== CONTRACT_VERSION) fail('capability_list.contract_version', 'must be v1')
  if (!Array.isArray(item.capabilities) || item.capabilities.length !== 19) fail('capability_list.capabilities', 'must contain exactly 19 rows')
  if (item.count !== 19) fail('capability_list.count', 'must be 19')
  if (item.public_mirror_count !== 17) fail('capability_list.public_mirror_count', 'must be 17')
  const capabilities = item.capabilities.map((capability, index) => decodeEngineCapability(capability, `capability_list.capabilities[${index}]`))
  const ids = capabilities.map((capability) => capability.engine_id)
  if (new Set(ids).size !== ids.length) fail('capability_list.capabilities', 'engine IDs must be unique')
  if (capabilities.some((capability) => !capability.reason_code || !capability.dependency_observations || !capability.operations)) fail('capability_list.capabilities', 'canonical metadata is required for every row')
  return { contract_version: CONTRACT_VERSION, capabilities, count: 19, public_mirror_count: 17 }
}

export function decodeWorkflowOutcome(value: unknown): WorkflowOutcome {
  const item = record(value, 'workflow')
  exactKeys(item, WORKFLOW_KEYS, 'workflow', [
    'contract_version',
    'workflow_id',
    'requested_engine_ids',
    'engine_outputs',
    'engine_failures',
    'execution_status',
    'synthesis_status',
  ])
  if (item.contract_version !== CONTRACT_VERSION) fail('workflow.contract_version', 'must be v1')
  const workflowId = stringValue(item.workflow_id, 'workflow.workflow_id', 80, engineIdPattern)
  if (!Array.isArray(item.requested_engine_ids) || item.requested_engine_ids.length < 1 || item.requested_engine_ids.length > 19) fail('workflow.requested_engine_ids', 'must contain 1 through 19 IDs')
  const requested = item.requested_engine_ids.map((engineId, index) => stringValue(engineId, `workflow.requested_engine_ids[${index}]`, 64, engineIdPattern))
  if (new Set(requested).size !== requested.length) fail('workflow.requested_engine_ids', 'must not contain duplicates')
  const outputsRecord = record(item.engine_outputs, 'workflow.engine_outputs')
  if (Object.keys(outputsRecord).length > 19) fail('workflow.engine_outputs', 'must contain at most 19 outputs')
  const engine_outputs: Record<string, WorkflowEngineOutput> = {}
  for (const [engineId, rawOutput] of Object.entries(outputsRecord)) {
    if (!engineIdPattern.test(engineId)) fail(`workflow.engine_outputs.${engineId}`, 'has an invalid engine ID')
    const output = record(rawOutput, `workflow.engine_outputs.${engineId}`)
    exactKeys(output, OUTPUT_KEYS, `workflow.engine_outputs.${engineId}`, ['result'])
    engine_outputs[engineId] = { result: record(output.result, `workflow.engine_outputs.${engineId}.result`) }
    if ('provenance' in output) engine_outputs[engineId].provenance = record(output.provenance, `workflow.engine_outputs.${engineId}.provenance`)
  }
  if (!Array.isArray(item.engine_failures) || item.engine_failures.length > 19) fail('workflow.engine_failures', 'must contain at most 19 failures')
  const engine_failures = item.engine_failures.map((rawFailure, index) => {
    const failure = record(rawFailure, `workflow.engine_failures[${index}]`)
    exactKeys(failure, FAILURE_KEYS, `workflow.engine_failures[${index}]`, FAILURE_KEYS)
    const message = stringValue(failure.message, `workflow.engine_failures[${index}].message`, 200)
    if (/https?:\/\/|token|secret|api[_-]?key/i.test(message)) fail(`workflow.engine_failures[${index}].message`, 'contains a sensitive value')
    return {
      engine_id: stringValue(failure.engine_id, `workflow.engine_failures[${index}].engine_id`, 64, engineIdPattern),
      error_code: enumValue(failure.error_code, workflowErrorValues, `workflow.engine_failures[${index}].error_code`),
      message,
    }
  })
  const failedIds = engine_failures.map((failure) => failure.engine_id)
  if (new Set(failedIds).size !== failedIds.length) fail('workflow.engine_failures', 'engine IDs must be unique')
  const outputIds = new Set(Object.keys(engine_outputs))
  const failureIds = new Set(failedIds)
  for (const engineId of outputIds) if (failureIds.has(engineId)) fail('workflow', 'an engine cannot both succeed and fail')
  const conserved = new Set([...outputIds, ...failureIds])
  if (conserved.size !== requested.length || requested.some((engineId) => !conserved.has(engineId))) fail('workflow', 'outputs and failures must conserve requested engine IDs')
  const execution_status = enumValue(item.execution_status, new Set<WorkflowExecutionStatus>(['complete', 'partial', 'failed']), 'workflow.execution_status')
  if (execution_status === 'complete' && (failureIds.size > 0 || outputIds.size !== requested.length)) fail('workflow.execution_status', 'complete requires only successful outputs')
  if (execution_status === 'partial' && (failureIds.size === 0 || outputIds.size === 0)) fail('workflow.execution_status', 'partial requires successes and failures')
  if (execution_status === 'failed' && (outputIds.size > 0 || failureIds.size === 0)) fail('workflow.execution_status', 'failed requires only failures')
  const synthesis_status = enumValue(item.synthesis_status, new Set<WorkflowSynthesisStatus>(['available', 'failed', 'unsupported']), 'workflow.synthesis_status')
  let synthesis: { text: string } | undefined
  if ('synthesis' in item) {
    const rawSynthesis = record(item.synthesis, 'workflow.synthesis')
    exactKeys(rawSynthesis, ['text'], 'workflow.synthesis', ['text'])
    synthesis = { text: stringValue(rawSynthesis.text, 'workflow.synthesis.text', 4000) }
  }
  if (synthesis_status === 'available' && !synthesis) fail('workflow.synthesis_status', 'available requires synthesis text')
  if ((synthesis_status === 'failed' || synthesis_status === 'unsupported') && synthesis) fail('workflow.synthesis_status', 'failed or unsupported cannot contain synthesis text')
  let engine_results: Record<string, WorkflowEngineOutput> | undefined
  if ('engine_results' in item) {
    const alias = decodeWorkflowOutputMap(item.engine_results, 'workflow.engine_results')
    if (!deepEqual(alias, engine_outputs)) fail('workflow.engine_results', 'must equal engine_outputs')
    engine_results = alias
  }
  return { contract_version: CONTRACT_VERSION, workflow_id: workflowId, requested_engine_ids: requested, engine_outputs, engine_failures, execution_status, synthesis_status, ...(synthesis ? { synthesis } : {}), ...(engine_results ? { engine_results } : {}) }
}

function decodeWorkflowOutputMap(value: unknown, path: string): Record<string, WorkflowEngineOutput> {
  const outputsRecord = record(value, path)
  if (Object.keys(outputsRecord).length > 19) fail(path, 'must contain at most 19 outputs')
  const outputs: Record<string, WorkflowEngineOutput> = {}
  for (const [engineId, rawOutput] of Object.entries(outputsRecord)) {
    const output = record(rawOutput, `${path}.${engineId}`)
    exactKeys(output, OUTPUT_KEYS, `${path}.${engineId}`, ['result'])
    outputs[engineId] = { result: record(output.result, `${path}.${engineId}.result`) }
    if ('provenance' in output) outputs[engineId].provenance = record(output.provenance, `${path}.${engineId}.provenance`)
  }
  return outputs
}

function stable(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(stable)
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.entries(value as UnknownRecord).sort(([a], [b]) => a.localeCompare(b)).map(([key, nested]) => [key, stable(nested)]))
  }
  return value
}

function deepEqual(left: unknown, right: unknown): boolean {
  return JSON.stringify(stable(left)) === JSON.stringify(stable(right))
}

export function isEngineCapabilityList(value: unknown): value is ContractEngineCapabilityList {
  try {
    decodeEngineCapabilityList(value)
    return true
  } catch {
    return false
  }
}

export function isWorkflowOutcome(value: unknown): value is WorkflowOutcome {
  try {
    decodeWorkflowOutcome(value)
    return true
  } catch {
    return false
  }
}
