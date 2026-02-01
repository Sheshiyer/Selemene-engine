import { Elysia, t } from 'elysia'
import type { EngineInput, ErrorResponse, HealthResponse } from '../types'
import { registry } from './registry'

const startTime = Date.now()

/**
 * Create the Elysia HTTP server with all routes
 */
export function createServer() {
  const app = new Elysia()
    // Health check endpoint
    .get(
      '/health',
      (): HealthResponse => ({
        status: 'healthy',
        engines: registry.list(),
        uptime_ms: Date.now() - startTime,
        version: '1.0.0',
      }),
    )

    // List all engines
    .get('/engines', () => ({
      engines: registry.listMetadata(),
      count: registry.count(),
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
        const engine = registry.get(params.id)
        if (!engine) {
          set.status = 404
          return {
            error: `Engine not found: ${params.id}`,
            error_code: 'ENGINE_NOT_FOUND',
          } as ErrorResponse
        }
        return engine.metadata() as any
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
        const engine = registry.get(params.id)
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

export { registry }
