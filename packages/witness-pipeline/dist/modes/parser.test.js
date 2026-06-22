import { describe, it, expect } from 'vitest';
import { parseModeDocument, getPassTemplate, getTargetWordsForRegister } from './parser.js';
const sampleMode = `---
mode: composite-dyad
subject_count:
  min: 2
  max: 2
roles:
  - subject-a
  - subject-b
target_words:
  min: 9000
  max: 11000
architecture: linear
pass_plan:
  - id: alpha
    title: Structural Field
    target_words: 3000
    template: pass-alpha-template
engine_overlay_weights:
  panchanga: 1.0
house_overlay: [1, 4, 7, 10]
bridge_mandates:
  - "Braid Vedic and HD data"
svg_topology: dyad-arc
register_variants:
  l1_l3:
    target_words:
      min: 8000
      max: 10000
    overrides:
      - pass_id: alpha
        template: pass-alpha-template-l1-l3
---

## pass-alpha-template
This is the alpha prompt.

## pass-alpha-template-l1-l3
This is the L1-L3 alpha prompt.

## lessons

### 2026-06-01 — Test lesson
**Question:** Does this work?
**Adopted:** Yes.
`;
describe('parseModeDocument', () => {
    it('parses frontmatter and body sections', () => {
        const parsed = parseModeDocument(sampleMode, 'composite-dyad.md');
        expect(parsed.frontmatter.mode).toBe('composite-dyad');
        expect(parsed.frontmatter.subject_count).toEqual({ min: 2, max: 2 });
        expect(parsed.frontmatter.pass_plan).toHaveLength(1);
        expect(parsed.frontmatter.pass_plan[0].template).toBe('pass-alpha-template');
        expect(parsed.sections['pass-alpha-template']).toBe('This is the alpha prompt.');
        expect(parsed.lessons).toHaveLength(1);
        expect(parsed.lessons[0].title).toBe('Test lesson');
    });
    it('throws when frontmatter is missing', () => {
        expect(() => parseModeDocument('no frontmatter', 'bad.md')).toThrow('missing leading');
    });
    it('resolves register-variant template override', () => {
        const parsed = parseModeDocument(sampleMode, 'composite-dyad.md');
        const l1 = getPassTemplate(parsed, 'alpha', 'l1_l3');
        expect(l1).toBe('This is the L1-L3 alpha prompt.');
        const l4 = getPassTemplate(parsed, 'alpha', 'l4_l5');
        expect(l4).toBe('This is the alpha prompt.');
    });
    it('returns register-specific target words', () => {
        const parsed = parseModeDocument(sampleMode, 'composite-dyad.md');
        expect(getTargetWordsForRegister(parsed, 'l1_l3')).toEqual({ min: 8000, max: 10000 });
        expect(getTargetWordsForRegister(parsed, 'l4_l5')).toEqual({ min: 9000, max: 11000 });
    });
});
//# sourceMappingURL=parser.test.js.map