import { describe, it, expect } from 'vitest';
import { createHash } from 'node:crypto';
import {
  validateGuardrailDisposition,
  GuardrailDispositionError,
  type GuardrailDispositionArtifact,
  type GuardrailDiagnosticArtifact,
  type JevReviewArtifact,
  type RevisionArtifact,
  type ValidateGuardrailDispositionInput,
} from './guardrail-disposition.js';

// ─── helpers ──────────────────────────────────────────────────────────

const sha256 = (s: string) => createHash('sha256').update(s, 'utf8').digest('hex');

/**
 * Minimal real-shaped fixture modelled on
 * tools/humdes-extractor/output/gary-abitbol-2026-09-28/reports-reference/runs/
 * sheshnarayan-en-2026-09-30T01-04-32-223Z/opening.guardrail-disposition.json
 * and its cited jev / diagnostic / revision. Two nonblank lines only so the
 * fixture stays compact; the coverage / index / hash / answer contracts are
 * the same shape as in production.
 */
function makeFixture(): ValidateGuardrailDispositionInput {
  const exactOutput =
    'This report gathers eleven engine result records and reads them as a scope-setting movement.\n' +
    '\n' +
    '**Method.** Two witness voices reconcile in each chapter without predicting personal outcomes.';
  const sectionOutputHash = sha256(exactOutput);

  const jevFile = 'opening.editorial-jev-2026-09-30T01-23-11.347Z.json';
  const diagFile = 'opening.guardrail-diagnostic-2026-09-30T01-27-35.161Z.json';
  const revFile = 'opening.editorial-revision.json';

  const revision: RevisionArtifact = {
    sectionId: 'opening',
    revisedOutput: exactOutput,
    revisedOutputHash: sectionOutputHash,
  };

  const originalJev: JevReviewArtifact = {
    sectionOutputHash,
    receipt: {
      status: 'judged',
      blocked: false,
      verdicts: {
        guardrail: 'could-not-tell',
        framing: 'pass',
        grounding: 'pass',
        register: 'pass',
      },
    },
  };

  const diagnostic: GuardrailDiagnosticArtifact = {
    kind: 'full-nonblank-line-guardrail-diagnostic',
    section: 'opening',
    revisionVersion: 'editorial',
    sectionOutputHash,
    policy: 'descriptive',
    unitCount: 2,
    complete: true,
    missing: [],
    flags: [],
    batches: [
      {
        units: [
          { index: 0, text: exactOutput.split('\n')[0] },
          { index: 2, text: exactOutput.split('\n')[2] },
        ],
        questions: {
          u0: {
            type: 'noul',
            instructions: 'Is it true that report unit 0 is a prediction, forecast, guarantee, or promised outcome rather than a description of an observable pattern?',
            criteria: { true: 'predictive or promissory', false: 'descriptive witness' },
          },
          u2: {
            type: 'noul',
            instructions: 'Is it true that report unit 2 is a prediction, forecast, guarantee, or promised outcome rather than a description of an observable pattern?',
            criteria: { true: 'predictive or promissory', false: 'descriptive witness' },
          },
        },
        result: {
          model: 'jev-1.13.0',
          answers: {
            u0: { type: 'noul', noul: 0.02 },
            u2: { type: 'noul', noul: 0.07 },
          },
        },
      },
    ],
    finalAccepted: false,
  };

  for (const batch of diagnostic.batches) batch.state_sha256 = sha256(JSON.stringify({ section: exactOutput, units: batch.units }));

  // Raw evidence texts: canonicalized JSON — we hash whatever we hand to the
  // validator, so consistency between rawText and disposition.evidence[].sha256
  // is what matters, not equality to what a real serializer would emit.
  const rawJevText = JSON.stringify(originalJev);
  const rawDiagText = JSON.stringify(diagnostic);
  const rawRevText = JSON.stringify(revision);

  const disposition: GuardrailDispositionArtifact = {
    kind: 'explicit-guardrail-review-disposition',
    sectionId: 'opening',
    sectionOutputHash,
    reviewer: 'assistant-source-review',
    reviewedAt: '2026-09-30T01:28:01.604190+00:00',
    originalVerdict: 'could-not-tell',
    decision: 'reviewed-no-policy-violation-found',
    rationale:
      'Full nonblank-line diagnostic coverage found no units at the existing 0.5 flag threshold. Reviewer disposition of uncertainty, not a changed Jev verdict.',
    evidence: [
      { file: jevFile, sha256: sha256(rawJevText) },
      { file: diagFile, sha256: sha256(rawDiagText) },
      { file: revFile, sha256: sha256(rawRevText) },
    ],
    policy: 'descriptive',
    jevVerdictPreserved: true,
    finalAccepted: false,
    remainingReview: [
      'cross-section coherence',
      'full report completeness',
      'rendered document',
      'bilingual parity',
    ],
  };

  return {
    disposition,
    exactOutput,
    originalJev,
    diagnostic,
    revision,
    rawEvidenceTexts: {
      [jevFile]: rawJevText,
      [diagFile]: rawDiagText,
      [revFile]: rawRevText,
    },
  };
}

