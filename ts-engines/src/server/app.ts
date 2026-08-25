import { swagger } from '@elysiajs/swagger'
import { Elysia, t } from 'elysia'
import { resolveClipDir, resolveStoredClip } from '../engines/raaga/clip'
import type {
  EngineHealthStatus,
  EngineInput,
  EngineMetadata,
  EnginesHealthResponse,
  ErrorResponse,
  HealthResponse,
  LivenessResponse,
  ReadinessResponse,
} from '../types'
import { isEngineValidationError } from '../utils'
import { EngineRegistry, registry } from './registry'

const startTime = Date.now()

async function runSelfCheck(engineRegistry: EngineRegistry): Promise<EngineHealthStatus[]> {
  return Promise.all(
    engineRegistry.all().map(async (engine) => {
      const startedAt = Date.now()
      const engineId = engine.metadata().id

      try {
        if (!engine.selfCheck) {
          return {
            engine_id: engineId,
            healthy: true,
            detail: 'default health check passed',
            latency_ms: Date.now() - startedAt,
          }
        }

        const result = await engine.selfCheck()
        return {
          engine_id: result.engine_id || engineId,
          healthy: result.healthy,
          detail: result.detail,
          latency_ms: result.latency_ms ?? Date.now() - startedAt,
        }
      } catch (error) {
        return {
          engine_id: engineId,
          healthy: false,
          detail: error instanceof Error ? error.message : 'unknown engine health error',
          latency_ms: Date.now() - startedAt,
        }
      }
    }),
  )
}

/**
 * Create the Elysia HTTP server with all routes
 */
