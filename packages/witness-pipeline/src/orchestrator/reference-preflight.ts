// ─── Reference Route Preflight ────────────────────────────────────────
// Opt-in preflight check for the reference-aligned route.
// Must be called before any generation begins when using the reference route.
//
// Verifies that:
//   1. A canonical dyad client adapter is configured (not local-draft mode)
//   2. A live grounding adapter is configured (not disabled/noop)
//   3. The grounding adapter's embedding space is compatible (no HTTP-410 retired models)
//   4. The section manifest passes structural validation
//
// A failed preflight BLOCKS generation and returns clear diagnostics.
// There is NO boolean opt-out — the reference route requires both dyad and grounding.
//
// NOTE: This does NOT make live LLM calls or production inference calls.
// It only verifies that the required client adapters are *configured* and
// structurally valid. Live call availability is a deployment concern.

import type { SectionManifest } from './section-manifest.js';
import { validateManifestStructure } from './section-manifest.js';

// ─── Dyad client adapter interface ──────────────────────────────────
// The canonical dyad must be injected via a client adapter, not hardcoded.
// This interface is the contract — local-draft mode must set isDraft=true,
// and the preflight blocks it for the reference route.

export type DyadAdapterMode = 'canonical' | 'local-draft';

export interface DyadClientAdapter {
  /** Mode declaration. Reference route requires 'canonical'. */
  mode: DyadAdapterMode;
  /** Name/version of the dyad runtime for receipt provenance */
  runtimeId: string;
  /** Whether the adapter has Aletheios configured */
  hasAletheios: boolean;
  /** Whether the adapter has Pichet configured */
  hasPichet: boolean;
  /** Whether the adapter has a synthesis stage configured */
  hasSynthesis: boolean;
}

// ─── Grounding adapter status ────────────────────────────────────────

export type GroundingAdapterStatus =
  | 'live'           // adapter configured, embedding space confirmed
  | 'disabled'       // explicitly disabled (NoopGroundingProvider)
  | 'retired-model'  // embedding model retired (e.g. HTTP 410 baai/bge-m3)
  | 'unconfigured';  // no adapter provided

export interface GroundingAdapterDescriptor {
  status: GroundingAdapterStatus;
  /** Identifies the embedding model or provider for audit trail */
  embeddingModel?: string;
  /** Human-readable reason for non-live status */
  reason?: string;
}

// ─── Preflight input ─────────────────────────────────────────────────

export interface ReferencePreflight {
  /** Dyad client adapter declaration. */
  dyadAdapter: DyadClientAdapter | null;
  /** Grounding adapter descriptor. */
  groundingDescriptor: GroundingAdapterDescriptor;
  /** Section manifest to structurally validate. */
  sectionManifest: SectionManifest;
}

// ─── Preflight result ────────────────────────────────────────────────

export interface PreflightResult {
  passed: boolean;
  blockers: string[];
  warnings: string[];
  /**
   * When passed=false, generation MUST NOT proceed.
   * This message is suitable for CLI output.
   */
  failMessage?: string;
}

/**
 * Run the reference route preflight check.
 *
 * Returns a PreflightResult. If `passed` is false, generation must be
 * aborted with the provided failMessage.
 *
 * No LLM calls, no network calls, no inference. This only validates
 * adapter configuration and structural soundness.
 */
