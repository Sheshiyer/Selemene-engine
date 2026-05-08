/**
 * Noesis TypeScript Engines Server
 * Provides HTTP API for archetypal consciousness engines:
 * - Tarot
 * - I-Ching
 * - Enneagram
 * - Sacred Geometry (stub)
 * - Sigil Forge (stub)
 * - Raaga (Carnatic melakarta sound therapy)
 */

import { createServer, registry } from './server'

import { EnneagramEngine } from './engines/enneagram'
import { IChingEngine } from './engines/i-ching'
import { RaagaEngine } from './engines/raaga'
import { SacredGeometryEngine } from './engines/sacred-geometry'
import { SigilForgeEngine } from './engines/sigil-forge'
// Import and register engines
import { TarotEngine } from './engines/tarot'

// Register engines
registry.register(new TarotEngine())
registry.register(new IChingEngine())
registry.register(new EnneagramEngine())
registry.register(new SacredGeometryEngine())
registry.register(new SigilForgeEngine())
registry.register(new RaagaEngine())

const PORT = process.env.PORT ? Number.parseInt(process.env.PORT) : 3001

const app = createServer()

app.listen(PORT, () => {
  console.log(`
╔══════════════════════════════════════════════════════════════╗
║           Noesis TypeScript Engines Server                   ║
╠══════════════════════════════════════════════════════════════╣
║  Port:     ${PORT.toString().padEnd(48)}║
║  Engines:  ${registry.count().toString().padEnd(48)}║
║  Status:   Running                                           ║
╚══════════════════════════════════════════════════════════════╝

Registered engines: ${registry.list().join(', ') || '(none yet)'}

Endpoints:
  GET  /health                    - Health check
  GET  /engines                   - List all engines
  GET  /engines/:id/info          - Get engine metadata
  POST /engines/:id/calculate     - Run calculation
`)
})

export { app }
