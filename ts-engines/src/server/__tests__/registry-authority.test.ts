import { describe, expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { EngineRegistry, registerTypeScriptRuntimeEngines } from '../registry'

interface RegistryRow {
  id: string
  runtime_class: string
}

interface RegistryAuthority {
  engines: RegistryRow[]
}

function canonicalTypeScriptIds(): string[] {
  const path = resolve(import.meta.dir, '../../../../contracts/v1/registries/engines.json')
  const authority = JSON.parse(readFileSync(path, 'utf8')) as RegistryAuthority
  return authority.engines
    .filter((row) => row.runtime_class === 'typescript')
    .map((row) => row.id)
    .sort()
}

describe('TypeScript runtime registry authority', () => {
  test('startup registration exactly matches the canonical TypeScript rows', () => {
    const registry = registerTypeScriptRuntimeEngines(new EngineRegistry())

    expect(registry.list().sort()).toEqual(canonicalTypeScriptIds())
    expect(registry.count()).toBe(6)
    expect(
      registry.listCapabilities().every((capability) => capability.runtime_kind === 'typescript'),
    ).toBe(true)
  })
})
