import { describe, it, expect } from 'vitest';
import { fileURLToPath } from 'node:url';
import { parseModeDoc } from '../modes/parser.js';
import {
  BUSINESS_TRIAD_SECTION_IDS,
  localizeBusinessTriadTitles,
} from './business-triad-support.js';

const loadMode = () =>
  parseModeDoc(fileURLToPath(new URL('../../modes/business-triad-reference.md', import.meta.url)));

describe('business-triad-reference mode', () => {
  it('parses with all required frontmatter and pass_plan', () => {
    const mode = loadMode();
    expect(mode.frontmatter.mode).toBe('business-triad-reference');
    expect(mode.frontmatter.subject_count).toEqual({ min: 3, max: 3 });
    expect(mode.frontmatter.roles).toEqual(['partner', 'partner', 'partner']);
    expect(mode.frontmatter.architecture).toBe('linear');
    expect(mode.frontmatter.svg_topology).toBe('triad-triangle');
    expect(mode.frontmatter.pass_plan).toHaveLength(12);
    const ids = mode.frontmatter.pass_plan.map((p) => p.id);
    expect(ids).toEqual([
      'opening',
      'part1',
      'part2',
      'part3',
      'part4',
      'part5',
      'part6',
      'part7',
      'part8',
      'part9',
      'part10',
      'part11',
    ]);
    for (const pass of mode.frontmatter.pass_plan) {
      expect(mode.sections[pass.template]).toBeTruthy();
    }
  });

  it('declares the mandatory bridge mandates against invented triad scoring', () => {
    const mode = loadMode();
    const mandateBlob = mode.frontmatter.bridge_mandates.join('\n').toLowerCase();
    expect(mandateBlob).toContain('per-subject attribution');
    expect(mandateBlob).toContain('never label anyone the primary client');
    expect(mandateBlob).toContain('do not invent a business penta chart');
    expect(mandateBlob).toContain('empty case as empty');
    expect(mandateBlob).toContain('no predictive claim');
  });

  it('exposes the same twelve canonical section ids to the helper', () => {
    const mode = loadMode();
    expect(BUSINESS_TRIAD_SECTION_IDS).toEqual(
      mode.frontmatter.pass_plan.map((p) => p.id),
    );
  });

  it('localizes titles to French without changing ids, target_words, or templates', () => {
    const mode = loadMode();
    const contract = mode.frontmatter.pass_plan.map(({ id, target_words, template }) => ({
      id,
      target_words,
      template,
    }));
    localizeBusinessTriadTitles(mode, 'fr');
    expect(
      mode.frontmatter.pass_plan.map(({ id, target_words, template }) => ({
        id,
        target_words,
        template,
      })),
    ).toEqual(contract);
    expect(mode.frontmatter.pass_plan[0].title.startsWith('Ouverture')).toBe(true);
    expect(mode.frontmatter.pass_plan[3].title).toContain('Partie III');
    expect(mode.frontmatter.pass_plan[11].title).toContain('Synthèse');
    const unique = new Set(mode.frontmatter.pass_plan.map((p) => p.title));
    expect(unique.size).toBe(12);
  });

  it('leaves English canonical titles untouched', () => {
    const mode = loadMode();
    const before = JSON.stringify(mode);
    localizeBusinessTriadTitles(mode, 'en');
    expect(JSON.stringify(mode)).toBe(before);
  });

  it('rejects unknown sections and unsupported languages', () => {
    expect(() => localizeBusinessTriadTitles(loadMode(), 'de')).toThrow('Unsupported');
    expect(() =>
      localizeBusinessTriadTitles(
        { frontmatter: { pass_plan: [{ id: 'invented', title: 'x' }] } },
        'fr',
      ),
    ).toThrow('Unknown');
  });
});
