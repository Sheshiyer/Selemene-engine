import { describe, expect, it } from 'vitest';
import { isCompleteReportRequest } from './types.js';

describe('isCompleteReportRequest', () => {
  it('requires normalized location for every subject', () => {
    const complete = isCompleteReportRequest({
      report_level: 'L3',
      report_mode: 'integrated-reading',
      subjects: [{
        role: 'primary',
        name: 'A',
        gender: 'female',
        birth_date: '1990-01-01',
        birth_time: '12:00',
        birth_time_confidence: 'exact',
        birth_location_query: 'Bangalore, India',
        normalized_location: {
          display_name: 'Bengaluru, Karnataka, India',
          latitude: 12.9716,
          longitude: 77.5946,
          timezone: 'Asia/Kolkata',
          provider: 'manual',
          confidence: 'selected',
        },
      }],
      output: { format: 'markdown', include_rubric: true, include_pattern_extraction: true },
    });

    expect(complete).toBe(true);
  });

  it('returns false when any subject lacks normalized_location', () => {
    const incomplete = isCompleteReportRequest({
      report_level: 'L3',
      report_mode: 'integrated-reading',
      subjects: [
        {
          role: 'primary',
          name: 'A',
          birth_date: '1990-01-01',
          birth_time_confidence: 'exact',
          birth_location_query: 'Bangalore, India',
          normalized_location: {
            display_name: 'Bengaluru',
            latitude: 12.97,
            longitude: 77.59,
            timezone: 'Asia/Kolkata',
            provider: 'nominatim',
            confidence: 'selected',
          },
        },
        {
          role: 'partner',
          name: 'B',
          birth_date: '1991-02-02',
          birth_time_confidence: 'exact',
          birth_location_query: 'Mumbai, India',
        },
      ],
      output: { format: 'markdown', include_rubric: true, include_pattern_extraction: true },
    });

    expect(incomplete).toBe(false);
  });

  it('returns false for empty subjects', () => {
    const empty = isCompleteReportRequest({
      report_level: 'L3',
      report_mode: 'integrated-reading',
      subjects: [],
      output: { format: 'markdown', include_rubric: true, include_pattern_extraction: true },
    });
    expect(empty).toBe(false);
  });
});
