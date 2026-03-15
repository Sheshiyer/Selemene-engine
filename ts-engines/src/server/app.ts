import { swagger } from '@elysiajs/swagger'
import { Elysia, t } from 'elysia'
import type {
  EngineHealthStatus,
  EngineInput,
  EnginesHealthResponse,
  ErrorResponse,
  HealthResponse,
  LivenessResponse,
  ReadinessResponse,
} from '../types'
import { EngineRegistry, registry } from './registry'
import { isEngineValidationError } from '../utils'

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
      ({
        params,
        set,
      }): ReturnType<typeof registry.get> extends infer T
        ? T extends undefined
          ? ErrorResponse
          : ReturnType<NonNullable<T>['metadata']>
        : never => {
        const engine = engineRegistry.get(params.id)
        if (!engine) {
          set.status = 404
          return {
            error: `Engine not found: ${params.id}`,
            error_code: 'ENGINE_NOT_FOUND',
          } as ErrorResponse
        }
        return engine.metadata() as unknown
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
        }),
      },
    )

  return app
}

export { EngineRegistry, registry }
