// ─── Resume Validation Tests ──────────────────────────────────────────
// Tests for validateResumeDir().
//
// Coverage (per spec):
//  1. Tampered output_hash (raw.synthesis changed) → rejects entire prior run
//  2. Tampered aletheios_hash (raw.aletheios changed) → rejects entire prior run
//  3. Tampered pichet_hash (raw.pichet changed) → rejects entire prior run
//  4. Changed inputs (inputHash mismatch) → rejects before reading any receipt
//  5. Changed language → rejects before reading any receipt
//  6. Changed subject → rejects before reading any receipt
//  7. Changed persona hash → rejects entire prior run
//  8. Changed runtime contract hash → rejects before reading any receipt
//  9. Failed section in prefix → prefix truncated before failed section (not a hard reject)
// 10. Missing receipt mid-sequence → prefix truncated at missing section
// 11. Jev-blocked receipt → prefix truncated before blocked section
// 12. Happy-path: successful continuation uses reused sections without re-running them
// 13. synthesis_input_hash mismatch → rejects entire prior run
// 14. passages_hash inconsistency (top-level vs retrieval) → rejects entire prior run

import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { promises as fs } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { createHash, randomBytes } from 'node:crypto';
import { validateResumeDir } from './resume-validation.js';
import type { CurrentRunIdentity, RunInputsIdentity } from './resume-validation.js';
import type { SectionExecutionReceipt } from './reference-execution.js';

// ─── Fixtures ─────────────────────────────────────────────────────────

function sha256(text: string): string {
  return createHash('sha256').update(text, 'utf8').digest('hex');
}

const ALETHEIOS_IDENTITY = '# aletheios IDENTITY\nYou are aletheios, the structural interpreter.';
const PICHET_IDENTITY = '# pichet IDENTITY\nYou are pichet, the experiential reflector.';
const ALETHEIOS_HASH = sha256(ALETHEIOS_IDENTITY);
const PICHET_HASH = sha256(PICHET_IDENTITY);

const RUNTIME_FILES = [
  'packages/witness-pipeline/modes/integrated-kundali-reference.md',
  'packages/witness-pipeline/src/orchestrator/integrated.ts',
  'packages/witness-pipeline/src/orchestrator/reference-execution.ts',
];
const RUNTIME_HASHES: Record<string, string> = Object.fromEntries(
  RUNTIME_FILES.map((f) => [f, sha256(`content-of-${f}`)]),
);

const ENGINE_JSON = JSON.stringify([{ engine_id: 'human-design', result: { hd_type: 'Generator' } }]);
const ENGINE_HASH = sha256(ENGINE_JSON);

const ACCOUNT_ID = 'a'.repeat(32);

function makeCurrentIdentity(overrides: Partial<CurrentRunIdentity> = {}): CurrentRunIdentity {
  return {
    sourceTexts: { 'sw:hd:type:generator:desc': 'passage1 text content', 'sw:gk:4:shadow_description': 'passage2 text content' },
    inputHash: ENGINE_HASH,
    subject: 'sheshnarayan',
    language: 'en',
    accountId: ACCOUNT_ID,
    index: 'witness-wisdom-corpus',
    aletheiosPersonaHash: ALETHEIOS_HASH,
    pichetPersonaHash: PICHET_HASH,
    runtimeHashes: { ...RUNTIME_HASHES },
    ...overrides,
  };
}

function makeInputsJson(overrides: Partial<RunInputsIdentity> = {}): RunInputsIdentity {
  return {
    inputHash: ENGINE_HASH,
    subject: 'sheshnarayan',
    language: 'en',
    accountId: ACCOUNT_ID,
    index: 'witness-wisdom-corpus',
    personaHashes: [ALETHEIOS_HASH, PICHET_HASH],
    runtimeHashes: { ...RUNTIME_HASHES },
    ...overrides,
  };
}

