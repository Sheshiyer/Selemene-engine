// ─── Reference Source Audit Tests ─────────────────────────────────────
// Covers the standalone audit module (no orchestrator wiring here).
//   1. Prompt contains the required strict-JSON schema keywords
//   2. Valid audit JSON with findings → status='changes-required'
//   3. Clean audit with coverage>0 → status='clean'
//   4. Clean audit with coverage=0 → status='insufficient' (fail-closed)
//   5. Malformed JSON → status='malformed', model_raw preserved verbatim
//   6. LLM throws → status='llm-error', input_sha256 still set
//   7. Passages-hash mismatch → status='malformed' before LLM is called
//   8. input_sha256 changes when accepted output, passages, or engine facts change
//   9. output_sha256 == sha256(model_raw)
//  10. Auditor prompt is deterministic (stable across identical calls)
//  11. hasActionableFindings distinguishes clarity vs error/unsupported

import { describe, it, expect, vi } from 'vitest';
import { createHash } from 'node:crypto';
import {
  runSourceAudit,
  buildAuditorSystemPrompt,
  buildAuditorUserPrompt,
  computeAuditInputHash,
  hasActionableFindings,
  extractJsonObject,
  buildAuditRepairPrompt,
  buildAuditRecoveryUserPrompt,
} from './reference-source-audit.js';
import type { RunSourceAuditInput } from './reference-source-audit.js';
import type { LlmCall } from './integrated.js';

function sha256(t: string): string {
  return createHash('sha256').update(t, 'utf8').digest('hex');
}

const PASSAGE_TEXT = 'Framework note: expression measures outward projection.';
const PASSAGE = { id: 'p1', source: 'corpus/numerology-desc.md', text: PASSAGE_TEXT };
const PASSAGES_HASH = sha256(PASSAGE_TEXT);
const ENGINE_FACTS = 'Engine: numerology | expression.value=8 | personality.value=4 | soul_urge.value=4';

function makeInput(overrides: Partial<RunSourceAuditInput> = {}): RunSourceAuditInput {
  return {
    sectionId: 'opening',
    sectionTitle: 'Opening',
    acceptedOutput: 'The opening notes a 4-cluster with expression at 8. Per corpus/numerology-desc.md: expression measures outward projection.',
    engineFacts: ENGINE_FACTS,
    passages: [PASSAGE],
    passagesHash: PASSAGES_HASH,
    model: 'test-model',
    ...overrides,
  };
}

describe('reference-source-audit: prompt contract', () => {
  it('system prompt contains strict-JSON schema keywords', () => {
    const sys = buildAuditorSystemPrompt();
    expect(sys).toContain('STRICT JSON');
    expect(sys).toContain('status');
    expect(sys).toContain('coverage');
    expect(sys).toContain('findings');
    expect(sys).toContain('claim_quoted');
    expect(sys).toContain('source_path_or_passage_id');
    expect(sys).toContain('required_correction');
    expect(sys).toContain('total_claims_audited');
    expect(sys).toContain('recount the exact supplied values');
    // Auditor is NOT Jev
    expect(sys).toContain('NOT the Jev');
  });

  it('user prompt inlines engine facts, passages, and accepted output', () => {
    const user = buildAuditorUserPrompt(makeInput());
    expect(user).toContain('Engine facts');
    expect(user).toContain('expression.value=8');
    expect(user).toContain('Passage 1');
    expect(user).toContain(PASSAGE_TEXT);
    expect(user).toContain('Accepted section output');
  });
});