export function createServer(engineRegistry: EngineRegistry = registry) {
  const app = new Elysia()
    .use(
      swagger({
        documentation: {
          info: {
            title: 'Noesis TS Engines API',
            version: '1.0.0',
            description:
              'TypeScript consciousness engines for Tryambakam Noesis — tarot, i-ching, enneagram, sacred-geometry, sigil-forge',
          },
        },
        path: '/docs',
      }),
    )
    // Health check endpoint
    .get(
      '/health',
      (): HealthResponse => ({
        status: 'healthy',
        engines: engineRegistry.list(),
        uptime_ms: Date.now() - startTime,
        version: '1.0.0',
      }),
    )
    .get(
      '/health/live',
      (): LivenessResponse => ({
        status: 'alive',
        uptime_ms: Date.now() - startTime,
        version: '1.0.0',
      }),
    )
    .get('/health/engines', async (): Promise<EnginesHealthResponse> => {
      const engines = await runSelfCheck(engineRegistry)
      const status = engines.every((engine) => engine.healthy) ? 'healthy' : 'degraded'
      return { status, engines }
    })
    .get('/health/ready', async ({ set }): Promise<ReadinessResponse> => {
      const engines = await runSelfCheck(engineRegistry)
      const failedEngines = engines
        .filter((engine) => !engine.healthy)
        .map((engine) => engine.engine_id)

      if (failedEngines.length > 0) {
        set.status = 503
      }

      return {
        status: failedEngines.length === 0 ? 'ready' : 'degraded',
        engines,
        failed_engines: failedEngines,
      }
    })

    // List all engines
    .get('/engines', () => ({
      engines: engineRegistry.listMetadata(),
      count: engineRegistry.count(),
    }))

    // Get engine info by ID
    .get(
      '/engines/:id/info',
      ({ params, set }): EngineMetadata | ErrorResponse => {
        const engine = engineRegistry.get(params.id)
        if (!engine) {
          set.status = 404
          return {
            error: `Engine not found: ${params.id}`,
            error_code: 'ENGINE_NOT_FOUND',
          } as ErrorResponse
        }
        return engine.metadata()
      },
      {
        params: t.Object({
          id: t.String(),
        }),
      },
    )

    // Calculate endpoint
    .post(
      '/engines/:id/calculate',
      async ({ params, body, set }) => {
        const engine = engineRegistry.get(params.id)
        if (!engine) {
          set.status = 404
          return {
            error: `Engine not found: ${params.id}`,
            error_code: 'ENGINE_NOT_FOUND',
          } as ErrorResponse
        }

        const meta = engine.metadata()

        // Check consciousness level
        if (body.consciousness_level < meta.required_phase) {
          set.status = 403
          return {
            error: `Insufficient consciousness level. Required: ${meta.required_phase}, provided: ${body.consciousness_level}`,
            error_code: 'PHASE_ACCESS_DENIED',
            details: {
              required_phase: meta.required_phase,
              provided_phase: body.consciousness_level,
            },
          } as ErrorResponse
        }

        try {
          const result = await engine.calculate(body as EngineInput)
          return result
        } catch (err) {
          if (isEngineValidationError(err)) {
            set.status = 422
            return {
              error: err.message,
              error_code: err.code,
              details: err.details,
            } as ErrorResponse
          }

          set.status = 500
          return {
            error: err instanceof Error ? err.message : 'Unknown error',
            error_code: 'CALCULATION_ERROR',
          } as ErrorResponse
        }
      },
      {
        params: t.Object({
          id: t.String(),
        }),
        body: t.Object({
          consciousness_level: t.Number({ minimum: 0, maximum: 5 }),
          parameters: t.Record(t.String(), t.Unknown()),
          seed: t.Optional(t.Number()),
          question: t.Optional(t.String()),
          // P1/P2 media per FROZEN (T-002/T-005/T-031) -- allow top-level for audio_ref/consent/image_data in raaga/sigil etc samples
          // Cites all mandatory: p1-w1-worker-bootstrap-packet.md + 3 extraction + P1W1-CONTRACTS-FROZEN.md + detailed-task-list T-031 + ext-contract-harness.ts + EXECUTION-STATUS + P1W2-HANDOFF + tags phase:integration-p1 wave:integration-w2 engine-raaga
          image_data: t.Optional(t.Any()),
          audio_ref: t.Optional(t.Any()),
          consent: t.Optional(t.Any()),
          quality: t.Optional(t.Any()),
        }),
      },
    )

    // Raaga clip store (raaga-clip-generation, p5-p4-next-batch)
    // Serves locally rendered raaga WAV clips written under RAAGA_CLIP_DIR (default <tmp>/raaga-clips).
    // Local-first: no external fetch; path-traversal guarded; 404 when clip absent.
    // Cites: goal-understanding.md (local-first), P1W1-CONTRACTS-FROZEN.md (generated_audio.clip_url),
    // gaps-and-improvements.md §4 (audio clip path missing), p1-w1-worker-bootstrap-packet.md
    // tags: phase:integration-p1 wave:integration-w2 area:engine-integration engine-raaga
    .get('/clips/raaga/:file', async ({ params, set }) => {
      const path = resolveStoredClip(resolveClipDir(), params.file)
      if (!path) {
        set.status = 404
        return {
          error: `Clip not found: ${params.file}`,
          error_code: 'CLIP_NOT_FOUND',
        } as ErrorResponse
      }
      set.headers['Content-Type'] = 'audio/wav'
      set.headers['Cache-Control'] = 'public, max-age=31536000, immutable'
      return new Response(await Bun.file(path).arrayBuffer(), {
        headers: { 'Content-Type': 'audio/wav' },
      })
    })

    // Suno bridge proxy routes (SUNO-02)
    // Forwards to the Suno API wrapper with SUNO_COOKIE injected server-side.
    // Endpoints: GET /suno/get_limit, POST /suno/custom_generate, GET /suno/get
    .get('/suno/get_limit', async ({ set }) => {
      const result = await proxyToSuno('/api/get_limit')
      set.status = result.status
      return result.body
    })
    .post('/suno/custom_generate', async ({ body, set }) => {
      const result = await proxyToSuno('/api/custom_generate', {
        method: 'POST',
        body: JSON.stringify(body),
      })
      set.status = result.status
      return result.body
    })
    .get('/suno/get', async ({ query, set }) => {
      const ids = (query as Record<string, string>).ids ?? ''
      const result = await proxyToSuno(`/api/get?ids=${encodeURIComponent(ids)}`)
      set.status = result.status
      return result.body
    })

  return app
}

// ---------------------------------------------------------------------------
// Suno proxy helper
// ---------------------------------------------------------------------------

const SUNO_BRIDGE_URL = process.env.SUNO_BRIDGE_URL ?? 'https://suno.ai'
const SUNO_COOKIE = process.env.SUNO_COOKIE ?? ''

async function proxyToSuno(
  path: string,
  init: RequestInit = {},
): Promise<{ status: number; body: unknown }> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    Accept: 'application/json',
  }
  if (SUNO_COOKIE) headers.Cookie = SUNO_COOKIE

  try {
    const res = await fetch(`${SUNO_BRIDGE_URL}${path}`, {
      ...init,
      headers: { ...headers, ...((init.headers as Record<string, string>) ?? {}) },
    })
    const body = await res.json().catch(() => ({ error: 'non-json response' }))
    return { status: res.status, body }
  } catch (err) {
    return {
      status: 502,
      body: {
        error: 'suno bridge unreachable',
        detail: err instanceof Error ? err.message : String(err),
      },
    }
  }
}

export { EngineRegistry, registry }
