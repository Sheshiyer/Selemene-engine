import { describe, it, expect, vi } from 'vitest';
import { createJevPassGate, buildPassQuestions, parsePassResult, JEV_ACT_THRESHOLD, JEV_ASK_THRESHOLD, type JevPassInput } from './pass-gate.js';
import type { JevResult } from './client.js';
import { IntegratedReadingOrchestrator } from '../orchestrator/integrated.js';
import { parseModeDoc } from '../modes/parser.js';
import { runFinalVerification } from '../orchestrator/final-verification.js';
import { resolve } from 'node:path';

const rubric = (over: Partial<JevPassInput['rubric']> = {}): JevPassInput['rubric'] => ({
  section_id: 'opening', title: 'Opening', target_words: 350, actual_words: 340, word_count_fit: 'pass', word_count_ratio: 0.97,
  deterministic_fact_count: 8, deterministic_fact_gate: 'pass', integrated_layer_count: 4, integrated_layering_gate: 'pass',
  guardrail_gate: 'pass', guardrail_violations: [], model_requested: 'x', model_used: 'x', latency_ms: 1, ...over,
});

const input = (over: Partial<JevPassInput> = {}): JevPassInput => ({
  passId: 'opening', passTitle: 'Opening', output: 'Descriptive section.', register: 'l1_l3',
  relationshipType: 'business-partners', subjectNames: ['A', 'B'], engineFacts: '### A\n- hd: Type Generator', rubric: rubric(), ...over,
});

const result = (over: Partial<JevResult['answers']> = {}): JevResult => ({
  model: 'jev-1.13.0',
  answers: {
    guardrail_clean: { type: 'noul', noul: 0.95 },
    relationship_framing_ok: { type: 'noul', noul: 0.9 },
    fact_grounding: { type: 'score', score: 2.6, legend: {}, probabilities: {}, confidence: 0.7 },
    register_fit: { type: 'choice', choice: 'l1_l3', probabilities: {}, confidence: 0.8 },
    ...over,
  },
});

