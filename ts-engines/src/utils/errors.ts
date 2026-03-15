export class EngineValidationError extends Error {
  code: string
  details?: Record<string, unknown>

  constructor(message: string, code = 'VALIDATION_ERROR', details?: Record<string, unknown>) {
    super(message)
    this.name = 'EngineValidationError'
    this.code = code
    this.details = details
  }
}

export function isEngineValidationError(err: unknown): err is EngineValidationError {
  return (
    !!err &&
    typeof err === 'object' &&
    (err as EngineValidationError).name === 'EngineValidationError'
  )
}
