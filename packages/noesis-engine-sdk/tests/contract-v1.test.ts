import { describe, expect, test } from 'bun:test'
import capabilityFixture from '../../../contracts/v1/fixtures/engine-capability.json'
import errorFixture from '../../../contracts/v1/fixtures/error.json'
import requestFixture from '../../../contracts/v1/fixtures/engine-request.json'
import resultFixture from '../../../contracts/v1/fixtures/engine-result.json'
import {
  CONTRACT_VERSION,
  type CapabilityAvailability,
  type ConsciousnessPhase,
  type ContractEngineCapability,
  type ContractEngineRequest,
  type ContractEngineResult,
  type ContractError,
  type RuntimeKind,
} from '../src/index.js'

const request: ContractEngineRequest = {
  ...requestFixture,
  contract_version: CONTRACT_VERSION,
  consciousness_level: requestFixture.consciousness_level as ConsciousnessPhase,
}
const result: ContractEngineResult = {
  ...resultFixture,
  contract_version: CONTRACT_VERSION,
  consciousness_level: resultFixture.consciousness_level as ConsciousnessPhase,
  provenance: {
    ...resultFixture.provenance,
    runtime_kind: resultFixture.provenance.runtime_kind as RuntimeKind,
  },
}
const error: ContractError = { ...errorFixture, contract_version: CONTRACT_VERSION }
const capability: ContractEngineCapability = {
  ...capabilityFixture,
  contract_version: CONTRACT_VERSION,
  availability: capabilityFixture.availability as CapabilityAvailability,
  runtime_kind: capabilityFixture.runtime_kind as RuntimeKind,
  required_phase: capabilityFixture.required_phase as ConsciousnessPhase,
}
const singularLegacyResult: ContractEngineResult = {
  contract_version: CONTRACT_VERSION,
  engine_id: 'numerology',
  result: {},
  consciousness_level: 2,
  witness_prompt: 'What is witnessed?',
  calculated_at: '2026-08-26T06:30:00Z',
  processing_time_ms: 1,
}

describe('canonical v1 fixtures', () => {
  test('engine SDK consumes request and result authority', () => {
    expect(CONTRACT_VERSION).toBe('v1')
    expect(request.contract_version).toBe(CONTRACT_VERSION)
    expect(request.image_data?.consent?.scopes).toEqual(['face-image'])
    expect(result.engine_id).toBe('numerology')
    expect(result.provenance?.runtime_kind).toBe('native')
    expect(result.witness_prompts).toHaveLength(1)
    expect(singularLegacyResult.provenance).toBeUndefined()
  })

  test('engine SDK consumes error and capability authority', () => {
    expect(error.error_code).toBe('VALIDATION_ERROR')
    expect(error.contract_version).toBe(CONTRACT_VERSION)
    expect(capability.availability).toBe('available')
    expect(capability.dependencies).toEqual([])
  })
})