describe('jev pass gate', () => {
  it('judges the complete reference section with its retrieved passages', async () => {
    const client = vi.fn(async () => result());
    const engineFacts = 'x'.repeat(7000) + 'LAST_ENGINE_FACT';
    const output = 'y'.repeat(14000) + 'FINAL_SECTION_CLAIM';
    const frameworkPassages = [{ id: 'sw:gk:4:life_theme', source: 'corpus', text: 'Attributed meaning' }];
    await createJevPassGate(client, 'shadow').judge(input({ engineFacts, output, frameworkPassages, fullContext: true }));
    const state = (client.mock.calls[0] as any)[0].state;
    expect(state.engine_facts).toBe(engineFacts);
    expect(state.section).toBe(output);
    expect(state.attributed_framework_passages).toEqual(frameworkPassages);
  });
  it('thresholds are exactly 0.5 and 0.85', () => {
    expect(JEV_ASK_THRESHOLD).toBe(0.5);
    expect(JEV_ACT_THRESHOLD).toBe(0.85);
  });

  it('asks a framing question only for known relationship types; options come from the register bands', () => {
    const q = buildPassQuestions(input());
    expect(Object.keys(q).sort()).toEqual(['fact_grounding', 'guardrail_clean', 'register_fit', 'relationship_framing_ok']);
    expect((q.register_fit as any).criteria).toHaveProperty('l1_l3');
    expect(Object.keys(buildPassQuestions(input({ relationshipType: undefined })))).not.toContain('relationship_framing_ok');
  });

  it('records skipped with the install command when no client is configured, never a counterfeit verdict', async () => {
    const r = await createJevPassGate(null, 'shadow').judge(input());
    expect(r.status).toBe('skipped');
    expect(r.reason).toContain('npx skills add typesafe-ai/skills');
    expect(r.answers).toBeUndefined();
    expect(r.blocked).toBe(false);
  });

  it('parses verdicts and flags disagreements with the regex rubric', () => {
    const r = parsePassResult(input({ rubric: rubric({ deterministic_fact_gate: 'warn' }) }), 'shadow', result(), 12);
    expect(r.status).toBe('judged');
    expect(r.verdicts).toEqual({ guardrail: 'pass', framing: 'pass', grounding: 'pass', register: 'pass' });
    expect(r.disagreements).toEqual(['grounding: rubric warn, jev pass']);
    expect(r.blocked).toBe(false);
  });

  it('low-confidence answers become could-not-tell, not fail', () => {
    const r = parsePassResult(input(), 'shadow', result({ guardrail_clean: { type: 'noul', noul: 0.6 }, register_fit: { type: 'choice', choice: 'l4_l5', probabilities: {}, confidence: 0.3 } }), 1);
    expect(r.verdicts?.guardrail).toBe('could-not-tell');
    expect(r.verdicts?.register).toBe('could-not-tell');
  });

  it('shadow mode never blocks; active mode blocks only a confident safety failure', () => {
    const bad = result({ guardrail_clean: { type: 'noul', noul: 0.05 } });
    expect(parsePassResult(input(), 'shadow', bad, 1).blocked).toBe(false);
    expect(parsePassResult(input(), 'active', bad, 1).blocked).toBe(true);
    const unsure = result({ guardrail_clean: { type: 'noul', noul: 0.4 } });
    expect(parsePassResult(input(), 'active', unsure, 1).blocked).toBe(false);
    expect(parsePassResult(input(), 'active', unsure, 1).verdicts?.guardrail).toBe('fail');
  });

  it('malformed answers and transport errors are recorded as error, not judged', async () => {
    expect(parsePassResult(input(), 'shadow', { model: 'x', answers: {} }, 1).status).toBe('error');
    const gate = createJevPassGate(async () => { throw new Error('timeout'); }, 'shadow');
    const r = await gate.judge(input());
    expect(r.status).toBe('error');
    expect(r.reason).toBe('timeout');
  });

  it('orchestrator attaches one receipt per pass and final verification honours active blocks', async () => {
    const mode = parseModeDoc(resolve(__dirname, '../../modes/business-partners.md'));
    const llm = vi.fn().mockResolvedValue('Descriptive business-partner pattern witness. No prediction. No diagnosis.');
    const client = vi.fn(async () => result({ guardrail_clean: { type: 'noul', noul: 0.02 } }));
    const run = async (gateMode: 'shadow' | 'active') => {
      const orch = new IntegratedReadingOrchestrator({ mode, llm, jevGate: createJevPassGate(client, gateMode) });
      return orch.run({
        subjectNames: ['A', 'B'],
        subjectRoles: [{ role: 'business-partner', name: 'A' }, { role: 'business-partner', name: 'B' }],
        relationshipContext: { type: 'business-partners', mapping_goal: 'g', sensitivity_level: 'medium' },
        engineResultsBySubject: [[], []],
        consciousnessLevel: 2,
      });
    };
    const shadow = await run('shadow');
    expect(shadow.jev_receipts?.length).toBe(mode.frontmatter.pass_plan.length);
    expect(shadow.passes.every((p) => p.jev?.status === 'judged' && !p.jev.blocked)).toBe(true);
    expect(runFinalVerification({ passes: shadow.passes }).passed).toBe(true);
    const state = (client.mock.calls[0] as any)[0].state;
    expect(state.relationship_type).toBe('business-partners');
    expect(state.section).toContain('Descriptive');

    const active = await run('active');
    const v = runFinalVerification({ passes: active.passes });
    expect(v.passed).toBe(false);
    expect(v.blockers[0]).toMatch(/:jev_blocked$/);
  });

  it('does not run when the gate is off', async () => {
    const mode = parseModeDoc(resolve(__dirname, '../../modes/business-partners.md'));
    const orch = new IntegratedReadingOrchestrator({ mode, llm: vi.fn().mockResolvedValue('x'), jevGate: createJevPassGate(vi.fn(), 'off') });
    const out = await orch.run({ subjectNames: ['A'], engineResultsBySubject: [[]], consciousnessLevel: 2 });
    expect(out.jev_receipts).toBeUndefined();
  });
});

