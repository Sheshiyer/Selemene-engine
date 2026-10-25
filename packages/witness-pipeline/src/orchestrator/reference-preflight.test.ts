import { describe, it, expect } from 'vitest';
import {
  runReferencePreflight,
  assertPreflightPassed,
  BGE_M3_RETIRED_DESCRIPTOR,
  DISABLED_GROUNDING_DESCRIPTOR,
  UNCONFIGURED_GROUNDING_DESCRIPTOR,
} from './reference-preflight.js';
import { buildManifestFromIds } from './section-manifest.js';
import type { DyadClientAdapter } from './reference-preflight.js';

const canonicalDyad: DyadClientAdapter = {
  mode: 'canonical',
  runtimeId: 'selemene-witness-v1',
  hasAletheios: true,
  hasPichet: true,
  hasSynthesis: true,
};

const validManifest = buildManifestFromIds(['opening', 'part1', 'part11']);

describe('runReferencePreflight', () => {
  it('passes with canonical dyad + live grounding + valid manifest', () => {
    const result = runReferencePreflight({
      dyadAdapter: canonicalDyad,
      groundingDescriptor: { status: 'live', embeddingModel: 'text-embedding-3-large' },
      sectionManifest: validManifest,
    });
    expect(result.passed).toBe(true);
    expect(result.blockers).toHaveLength(0);
  });

  it('blocks when dyad adapter is null', () => {
    const result = runReferencePreflight({
      dyadAdapter: null,
      groundingDescriptor: { status: 'live' },
      sectionManifest: validManifest,
    });
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('dyad:not_configured'))).toBe(true);
  });

  it('blocks when dyad is local-draft mode', () => {
    const draftDyad: DyadClientAdapter = { ...canonicalDyad, mode: 'local-draft', runtimeId: 'handwritten-draft' };
    const result = runReferencePreflight({
      dyadAdapter: draftDyad,
      groundingDescriptor: { status: 'live' },
      sectionManifest: validManifest,
    });
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('local_draft_mode'))).toBe(true);
  });

  it('blocks when dyad is missing Aletheios', () => {
    const partial: DyadClientAdapter = { ...canonicalDyad, hasAletheios: false };
    const result = runReferencePreflight({
      dyadAdapter: partial,
      groundingDescriptor: { status: 'live' },
      sectionManifest: validManifest,
    });
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('aletheios_missing'))).toBe(true);
  });

  it('blocks on retired BGE-M3 grounding model', () => {
    const result = runReferencePreflight({
      dyadAdapter: canonicalDyad,
      groundingDescriptor: BGE_M3_RETIRED_DESCRIPTOR,
      sectionManifest: validManifest,
    });
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('retired_model') && b.includes('baai/bge-m3'))).toBe(true);
    // Must mention that dimension compatibility alone is insufficient
    expect(result.blockers.some((b) => b.includes('dimension'))).toBe(true);
  });

  it('blocks on disabled grounding (no opt-out boolean weakens this)', () => {
    const result = runReferencePreflight({
      dyadAdapter: canonicalDyad,
      groundingDescriptor: DISABLED_GROUNDING_DESCRIPTOR,
      sectionManifest: validManifest,
    });
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('grounding:disabled'))).toBe(true);
  });

  it('blocks on unconfigured grounding', () => {
    const result = runReferencePreflight({
      dyadAdapter: canonicalDyad,
      groundingDescriptor: UNCONFIGURED_GROUNDING_DESCRIPTOR,
      sectionManifest: validManifest,
    });
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('grounding:unconfigured'))).toBe(true);
  });

  it('blocks on manifest with duplicate section IDs', () => {
    const dupManifest = { sections: [{ id: 'opening', required: true }, { id: 'opening', required: true }] };
    const result = runReferencePreflight({
      dyadAdapter: canonicalDyad,
      groundingDescriptor: { status: 'live' },
      sectionManifest: dupManifest,
    });
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('duplicate_section_id'))).toBe(true);
  });

  it('includes all blockers in failMessage', () => {
    const result = runReferencePreflight({
      dyadAdapter: null,
      groundingDescriptor: BGE_M3_RETIRED_DESCRIPTOR,
      sectionManifest: validManifest,
    });
    expect(result.passed).toBe(false);
    expect(result.failMessage).toContain('REFERENCE ROUTE PREFLIGHT FAILED');
    expect(result.failMessage).toContain('dyad:not_configured');
    expect(result.failMessage).toContain('retired_model');
  });
});

describe('assertPreflightPassed', () => {
  it('does not throw when preflight passes', () => {
    expect(() => assertPreflightPassed({
      dyadAdapter: canonicalDyad,
      groundingDescriptor: { status: 'live', embeddingModel: 'text-embedding-3-large' },
      sectionManifest: validManifest,
    })).not.toThrow();
  });

  it('throws with full fail message when preflight fails', () => {
    expect(() => assertPreflightPassed({
      dyadAdapter: null,
      groundingDescriptor: DISABLED_GROUNDING_DESCRIPTOR,
      sectionManifest: validManifest,
    })).toThrow(/REFERENCE ROUTE PREFLIGHT FAILED/);
  });
});