describe('reference-source-audit: happy paths', () => {
  it('returns changes-required with structured findings for valid audit JSON', async () => {
    const auditJson = {
      status: 'changes-required',
      coverage: { total_claims_audited: 4, sources_referenced: 1 },
      findings: [
        {
          id: 'OPENING-NUM-01',
          severity: 'error',
          claim_quoted: '4-cluster with expression at 8',
          source_reference: {
            source_path_or_passage_id: 'engine:numerology.result.expression.value',
            supporting_values: { expression: 8, personality: 4 },
          },
          analysis: 'Expression value is 8, not 4; grouping it with the 4-cluster contradicts the engine value.',
          required_correction: 'Remove expression from the 4-cluster; preserve Expression 8 explicitly.',
        },
      ],
    };
    const llm: LlmCall = vi.fn(async () => JSON.stringify(auditJson));
    const receipt = await runSourceAudit(makeInput(), llm);
    expect(receipt.status).toBe('changes-required');
    expect(receipt.findings).toHaveLength(1);
    expect(receipt.findings[0].source_reference.source_path_or_passage_id).toContain('numerology');
    expect(receipt.coverage.total_claims_audited).toBe(4);
    expect(receipt.input_sha256).toHaveLength(64);
    expect(receipt.output_sha256).toBe(sha256(JSON.stringify(auditJson)));
    expect(receipt.model_raw).toBe(JSON.stringify(auditJson));
    expect(llm).toHaveBeenCalledOnce();
  });

  it('returns clean when auditor finds nothing and coverage>0', async () => {
    const audit = { status: 'clean', coverage: { total_claims_audited: 1, sources_referenced: 1 }, claim_checks: [{ claim_quoted: 'expression at 8', source_reference: { source_path_or_passage_id: 'numerology.result.expression.value', supporting_values: 8 } }], findings: [] };
    const llm: LlmCall = vi.fn(async () => JSON.stringify(audit));
    const receipt = await runSourceAudit(makeInput(), llm);
    expect(receipt.status).toBe('clean');
    expect(receipt.findings).toHaveLength(0);
    expect(receipt.coverage.total_claims_audited).toBe(1);
  });

  it('tolerates markdown fences around JSON', async () => {
    const audit = { status: 'clean', coverage: { total_claims_audited: 1, sources_referenced: 1 }, claim_checks: [{ claim_quoted: 'expression at 8', source_reference: { source_path_or_passage_id: 'numerology.result.expression.value', supporting_values: 8 } }], findings: [] };
    const llm: LlmCall = vi.fn(async () => '```json\n' + JSON.stringify(audit) + '\n```\n');
    const receipt = await runSourceAudit(makeInput(), llm);
    expect(receipt.status).toBe('clean');
  });
});

describe('reference-source-audit: fail-closed behaviors', () => {
  it('malformed JSON → status=malformed and model_raw preserved verbatim', async () => {
    const garbage = 'the model refused to output JSON and wrote prose instead';
    const llm: LlmCall = vi.fn(async () => garbage);
    const receipt = await runSourceAudit(makeInput(), llm, { maxResponseRepairs: 0 });
    expect(receipt.status).toBe('malformed');
    expect(receipt.model_raw).toBe(garbage);
    expect(receipt.output_sha256).toBe(sha256(garbage));
    expect(receipt.reason).toBeDefined();
    expect(llm).toHaveBeenCalledOnce();
    expect(receipt.response_attempts).toBeUndefined();
  });

  it('invalid finding shape (missing source_reference) → malformed', async () => {
    const bad = {
      status: 'changes-required',
      coverage: { total_claims_audited: 1, sources_referenced: 0 },
      findings: [{ id: 'X', severity: 'error', claim_quoted: 'x', analysis: 'y', required_correction: 'z' }],
    };
    const llm: LlmCall = vi.fn(async () => JSON.stringify(bad));
    const receipt = await runSourceAudit(makeInput(), llm, { maxResponseRepairs: 0 });
    expect(receipt.status).toBe('malformed');
    expect(receipt.reason).toContain('source_reference');
    expect(llm).toHaveBeenCalledOnce();
  });

  it('clean status with zero claims → insufficient (fail-closed)', async () => {
    const suspicious = { status: 'clean', coverage: { total_claims_audited: 0, sources_referenced: 0 }, findings: [] };
    const llm: LlmCall = vi.fn(async () => JSON.stringify(suspicious));
    const receipt = await runSourceAudit(makeInput(), llm, { maxResponseRepairs: 0 });
    expect(receipt.status).toBe('insufficient');
    expect(receipt.reason).toContain('zero claims');
    expect(llm).toHaveBeenCalledOnce();
  });

  it('LLM throw → llm-error with input_sha256 still populated', async () => {
    const llm: LlmCall = vi.fn(async () => { throw new Error('audit LLM down'); });
    const receipt = await runSourceAudit(makeInput(), llm);
    expect(receipt.status).toBe('llm-error');
    expect(receipt.reason).toContain('audit LLM down');
    expect(receipt.input_sha256).toHaveLength(64);
    expect(receipt.output_sha256).toBe('');
    expect(receipt.model_raw).toBe('');
    // LLM errors are not eligible for MODEL-response repair.
    expect(llm).toHaveBeenCalledOnce();
    expect(receipt.response_attempts).toBeUndefined();
  });

  it('passages hash mismatch → malformed BEFORE the LLM is called', async () => {
    const llm = vi.fn(async () => '{}');
    const receipt = await runSourceAudit(makeInput({ passagesHash: 'wrong-hash-value' }), llm as LlmCall);
    expect(receipt.status).toBe('malformed');
    expect(receipt.reason).toContain('passages hash mismatch');
    expect(llm).not.toHaveBeenCalled();
    expect(receipt.response_attempts).toBeUndefined();
  });
});

