import { describe, it, expect } from 'vitest';
import { runFinalVerification } from './final-verification.js';
import type { PassResult } from './integrated.js';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const failingPass = {
  id: 'opening',
  rubric: { placeholder_gate: 'fail', chart_fidelity_gate: 'pass' },
} as unknown as PassResult;

const passingPass = {
  id: 'opening',
  rubric: { placeholder_gate: 'pass', chart_fidelity_gate: 'pass' },
} as unknown as PassResult;

describe('runFinalVerification', () => {
  it('fails when any section has placeholder_gate fail', () => {
    const result = runFinalVerification({ passes: [failingPass], pdfPath: 'does-not-exist.pdf' });
    expect(result.passed).toBe(false);
    expect(result.blockers).toContain('opening:placeholder_gate');
  });

  it('passes when all gates pass and pdf exists', () => {
    const tmpPdf = path.join(os.tmpdir(), 'selemene-test.pdf');
    fs.writeFileSync(tmpPdf, '%PDF-1.4 test');
    const result = runFinalVerification({ passes: [passingPass], pdfPath: tmpPdf });
    fs.unlinkSync(tmpPdf);
    expect(result.passed).toBe(true);
  });
});
