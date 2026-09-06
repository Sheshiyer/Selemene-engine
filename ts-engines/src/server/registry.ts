import { EnneagramEngine } from '../engines/enneagram'
import { IChingEngine } from '../engines/i-ching'
import { RaagaEngine } from '../engines/raaga'
import { SacredGeometryEngine } from '../engines/sacred-geometry'
import { SigilForgeEngine } from '../engines/sigil-forge'
import { TarotEngine } from '../engines/tarot'
import type {
  CapabilityAvailability,
  ConsciousnessEngine,
  ContractEngineCapability,
  EngineMetadata,
} from '../types'

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
