import { describe, it, expect } from 'vitest';
import { createHash } from 'node:crypto';
import { referenceModeFile, triadEngineFactsFromSnapshots } from './reference-mode-contract.js';
import { BUSINESS_TRIAD_SLUGS, BUSINESS_TRIAD_DISPLAY_NAMES } from './business-triad-support.js';
const hash = (s: string) => createHash('sha256').update(s).digest('hex');
function fixture() {
  const snapshots = Object.fromEntries(BUSINESS_TRIAD_SLUGS.map((s, i) => [s, JSON.stringify([{ engine_id: 'human-design', result: { profile: `${i + 1}/3` } }])]));
  const inputs = { route: 'business-triad-reference', language: 'en', subjects: BUSINESS_TRIAD_SLUGS.map(slug => ({ slug, name: BUSINESS_TRIAD_DISPLAY_NAMES[slug], role: 'partner', inputHash: hash(snapshots[slug]), engineCount: 1 })) };
  return { snapshots, inputs };
}
describe('repository reference mode selection', () => {
  it.each(['en', 'fr'])('keeps existing individual contract for %s', language => {
    expect(referenceModeFile({ subject: 'gary-abitbol', language })).toBe('integrated-kundali-reference.md');
  });
  it('allows only the bound triad contract', () => {
    expect(referenceModeFile(fixture().inputs)).toBe('business-triad-reference.md');
    expect(() => referenceModeFile({ route: '../../custom', language: 'en', subject: 'gary-abitbol' })).toThrow(/Unknown saved reference route/);
  });
  it.each(['missing', 'order', 'role', 'name', 'hash'])('rejects %s subject bindings', kind => {
    const { inputs } = fixture();
    if (kind === 'missing') inputs.subjects.pop();
    if (kind === 'order') inputs.subjects.reverse();
    if (kind === 'role') inputs.subjects[0].role = 'primary';
    if (kind === 'name') inputs.subjects[0].name = 'Anonymous';
    if (kind === 'hash') inputs.subjects[0].inputHash = '';
    expect(() => referenceModeFile(inputs)).toThrow();
  });
  it('keeps three separate named source blocks and full records', () => {
    const { inputs, snapshots } = fixture();
    const text = triadEngineFactsFromSnapshots(inputs, snapshots);
    for (const slug of BUSINESS_TRIAD_SLUGS) expect(text).toContain(`### ${BUSINESS_TRIAD_DISPLAY_NAMES[slug]} (partner)`);
    const full = JSON.parse(text.split('\n\n').at(-1)!);
    expect(full.map((r: any) => r.records[0].result.profile)).toEqual(['1/3', '2/3', '3/3']);
  });
  it('rejects swapped snapshot bytes and changed record counts', () => {
    const { inputs, snapshots } = fixture();
    expect(() => triadEngineFactsFromSnapshots(inputs, { ...snapshots, 'gary-abitbol': snapshots['mohan-kumar'] })).toThrow(/hash mismatch/);
    inputs.subjects[0].engineCount = 2;
    expect(() => triadEngineFactsFromSnapshots(inputs, snapshots)).toThrow(/record count mismatch/);
  });
});
