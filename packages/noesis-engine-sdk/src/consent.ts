/**
 * Local-first consent enforcement (goal-understanding.md invariant; Sankalpa ISA).
 *
 * Mirrors the harness guard (scripts/ext-contract-harness.ts:31-41) but FAILS CLOSED:
 * the SDK throws ConsentError before any network call when media or generative output
 * is requested without a valid grant — Prong2 Sankalpa owns local preview; escalation
 * to Selemene backend is explicit opt-in only.
 *
 * Consent scopes match the FROZEN harness samples (ext-contract-harness.ts:75-78).
 *
 * Cites: P1W1-CONTRACTS-FROZEN.md, goal-understanding.md, gaps-and-improvements.md
 * (§5 consent/privacy gaps), engine-media-contracts.ts (sankalpa assertConsentForBackend).
 *
 * Tags: phase:integration-p1 wave:integration-w2 area:engine-integration
 */

import { ConsentError } from './errors'
import type { Consent } from './types'

/** FROZEN consent scopes (scripts/ext-contract-harness.ts:75-78). */
export const CONSENT_SCOPES = {
  BIOFIELD_CAPTURE: 'biofield-capture',
  FACE_IMAGE: 'face-image',
  RAAGA_AUDIO: 'raaga-audio',
  SIGIL_GEN: 'sigil-gen',
} as const

export type ConsentScope = (typeof CONSENT_SCOPES)[keyof typeof CONSENT_SCOPES]

/** Resolve the effective consent for a call: explicit input consent wins, then per-media consent. */
export function resolveConsent(
  ...candidates: Array<Consent | undefined>
): Consent | undefined {
  for (const candidate of candidates) {
    if (candidate) return candidate
  }
  return undefined
}

/**
 * Throw ConsentError unless `consent` is granted AND includes `requiredScope`.
 * Called before every network call that carries media or requests generative output.
 * Local-first: this is the only guard; there is no silent fallback upload.
 */
export function requireConsent(
  consent: Consent | undefined,
  requiredScope: ConsentScope | string,
  engineId: string,
): asserts consent is Consent {
  if (!consent || consent.granted !== true || !consent.scopes.includes(requiredScope)) {
    throw new ConsentError(engineId, requiredScope)
  }
}

/**
 * Soft freshness check (harness warns past 30m). Returns age in ms; caller decides policy.
 * The SDK does not auto-reject stale consent — expiry policy belongs to the caller (Sankalpa).
 */
export function consentAgeMs(consent: Consent, now: number = Date.now()): number {
  const ts = new Date(consent.timestamp).getTime()
  return Number.isFinite(ts) ? now - ts : Number.POSITIVE_INFINITY
}

/** Convenience grant factory for tests/harness/Sankalpa consent gates (timestamped, opaque token). */
export function createConsent(scopes: string[], token?: string): Consent {
  return {
    granted: true,
    scopes,
    timestamp: new Date().toISOString(),
    token: token ?? `consent-${Date.now()}-${Math.random().toString(36).slice(2)}`,
  }
}