function makeReceipt(sectionId: string, overrides: Partial<SectionExecutionReceipt> = {}): SectionExecutionReceipt {
  const aletheiosRaw = `Aletheios output for ${sectionId} — detailed structural analysis with sufficient length.`;
  const pichetRaw = `Pichet output for ${sectionId} — lived experiential reflection with sufficient length.`;
  const synthesisRaw = `## ${sectionId}\n\nSynthesis of ${sectionId}. This is the accepted output that was generated.\n\nOne open question: how does the subject orient to this?`;
  const synthesisInputText = `[prompt]\n\n## Aletheios (structural interpretation)\n${aletheiosRaw}\n\n## Pichet (experiential reflection)\n${pichetRaw}\n\nWrite the reconciled synthesis section now.`;
  const passagesText = 'passage1 text content\npassage2 text content';
  const passagesHash = sha256(passagesText);

  return {
    section_id: sectionId,
    outcome: 'ok',
    source_ids: ['sw:hd:type:generator:desc', 'sw:gk:4:shadow_description'],
    source_hashes: [sha256('passage1 text content'), sha256('passage2 text content')],
    passages_hash: passagesHash,
    aletheios_hash: sha256(aletheiosRaw),
    pichet_hash: sha256(pichetRaw),
    synthesis_input_hash: sha256(synthesisInputText),
    synthesis_input_text: synthesisInputText,
    prior_section_content_hashes: {},
    output_hash: sha256(synthesisRaw),
    aletheios_persona: { name: 'aletheios', sourcePath: '/agents/aletheios/IDENTITY.md', sourceHash: ALETHEIOS_HASH },
    pichet_persona: { name: 'pichet', sourcePath: '/agents/pichet/IDENTITY.md', sourceHash: PICHET_HASH },
    raw: { aletheios: aletheiosRaw, pichet: pichetRaw, synthesis: synthesisRaw },
    retrieval: {
      section_id: sectionId,
      state: 'success',
      count: 2,
      source_ids: ['sw:hd:type:generator:desc', 'sw:gk:4:shadow_description'],
      passages_hash: passagesHash,
    },
    attempts: [{ attempt: 1, outcome: 'ok', aletheios_raw: aletheiosRaw, pichet_raw: pichetRaw, synthesis_raw: synthesisRaw }],
    ...overrides,
  };
}

// ─── Test helpers ──────────────────────────────────────────────────────

let tmpDir: string;

beforeEach(async () => {
  tmpDir = join(tmpdir(), `resume-test-${randomBytes(6).toString('hex')}`);
  await fs.mkdir(tmpDir, { recursive: true });
});

afterEach(async () => {
  await fs.rm(tmpDir, { recursive: true, force: true });
});

async function writeRun(inputsJson: RunInputsIdentity, receipts: Record<string, SectionExecutionReceipt>): Promise<string> {
  const runDir = join(tmpDir, `run-${randomBytes(4).toString('hex')}`);
  await fs.mkdir(runDir, { recursive: true });
  await fs.writeFile(join(runDir, 'inputs.json'), JSON.stringify(inputsJson));
  for (const [sectionId, receipt] of Object.entries(receipts)) {
    await fs.writeFile(join(runDir, `${sectionId}.receipt.json`), JSON.stringify(receipt));
  }
  return runDir;
}

// ─── Tests ────────────────────────────────────────────────────────────

