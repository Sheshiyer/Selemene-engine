import { describe, expect, it } from 'vitest';
import { buildReportIntakeQuestions } from './questions.js';

describe('buildReportIntakeQuestions', () => {
  it('includes gender and location confirmation questions', () => {
    const questions = buildReportIntakeQuestions({ subjectCount: 1, relationship: false });
    const headers = questions.map((q) => q.header);

    expect(headers).toContain('Gender');
    expect(headers).toContain('Birthplace');
    expect(headers).toContain('Confirm Location');
  });
});