describe('reference-source-audit: hash binding', () => {
  it('input_sha256 changes when accepted output changes', () => {
    const h1 = computeAuditInputHash({ acceptedOutput: 'A', passagesHash: PASSAGES_HASH, engineFacts: ENGINE_FACTS });
    const h2 = computeAuditInputHash({ acceptedOutput: 'B', passagesHash: PASSAGES_HASH, engineFacts: ENGINE_FACTS });
    expect(h1).not.toBe(h2);
  });

  it('input_sha256 changes when passages hash changes', () => {
    const h1 = computeAuditInputHash({ acceptedOutput: 'A', passagesHash: PASSAGES_HASH, engineFacts: ENGINE_FACTS });
    const h2 = computeAuditInputHash({ acceptedOutput: 'A', passagesHash: sha256('different'), engineFacts: ENGINE_FACTS });
    expect(h1).not.toBe(h2);
  });

  it('input_sha256 changes when engine facts change', () => {
    const h1 = computeAuditInputHash({ acceptedOutput: 'A', passagesHash: PASSAGES_HASH, engineFacts: ENGINE_FACTS });
    const h2 = computeAuditInputHash({ acceptedOutput: 'A', passagesHash: PASSAGES_HASH, engineFacts: ENGINE_FACTS + ' | extra' });
    expect(h1).not.toBe(h2);
  });
});

describe('reference-source-audit: helpers', () => {
  it('extractJsonObject handles balanced braces and strings with braces', () => {
    const inp = 'prefix {"a":"b}c","d":{"e":1}} suffix';
    expect(extractJsonObject(inp)).toBe('{"a":"b}c","d":{"e":1}}');
  });

  it('hasActionableFindings ignores clarity but flags error/unsupported', () => {
    expect(hasActionableFindings([])).toBe(false);
    expect(hasActionableFindings([{ id: 'a', severity: 'clarity', claim_quoted: 'x', source_reference: { source_path_or_passage_id: 'p' }, analysis: 'a', required_correction: 'c' }])).toBe(false);
    expect(hasActionableFindings([{ id: 'a', severity: 'error', claim_quoted: 'x', source_reference: { source_path_or_passage_id: 'p' }, analysis: 'a', required_correction: 'c' }])).toBe(true);
    expect(hasActionableFindings([{ id: 'a', severity: 'unsupported', claim_quoted: 'x', source_reference: { source_path_or_passage_id: 'p' }, analysis: 'a', required_correction: 'c' }])).toBe(true);
  });

  it('buildAuditRepairPrompt inlines findings, sources, and required subsection IDs', () => {
    const prompt = buildAuditRepairPrompt({
      previousOutput: 'previous draft text',
      engineFacts: ENGINE_FACTS,
      passages: [PASSAGE],
      actionableFindings: [{
        id: 'F1',
        severity: 'error',
        claim_quoted: 'wrong claim',
        source_reference: { source_path_or_passage_id: 'engine:x' },
        analysis: 'analysis text',
        required_correction: 'do the right thing',
      }],
      requiredSubsectionIds: ['2.1', '2.2'],
      targetWords: 400,
    });
    expect(prompt).toContain('wrong claim');
    expect(prompt).toContain('do the right thing');
    expect(prompt).toContain('2.1, 2.2');
    expect(prompt).toContain(PASSAGE_TEXT);
    expect(prompt).toContain('previous draft text');
    expect(prompt).toContain('MINIMAL, TARGETED');
  });
});