describe('resume-validation: identity block checks (reject before reading receipts)', () => {
  it('rejects when inputHash differs (subject data changed)', async () => {
    const runDir = await writeRun(
      makeInputsJson({ inputHash: 'different-' + sha256('other-data') }),
      { opening: makeReceipt('opening') },
    );
    const result = await validateResumeDir(runDir, ['opening', 'part1'], makeCurrentIdentity());
    expect(result.valid).toBe(false);
    expect('reason' in result ? result.reason : undefined).toContain('inputHash mismatch');
  });

  it('rejects when language differs', async () => {
    const runDir = await writeRun(
      makeInputsJson({ language: 'fr' }),
      { opening: makeReceipt('opening') },
    );
    const result = await validateResumeDir(runDir, ['opening', 'part1'], makeCurrentIdentity({ language: 'en' }));
    expect(result.valid).toBe(false);
    expect('reason' in result ? result.reason : undefined).toContain('language mismatch');
  });

  it('rejects when subject differs', async () => {
    const runDir = await writeRun(
      makeInputsJson({ subject: 'gary-abitbol' }),
      { opening: makeReceipt('opening') },
    );
    const result = await validateResumeDir(runDir, ['opening'], makeCurrentIdentity({ subject: 'sheshnarayan' }));
    expect(result.valid).toBe(false);
    expect('reason' in result ? result.reason : undefined).toContain('subject mismatch');
  });

  it('rejects when a runtime contract file hash changed', async () => {
    const alteredHashes = { ...RUNTIME_HASHES, 'packages/witness-pipeline/src/orchestrator/integrated.ts': sha256('different-content') };
    const runDir = await writeRun(
      makeInputsJson({ runtimeHashes: alteredHashes }),
      { opening: makeReceipt('opening') },
    );
    const result = await validateResumeDir(runDir, ['opening'], makeCurrentIdentity());
    expect(result.valid).toBe(false);
    expect('reason' in result ? result.reason : undefined).toContain('runtimeHashes');
    expect('reason' in result ? result.reason : undefined).toContain('integrated.ts');
  });

  it('rejects when current has runtime keys absent in prior (expanded contract)', async () => {
    const smallerHashes = { 'packages/witness-pipeline/modes/integrated-kundali-reference.md': RUNTIME_HASHES['packages/witness-pipeline/modes/integrated-kundali-reference.md'] };
    const runDir = await writeRun(
      makeInputsJson({ runtimeHashes: smallerHashes }),
      { opening: makeReceipt('opening') },
    );
    const result = await validateResumeDir(runDir, ['opening'], makeCurrentIdentity());
    expect(result.valid).toBe(false);
    expect('reason' in result ? result.reason : undefined).toContain('runtimeHashes');
  });

  it('rejects when aletheios persona hash changed', async () => {
    const runDir = await writeRun(
      makeInputsJson({ personaHashes: [sha256('old-aletheios-text'), PICHET_HASH] }),
      { opening: makeReceipt('opening') },
    );
    const result = await validateResumeDir(runDir, ['opening'], makeCurrentIdentity());
    expect(result.valid).toBe(false);
    expect('reason' in result ? result.reason : undefined).toContain('aletheios persona hash mismatch');
  });

  it('rejects when pichet persona hash changed', async () => {
    const runDir = await writeRun(
      makeInputsJson({ personaHashes: [ALETHEIOS_HASH, sha256('old-pichet-text')] }),
      { opening: makeReceipt('opening') },
    );
    const result = await validateResumeDir(runDir, ['opening'], makeCurrentIdentity());
    expect(result.valid).toBe(false);
    expect('reason' in result ? result.reason : undefined).toContain('pichet persona hash mismatch');
  });

  it('rejects when inputs.json is missing entirely', async () => {
    const runDir = join(tmpDir, 'empty-run');
    await fs.mkdir(runDir, { recursive: true });
    const result = await validateResumeDir(runDir, ['opening'], makeCurrentIdentity());
    expect(result.valid).toBe(false);
    expect('reason' in result ? result.reason : undefined).toContain('cannot read prior inputs.json');
  });
});

