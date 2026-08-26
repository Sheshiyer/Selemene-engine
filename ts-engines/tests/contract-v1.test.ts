import { describe, expect, test } from 'bun:test'
import capabilityFixture from '../../contracts/v1/fixtures/engine-capability.json'
import errorFixture from '../../contracts/v1/fixtures/error.json'
import requestFixture from '../../contracts/v1/fixtures/engine-request.json'
import resultFixture from '../../contracts/v1/fixtures/engine-result.json'
import {
  CONTRACT_VERSION,
  type CapabilityAvailability,
  type ConsciousnessPhase,
  type ContractEngineCapability,
  type ContractEngineRequest,
  type ContractEngineResult,
  type ContractError,
  type RuntimeKind,
} from '../src/types'

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

describe('contract authority v1', () => {
  test('TypeScript engine boundary consumes request and result fixtures', () => {
    expect(request.contract_version).toBe(CONTRACT_VERSION)
    expect(request.consciousness_level).toBe(2)
    expect(result.contract_version).toBe(CONTRACT_VERSION)
    expect(result.provenance.cached).toBe(false)
  })

  test('TypeScript engine boundary consumes error and capability fixtures', () => {
    expect(error.status).toBe(422)
    expect(capability.runtime_kind).toBe('native')
    expect(capability.availability).toBe('available')
  })
})
