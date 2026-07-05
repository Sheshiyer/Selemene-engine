import { describe, expect, it } from 'vitest';
import {
  normalizeManualLocation,
  confirmNormalizedLocation,
  isCompleteReportRequest,
  type ReportGenerationRequest,
} from './index.js';

describe('intake completeness contract', () => {
  it('blocks report generation until normalized_location is present on all subjects', () => {
    const req: ReportGenerationRequest = {
      report_level: 'L3',
      report_mode: 'integrated-reading',
      subjects: [
        {
          role: 'primary',
          name: 'Test',
          birth_date: '1990-01-01',
          birth_time_confidence: 'exact',
          birth_location_query: 'Bengaluru, India',
        },
      ],
      output: { format: 'markdown', include_rubric: true, include_pattern_extraction: true },
    };

    expect(isCompleteReportRequest(req)).toBe(false);

    req.subjects[0].normalized_location = confirmNormalizedLocation({
      manual: {
        displayName: 'Bengaluru, Karnataka, India',
        latitude: 12.9716,
        longitude: 77.5946,
        timezone: 'Asia/Kolkata',
      },
    });

    expect(isCompleteReportRequest(req)).toBe(true);
  });
});
