import { describe, it, expect } from 'vitest';
import {
  validateManifestStructure,
  validatePassResultsAgainstManifest,
  buildManifestFromIds,
} from './section-manifest.js';

describe('validateManifestStructure', () => {
  it('passes a clean manifest', () => {
    const manifest = buildManifestFromIds(['opening', 'part1', 'part2', 'part11']);
    const result = validateManifestStructure(manifest);
    expect(result.passed).toBe(true);
    expect(result.blockers).toHaveLength(0);
  });

  it('blocks empty section ID', () => {
    const manifest = { sections: [{ id: '', required: true }] };
    const result = validateManifestStructure(manifest);
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('empty_section_id'))).toBe(true);
  });

  it('blocks whitespace-only section ID', () => {
    const manifest = { sections: [{ id: '   ', required: true }] };
    const result = validateManifestStructure(manifest);
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('empty_section_id'))).toBe(true);
  });

  it('blocks duplicate section IDs', () => {
    const manifest = buildManifestFromIds(['opening', 'part1', 'opening']);
    const result = validateManifestStructure(manifest);
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('duplicate_section_id') && b.includes('opening'))).toBe(true);
  });
});

describe('validatePassResultsAgainstManifest', () => {
  const manifest = buildManifestFromIds(['opening', 'part1', 'part11']);

  it('passes when all required sections present', () => {
    const result = validatePassResultsAgainstManifest(['opening', 'part1', 'part11'], manifest);
    expect(result.passed).toBe(true);
  });

  it('blocks unknown section IDs in results', () => {
    const result = validatePassResultsAgainstManifest(['opening', 'part1', 'part11', 'phantom'], manifest);
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('unknown_section_id') && b.includes('phantom'))).toBe(true);
  });

  it('blocks missing required sections', () => {
    const result = validatePassResultsAgainstManifest(['opening', 'part1'], manifest);
    expect(result.passed).toBe(false);
    expect(result.blockers.some((b) => b.includes('missing_required_section') && b.includes('part11'))).toBe(true);
  });

  it('warns on missing optional sections but does not block', () => {
    const mixedManifest = {
      sections: [
        { id: 'opening', required: true },
        { id: 'part1', required: true },
        { id: 'optional-appendix', required: false },
      ],
    };
    const result = validatePassResultsAgainstManifest(['opening', 'part1'], mixedManifest);
    expect(result.passed).toBe(true);
    expect(result.warnings.some((w) => w.includes('optional-appendix'))).toBe(true);
  });
});

describe('buildManifestFromIds', () => {
  it('marks all as required by default', () => {
    const manifest = buildManifestFromIds(['opening', 'part1']);
    expect(manifest.sections.every((s) => s.required)).toBe(true);
  });

  it('respects a requiredIds set', () => {
    const manifest = buildManifestFromIds(['opening', 'part1', 'appendix'], new Set(['opening', 'part1']));
    const appendix = manifest.sections.find((s) => s.id === 'appendix');
    expect(appendix?.required).toBe(false);
  });
});