describe('resume-validation: tampered receipt detection (reject entire prior run)', () => {
  it('rejects entire run when output_hash does not match raw.synthesis', async () => {
    const receipt = makeReceipt('opening');
    // Tamper: change raw.synthesis but leave output_hash unchanged
    const tamperedReceipt = { ...receipt, raw: { ...receipt.raw, synthesis: receipt.raw.synthesis + ' TAMPERED' } };
    const runDir = await writeRun(makeInputsJson(), { opening: tamperedReceipt });

    const result = await validateResumeDir(runDir, ['opening', 'part1'], makeCurrentIdentity());
    expect(result.valid).toBe(false);
    expect('reason' in result ? result.reason : undefined).toContain('output_hash mismatch');
    expect('reason' in result ? result.reason : undefined).toContain('opening');
    expect('reason' in result ? result.reason : undefined).toContain('tampered or corrupted');
  });

  it('rejects entire run when aletheios_hash does not match raw.aletheios', async () => {
    const receipt = makeReceipt('opening');
    const tamperedReceipt = { ...receipt, raw: { ...receipt.raw, aletheios: receipt.raw.aletheios + ' TAMPERED' } };
    const runDir = await writeRun(makeInputsJson(), { opening: tamperedReceipt });

    const result = await validateResumeDir(runDir, ['opening', 'part1'], makeCurrentIdentity());
    expect(result.valid).toBe(false);
    expect('reason' in result ? result.reason : undefined).toContain('aletheios_hash does not match raw.aletheios');
    expect('reason' in result ? result.reason : undefined).toContain('tampered');
  });

  it('rejects entire run when pichet_hash does not match raw.pichet', async () => {
    const receipt = makeReceipt('opening');
    const tamperedReceipt = { ...receipt, raw: { ...receipt.raw, pichet: receipt.raw.pichet + ' TAMPERED' } };
    const runDir = await writeRun(makeInputsJson(), { opening: tamperedReceipt });

    const result = await validateResumeDir(runDir, ['opening', 'part1'], makeCurrentIdentity());
    expect(result.valid).toBe(false);
    expect('reason' in result ? result.reason : undefined).toContain('pichet_hash does not match raw.pichet');
    expect('reason' in result ? result.reason : undefined).toContain('tampered');
  });

  it('rejects entire run when synthesis_input_hash does not match synthesis_input_text', async () => {
    const receipt = makeReceipt('opening');
    // Tamper: change synthesis_input_text but not the hash
    const tamperedReceipt = { ...receipt, synthesis_input_text: receipt.synthesis_input_text + ' INJECTED CONTENT' };
    const runDir = await writeRun(makeInputsJson(), { opening: tamperedReceipt });

    const result = await validateResumeDir(runDir, ['opening', 'part1'], makeCurrentIdentity());
    expect(result.valid).toBe(false);
    expect('reason' in result ? result.reason : undefined).toContain('synthesis_input_hash does not match synthesis_input_text');
    expect('reason' in result ? result.reason : undefined).toContain('tampered');
  });

  it('rejects entire run when receipt-level passages_hash != retrieval.passages_hash', async () => {
    const receipt = makeReceipt('opening');
    // Tamper: change only the top-level passages_hash to create an inconsistency
    const tamperedReceipt = { ...receipt, passages_hash: sha256('different-passages') };
    const runDir = await writeRun(makeInputsJson(), { opening: tamperedReceipt });

    const result = await validateResumeDir(runDir, ['opening', 'part1'], makeCurrentIdentity());
    expect(result.valid).toBe(false);
    expect('reason' in result ? result.reason : undefined).toContain('passages_hash');
    expect('reason' in result ? result.reason : undefined).toContain('inconsistency');
  });

  it('rejects entire run when aletheios_persona.sourceHash in receipt differs from current persona', async () => {
    const receipt = makeReceipt('opening');
    const tamperedReceipt = {
      ...receipt,
      aletheios_persona: { ...receipt.aletheios_persona, sourceHash: sha256('old-aletheios') },
    };
    const runDir = await writeRun(makeInputsJson(), { opening: tamperedReceipt });

    const result = await validateResumeDir(runDir, ['opening', 'part1'], makeCurrentIdentity());
    expect(result.valid).toBe(false);
    expect('reason' in result ? result.reason : undefined).toContain('aletheios_persona.sourceHash');
    expect('reason' in result ? result.reason : undefined).toContain('persona changed');
  });

  it('rejects entire run when section_id in receipt file does not match filename', async () => {
    const receipt = makeReceipt('WRONG_SECTION_ID');
    const runDir = await writeRun(makeInputsJson(), { opening: receipt });

    const result = await validateResumeDir(runDir, ['opening', 'part1'], makeCurrentIdentity());
    expect(result.valid).toBe(false);
    expect('reason' in result ? result.reason : undefined).toContain('malformed receipt');
  });
});

