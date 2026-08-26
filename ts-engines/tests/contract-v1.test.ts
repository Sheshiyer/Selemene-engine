import { describe, expect, test } from 'bun:test'
import {
  CONTRACT_VERSION,
  type ContractEngineCapability,
  type ContractEngineRequest,
  type ContractEngineResult,
  type ContractError,
} from '../src/types'

async function fixture<T>(name: string): Promise<T> {
  return (await Bun.file(new URL(`../../contracts/v1/fixtures/${name}`, import.meta.url)).json()) as T
}

describe('contract authority v1', () => {
  test('TypeScript engine boundary consumes request and result fixtures', async () => {
    const request = await fixture<ContractEngineRequest>('engine-request.json')
    const result = await fixture<ContractEngineResult>('engine-result.json')

    expect(request.contract_version).toBe(CONTRACT_VERSION)
    expect(request.consciousness_level).toBe(2)
    expect(result.contract_version).toBe(CONTRACT_VERSION)
    expect(result.provenance.cached).toBe(false)
  })

  test('TypeScript engine boundary consumes error and capability fixtures', async () => {
    const error = await fixture<ContractError>('error.json')
    const capability = await fixture<ContractEngineCapability>('engine-capability.json')

    expect(error.status).toBe(422)
    expect(capability.runtime_kind).toBe('native')
    expect(capability.availability).toBe('available')
  })
})