it('rejects the live false-clean pattern: a claimed count with no inspectable evidence', async () => {
  const raw = JSON.stringify({status: 'clean', coverage: {total_claims_audited: 61, sources_referenced: 11}, findings: []});
  const receipt = await runSourceAudit(makeInput(), async () => raw, { maxResponseRepairs: 0 });
  expect(receipt.status).toBe('insufficient');
  expect(receipt.model_raw).toBe(raw);
});
it('rejects a clean ledger that quotes text absent from the reviewed output', async () => {
  const raw = JSON.stringify({status: 'clean', coverage: {total_claims_audited: 1, sources_referenced: 1}, findings: [], claim_checks: [{claim_quoted: 'invented claim', source_reference: {source_path_or_passage_id: 'numerology.result.expression.value', supporting_values: 8}}]});
  expect((await runSourceAudit(makeInput(), async () => raw, { maxResponseRepairs: 0 })).status).toBe('insufficient');
});

it('matches rendered quotations across bold delimiters without changing values', async () => {
  const raw = JSON.stringify({status: 'clean', coverage: {total_claims_audited: 1, sources_referenced: 1}, findings: [], claim_checks: [{claim_quoted: 'Expression is 8.', source_reference: {source_path_or_passage_id: 'numerology.result.expression.value', supporting_values: 8}}]});
  expect((await runSourceAudit(makeInput({acceptedOutput: '**Expression** is 8.'}), async () => raw)).status).toBe('clean');
  expect((await runSourceAudit(makeInput({acceptedOutput: '**Expression** is 4.'}), async () => raw, { maxResponseRepairs: 0 })).status).toBe('insufficient');
});