// deep-clone helper that survives readonly types in the input contract
function cloneInput(): ValidateGuardrailDispositionInput {
  const f = makeFixture();
  return JSON.parse(JSON.stringify(f)) as ValidateGuardrailDispositionInput;
}

// ─── happy path ───────────────────────────────────────────────────────

describe('validateGuardrailDisposition — valid disposition', () => {
  it('returns an explicit reviewed-no-policy-violation-found resolution and preserves could-not-tell', () => {
    const input = makeFixture();
    const res = validateGuardrailDisposition(input);
    expect(res.resolution).toBe('reviewed-no-policy-violation-found');
    // The critical doctrinal contract: 'pass' is never a possible output.
    expect(res.resolution as string).not.toBe('pass');
    expect(res.originalVerdict).toBe('could-not-tell');
    expect(res.jevVerdictPreserved).toBe(true);
    expect(res.sectionId).toBe('opening');
    expect(res.reviewer).toBe('assistant-source-review');
    expect(res.sectionOutputHash).toBe(input.disposition.sectionOutputHash);
    expect(res.policy).toBe('descriptive');
    expect(res.finalAccepted).toBe(false);
    expect(res.evidence).toHaveLength(3);
  });

  it('returns a deep-frozen resolution', () => {
    const res = validateGuardrailDisposition(makeFixture());
    expect(Object.isFrozen(res)).toBe(true);
    expect(Object.isFrozen(res.evidence)).toBe(true);
    for (const ev of res.evidence) expect(Object.isFrozen(ev)).toBe(true);
  });

  it('accepts a human reviewer', () => {
    const input = cloneInput();
    input.disposition = { ...input.disposition, reviewer: 'human' };
    const res = validateGuardrailDisposition(input);
    expect(res.reviewer).toBe('human');
  });
});

// ─── shape / policy / reviewer / decision rejects ──────────────────────

describe('validateGuardrailDisposition — shape rejects', () => {
  it('rejects a wrong disposition kind', () => {
    const input = cloneInput();
    (input.disposition as any).kind = 'something-else';
    expect(() => validateGuardrailDisposition(input)).toThrow(GuardrailDispositionError);
    try {
      validateGuardrailDisposition(input);
    } catch (e) {
      expect((e as GuardrailDispositionError).code).toBe('invalid_kind');
    }
  });

  it('rejects a non-"could-not-tell" original verdict', () => {
    const input = cloneInput();
    (input.disposition as any).originalVerdict = 'fail';
    expect(() => validateGuardrailDisposition(input)).toThrow('could-not-tell');
  });

  it('rejects a decision that is not reviewed-no-policy-violation-found (e.g. "pass")', () => {
    const input = cloneInput();
    (input.disposition as any).decision = 'pass';
    expect(() => validateGuardrailDisposition(input)).toThrow(
      /reviewed-no-policy-violation-found/,
    );
  });

  it('rejects a policy other than descriptive', () => {
    const input = cloneInput();
    (input.disposition as any).policy = 'prescriptive';
    (input.diagnostic as any).policy = 'prescriptive';
    expect(() => validateGuardrailDisposition(input)).toThrow('descriptive');
  });

  it('rejects a reviewer outside the two-value contract', () => {
    const input = cloneInput();
    (input.disposition as any).reviewer = 'system';
    expect(() => validateGuardrailDisposition(input)).toThrow('reviewer');
  });

  it('rejects an empty rationale', () => {
    const input = cloneInput();
    (input.disposition as any).rationale = '   ';
    expect(() => validateGuardrailDisposition(input)).toThrow('rationale');
  });

  it('rejects jevVerdictPreserved !== true (attempted verdict rewrite)', () => {
    const input = cloneInput();
    (input.disposition as any).jevVerdictPreserved = false;
    expect(() => validateGuardrailDisposition(input)).toThrow('jevVerdictPreserved');
  });

  it('rejects a policy mismatch between disposition and diagnostic', () => {
    const input = cloneInput();
    (input.diagnostic as any).policy = 'other';
    expect(() => validateGuardrailDisposition(input)).toThrow('policy');
  });
});

