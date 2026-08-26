import { describe, expect, test } from 'bun:test'
import {
  CONTRACT_VERSION,
  type ContractEngineCapability,
  type ContractEngineRequest,
  type ContractEngineResult,
  type ContractError,
} from '../src/index.js'

async function fixture<T>(name: string): Promise<T> {
  const path = new URL(`../../../contracts/v1/fixtures/${name}`, import.meta.url)
  return (await Bun.file(path).json()) as T
}

describe('canonical v1 fixtures', () => {
  test('engine SDK consumes request and result authority', async () => {
    const request = await fixture<ContractEngineRequest>('engine-request.json')
    const result = await fixture<ContractEngineResult>('engine-result.json')

    expect(CONTRACT_VERSION).toBe('v1')
    expect(request.contract_version).toBe(CONTRACT_VERSION)
    expect(request.image_data?.consent?.scopes).toEqual(['face-image'])
    expect(result.engine_id).toBe('numerology')
    expect(result.provenance.runtime_kind).toBe('native')
    expect(result.witness_prompts).toHaveLength(1)
  })

  test('engine SDK consumes error and capability authority', async () => {
    const error = await fixture<ContractError>('error.json')
    const capability = await fixture<ContractEngineCapability>('engine-capability.json')

    expect(error.error_code).toBe('VALIDATION_ERROR')
    expect(error.contract_version).toBe(CONTRACT_VERSION)
    expect(capability.availability).toBe('available')
    expect(capability.dependencies).toEqual([])
  })
})
