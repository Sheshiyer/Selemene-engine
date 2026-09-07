import { EnneagramEngine } from '../engines/enneagram'
import { IChingEngine } from '../engines/i-ching'
import { RaagaEngine } from '../engines/raaga'
import { SacredGeometryEngine } from '../engines/sacred-geometry'
import { SigilForgeEngine } from '../engines/sigil-forge'
import { TarotEngine } from '../engines/tarot'
import type {
  CapabilityAvailability,
  CapabilityOperations,
  CapabilityReasonCode,
  ConsciousnessEngine,
  ContractEngineCapability,
  DependencyObservation,
  EngineMetadata,
} from '../types'

export interface CapabilityObservation {
  availability: CapabilityAvailability
  reason_code: CapabilityReasonCode
  dependency_observations: DependencyObservation[]
  operations: CapabilityOperations
}

const TYPESCRIPT_ENGINE_ORDER = [
  'enneagram',
  'i-ching',
  'raaga',
  'sacred-geometry',
  'sigil-forge',
  'tarot',
] as const

const WITNESS_ELIGIBLE = new Set<string>([
  'enneagram',
  'i-ching',
  'sacred-geometry',
  'sigil-forge',
  'tarot',
])

/**
 * Registry of all TypeScript consciousness engines
 * Engines register themselves here on startup
 */
export class EngineRegistry {
  private engines: Map<string, ConsciousnessEngine> = new Map()

  /** Register an engine */
  register(engine: ConsciousnessEngine): void {
    const meta = engine.metadata()
    this.engines.set(meta.id, engine)
    console.log(`[Registry] Registered engine: ${meta.id} (${meta.name})`)
  }

  /** Get an engine by ID */
  get(id: string): ConsciousnessEngine | undefined {
    return this.engines.get(id)
  }

  /** Check if an engine exists */
  has(id: string): boolean {
    return this.engines.has(id)
  }

  /** Get all engine IDs */
  list(): string[] {
    return Array.from(this.engines.keys())
  }

  /** Get all engine metadata */
  listMetadata(): EngineMetadata[] {
    return Array.from(this.engines.values()).map((e) => e.metadata())
  }

  /** Get all engine capabilities in the canonical v1 shape */
  listCapabilities(
    availabilityByEngineId: Map<string, CapabilityAvailability> = new Map(),
  ): ContractEngineCapability[] {
    return Array.from(this.engines.values()).map((engine) => {
      const meta = engine.metadata()
      return {
        contract_version: 'v1',
        engine_id: meta.id,
        display_name: meta.name,
        availability: availabilityByEngineId.get(meta.id) ?? 'declared',
        runtime_kind: 'typescript',
        dependencies: [],
        required_phase: meta.required_phase,
        implementation_version: meta.version,
      }
    })
  }

  /** Project explicit, bounded self-check observations into canonical rows. */
  listCapabilityObservations(
    observations: Map<string, CapabilityObservation> = new Map(),
  ): ContractEngineCapability[] {
    return TYPESCRIPT_ENGINE_ORDER.flatMap((engineId) => {
      const engine = this.engines.get(engineId)
      if (!engine) return []
      const meta = engine.metadata()
      const observation = observations.get(engineId) ?? {
        availability: 'declared' as const,
        reason_code: 'NOT_OBSERVED' as const,
        dependency_observations: [],
        operations: {
          calculate: 'supported' as const,
          validate: 'unsupported' as const,
          witness_eligible: WITNESS_ELIGIBLE.has(engineId),
        },
      }
      return [
        {
          contract_version: 'v1' as const,
          engine_id: meta.id,
          display_name: meta.name,
          availability: observation.availability,
          runtime_kind: 'typescript' as const,
          dependencies: ['typescript:bridge'],
          required_phase: meta.required_phase,
          implementation_version: meta.version,
          reason_code: observation.reason_code,
          dependency_observations: observation.dependency_observations,
          operations: observation.operations,
        },
      ]
    })
  }

  /** Get all engine instances */
  all(): ConsciousnessEngine[] {
    return Array.from(this.engines.values())
  }

  /** Get engine count */
  count(): number {
    return this.engines.size
  }
}

/** Register the complete TypeScript runtime set used by the server entrypoint. */
export function registerTypeScriptRuntimeEngines(engineRegistry: EngineRegistry): EngineRegistry {
  engineRegistry.register(new TarotEngine())
  engineRegistry.register(new IChingEngine())
  engineRegistry.register(new EnneagramEngine())
  engineRegistry.register(new SacredGeometryEngine())
  engineRegistry.register(new SigilForgeEngine())
  engineRegistry.register(new RaagaEngine())
  return engineRegistry
}

/** Global engine registry singleton */
export const registry = new EngineRegistry()
