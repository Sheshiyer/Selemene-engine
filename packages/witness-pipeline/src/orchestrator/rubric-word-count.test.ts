import { expect, it } from 'vitest';
import { countReportWords } from './rubric.js';

it('counts table content identically to prose without charging for Markdown syntax', () => {
  const table = '## Identity\n\n| Field | Value |\n| --- | --- |\n| Profile | 2/4 |\n\n---';
  expect(countReportWords(table)).toBe(countReportWords('Identity Field Value Profile 2/4'));
});
it('retains accented language, source labels and exact numeric values', () => {
  expect(countReportWords('Énergie thème 23°14 sw:hd:type:generator:desc — | **')).toBe(4);
});
