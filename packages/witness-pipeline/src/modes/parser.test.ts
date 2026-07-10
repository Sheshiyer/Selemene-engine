import { describe, it, expect, vi } from 'vitest';
import { parseModeDocument, getPassTemplate, getTargetWordsForRegister } from './parser.js';
import { IntegratedReadingOrchestrator } from '../orchestrator/integrated.js';
import type { SelemeneEngineOutput } from '../index.js';

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

  it('parses optional report level metadata', () => {
    const doc = parseModeDocument(`---
mode: test-report
report_level: L3
subject_count: { min: 1, max: 1 }
roles: [subject]
target_words: { min: 100, max: 200 }
architecture: linear
pass_plan:
  - id: synthesis
    title: Synthesis
    target_words: 100
    template: synthesis-pass
engine_overlay_weights: { panchanga: 1 }
house_overlay: [1]
bridge_mandates: ["Use facts"]
svg_topology: dyad-arc
---

## synthesis-pass
x
`, 'report-level.md');
    expect(doc.frontmatter.report_level).toBe('L3');
  });

  it('accepts L4 and L5 as valid report_level values', () => {
    const l4 = parseModeDocument(`---
mode: test-l4
report_level: L4
subject_count: { min: 1, max: 1 }
roles: [subject]
target_words: { min: 100, max: 200 }
architecture: linear
pass_plan:
  - id: synthesis
    title: Synthesis
    target_words: 100
    template: synthesis-pass
engine_overlay_weights: { panchanga: 1 }
house_overlay: [1]
bridge_mandates: ["Use facts"]
svg_topology: dyad-arc
---

## synthesis-pass
x
`, 'l4.md');
    expect(l4.frontmatter.report_level).toBe('L4');

    const l5 = parseModeDocument(`---
mode: test-l5
report_level: L5
subject_count: { min: 1, max: 1 }
roles: [subject]
target_words: { min: 100, max: 200 }
architecture: linear
pass_plan:
  - id: synthesis
    title: Synthesis
    target_words: 100
    template: synthesis-pass
engine_overlay_weights: { panchanga: 1 }
house_overlay: [1]
bridge_mandates: ["Use facts"]
svg_topology: dyad-arc
---

## synthesis-pass
x
`, 'l5.md');
    expect(l5.frontmatter.report_level).toBe('L5');
  });

  it('loads unmarried-partners mode with correct relationship_type', () => {
    const parsed = parseModeDocument(`---
mode: unmarried-partners
subject_count: { min: 2, max: 2 }
roles:
  - partner
  - partner
target_words: { min: 3500, max: 5500 }
architecture: linear
pass_plan:
  - id: opening
    title: Opening
    target_words: 350
    template: opening-template
engine_overlay_weights: { "human-design": 1 }
house_overlay: [1, 7]
bridge_mandates: ["Use unmarried-partners language only"]
svg_topology: dyad-arc
relationship_types: ["unmarried-partners"]
report_level: L2
---

## opening-template
# {{relationship_header}}
Non-predictive witness for unmarried partners.
`, 'unmarried-partners.md');
    expect(parsed.frontmatter.mode).toBe('unmarried-partners');
    expect(parsed.frontmatter.relationship_types).toContain('unmarried-partners');
    expect(parsed.frontmatter.report_level).toBe('L2');
  });

  it('loads married-partners mode with correct relationship_type', () => {
    const parsed = parseModeDocument(`---
mode: married-partners
subject_count: { min: 2, max: 2 }
roles:
  - partner
  - partner
target_words: { min: 3500, max: 5500 }
architecture: linear
pass_plan:
  - id: opening
    title: Opening
    target_words: 350
    template: opening-template
engine_overlay_weights: { "human-design": 1 }
house_overlay: [1, 7]
bridge_mandates: ["Use married-partners language only"]
svg_topology: dyad-arc
relationship_types: ["married-partners"]
report_level: L2
---

## opening-template
# {{relationship_header}}
Non-predictive witness for married partners.
`, 'married-partners.md');
    expect(parsed.frontmatter.mode).toBe('married-partners');
    expect(parsed.frontmatter.relationship_types).toContain('married-partners');
    expect(parsed.frontmatter.report_level).toBe('L2');
  });

  it('exercises triad-triangle topology via parser + orchestrator mock', async () => {
    const triadDoc = parseModeDocument(`---
mode: triad-triangle-test
subject_count: { min: 3, max: 3 }
roles:
  - partner
  - partner
  - partner
target_words: { min: 300, max: 500 }
architecture: linear
pass_plan:
  - id: opening
    title: Opening
    target_words: 100
    template: opening-template
  - id: field
    title: Field
    target_words: 150
    template: field-template
engine_overlay_weights: { "human-design": 1 }
house_overlay: [1, 7]
bridge_mandates: ["Use triad language"]
svg_topology: triad-triangle
relationship_types: ["unmarried-partners"]
report_level: L2
---

## opening-template
# {{relationship_header}}
Triad field for {{subject_names}}.

## field-template
Witness the triangle between the three partners.
`, 'triad-triangle-test.md');

    expect(triadDoc.frontmatter.svg_topology).toBe('triad-triangle');
    expect(triadDoc.frontmatter.relationship_types).toContain('unmarried-partners');

    const llm = vi.fn().mockResolvedValue('TRIAD OUTPUT');
    const orchestrator = new IntegratedReadingOrchestrator({ mode: triadDoc, llm });
    const mockEng: SelemeneEngineOutput[] = [{
      engine_id: 'human-design',
      result: { type: 'Generator' },
      witness_prompt: 'Observe.',
      consciousness_level: 2,
      metadata: { calculation_time_ms: 1, backend: 'native', precision_achieved: 'standard', cached: false, timestamp: '2026-06-22T00:00:00Z', engine_version: '1' },
      envelope_version: '1',
    }];
    const result = await orchestrator.run({
      subjectNames: ['A', 'B', 'C'],
      engineResultsBySubject: [mockEng, mockEng, mockEng],
      consciousnessLevel: 2,
    });
    expect(result.mode).toBe('triad-triangle-test');
    expect(result.passes.length).toBe(2);
    expect(result.assembled).toContain('TRIAD OUTPUT');
  });

});