describe('jev sentence flagging and retry loop', () => {
  it('splits sentences, asks one noul per sentence, and returns those at or above 0.5 highest first', async () => {
    const { findPredictiveSentences, splitSentences } = await import('./pass-gate.js');
    const text = '# Heading\n\nThe shared 43-23 channel appears in both charts and is observable today. Success likely emerges through honoring both responses over the coming years. Short.';
    expect(splitSentences(text).length).toBe(2);
    const client = vi.fn(async (req: any) => ({
      model: 'jev-1.13.0',
      answers: Object.fromEntries(Object.keys(req.questions).map((k, i) => [k, { type: 'noul', noul: i === 1 ? 0.91 : 0.08 }])),
    }));
    const flagged = await findPredictiveSentences(client as any, text);
    expect(flagged).toEqual([{ sentence: 'Success likely emerges through honoring both responses over the coming years.', score: 0.91 }]);
    expect((client.mock.calls[0] as any)[0].state.sentences.length).toBe(2);
  });

  it('retries a confidently failed pass with the flagged sentences quoted and keeps the better draft', async () => {
    const mode = parseModeDoc(resolve(__dirname, '../../modes/business-partners.md'));
    let call = 0;
    const llm = vi.fn(async (_s: string, user: string) => {
      call++;
      if (user.includes('## Revision required')) {
        expect(user).toContain('Sentences to remove or reframe');
        expect(user).toContain('Success likely emerges');
        return 'Revised descriptive witness. The 43-23 channel is present in both charts. No prediction. No diagnosis.';
      }
      return 'Draft one. Success likely emerges through honoring both responses over the coming years. No diagnosis.';
    });
    const client = vi.fn(async (req: any) => {
      if (req.state.sentences) {
        return { model: 'j', answers: Object.fromEntries(req.state.sentences.map((s: any) => [`s${s.i}`, { type: 'noul', noul: /likely/.test(s.sentence) ? 0.9 : 0.1 }])) };
      }
      const revised = /Revised/.test(req.state.section);
      return result({ guardrail_clean: { type: 'noul', noul: revised ? 0.92 : 0.1 } });
    });
    const orch = new IntegratedReadingOrchestrator({ mode, llm, jevGate: createJevPassGate(client as any, 'shadow'), jevRetry: { maxRetries: 1 } });
    const out = await orch.run({
      subjectNames: ['A', 'B'], subjectRoles: [{ role: 'business-partner', name: 'A' }, { role: 'business-partner', name: 'B' }],
      relationshipContext: { type: 'business-partners', mapping_goal: 'g', sensitivity_level: 'medium' },
      engineResultsBySubject: [[], []], consciousnessLevel: 2,
    });
    const passes = mode.frontmatter.pass_plan.length;
    expect(llm).toHaveBeenCalledTimes(passes * 2);
    for (const p of out.passes) {
      expect(p.output).toContain('Revised');
      expect(p.jev?.chosen).toBe('revision');
      expect(p.jev?.original_guardrail_clean).toBe(0.1);
      expect(p.jev?.answers?.guardrail_clean).toBe(0.92);
      expect(p.jev?.retries?.[0].accepted).toBe(true);
      expect(p.jev?.retries?.[0].flagged_sentences[0].sentence).toContain('Success likely');
    }
    expect(out.assembled).not.toContain('Success likely');
    expect(() => JSON.stringify(out)).not.toThrow();
    expect(JSON.parse(JSON.stringify(out)).passes[0].jev.retries[0].receipt.retries).toBeUndefined();
  });

  it('does not retry a clean pass and keeps the original when the revision is not better', async () => {
    const mode = parseModeDoc(resolve(__dirname, '../../modes/business-partners.md'));
    const llm = vi.fn(async () => 'Draft. No prediction. No diagnosis.');
    const clean = vi.fn(async () => result());
    const orchClean = new IntegratedReadingOrchestrator({ mode, llm, jevGate: createJevPassGate(clean as any, 'shadow'), jevRetry: { maxRetries: 2 } });
    const outClean = await orchClean.run({ subjectNames: ['A'], engineResultsBySubject: [[]], consciousnessLevel: 2 });
    expect(llm).toHaveBeenCalledTimes(mode.frontmatter.pass_plan.length);
    expect(outClean.passes[0].jev?.retries).toBeUndefined();

    const alwaysBad = vi.fn(async (req: any) => req.state.sentences
      ? { model: 'j', answers: {} }
      : result({ guardrail_clean: { type: 'noul', noul: 0.2 } }));
    const orchBad = new IntegratedReadingOrchestrator({ mode, llm: vi.fn(async () => 'Still predictive draft with enough words here.'), jevGate: createJevPassGate(alwaysBad as any, 'shadow'), jevRetry: { maxRetries: 1 } });
    const outBad = await orchBad.run({ subjectNames: ['A'], engineResultsBySubject: [[]], consciousnessLevel: 2 });
    expect(outBad.passes[0].jev?.chosen).toBe('original');
    expect(outBad.passes[0].jev?.retries?.[0].accepted).toBe(false);
    expect(outBad.passes[0].jev?.retries?.[0].flagged_sentences).toEqual([]);
  });
});