describe('resume-validation: prefix truncation (not hard reject)', () => {
  it('truncates prefix at first failed section, does not include it in reused', async () => {
    const receipts = {
      opening: makeReceipt('opening'),
      part1: makeReceipt('part1', { outcome: 'failed', raw: { aletheios: '', pichet: '', synthesis: '' }, output_hash: '' }),
      part2: makeReceipt('part2'),
    };
    const runDir = await writeRun(makeInputsJson(), receipts);

    const result = await validateResumeDir(runDir, ['opening', 'part1', 'part2'], makeCurrentIdentity());
    // Should succeed with only opening reused (part1 is failed, part2 not included)
    expect(result.valid).toBe(true);
    if (!result.valid) return;
    expect(result.reusedSections.map(s => s.section_id)).toEqual(['opening']);
    // part2 should NOT be included (non-contiguous)
  });

  it('truncates prefix at missing receipt mid-sequence', async () => {
    const receipts = {
      opening: makeReceipt('opening'),
      // part1 receipt file is absent
      part2: makeReceipt('part2'),
    };
    const runDir = await writeRun(makeInputsJson(), receipts);

    const result = await validateResumeDir(runDir, ['opening', 'part1', 'part2'], makeCurrentIdentity());
    expect(result.valid).toBe(true);
    if (!result.valid) return;
    expect(result.reusedSections.map(s => s.section_id)).toEqual(['opening']);
  });

  it('truncates prefix at Jev-blocked section, never reuses blocked output', async () => {
    const blockedReceipt = makeReceipt('part1', {
      jev: {
        pass_id: 'part1',
        mode: 'active' as const,
        status: 'judged' as const,
        blocked: true,
        disagreements: ['guardrail: predictive framing'],
        latency_ms: 100,
      },
    });
    const receipts = {
      opening: makeReceipt('opening'),
      part1: blockedReceipt,
      part2: makeReceipt('part2'),
    };
    const runDir = await writeRun(makeInputsJson(), receipts);

    const result = await validateResumeDir(runDir, ['opening', 'part1', 'part2'], makeCurrentIdentity());
    expect(result.valid).toBe(true);
    if (!result.valid) return;
    expect(result.reusedSections.map(s => s.section_id)).toEqual(['opening']);
    // part1 was Jev-blocked — never reused
    // part2 not included (non-contiguous)
  });

  it('returns valid:false when no receipts exist at all', async () => {
    const runDir = await writeRun(makeInputsJson(), {});
    const result = await validateResumeDir(runDir, ['opening', 'part1'], makeCurrentIdentity());
    expect(result.valid).toBe(false);
    expect('reason' in result ? result.reason : undefined).toContain('receipt not found in prior run');
  });

  it('returns valid:false when first section is immediately failed', async () => {
    const receipts = {
      opening: makeReceipt('opening', { outcome: 'failed', raw: { aletheios: '', pichet: '', synthesis: '' }, output_hash: '' }),
    };
    const runDir = await writeRun(makeInputsJson(), receipts);
    const result = await validateResumeDir(runDir, ['opening', 'part1'], makeCurrentIdentity());
    expect(result.valid).toBe(false);
    expect('reason' in result ? result.reason : undefined).toContain('no reusable sections found');
  });
});

describe('resume-validation: happy-path successful continuation', () => {
  it('returns reusedSections with correct output text for a fully successful prior prefix', async () => {
    const openingReceipt = makeReceipt('opening');
    const part1Receipt = makeReceipt('part1');

    const runDir = await writeRun(makeInputsJson(), {
      opening: openingReceipt,
      part1: part1Receipt,
      // part2 is absent — simulates the point where prior run stopped
    });

    const result = await validateResumeDir(
      runDir,
      ['opening', 'part1', 'part2', 'part3'],
      makeCurrentIdentity(),
    );

    expect(result.valid).toBe(true);
    if (!result.valid) return;
    expect(result.reusedSections).toHaveLength(2);
    expect(result.reusedSections[0].section_id).toBe('opening');
    expect(result.reusedSections[1].section_id).toBe('part1');

    // acceptedOutput must be the raw synthesis text (the actual accepted output)
    expect(result.reusedSections[0].acceptedOutput).toBe(openingReceipt.raw.synthesis);
    expect(result.reusedSections[1].acceptedOutput).toBe(part1Receipt.raw.synthesis);

    // synthesis_input_text threaded through for prior-context
    expect(result.reusedSections[0].synthesis_input_text).toBe(openingReceipt.synthesis_input_text);

    // Original receipts are preserved immutably
    expect(result.reusedSections[0].receipt).toEqual(openingReceipt);
    expect(result.reusedSections[1].receipt).toEqual(part1Receipt);

    // Provenance contains summary
    expect(result.provenance).toContain('opening');
    expect(result.provenance).toContain('part1');
    expect(result.priorRunDir).toBe(runDir);
  });

  it('accepts repaired (outcome=repaired) sections in the prefix', async () => {
    const repairedReceipt = makeReceipt('opening', { outcome: 'repaired' });
    const runDir = await writeRun(makeInputsJson(), { opening: repairedReceipt });

    const result = await validateResumeDir(runDir, ['opening', 'part1'], makeCurrentIdentity());
    expect(result.valid).toBe(true);
    if (!result.valid) return;
    expect(result.reusedSections[0].section_id).toBe('opening');
    expect(result.reusedSections[0].receipt.outcome).toBe('repaired');
  });

  it('accepts sections with non-blocked shadow Jev disagreements', async () => {
    const receipt = makeReceipt('opening', {
      jev: {
        pass_id: 'opening',
        mode: 'shadow' as const,
        status: 'judged' as const,
        blocked: false,
        disagreements: ['grounding: rubric-pass jev-fail'],
        latency_ms: 80,
      },
    });
    const runDir = await writeRun(makeInputsJson(), { opening: receipt });

    const result = await validateResumeDir(runDir, ['opening', 'part1'], makeCurrentIdentity());
    expect(result.valid).toBe(true);
    if (!result.valid) return;
    expect(result.reusedSections[0].section_id).toBe('opening');
    // shadow disagreements preserved but not blocking
    expect(result.reusedSections[0].receipt.jev?.disagreements).toContain('grounding: rubric-pass jev-fail');
  });

  it('returns all sections when every section has a successful receipt', async () => {
    const sectionIds = ['opening', 'part1', 'part2', 'part3'];
    const receipts = Object.fromEntries(sectionIds.map(id => [id, makeReceipt(id)]));
    const runDir = await writeRun(makeInputsJson(), receipts);

    const result = await validateResumeDir(runDir, sectionIds, makeCurrentIdentity());
    expect(result.valid).toBe(true);
    if (!result.valid) return;
    expect(result.reusedSections.map(s => s.section_id)).toEqual(sectionIds);
  });
});

