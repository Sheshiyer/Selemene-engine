import { describe, it, expect } from 'vitest';
import { parseModeDoc } from './parser.js';
import { resolve } from 'node:path';

describe('parseModeDoc (file-based)', () => {
  it('parses a real mode document from disk', () => {
    const path = resolve(__dirname, 'fixtures/composite-dyad.md');
    const doc = parseModeDoc(path);
    expect(doc.frontmatter.mode).toBe('composite-dyad');
    expect(doc.frontmatter.pass_plan).toHaveLength(1);
    expect(doc.sections['pass-alpha-template']).toContain('This is the alpha prompt');
    expect(doc.lessons).toHaveLength(1);
  });

  it('resolves register variants from file', () => {
    const path = resolve(__dirname, 'fixtures/composite-dyad.md');
    const doc = parseModeDoc(path);
    const l1 = doc.frontmatter.register_variants?.l1_l3;
    expect(l1?.target_words).toEqual({ min: 8000, max: 10000 });
  });
});