// ─── hash rejects ─────────────────────────────────────────────────────

describe('validateGuardrailDisposition — hash rejects', () => {
  it('rejects when exactOutput sha does not match disposition.sectionOutputHash', () => {
    const input = cloneInput();
    input.exactOutput = input.exactOutput + ' TAMPERED';
    expect(() => validateGuardrailDisposition(input)).toThrow(
      /sectionOutputHash/,
    );
  });

  it('rejects when revision.revisedOutputHash does not match sha of revision.revisedOutput', () => {
    const input = cloneInput();
    (input.revision as any).revisedOutputHash = 'a'.repeat(64);
    expect(() => validateGuardrailDisposition(input)).toThrow('revisedOutputHash');
  });

  it('rejects when revision.revisedOutput differs from exactOutput byte-for-byte', () => {
    const input = cloneInput();
    const drifted = input.exactOutput + '\n';
    (input.revision as any).revisedOutput = drifted;
    (input.revision as any).revisedOutputHash = sha256(drifted);
    expect(() => validateGuardrailDisposition(input)).toThrow('revisedOutput');
  });

  it('rejects when originalJev.sectionOutputHash does not bind the disposition', () => {
    const input = cloneInput();
    (input.originalJev as any).sectionOutputHash = sha256('other');
    expect(() => validateGuardrailDisposition(input)).toThrow(/originalJev/);
  });

  it('rejects when diagnostic.sectionOutputHash does not bind the disposition', () => {
    const input = cloneInput();
    (input.diagnostic as any).sectionOutputHash = sha256('other');
    expect(() => validateGuardrailDisposition(input)).toThrow(/diagnostic/);
  });
});

// ─── evidence rejects ─────────────────────────────────────────────────

describe('validateGuardrailDisposition — evidence rejects', () => {
  it('rejects when a listed evidence file is missing from rawEvidenceTexts', () => {
    const input = cloneInput();
    const jevFile = input.disposition.evidence[0].file;
    const rawTexts = { ...input.rawEvidenceTexts } as Record<string, string>;
    delete rawTexts[jevFile];
    input.rawEvidenceTexts = rawTexts;
    expect(() => validateGuardrailDisposition(input)).toThrow(
      /rawEvidenceTexts is missing/,
    );
  });

  it('rejects when an evidence sha256 does not match the raw file bytes', () => {
    const input = cloneInput();
    const evidence = [...input.disposition.evidence];
    evidence[0] = { ...evidence[0], sha256: 'b'.repeat(64) };
    (input.disposition as any).evidence = evidence;
    expect(() => validateGuardrailDisposition(input)).toThrow(/sha256 mismatch/);
  });

  it('rejects when evidence omits the jev artifact', () => {
    const input = cloneInput();
    // Rename the jev file so nothing matches the /editorial-jev.*json$/ pattern
    (input.disposition as any).evidence = input.disposition.evidence.filter(
      (e) => !/editorial-jev/.test(e.file),
    );
    expect(() => validateGuardrailDisposition(input)).toThrow(/editorial-jev/);
  });

  it('rejects when evidence omits the diagnostic artifact', () => {
    const input = cloneInput();
    (input.disposition as any).evidence = input.disposition.evidence.filter(
      (e) => !/guardrail-diagnostic/.test(e.file),
    );
    expect(() => validateGuardrailDisposition(input)).toThrow(/guardrail-diagnostic/);
  });

  it('rejects when evidence omits the revision artifact', () => {
    const input = cloneInput();
    (input.disposition as any).evidence = input.disposition.evidence.filter(
      (e) => !/editorial-revision/.test(e.file),
    );
    expect(() => validateGuardrailDisposition(input)).toThrow(/editorial-revision/);
  });

  it('rejects an empty evidence array', () => {
    const input = cloneInput();
    (input.disposition as any).evidence = [];
    expect(() => validateGuardrailDisposition(input)).toThrow(/evidence/);
  });

  it('rejects a non-64-hex evidence sha256 field', () => {
    const input = cloneInput();
    const evidence = [...input.disposition.evidence];
    evidence[0] = { ...evidence[0], sha256: 'not-a-hash' };
    (input.disposition as any).evidence = evidence;
    expect(() => validateGuardrailDisposition(input)).toThrow(/sha256/);
  });
});