describe('reference-source-audit: audit-only response repair', () => {
  const CLEAN_QUOTE = 'expression at 8';
  const cleanAuditRaw = () => JSON.stringify({
    status: 'clean',
    coverage: { total_claims_audited: 1, sources_referenced: 1 },
    findings: [],
    claim_checks: [{
      claim_quoted: CLEAN_QUOTE,
      source_reference: { source_path_or_passage_id: 'engine:numerology.result.expression.value', supporting_values: 8 },
    }],
  });

  it('recovers a malformed first response with one additional audit-only call', async () => {
    const cleanRaw = cleanAuditRaw();
    const llm = vi.fn()
      .mockResolvedValueOnce('not JSON at all — model rambled')
      .mockResolvedValueOnce(cleanRaw);
    const input = makeInput();
    const receipt = await runSourceAudit(input, llm as LlmCall);

    expect(receipt.status).toBe('clean');
    // Final receipt reflects the recovery response.
    expect(receipt.model_raw).toBe(cleanRaw);
    expect(receipt.output_sha256).toBe(sha256(cleanRaw));
    // Section input hash is unchanged — bound to the audit input, not the response.
    expect(receipt.input_sha256).toBe(computeAuditInputHash({
      acceptedOutput: input.acceptedOutput,
      passagesHash: input.passagesHash,
      engineFacts: input.engineFacts,
    }));
    // Exactly one repair call.
    expect(llm).toHaveBeenCalledTimes(2);
    // Original failed response is preserved.
    expect(receipt.response_attempts).toBeDefined();
    expect(receipt.response_attempts).toHaveLength(2);
    expect(receipt.response_attempts![0].attempt).toBe(1);
    expect(receipt.response_attempts![0].status).toBe('malformed');
    expect(receipt.response_attempts![0].model_raw).toBe('not JSON at all — model rambled');
    expect(receipt.response_attempts![1].attempt).toBe(2);
    expect(receipt.response_attempts![1].status).toBe('clean');
    // Recovery prompt is retained for inspection (text and hash).
    expect(receipt.response_attempts![1].recovery_prompt).toBeDefined();
    expect(receipt.response_attempts![1].recovery_prompt_sha256).toHaveLength(64);
    expect(receipt.response_attempts![1].recovery_prompt_sha256)
      .toBe(sha256(receipt.response_attempts![1].recovery_prompt!));
  });

  it('recovers a paraphrased-quote (insufficient) response with one audit-only call', async () => {
    // First response is structurally valid JSON but claim_quoted is paraphrased.
    const paraphrased = JSON.stringify({
      status: 'clean',
      coverage: { total_claims_audited: 1, sources_referenced: 1 },
      findings: [],
      claim_checks: [{
        claim_quoted: 'the expression value is eight', // paraphrase — not in section
        source_reference: { source_path_or_passage_id: 'engine:numerology.result.expression.value', supporting_values: 8 },
      }],
    });
    const cleanRaw = cleanAuditRaw();
    const llm = vi.fn()
      .mockResolvedValueOnce(paraphrased)
      .mockResolvedValueOnce(cleanRaw);
    const receipt = await runSourceAudit(makeInput(), llm as LlmCall);

    expect(receipt.status).toBe('clean');
    expect(receipt.model_raw).toBe(cleanRaw);
    expect(llm).toHaveBeenCalledTimes(2);
    expect(receipt.response_attempts).toHaveLength(2);
    expect(receipt.response_attempts![0].status).toBe('insufficient');
    expect(receipt.response_attempts![0].reason)
      .toContain('clean audit lacks a complete, quoted claim evidence ledger');
    // Original raw failed audit is retained verbatim.
    expect(receipt.response_attempts![0].model_raw).toBe(paraphrased);
    // Recovery user prompt gives the model the explicit contiguity feedback.
    const recoveryPrompt = receipt.response_attempts![1].recovery_prompt!;
    expect(recoveryPrompt).toContain('CONTIGUOUS, EXACT substring');
    expect(recoveryPrompt).toContain('Do NOT paraphrase');
    expect(recoveryPrompt).toContain('Do NOT quote from the engine facts');
    expect(recoveryPrompt).toContain('Do NOT quote from the retrieved framework passages');
    // The failed response and its reason are included as QUOTED DATA.
    expect(recoveryPrompt).toContain('QUOTED DATA');
    expect(recoveryPrompt).toContain(paraphrased);
  });

  it('keeps terminal status when the recovery response is still invalid', async () => {
    // Both responses are malformed — recovery cannot rescue this.
    const firstBad = 'still not JSON';
    const secondBad = 'still not JSON either';
    const llm = vi.fn()
      .mockResolvedValueOnce(firstBad)
      .mockResolvedValueOnce(secondBad);
    const receipt = await runSourceAudit(makeInput(), llm as LlmCall);

    expect(receipt.status).toBe('malformed');
    // Final receipt reflects the LAST (recovery) response — audit is not clean.
    expect(receipt.model_raw).toBe(secondBad);
    expect(llm).toHaveBeenCalledTimes(2);
    expect(receipt.response_attempts).toHaveLength(2);
    expect(receipt.response_attempts![0].status).toBe('malformed');
    expect(receipt.response_attempts![0].model_raw).toBe(firstBad);
    expect(receipt.response_attempts![1].status).toBe('malformed');
    expect(receipt.response_attempts![1].model_raw).toBe(secondBad);
  });

  it('never repairs an input-hash mismatch — makes zero LLM calls', async () => {
    const llm = vi.fn(async () => cleanAuditRaw());
    const receipt = await runSourceAudit(
      makeInput({ passagesHash: 'declared-hash-that-does-not-match' }),
      llm as LlmCall,
      // Default is 1; the mismatch must short-circuit BEFORE any LLM call.
    );
    expect(receipt.status).toBe('malformed');
    expect(receipt.reason).toContain('passages hash mismatch');
    expect(llm).not.toHaveBeenCalled();
    expect(receipt.response_attempts).toBeUndefined();
  });

  it('never regenerates prose or mutates section output during recovery', async () => {
    const originalOutput = 'The opening notes a 4-cluster with expression at 8. Per corpus/numerology-desc.md: expression measures outward projection.';
    const input = makeInput({ acceptedOutput: originalOutput });
    const firstBad = 'garbage';
    const cleanRaw = cleanAuditRaw();
    const llm = vi.fn()
      .mockResolvedValueOnce(firstBad)
      .mockResolvedValueOnce(cleanRaw);
    const receipt = await runSourceAudit(input, llm as LlmCall);

    // The section text handed in must be exactly what was returned to the caller
    // in unrelated pipeline state — this module never edits prose. We assert on
    // the input object staying the same reference/value and on the recovery
    // prompt containing (not rewriting) the section text.
    expect(input.acceptedOutput).toBe(originalOutput);
    const recoveryPrompt = receipt.response_attempts![1].recovery_prompt!;
    expect(recoveryPrompt).toContain(originalOutput);
    // Recovery prompt does NOT contain any "compose", "rewrite", "revise" verbs
    // that would signal a prose-regeneration request.
    expect(recoveryPrompt.toLowerCase()).not.toContain('rewrite');
    expect(recoveryPrompt.toLowerCase()).not.toContain('regenerate');
    expect(recoveryPrompt.toLowerCase()).not.toContain('compose');
  });

  it('retains the raw failed audit even when the recovery succeeds', async () => {
    const failedRaw = JSON.stringify({
      status: 'clean',
      coverage: { total_claims_audited: 42, sources_referenced: 7 },
      findings: [],
      // No claim_checks — insufficient.
    });
    const cleanRaw = cleanAuditRaw();
    const llm = vi.fn()
      .mockResolvedValueOnce(failedRaw)
      .mockResolvedValueOnce(cleanRaw);
    const receipt = await runSourceAudit(makeInput(), llm as LlmCall);

    expect(receipt.status).toBe('clean');
    expect(receipt.response_attempts![0].model_raw).toBe(failedRaw);
    expect(receipt.response_attempts![0].output_sha256).toBe(sha256(failedRaw));
    expect(receipt.response_attempts![0].status).toBe('insufficient');
  });

  it('maxResponseRepairs: 0 disables recovery entirely (single-call semantics)', async () => {
    const llm = vi.fn(async () => 'malformed');
    const receipt = await runSourceAudit(makeInput(), llm as LlmCall, { maxResponseRepairs: 0 });
    expect(receipt.status).toBe('malformed');
    expect(llm).toHaveBeenCalledOnce();
    expect(receipt.response_attempts).toBeUndefined();
  });

  it('caps at one repair — a numeric > 1 does not unlock more calls', async () => {
    const llm = vi.fn(async () => 'malformed');
    // @ts-expect-error — deliberately probing the cap with an out-of-type value.
    const receipt = await runSourceAudit(makeInput(), llm as LlmCall, { maxResponseRepairs: 5 });
    expect(receipt.status).toBe('malformed');
    // At most 2 calls total (1 original + 1 recovery), never more.
    expect(llm).toHaveBeenCalledTimes(2);
  });

  it('buildAuditRecoveryUserPrompt inlines section, passages, prior response and reason', () => {
    const prompt = buildAuditRecoveryUserPrompt({
      input: makeInput(),
      priorRaw: '{"status":"clean","coverage":{"total_claims_audited":1,"sources_referenced":1},"findings":[],"claim_checks":[{"claim_quoted":"paraphrase","source_reference":{"source_path_or_passage_id":"x","supporting_values":1}}]}',
      priorReason: 'clean audit lacks a complete, quoted claim evidence ledger',
    });
    // Includes the same audit surface as the original prompt.
    expect(prompt).toContain('Engine facts');
    expect(prompt).toContain('expression.value=8');
    expect(prompt).toContain('Accepted section output');
    // Feedback surface.
    expect(prompt).toContain('QUOTED DATA');
    expect(prompt).toContain('CONTIGUOUS, EXACT substring');
    expect(prompt).toContain('clean audit lacks a complete');
    expect(prompt).toContain('paraphrase'); // the prior claim_quoted is echoed
  });
});