describe('guardrail policy', () => {
  it('asks the forecast-allowed question and sentence clause when the policy says so', async () => {
    const { GUARDRAIL_QUESTION, SENTENCE_QUESTION, findPredictiveSentences } = await import('./pass-gate.js');
    const q = buildPassQuestions(input({ guardrailPolicy: 'forecast-allowed' }));
    expect((q.guardrail_clean as any).instructions).toBe(GUARDRAIL_QUESTION['forecast-allowed'][0]);
    expect((buildPassQuestions(input()).guardrail_clean as any).instructions).toBe(GUARDRAIL_QUESTION.descriptive[0]);
    expect(GUARDRAIL_QUESTION.descriptive[0]).toContain('Engine-recorded dates');
    const client = vi.fn(async (req: any) => ({ model: 'j', answers: Object.fromEntries(Object.keys(req.questions).map((k) => [k, { type: 'noul', noul: 0.1 }])) }));
    await findPredictiveSentences(client as any, 'The Jupiter period tends to emphasise study and teaching over these years.', 'forecast-allowed');
    expect((client.mock.calls[0] as any)[0].questions.s0.instructions).toContain(SENTENCE_QUESTION['forecast-allowed'][0]);
  });

  it('orchestrator reads the policy from the mode doc and puts the matching voice rules in the system prompt', async () => {
    const { VOICE_RULES } = await import('../orchestrator/integrated.js');
    const l0 = parseModeDoc(resolve(__dirname, '../../modes/integrated-kundali-l0.md'));
    expect(l0.frontmatter.jev_guardrail).toBe('forecast-allowed');
    const systems: string[] = [];
    const client = vi.fn(async (req: any) => { expect(req.questions.guardrail_clean.instructions).toContain('dated planetary periods'); return result(); });
    const orch = new IntegratedReadingOrchestrator({ mode: l0, llm: vi.fn(async (sys: string) => { systems.push(sys); return 'x'; }), jevGate: createJevPassGate(client as any, 'shadow') });
    await orch.run({ subjectNames: ['A'], engineResultsBySubject: [[]], consciousnessLevel: 5 });
    expect(systems[0]).toContain(VOICE_RULES['forecast-allowed']);
    const bp = parseModeDoc(resolve(__dirname, '../../modes/business-partners.md'));
    expect(bp.frontmatter.jev_guardrail).toBeUndefined();
    const sys2: string[] = []; const users: string[] = [];
    await new IntegratedReadingOrchestrator({ mode: bp, llm: vi.fn(async (sys: string, user: string) => { sys2.push(sys); users.push(user); return 'x'; }) }).run({ subjectNames: ['A', 'B'], engineResultsBySubject: [[], []], consciousnessLevel: 2 });
    expect(sys2[0]).toContain(VOICE_RULES.descriptive);
    // lessons reach the user prompt even though this template declares no {{lessons_summary}}
    expect(users[0]).toContain('Prior Autoresearch Findings');
    expect(users[0]).toContain('Jev shadow matrix');
  });

  it('every mode doc parses, and the migrated partner-synastry doc has four passes and a dyad shape', async () => {
    const { readdirSync } = await import('node:fs');
    const dir = resolve(__dirname, '../../modes');
    for (const f of readdirSync(dir).filter((x) => x.endsWith('.md'))) expect(() => parseModeDoc(resolve(dir, f))).not.toThrow();
    const ps = parseModeDoc(resolve(dir, 'partner-synastry.md'));
    expect(ps.frontmatter.pass_plan.map((p) => p.id)).toEqual(['opening', 'structural-compatibility', 'energetic-dance', 'synthesis']);
    expect(ps.frontmatter.subject_count).toEqual({ min: 2, max: 2 });
    expect(ps.lessons.length).toBe(1);
  });

  it('parser rejects an unknown jev_guardrail value', async () => {
    const { parseModeDocument } = await import('../modes/parser.js');
    const raw = require('node:fs').readFileSync(resolve(__dirname, '../../modes/business-partners.md'), 'utf8').replace('mode: business-partners', 'mode: business-partners\njev_guardrail: anything-goes');
    expect(() => parseModeDocument(raw, "inline")).toThrow(/invalid jev_guardrail/);
  });
});
