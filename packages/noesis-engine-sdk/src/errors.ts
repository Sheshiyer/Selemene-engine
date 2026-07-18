/**
 * Error types for @selemene/engine-sdk.
 *
 * Cites: p1-w1-worker-bootstrap-packet.md (evidence before claim), goal-understanding.md
 * (local-first + explicit consent invariant), packages/biofield-api-client/src/biofield-client.ts
 * (error resolution pattern).
 *
 * Tags: phase:integration-p1 wave:integration-w2 area:engine-integration
 */

/** Base SDK error. Carries HTTP status (0 for client-side guards) + optional server error code/details. */
export class EngineSdkError extends Error {
  readonly status: number
  readonly code?: string
  readonly details?: unknown

  constructor(message: string, status: number, code?: string, details?: unknown) {
    super(message)
    this.name = 'EngineSdkError'
    this.status = status
    this.code = code
    this.details = details
  }
}

/**
 * Thrown synchronously BEFORE any network call when media or generative output is
 * requested without a valid consent grant (local-first invariant; goal-understanding.md).
 * status is 0 because no request ever left the client.
 */
export class ConsentError extends EngineSdkError {
  readonly engineId: string
  readonly requiredScope: string

  constructor(engineId: string, requiredScope: string) {
    super(
      `[${engineId}] consent required: scope "${requiredScope}" must be granted before ` +
        `sending media or requesting generative output (local-first; no network call made).`,
      0,
      'CONSENT_REQUIRED',
    )
    this.name = 'ConsentError'
    this.engineId = engineId
    this.requiredScope = requiredScope
  }
}
