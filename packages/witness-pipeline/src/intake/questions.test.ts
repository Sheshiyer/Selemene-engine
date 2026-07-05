import { describe, expect, it } from 'vitest';
import { buildReportIntakeQuestions } from './questions.js';

describe('buildReportIntakeQuestions', () => {
  it('includes Birthplace and Confirm Location questions', () => {
    const qs = buildReportIntakeQuestions({ subjectCount: 1, relationship: false });
    const headers = qs.map((q) => q.header);
    expect(headers).toContain('Birthplace');
    expect(headers).toContain('Confirm Location');
  });
});
