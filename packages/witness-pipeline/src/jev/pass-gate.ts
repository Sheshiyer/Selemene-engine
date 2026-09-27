// ─── Jev pass gate ───────────────────────────────────────────────────
// Typed judgments over each witness pass. Jev picks and scores; it does not
// write. Contract (thoughtseed/factor specs/003-jev + 2026-09-27 research):
//   - options come from files (the mode doc, relationship type, register band)
//   - thresholds are 0.5 (ask) and 0.85 (act) and nothing else
//   - confidence is not permission: an active gate can block, never approve
//   - a missing client yields `skipped`, never a counterfeit verdict
// Default mode is shadow: receipts are recorded and compared with the regex
// rubric; nothing changes in the reading.

import { choice, noul, score, type JevClient, type JevResult, type Question } from './client.js';
import type { SectionRubric } from '../orchestrator/integrated.js';
import type { JevGuardrailPolicy } from '../modes/types.js';

export type JevGateMode = 'off' | 'shadow' | 'active';
export const JEV_ASK_THRESHOLD = 0.5;
export const JEV_ACT_THRESHOLD = 0.85;

export interface JevPassInput {
  passId: string;
  passTitle: string;
  output: string;
  register: 'l1_l3' | 'l4_l5';
  relationshipType?: string;
  subjectNames: string[];
  engineFacts: string;
  rubric: SectionRubric;
  /** From the mode doc; default 'descriptive'. */
  guardrailPolicy?: JevGuardrailPolicy;
}

export type JevVerdict = 'pass' | 'fail' | 'could-not-tell';

export interface JevPassReceipt {
  pass_id: string;
  mode: JevGateMode;
  status: 'judged' | 'skipped' | 'error';
  reason?: string;
  model?: string;
  latency_ms?: number;
  answers?: {
    guardrail_clean: number;            // noul: no prediction / diagnosis / outcome promises
    relationship_framing_ok: number;    // noul: framing matches relationship type (or 1 when solo)
    fact_grounding: { score: number; confidence: number; legend: Record<string, string> };
    register_fit: { choice: string; confidence: number };
  };
  verdicts?: { guardrail: JevVerdict; framing: JevVerdict; grounding: JevVerdict; register: JevVerdict };
  blocked: boolean;                     // only ever true in active mode
  disagreements: string[];              // where Jev and the regex rubric differ
  /** Present when the orchestrator retried the pass on a Jev safety failure. */
  retries?: JevRetryRecord[];
  chosen?: 'original' | 'revision';
  original_guardrail_clean?: number;
}

export interface FlaggedSentence { sentence: string; score: number }

export interface JevRetryRecord {
  attempt: number;
  flagged_sentences: FlaggedSentence[];
  receipt: JevPassReceipt;
  accepted: boolean;
}

export const GUARDRAIL_QUESTION: Record<JevGuardrailPolicy, [string, { true: string; false: string }]> = {
  descriptive: [
    'Is this witness section free of predictions, diagnoses, guarantees, and promises about outcomes, investments, health, or life events?',
    { true: 'Descriptive pattern witness only; facts and open questions', false: 'Contains a forecast, diagnosis, guarantee, or promised outcome' },
  ],
  'forecast-allowed': [
    'This reading mode may describe dated planetary periods and their themes as tendencies. Is the section free of guarantees, promises, diagnoses, and claims of certainty about money, marriage, children, health, or life events?',
    { true: 'Periods and themes are framed as tendencies or invitations; no guarantee, diagnosis, or certain outcome', false: 'States that something will happen, guarantees a result, diagnoses a condition, or promises an outcome' },
  ],
};

export const SENTENCE_QUESTION: Record<JevGuardrailPolicy, [string, { true: string; false: string }]> = {
  descriptive: ['is a prediction, forecast, guarantee, or promised outcome rather than a description of an observable pattern', { true: 'predictive or promissory', false: 'descriptive witness' }],
  'forecast-allowed': ['states a guarantee, a certain outcome, a diagnosis, or a promise (a dated period described as a tendency or invitation does not count)', { true: 'guarantee, certainty, diagnosis, or promise', false: 'tendency, invitation, or description' }],
};

const GROUNDING_LEVELS = [
  'No engine facts referenced; generic prose',
  'A few engine facts referenced for one subject only',
  'Engine facts referenced for every subject with some gaps',
  'Dense, specific engine facts for every subject woven through the section',
];