describe('resume source and parse integrity', () => {
  it('rejects changed retrieved text even when receipt hash fields agree', async () => {
    const runDir = await writeRun(makeInputsJson(), { opening: makeReceipt('opening') });
    const result = await validateResumeDir(runDir, ['opening'], makeCurrentIdentity({ sourceTexts: {} }));
    expect(result.valid).toBe(false);
  });
  it('rejects malformed JSON after an intact prefix instead of hiding corruption', async () => {
    const runDir = await writeRun(makeInputsJson(), { opening: makeReceipt('opening') });
    await fs.writeFile(join(runDir, 'part1.receipt.json'), '{broken');
    const result = await validateResumeDir(runDir, ['opening', 'part1'], makeCurrentIdentity());
    expect(result.valid).toBe(false);
  });
});

it('continues with full reused text while calling the LLM only for the next section', async () => {
  const { IntegratedReadingOrchestrator } = await import('./integrated.js');
  const { parseModeDoc } = await import('../modes/parser.js');
  const mode = parseModeDoc(join(process.cwd(), 'modes/integrated-kundali-reference.md'));
  mode.frontmatter.pass_plan = mode.frontmatter.pass_plan.slice(0, 2).map(p => ({ ...p, target_words: 100 }));
  mode.sections[mode.frontmatter.pass_plan[1].template] = 'Write the next section with {{prior_pass}}.';
  const receipt = makeReceipt('opening');
  const calls: string[] = [];
  const checkpoints: string[] = [];
  const result = await new IntegratedReadingOrchestrator({
    mode, llm: async (_system, prompt) => { calls.push(prompt); return 'Supported reflection '.repeat(50); },
    reusedSections: new Map([['opening', { output: receipt.raw.synthesis, receipt }]]),
    referenceExecution: {
      enabled: true,
      aletheiosPersona: { name: 'aletheios', identityText: ALETHEIOS_IDENTITY, sourceHash: ALETHEIOS_HASH, sourcePath: '/aletheios' },
      pichetPersona: { name: 'pichet', identityText: PICHET_IDENTITY, sourceHash: PICHET_HASH, sourcePath: '/pichet' },
      retriever: async sectionId => ({ passages: [{ id: 'sw:test', source: 'test', text: 'Framework.' }], receipt: {
        section_id: sectionId, state: 'success', count: 1, source_ids: ['sw:test'], passages_hash: sha256('Framework.'),
      } }),
    },
    sectionCheckpoint: async r => { checkpoints.push(r.section_id); },
  }).run({ subjectNames: ['Test'], engineResultsBySubject: [[]], consciousnessLevel: 4, language: 'en' });
  expect(calls).toHaveLength(3);
  expect(calls[2]).toContain(receipt.raw.synthesis);
  expect(result.passes.map(p => p.id)).toEqual(['opening', 'part1']);
  expect(result.passes[0].output).toBe(receipt.raw.synthesis);
  expect(checkpoints).toEqual(['opening', 'part1']);
});

