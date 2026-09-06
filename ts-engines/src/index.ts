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

import { createServer, registerTypeScriptRuntimeEngines, registry } from './server'

// Register engines
registerTypeScriptRuntimeEngines(registry)

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
