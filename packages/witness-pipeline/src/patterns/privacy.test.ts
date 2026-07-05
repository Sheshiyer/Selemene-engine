import { describe, expect, it } from 'vitest';
import { scrubPrivateBirthData } from './privacy.js';

describe('scrubPrivateBirthData', () => {
  it('rejects pass output containing birth date or coordinates', () => {
    const { scrubbed, hadPrivate } = scrubPrivateBirthData('Born 1960-06-01 at 12.972,77.594');
    expect(hadPrivate).toBe(true);
    expect(scrubbed).not.toMatch(/1960-06-01|12\.972|77\.594/);
  });
});