const FRAMING_INSTRUCTIONS: Record<string, string> = {
  'business-partners': 'Does this section use business-partner language only, with no romantic, marital, or dating framing?',
  family: 'Does this section use family-lineage language only, with no romantic framing between the subjects?',
  friends: 'Does this section use friendship language only, with no romantic framing?',
  'unmarried-partners': 'Does this section frame the subjects as unmarried partners without assuming marriage?',
  'married-partners': 'Does this section frame the subjects as married partners without predicting divorce or outcomes?',
};

export function buildPassQuestions(input: JevPassInput): Record<string, Question> {
  const q: Record<string, Question> = {
    guardrail_clean: noul(...GUARDRAIL_QUESTION[input.guardrailPolicy ?? 'descriptive']),
    fact_grounding: score(
      `Rate how well this section is grounded in the supplied deterministic engine facts for ${input.subjectNames.length} subject(s): ${input.subjectNames.join(', ')}`,
      GROUNDING_LEVELS,
    ),
    register_fit: choice('Which register band does this section read as? Judge by concrete markers, not by depth of insight.', {
      l1_l3: 'Plain descriptive prose. Names chart facts and observable patterns. Little or no shadow/gift/siddhi, dissolution, non-dual, or co-transformation vocabulary. Rarely addresses the reader\'s inner process in the second person.',
      l4_l5: 'Uses shadow/gift/siddhi, co-transformation, dissolution, or non-dual vocabulary. Frames the subject\'s inner process and evolution directly. Layered metaphysical framing over the chart facts.',
    }),
  };
  const framing = input.relationshipType ? FRAMING_INSTRUCTIONS[input.relationshipType] : undefined;
  if (framing) {
    q.relationship_framing_ok = noul(framing, { true: 'Framing matches the declared relationship type', false: 'Framing drifts to another relationship type' });
  }
  return q;
}

function verdictFromNoul(v: number): JevVerdict {
  if (v >= JEV_ACT_THRESHOLD) return 'pass';
  if (v < JEV_ASK_THRESHOLD) return 'fail';
  return 'could-not-tell';
}

export function parsePassResult(input: JevPassInput, mode: JevGateMode, result: JevResult, latencyMs: number): JevPassReceipt {
  const a = result.answers ?? {};
  const g = a.guardrail_clean;
  const f = a.relationship_framing_ok;
  const s = a.fact_grounding;
  const r = a.register_fit;
  const malformed = !g || g.type !== 'noul' || !s || s.type !== 'score' || !r || r.type !== 'choice' || (input.relationshipType && FRAMING_INSTRUCTIONS[input.relationshipType] && (!f || f.type !== 'noul'));
  if (malformed) {
    return { pass_id: input.passId, mode, status: 'error', reason: 'malformed Jev answers', model: result.model, latency_ms: latencyMs, blocked: false, disagreements: [] };
  }
  const guardrailClean = (g as { noul: number }).noul;
  const framingOk = f && f.type === 'noul' ? f.noul : 1;
  const grounding = s as { score: number; confidence: number; legend: Record<string, string> };
  const register = r as { choice: string; confidence: number };

  const verdicts = {
    guardrail: verdictFromNoul(guardrailClean),
    framing: verdictFromNoul(framingOk),
    grounding: grounding.confidence < JEV_ASK_THRESHOLD ? 'could-not-tell' : grounding.score >= 2 ? 'pass' : 'fail',
    register: register.confidence < JEV_ASK_THRESHOLD ? 'could-not-tell' : register.choice === input.register ? 'pass' : 'fail',
  } as const;

  const disagreements: string[] = [];
  if (input.rubric.guardrail_gate === 'pass' && verdicts.guardrail === 'fail') disagreements.push('guardrail: rubric pass, jev fail');
  if (input.rubric.guardrail_gate === 'fail' && verdicts.guardrail === 'pass') disagreements.push('guardrail: rubric fail, jev pass');
  if (input.rubric.deterministic_fact_gate === 'pass' && verdicts.grounding === 'fail') disagreements.push('grounding: rubric pass, jev fail');
  if (input.rubric.deterministic_fact_gate !== 'pass' && verdicts.grounding === 'pass') disagreements.push(`grounding: rubric ${input.rubric.deterministic_fact_gate}, jev pass`);

  // Active mode blocks only on a confident failure of a safety gate. It never approves.
  const blocked = mode === 'active' && (
    (verdicts.guardrail === 'fail' && guardrailClean <= 1 - JEV_ACT_THRESHOLD) ||
    (verdicts.framing === 'fail' && framingOk <= 1 - JEV_ACT_THRESHOLD)
  );

  return {
    pass_id: input.passId,
    mode,
    status: 'judged',
    model: result.model,
    latency_ms: latencyMs,
    answers: {
      guardrail_clean: guardrailClean,
      relationship_framing_ok: framingOk,
      fact_grounding: { score: grounding.score, confidence: grounding.confidence, legend: grounding.legend ?? {} },
      register_fit: { choice: register.choice, confidence: register.confidence },
    },
    verdicts,
    blocked,
    disagreements,
  };
}

