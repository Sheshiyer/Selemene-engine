import type { ConsciousnessEngine, EngineMetadata } from '../types'

/**
 * Registry of all TypeScript consciousness engines
 * Engines register themselves here on startup
 */
class EngineRegistry {
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

  /** Get engine count */
  count(): number {
    return this.engines.size
  }
}

/** Global engine registry singleton */
export const registry = new EngineRegistry()
