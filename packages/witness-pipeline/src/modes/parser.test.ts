import { describe, it, expect } from 'vitest';
import { parseModeDocument } from './parser.js';

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
---

## pass-alpha-template
This is the alpha prompt.

## lessons

### 2026-06-01 — Test lesson
**Question:** Does this work?
**Adopted:** Yes.
`;

describe('parseModeDocument', () => {
  it('parses frontmatter and body sections', () => {
    const parsed = parseModeDocument(sampleMode, 'composite-dyad.md');
    expect(parsed.mode).toBe('composite-dyad');
    expect(parsed.subject_count).toEqual({ min: 2, max: 2 });
    expect(parsed.pass_plan).toHaveLength(1);
    expect(parsed.pass_plan[0].template).toBe('pass-alpha-template');
    expect(parsed.templates['pass-alpha-template']).toBe('This is the alpha prompt.');
    expect(parsed.lessons).toHaveLength(1);
    expect(parsed.lessons[0].title).toBe('Test lesson');
  });

  it('throws when frontmatter is missing', () => {
    expect(() => parseModeDocument('no frontmatter', 'bad.md')).toThrow('Missing frontmatter');
  });
});