export function runReferencePreflight(input: ReferencePreflight): PreflightResult {
  const blockers: string[] = [];
  const warnings: string[] = [];

  // ─── 1. Dyad adapter check ──────────────────────────────────────

  if (!input.dyadAdapter) {
    blockers.push('preflight:dyad:not_configured — reference route requires a canonical dyad client adapter; none provided');
  } else {
    const d = input.dyadAdapter;
    if (d.mode !== 'canonical') {
      blockers.push(
        `preflight:dyad:local_draft_mode — adapter "${d.runtimeId}" is marked local-draft; ` +
        'reference route requires a canonical dyad adapter. ' +
        'Do not imply local-draft prompts are the deployed Selemene runtime.',
      );
    }
    if (!d.hasAletheios) {
      blockers.push(`preflight:dyad:aletheios_missing — adapter "${d.runtimeId}" has no Aletheios voice configured`);
    }
    if (!d.hasPichet) {
      blockers.push(`preflight:dyad:pichet_missing — adapter "${d.runtimeId}" has no Pichet voice configured`);
    }
    if (!d.hasSynthesis) {
      blockers.push(`preflight:dyad:synthesis_missing — adapter "${d.runtimeId}" has no synthesis stage configured`);
    }
  }

  // ─── 2. Grounding adapter check ─────────────────────────────────

  const g = input.groundingDescriptor;
  switch (g.status) {
    default:
      blockers.push('preflight:grounding:invalid_status');
      break;
    case 'live':
      // All good — grounding is configured and embedding space is confirmed
      break;
    case 'disabled':
      // No opt-out boolean may weaken this for the reference route
      blockers.push(
        'preflight:grounding:disabled — reference route requires live grounding; ' +
        `disabled state is not allowed. ${g.reason ?? ''}`.trim(),
      );
      break;
    case 'retired-model':
      blockers.push(
        `preflight:grounding:retired_model — embedding model "${g.embeddingModel ?? 'unknown'}" is retired ` +
        `(${g.reason ?? 'HTTP 410 or end-of-life'}). ` +
        'Do not substitute a merely dimension-compatible model; its vector space must match or a separately authorized corpus migration is needed. ' +
        'This is an explicit pilot blocker.',
      );
      break;
    case 'unconfigured':
      blockers.push(
        'preflight:grounding:unconfigured — reference route requires a grounding adapter; none provided',
      );
      break;
  }

  // ─── 3. Section manifest structural check ───────────────────────

  const manifestCheck = validateManifestStructure(input.sectionManifest);
  blockers.push(...manifestCheck.blockers);
  warnings.push(...manifestCheck.warnings);

  // ─── Result ─────────────────────────────────────────────────────

  const passed = blockers.length === 0;
  const failMessage = passed
    ? undefined
    : [
        'REFERENCE ROUTE PREFLIGHT FAILED — generation blocked.',
        '',
        'Blockers:',
        ...blockers.map((b) => `  • ${b}`),
        warnings.length > 0 ? '' : null,
        ...(warnings.length > 0 ? ['Warnings:', ...warnings.map((w) => `  ~ ${w}`)] : []),
      ].filter((l) => l !== null).join('\n');

  return { passed, blockers, warnings, failMessage };
}

/**
 * Assert preflight passes and throw with the full fail message if not.
 * Use in CLI scripts before starting generation.
 *
 * @throws Error if preflight fails
 */
export function assertPreflightPassed(preflight: ReferencePreflight): void {
  const result = runReferencePreflight(preflight);
  if (!result.passed) {
    throw new Error(result.failMessage);
  }
}

// ─── Descriptor constructors ─────────────────────────────────────────

/** The baai/bge-m3 model reached end of life on 2026-08-25. */
export const BGE_M3_RETIRED_DESCRIPTOR: GroundingAdapterDescriptor = {
  status: 'retired-model',
  embeddingModel: 'baai/bge-m3',
  reason: 'HTTP 410 — baai/bge-m3 reached end of life on 2026-08-25T09:00:00Z. Vector space compatibility is not guaranteed by dimension alone.',
};

/** Disabled grounding descriptor (no provider configured). */
export const DISABLED_GROUNDING_DESCRIPTOR: GroundingAdapterDescriptor = {
  status: 'disabled',
  reason: 'NoopGroundingProvider or provider=null',
};

/** Unconfigured grounding (no descriptor provided). */
export const UNCONFIGURED_GROUNDING_DESCRIPTOR: GroundingAdapterDescriptor = {
  status: 'unconfigured',
};
