import type { PassResult } from './integrated.js';
import fs from 'node:fs';

export interface FinalVerificationInput {
  passes: PassResult[];
  pdfPath?: string;
}
// A Jev receipt marked `blocked` (active mode, confident safety failure) is a blocker.
// Shadow receipts never block.

export interface FinalVerificationResult {
  passed: boolean;
  blockers: string[];
}

export function runFinalVerification(input: FinalVerificationInput): FinalVerificationResult {
  const blockers: string[] = [];
  for (const pass of input.passes) {
    const r = pass.rubric as any;
    if (r.placeholder_gate === 'fail') blockers.push(`${pass.id}:placeholder_gate`);
    if (r.chart_fidelity_gate === 'fail') blockers.push(`${pass.id}:chart_fidelity_gate`);
    if (pass.jev?.blocked) blockers.push(`${pass.id}:jev_blocked`);
  }
  if (input.pdfPath && !fs.existsSync(input.pdfPath)) {
    blockers.push('pdf:missing');
  }
  return { passed: blockers.length === 0, blockers };
}