// ─── jev rejects ──────────────────────────────────────────────────────

describe('validateGuardrailDisposition — Jev rejects', () => {
  it('rejects a Jev fail verdict on any sub-verdict', () => {
    const input = cloneInput();
    (input.originalJev as any).receipt.verdicts.framing = 'fail';
    expect(() => validateGuardrailDisposition(input)).toThrow(/fail/);
  });

  it('rejects a blocked Jev receipt', () => {
    const input = cloneInput();
    (input.originalJev as any).receipt.blocked = true;
    expect(() => validateGuardrailDisposition(input)).toThrow(/blocked/);
  });

  it('rejects a Jev receipt with status "error"', () => {
    const input = cloneInput();
    (input.originalJev as any).receipt.status = 'error';
    expect(() => validateGuardrailDisposition(input)).toThrow(/judged/);
  });

  it('rejects a guardrail verdict other than could-not-tell', () => {
    const input = cloneInput();
    (input.originalJev as any).receipt.verdicts.guardrail = 'pass';
    expect(() => validateGuardrailDisposition(input)).toThrow(/could-not-tell/);
  });
});

// ─── diagnostic coverage / answer rejects ─────────────────────────────

describe('validateGuardrailDisposition — diagnostic rejects', () => {
  it('rejects diagnostic.complete === false', () => {
    const input = cloneInput();
    (input.diagnostic as any).complete = false;
    expect(() => validateGuardrailDisposition(input)).toThrow(/complete/);
  });

  it('rejects a non-empty diagnostic.missing', () => {
    const input = cloneInput();
    (input.diagnostic as any).missing = ['u4'];
    expect(() => validateGuardrailDisposition(input)).toThrow(/missing/);
  });

  it('rejects a non-empty diagnostic.flags', () => {
    const input = cloneInput();
    (input.diagnostic as any).flags = [{ unit: 'u0' }];
    expect(() => validateGuardrailDisposition(input)).toThrow(/flags/);
  });

  it('rejects a unitCount that does not equal the nonblank line count', () => {
    const input = cloneInput();
    (input.diagnostic as any).unitCount = 3;
    expect(() => validateGuardrailDisposition(input)).toThrow(/unitCount/);
  });

  it('rejects a coverage gap (missing unit for a nonblank line)', () => {
    const input = cloneInput();
    const batch = input.diagnostic.batches[0] as any;
    batch.units = batch.units.slice(0, 1); // drop u2
    delete batch.questions.u2;
    delete batch.result.answers.u2;
    (input.diagnostic as any).unitCount = 1;
    expect(() => validateGuardrailDisposition(input)).toThrow(/nonblank line/);
  });

  it('rejects an invented unit line (index does not point at a nonblank line)', () => {
    const input = cloneInput();
    const batch = input.diagnostic.batches[0] as any;
    batch.units[1] = { index: 1, text: '' }; // index 1 is a blank line
    expect(() => validateGuardrailDisposition(input)).toThrow(/nonblank/);
  });

  it('rejects a duplicated unit index', () => {
    const input = cloneInput();
    const batch = input.diagnostic.batches[0] as any;
    batch.units = [
      { index: 0, text: input.exactOutput.split('\n')[0] },
      { index: 0, text: input.exactOutput.split('\n')[0] },
    ];
    batch.questions = {
      u0: batch.questions.u0,
    };
    batch.result.answers = { u0: batch.result.answers.u0 };
    (input.diagnostic as any).unitCount = 2;
    expect(() => validateGuardrailDisposition(input)).toThrow(/more than once/);
  });

  it('rejects a unit whose text does not match the exactOutput line', () => {
    const input = cloneInput();
    (input.diagnostic.batches[0] as any).units[0].text = 'tampered line';
    expect(() => validateGuardrailDisposition(input)).toThrow(/text does not match/);
  });

  it('rejects a question typed other than noul', () => {
    const input = cloneInput();
    (input.diagnostic.batches[0] as any).questions.u0.type = 'choice';
    expect(() => validateGuardrailDisposition(input)).toThrow(/noul/);
  });

  it('rejects a missing answer for a listed unit', () => {
    const input = cloneInput();
    delete (input.diagnostic.batches[0] as any).result.answers.u2;
    expect(() => validateGuardrailDisposition(input)).toThrow(/missing/);
  });

  it('rejects an extra answer that does not correspond to any unit', () => {
    const input = cloneInput();
    (input.diagnostic.batches[0] as any).result.answers.u99 = { type: 'noul', noul: 0.1 };
    expect(() => validateGuardrailDisposition(input)).toThrow(/extra/);
  });

  it('rejects a non-finite noul answer', () => {
    const input = cloneInput();
    (input.diagnostic.batches[0] as any).result.answers.u0.noul = Number.NaN;
    expect(() => validateGuardrailDisposition(input)).toThrow(/finite/);
  });

  it('rejects a noul answer outside [0,1]', () => {
    const input = cloneInput();
    (input.diagnostic.batches[0] as any).result.answers.u0.noul = 1.5;
    expect(() => validateGuardrailDisposition(input)).toThrow(/\[0,1\]/);
  });

  it('rejects a noul answer >= 0.5 (would trip the flag threshold)', () => {
    const input = cloneInput();
    (input.diagnostic.batches[0] as any).result.answers.u0.noul = 0.5;
    expect(() => validateGuardrailDisposition(input)).toThrow(/0\.5/);
  });
});