describe('auditor prompt extensions (Phase B additive)', () => {
  const microInterpretations = [{
    engine_id: 'numerology',
    claim: 'Expression is 8',
    source_field: 'expression.value',
    source_value: '8',
    interpretation_tradition: 'numerology',
    confidence: 'direct' as const,
    contradictions: [],
    uncertainty_note: null,
  }];

  it('base system prompt contains the mandatory source-correction rules', async () => {
    const mod = await import('./reference-source-audit.js');
    const prompt = mod.buildAuditorSystemPrompt();
    expect(prompt).toContain('SOURCE-CORRECTION RULES');
    expect(prompt).toContain('Rao house 11 includes older siblings');
    expect(prompt).toContain('Mohan Ketu pratyantar duration 24.857 days');
    expect(prompt).toContain('Gary Sun pratyantar start 2027-06-05');
    expect(prompt).toContain('Gary birth-time confidence is approximate');
    expect(prompt).toContain('unsupported Darakaraka ranking');
    expect(prompt).toContain('Legacy reference facts');
  });

  it('base system prompt without coverage flag does not include MICRO-INTERPRETATION COVERAGE block', async () => {
    const mod = await import('./reference-source-audit.js');
    const prompt = mod.buildAuditorSystemPrompt();
    expect(prompt).not.toContain('MICRO-INTERPRETATION COVERAGE');
  });

  it('coverage flag adds the MICRO-INTERPRETATION COVERAGE block', async () => {
    const mod = await import('./reference-source-audit.js');
    const prompt = mod.buildAuditorSystemPrompt({ withMicroInterpretationCoverage: true });
    expect(prompt).toContain('MICRO-INTERPRETATION COVERAGE');
    expect(prompt).toContain('micro_interpretation_coverage');
    expect(prompt).toContain('uncovered_claims');
    expect(prompt).toContain('orphaned_interpretations');
  });

  it('user prompt inlines micro-interpretations as QUOTED DATA when supplied', async () => {
    const mod = await import('./reference-source-audit.js');
    const mi = [{
      engine_id: 'vedic-kundali',
      claim: 'Lagna Scorpio',
      source_field: 'lagna.sign',
      source_value: 'Scorpio',
      interpretation_tradition: 'Vedic Jyotish',
      confidence: 'direct' as const,
      contradictions: [],
      uncertainty_note: null,
    }];
    const prompt = mod.buildAuditorUserPrompt({
      sectionId: 'part1', sectionTitle: 'Part I',
      acceptedOutput: 'x',
      engineFacts: 'engine facts',
      passages: [], passagesHash: '',
      microInterpretations: mi,
    });
    expect(prompt).toContain('Private micro-interpretation array');
    expect(prompt).toContain('QUOTED DATA');
    expect(prompt).toContain('"engine_id": "vedic-kundali"');
    expect(prompt).toContain('"claim": "Lagna Scorpio"');
  });

  it('user prompt omits the micro-interpretation block when array is empty or absent', async () => {
    const mod = await import('./reference-source-audit.js');
    const base = {
      sectionId: 'part1', sectionTitle: 'Part I',
      acceptedOutput: 'x',
      engineFacts: 'engine facts',
      passages: [], passagesHash: '',
    };
    const promptEmpty = mod.buildAuditorUserPrompt({ ...base, microInterpretations: [] });
    expect(promptEmpty).not.toContain('Private micro-interpretation array');
    const promptAbsent = mod.buildAuditorUserPrompt(base);
    expect(promptAbsent).not.toContain('Private micro-interpretation array');
  });

  it('fails closed when micro-interpretations are supplied but coverage is missing', async () => {
    const raw = JSON.stringify({
      status: 'clean',
      coverage: { total_claims_audited: 1, sources_referenced: 1 },
      findings: [],
      claim_checks: [{
        claim_quoted: 'expression at 8',
        source_reference: { source_path_or_passage_id: 'expression.value', supporting_values: 8 },
      }],
    });
    const receipt = await runSourceAudit(
      makeInput({ microInterpretations }),
      async () => raw,
      { maxResponseRepairs: 0 },
    );
    expect(receipt.status).toBe('malformed');
    expect(receipt.reason).toContain('micro_interpretation_coverage');
  });

  it('fails closed on orphaned micro-interpretations and preserves their identifiers', async () => {
    const raw = JSON.stringify({
      status: 'changes-required',
      coverage: { total_claims_audited: 1, sources_referenced: 1 },
      findings: [],
      micro_interpretation_coverage: {
        uncovered_claims: [],
        orphaned_interpretations: ['numerology:expression.value'],
      },
    });
    const receipt = await runSourceAudit(
      makeInput({ microInterpretations }),
      async () => raw,
      { maxResponseRepairs: 0 },
    );
    expect(receipt.status).toBe('insufficient');
    expect(receipt.reason).toContain('orphaned micro-interpretations');
    expect(receipt.micro_interpretation_coverage?.orphaned_interpretations)
      .toEqual(['numerology:expression.value']);
  });

  it('requires every uncovered claim to have a matching actionable finding', async () => {
    const raw = JSON.stringify({
      status: 'changes-required',
      coverage: { total_claims_audited: 1, sources_referenced: 1 },
      findings: [{
        id: 'CLARITY-1', severity: 'clarity', claim_quoted: 'expression at 8',
        source_reference: { source_path_or_passage_id: 'expression.value', supporting_values: 8 },
        analysis: 'Needs clearer attribution.', required_correction: 'Clarify it.',
      }],
      micro_interpretation_coverage: {
        uncovered_claims: ['expression at 8'], orphaned_interpretations: [],
      },
    });
    const receipt = await runSourceAudit(
      makeInput({ microInterpretations }),
      async () => raw,
      { maxResponseRepairs: 0 },
    );
    expect(receipt.status).toBe('insufficient');
    expect(receipt.reason).toContain('lack actionable findings');
  });

  it('stores empty coverage arrays on a clean corrected-route audit', async () => {
    const raw = JSON.stringify({
      status: 'clean',
      coverage: { total_claims_audited: 1, sources_referenced: 1 },
      findings: [],
      claim_checks: [{
        claim_quoted: 'expression at 8',
        source_reference: { source_path_or_passage_id: 'expression.value', supporting_values: 8 },
      }],
      micro_interpretation_coverage: { uncovered_claims: [], orphaned_interpretations: [] },
    });
    const receipt = await runSourceAudit(makeInput({ microInterpretations }), async () => raw);
    expect(receipt.status).toBe('clean');
    expect(receipt.micro_interpretation_coverage).toEqual({
      uncovered_claims: [], orphaned_interpretations: [],
    });
  });
});