// ─── Mixed-source resume gate ─────────────────────────────────────────
// A prior receipt with primary_source_ids can only be safely resumed
// when the current run supplies the same primary registry (merged into
// `current.sourceTexts`). Missing text → actionable error. Matching
// text → the section is reused. This preserves the CF-only backwards
// compatibility (no primary_source_ids on the receipt keeps the old
// behaviour of the missing-text error message).

describe('resume-validation: mixed-source primary registry gate', () => {
  it('gives an actionable primary-registry error when a prior primary id is missing from current sourceTexts', async () => {
    const receipt = makeReceipt('opening', {
      source_ids: ['sw:hd:type:generator:desc', 'primary:rao2000:house:2'],
      source_hashes: [sha256('passage1 text content'), sha256('primary text content')],
      passages_hash: sha256(['passage1 text content', 'primary text content'].join('\n')),
      primary_source_ids: ['primary:rao2000:house:2'],
      cf_source_ids: ['sw:hd:type:generator:desc'],
      retrieval: {
        section_id: 'opening', state: 'success', count: 2,
        source_ids: ['sw:hd:type:generator:desc', 'primary:rao2000:house:2'],
        passages_hash: sha256(['passage1 text content', 'primary text content'].join('\n')),
      },
    });
    const runDir = await writeRun(makeInputsJson(), { opening: receipt });
    // Current identity supplies ONLY the CF text — primary text missing.
    const result = await validateResumeDir(runDir, ['opening'], makeCurrentIdentity({
      sourceTexts: { 'sw:hd:type:generator:desc': 'passage1 text content' },
    }));
    expect(result.valid).toBe(false);
    const reason = 'reason' in result ? result.reason : '';
    expect(reason).toContain('primary passage registry not supplied');
    expect(reason).toContain('--primary-registry');
    expect(reason).toContain('primary:rao2000:house:2');
  });

  it('accepts a prior mixed-source receipt when current sourceTexts merges the primary text with matching hash', async () => {
    const receipt = makeReceipt('opening', {
      source_ids: ['sw:hd:type:generator:desc', 'primary:rao2000:house:2'],
      source_hashes: [sha256('passage1 text content'), sha256('primary text content')],
      passages_hash: sha256(['passage1 text content', 'primary text content'].join('\n')),
      primary_source_ids: ['primary:rao2000:house:2'],
      cf_source_ids: ['sw:hd:type:generator:desc'],
      retrieval: {
        section_id: 'opening', state: 'success', count: 2,
        source_ids: ['sw:hd:type:generator:desc', 'primary:rao2000:house:2'],
        passages_hash: sha256(['passage1 text content', 'primary text content'].join('\n')),
      },
    });
    const runDir = await writeRun(makeInputsJson(), { opening: receipt });
    const result = await validateResumeDir(runDir, ['opening', 'part1'], makeCurrentIdentity({
      sourceTexts: {
        'sw:hd:type:generator:desc': 'passage1 text content',
        'primary:rao2000:house:2': 'primary text content',
      },
    }));
    expect(result.valid).toBe(true);
    if (result.valid) {
      expect(result.reusedSections.map(s => s.section_id)).toEqual(['opening']);
    }
  });

  it('keeps the pre-existing CF corpus-drift error message when the missing id is a CF id (no primary hint)', async () => {
    // Backwards compatibility: on a CF-only prior receipt, missing text
    // must not mention the primary-registry flag.
    const runDir = await writeRun(makeInputsJson(), { opening: makeReceipt('opening') });
    const result = await validateResumeDir(runDir, ['opening'], makeCurrentIdentity({
      sourceTexts: { /* second CF id missing */ 'sw:hd:type:generator:desc': 'passage1 text content' },
    }));
    expect(result.valid).toBe(false);
    const reason = 'reason' in result ? result.reason : '';
    expect(reason).toContain('CF passage text not in current preflight');
    expect(reason).not.toContain('--primary-registry');
  });
});