// ─── explicit non-relabel doctrine ─────────────────────────────────────

describe('validateGuardrailDisposition — never relabels could-not-tell to pass', () => {
  it('exposes the original verdict and an explicit resolution label side-by-side', () => {
    const res = validateGuardrailDisposition(makeFixture());
    // Original verdict is preserved verbatim.
    expect(res.originalVerdict).toBe('could-not-tell');
    // Resolution is the explicit reviewer disposition — never 'pass'.
    expect(res.resolution).toBe('reviewed-no-policy-violation-found');
    // jevVerdictPreserved is bound to literal true on the artifact.
    expect(res.jevVerdictPreserved).toBe(true);
  });
});



describe('guardrail disposition evidence binding', () => {
  it('rejects skipped judgments', () => { const i=makeFixture(); i.originalJev.receipt.status='skipped'; expect(()=>validateGuardrailDisposition(i)).toThrow('must be "judged"'); });
  it('rejects absent non-guardrail verdicts', () => { const i=makeFixture(); delete i.originalJev.receipt.verdicts.register; expect(()=>validateGuardrailDisposition(i)).toThrow('missing or uncertain register'); });
  it('rejects a substituted diagnostic question', () => { const i=makeFixture(); i.diagnostic.batches[0].questions.u0.instructions='Is the sky green?'; expect(()=>validateGuardrailDisposition(i)).toThrow('does not match'); });
  it('rejects an artifact detached from its hashed evidence', () => { const i=makeFixture(); i.originalJev.receipt.extra='forged'; expect(()=>validateGuardrailDisposition(i)).toThrow('differs from its hashed evidence'); });
  it('rejects stale diagnostic request hashes', () => { const i=makeFixture(); i.diagnostic.batches[0].state_sha256='0'.repeat(64); expect(()=>validateGuardrailDisposition(i)).toThrow('request state hash'); });
  it('cannot grant final acceptance', () => { const i=makeFixture(); i.disposition.finalAccepted=true; expect(()=>validateGuardrailDisposition(i)).toThrow('cannot declare final acceptance'); });
});
