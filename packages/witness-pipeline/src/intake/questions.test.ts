import { describe, expect, it } from 'vitest';
import { buildReportIntakeQuestions, getLanguageQuestion } from './questions.js';

describe('buildReportIntakeQuestions', () => {
  it('includes Birthplace and Confirm Location questions', () => {
    const qs = buildReportIntakeQuestions({ subjectCount: 1, relationship: false });
    const headers = qs.map((q) => q.header);
    expect(headers).toContain('Birthplace');
    expect(headers).toContain('Confirm Location');
  });
});

it('exposes language question with en as default option', () => {
  const q = getLanguageQuestion();
  expect(q.header).toBe('Language');
  const labels = (q.options ?? []).map(o => o.label);
  expect(labels).toContain('en');
});