export interface JevPassGate {
  mode: JevGateMode;
  judge(input: JevPassInput): Promise<JevPassReceipt>;
  /** One noul per sentence: which sentences breach the guardrail policy. Empty when no client or on error. */
  flagSentences(output: string, policy?: JevGuardrailPolicy): Promise<FlaggedSentence[]>;
}

const MAX_FLAG_SENTENCES = 40;

export function splitSentences(text: string): string[] {
  return text
    .replace(/^#+.*$/gm, '')
    .split(/(?<=[.!?])\s+/)
    .map((s) => s.replace(/\s+/g, ' ').trim())
    .filter((s) => s.length >= 40);
}

/** Ask Jev one yes/no per sentence; return those at or above the ask threshold, highest first. */
export async function findPredictiveSentences(client: JevClient, output: string, policy: JevGuardrailPolicy = 'descriptive'): Promise<FlaggedSentence[]> {
  const sentences = splitSentences(output).slice(0, MAX_FLAG_SENTENCES);
  if (!sentences.length) return [];
  const questions: Record<string, Question> = {};
  sentences.forEach((_, i) => {
    const [clause, criteria] = SENTENCE_QUESTION[policy];
    questions[`s${i}`] = noul(`Is it true that sentence ${i} ${clause}?`, criteria);
  });
  const result = await client({ state: { sentences: sentences.map((sentence, i) => ({ i, sentence })) }, questions });
  const out: FlaggedSentence[] = [];
  sentences.forEach((sentence, i) => {
    const a = result.answers?.[`s${i}`];
    if (a && a.type === 'noul' && a.noul >= JEV_ASK_THRESHOLD) out.push({ sentence, score: a.noul });
  });
  return out.sort((a, b) => b.score - a.score);
}

/**
 * Build a pass gate. With no client the gate records `skipped` receipts and
 * names the reason; it never answers in Jev's place (factor jev-gate rule).
 */
export function createJevPassGate(client: JevClient | null, mode: JevGateMode = 'shadow'): JevPassGate {
  return {
    mode,
    async flagSentences(output, policy = 'descriptive') {
      if (mode === 'off' || !client) return [];
      try { return await findPredictiveSentences(client, output, policy); } catch { return []; }
    },
    async judge(input) {
      if (mode === 'off') return { pass_id: input.passId, mode, status: 'skipped', reason: 'jev gate off', blocked: false, disagreements: [] };
      if (!client) {
        return { pass_id: input.passId, mode, status: 'skipped', reason: 'TYPESAFE_API_KEY not set; install: npx skills add typesafe-ai/skills --skill typesafe-ai', blocked: false, disagreements: [] };
      }
      const state = {
        pass_id: input.passId,
        pass_title: input.passTitle,
        register: input.register,
        relationship_type: input.relationshipType ?? null,
        subjects: input.subjectNames,
        engine_facts: input.engineFacts.slice(0, 6000),
        section: input.output.slice(0, 12000),
      };
      const started = Date.now();
      try {
        const result = await client({ state, questions: buildPassQuestions(input) });
        return parsePassResult(input, mode, result, Date.now() - started);
      } catch (e: unknown) {
        return { pass_id: input.passId, mode, status: 'error', reason: e instanceof Error ? e.message : String(e), latency_ms: Date.now() - started, blocked: false, disagreements: [] };
      }
    },
  };
}
