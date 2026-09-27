// ─── Integrated Reading Orchestrator ─────────────────────────────────
// Multi-pass reading orchestrator driven by a parsed mode document.

import type { ParsedModeDoc, RegisterBand, SelemeneEngineOutput } from '../index.js';
import { getPassTemplate, getTargetWordsForRegister, summarizeLessons } from '../modes/parser.js';
import { auditSectionOutput } from './rubric.js';
import { extractReportPatterns } from '../patterns/extractor.js';
import type { ExtractedPattern } from '../patterns/types.js';
import type { PatternVectorRetriever, RetrievedPattern } from '../patterns/retrieval.js';
import { renderRetrievedPatternsForPrompt } from '../patterns/retrieval.js';
import { renderFolioRelationshipHeader } from './folio-header.js';
import { buildEngineFactsBlock } from './engine-facts.js';
import type { JevPassGate, JevPassReceipt, JevRetryRecord } from '../jev/pass-gate.js';

export interface OrchestratorInput {
  subjectNames: string[];
  engineResultsBySubject: SelemeneEngineOutput[][];
  consciousnessLevel: number;
  retriever?: PatternVectorRetriever;
  retrievalQuery?: string;
  retrievalFilters?: import('../patterns/retrieval.js').RetrievalFilters;
  // additive for matrix / relationship / language
  subjectRoles?: Array<{ role: string; label?: string; name: string }>;
  relationshipContext?: { type: string; mapping_goal: string; sensitivity_level: 'low' | 'medium' | 'high' };
  language?: string;
}

export type RubricGate = 'pass' | 'warn' | 'fail';

export interface SectionRubric {
  section_id: string;
  title: string;
  target_words: number;
  actual_words: number;
  word_count_fit: RubricGate;
  word_count_ratio: number;
  deterministic_fact_count: number;
  deterministic_fact_gate: RubricGate;
  integrated_layer_count: number;
  integrated_layering_gate: RubricGate;
  guardrail_gate: 'pass' | 'fail';
  guardrail_violations: string[];
  model_requested: string;
  model_used: string;
  latency_ms: number;
  chart_fidelity_score?: number;
  chart_fidelity_details?: string[];
}

export interface PassResult {
  id: string;
  title: string;
  output: string;
  rubric: SectionRubric;
  /** Present when a Jev pass gate was configured (shadow or active). */
  jev?: JevPassReceipt;
}

export interface OrchestratorOutput {
  mode: string;
  subject_names: string[];
  register: RegisterBand;
  relationship_header?: string;
  passes: PassResult[];
  assembled: string;
  patterns: ExtractedPattern[];
  retrieved_patterns?: RetrievedPattern[];
  /** One receipt per pass when a Jev gate ran; absent otherwise. */
  jev_receipts?: JevPassReceipt[];
}

export interface LlmCall {
  (system: string, user: string, options: { max_tokens: number }): Promise<string>;
}

export interface OrchestratorOptions {
  mode: ParsedModeDoc;
  llm: LlmCall;
  retriever?: PatternVectorRetriever;
  /** Optional typed-judgment gate (TypeSafe Jev). Shadow mode records only. */
  jevGate?: JevPassGate;
  /** Retry a pass when Jev confidently fails guardrail or framing. Default 0 (record only). */
  jevRetry?: { maxRetries: number };
}

function resolveRegister(level: number): RegisterBand {
  return level <= 3 ? 'l1_l3' : 'l4_l5';
}

function resolveTargetWords(doc: ParsedModeDoc, register: RegisterBand, passId?: string): { min: number; max: number } {
  const variant = doc.frontmatter.register_variants?.[register];
  const base = variant?.target_words ?? doc.frontmatter.target_words;
  if (!passId) return base;
  const pass = doc.frontmatter.pass_plan.find((p) => p.id === passId);
  if (!pass) return base;
  // If the variant overrides this pass, use a tight range around the pass target.
  const override = variant?.overrides?.find((o) => o.pass_id === passId);
  if (override) {
    return { min: Math.round(pass.target_words * 0.9), max: pass.target_words };
  }
  return { min: Math.round(pass.target_words * 0.9), max: pass.target_words };
}

export class IntegratedReadingOrchestrator {
  private mode: ParsedModeDoc;
  private llm: LlmCall;
  private retriever?: PatternVectorRetriever;
  private jevGate?: JevPassGate;
  private jevMaxRetries: number;

  constructor(opts: OrchestratorOptions) {
    this.mode = opts.mode;
    this.llm = opts.llm;
    this.retriever = opts.retriever;
    this.jevGate = opts.jevGate;
    this.jevMaxRetries = opts.jevRetry?.maxRetries ?? 0;
  }

  async run(input: OrchestratorInput): Promise<OrchestratorOutput> {
    const register = resolveRegister(input.consciousnessLevel);
    const passOutputs: PassResult[] = [];
    let assembled = '';

    const relationship_header = input.relationshipContext
      ? renderFolioRelationshipHeader({
          subjectRoles: (input.subjectRoles || []).map(r => ({ role: r.role, name: r.name, label: r.label })),
          relationshipContext: input.relationshipContext,
        })
      : undefined;

    let retrieved: RetrievedPattern[] = [];
    const effectiveRetriever = input.retriever ?? this.retriever;
    if (effectiveRetriever && input.retrievalQuery) {
      try {
        retrieved = await effectiveRetriever.retrieveSimilar(
          input.retrievalQuery,
          input.retrievalFilters,
          5,
        );
      } catch {
        retrieved = [];
      }
    }
    const retrievedBlock = renderRetrievedPatternsForPrompt(retrieved);
    const engineFacts = buildEngineFactsBlock({
      subjectNames: input.subjectNames,
      subjectRoles: input.subjectRoles,
      engineResultsBySubject: input.engineResultsBySubject,
    });
    const allEngineResults = input.engineResultsBySubject.flat();

    for (const pass of this.mode.frontmatter.pass_plan) {
      const prior = assembled.slice(-4000);
      const templateContent = getPassTemplate(this.mode, pass.id, register);
      const hasFactsPlaceholder = /\{\{engine_facts\}\}/.test(templateContent);
      const rendered = this.renderPassTemplate(templateContent, pass, input, prior, register, engineFacts);
      // Ground every pass in deterministic engine facts: substitute the placeholder when the
      // template declares one, otherwise append the block so no pass runs on names alone.
      const basePrompt = hasFactsPlaceholder
        ? rendered
        : `${rendered}\n\n## Engine facts (deterministic, per subject)\n${engineFacts}`;
      const prompt = retrievedBlock ? `${basePrompt}\n\n${retrievedBlock}` : basePrompt;
      const system = this.buildSystemPrompt(pass, input, register);
      const { max } = resolveTargetWords(this.mode, register, pass.id);
      const model = pass.model ?? 'tier-default';
      const judgeInput = (output: string, rubric: SectionRubric) => ({
        passId: pass.id, passTitle: pass.title, output, register,
        relationshipType: input.relationshipContext?.type, subjectNames: input.subjectNames, engineFacts, rubric,
      });
      const produce = async (userPrompt: string) => {
        const started = Date.now();
        const output = await this.llm(system, userPrompt, { max_tokens: Math.round(max * 2) });
        const rubric = auditSectionOutput({
          sectionId: pass.id, title: pass.title, targetWords: pass.target_words, output,
          modelRequested: model, modelUsed: model, latencyMs: Date.now() - started,
          engineResults: allEngineResults, relationshipType: input.relationshipContext?.type,
        });
        const jev = this.jevGate && this.jevGate.mode !== 'off' ? await this.jevGate.judge(judgeInput(output, rubric)) : undefined;
        return { output, rubric, jev };
      };

      let best = await produce(prompt);
      const history: JevRetryRecord[] = [];
      const originalGuardrail = best.jev?.answers?.guardrail_clean;
      const needsRetry = (r?: JevPassReceipt) =>
        !!r && r.status === 'judged' && (r.verdicts?.guardrail === 'fail' || r.verdicts?.framing === 'fail');
      for (let attempt = 1; attempt <= this.jevMaxRetries && needsRetry(best.jev); attempt++) {
        const flagged = await this.jevGate!.flagSentences(best.output);
        const revisionPrompt = `${prompt}

## Revision required (attempt ${attempt})
A typed judge rated the previous draft as predictive or off-frame (guardrail_clean=${best.jev!.answers!.guardrail_clean.toFixed(2)}, framing_ok=${best.jev!.answers!.relationship_framing_ok.toFixed(2)}).
Rewrite the whole section as descriptive pattern witness for the declared relationship type. No forecasts, guarantees, "will", "likely", "ensures", "success", or promised outcomes. Keep every engine fact and the section length.
${flagged.length ? `Sentences to remove or reframe:\n${flagged.map((f) => `- "${f.sentence}"`).join('\n')}` : 'Reframe every sentence that states what the partnership will do or produce.'}

Previous draft:
${best.output}`;
        const candidate = await produce(revisionPrompt);
        const improved = candidate.jev?.status === 'judged' && best.jev?.status === 'judged'
          && (candidate.jev.answers!.guardrail_clean > best.jev.answers!.guardrail_clean)
          && candidate.jev.answers!.relationship_framing_ok >= best.jev.answers!.relationship_framing_ok - 0.05;
        // Store a snapshot of the candidate receipt: if it is accepted it becomes the pass receipt,
        // and the history must not point back at that same object (JSON cycle).
        history.push({ attempt, flagged_sentences: flagged, receipt: { ...candidate.jev! }, accepted: !!improved });
        if (improved) best = candidate;
      }

      const passResult: PassResult = { id: pass.id, title: pass.title, output: best.output, rubric: best.rubric };
      if (best.jev) {
        passResult.jev = best.jev;
        if (history.length) {
          passResult.jev.retries = history;
          passResult.jev.chosen = history.some((h) => h.accepted) ? 'revision' : 'original';
          passResult.jev.original_guardrail_clean = originalGuardrail;
        }
      }
      const output = best.output;
      passOutputs.push(passResult);
      assembled += `\n\n## ${pass.title}\n\n${output}`;
    }

    const patterns = extractReportPatterns({
      mode: this.mode.frontmatter.mode,
      reportLevel: (this.mode.frontmatter as any).report_level ?? 'L3',
      subjectNames: input.subjectNames,
      passes: passOutputs,
      language: input.language,
      relationship_type: input.relationshipContext?.type,
    });

    if (relationship_header) {
      assembled = `${relationship_header}\n\n${assembled}`;
    }

    const out: OrchestratorOutput = {
      mode: this.mode.frontmatter.mode,
      subject_names: input.subjectNames,
      register,
      passes: passOutputs,
      assembled: assembled.trim(),
      patterns,
    };
    if (relationship_header) (out as any).relationship_header = relationship_header;
    if (retrieved.length) out.retrieved_patterns = retrieved;
    if (this.jevGate && this.jevGate.mode !== 'off') out.jev_receipts = passOutputs.map((p) => p.jev!).filter(Boolean);
    return out;
  }

  private renderPassTemplate(
    template: string,
    pass: { id: string; title: string; target_words: number },
    input: OrchestratorInput,
    priorPass: string,
    register: RegisterBand,
    engineFacts = '',
  ): string {
    const overlaySummary = this.buildOverlaySummary();
    const bridgeMandates = this.mode.frontmatter.bridge_mandates.map((m) => `- ${m}`).join('\n');
    const lessonsSummary = summarizeLessons(this.mode.lessons, 5);

    const subjectRolesStr = input.subjectRoles && input.subjectRoles.length > 0
      ? input.subjectRoles.map((r) => `${r.name} (${r.role}${r.label ? ` — ${r.label}` : ''})`).join(', ')
      : input.subjectNames.join(', ');

    const relationshipHeader = input.relationshipContext
      ? `${input.subjectRoles?.map((r) => r.role).join('-') || 'Relationship'} ${input.relationshipContext.type} — non-predictive pattern witness`
      : '';

    const relationshipCtxJson = input.relationshipContext ? JSON.stringify(input.relationshipContext) : '';

    return template
      .replace(/\{\{subject_names\}\}/g, input.subjectNames.join(', '))
      .replace(/\{\{subject_roles\}\}/g, subjectRolesStr)
      .replace(/\{\{relationship_header\}\}/g, relationshipHeader)
      .replace(/\{\{relationship_context\}\}/g, relationshipCtxJson)
      .replace(/\{\{mapping_goal\}\}/g, input.relationshipContext?.mapping_goal || '')
      .replace(/\{\{prior_pass\}\}/g, priorPass)
      .replace(/\{\{overlay_summary\}\}/g, overlaySummary)
      .replace(/\{\{bridge_mandates\}\}/g, bridgeMandates)
      .replace(/\{\{lessons_summary\}\}/g, lessonsSummary)
      .replace(/\{\{register\}\}/g, register)
      .replace(/\{\{pass_id\}\}/g, pass.id)
      .replace(/\{\{target_words\}\}/g, String(pass.target_words))
      .replace(/\{\{language\}\}/g, input.language ?? 'en')
      .replace(/\{\{engine_facts\}\}/g, engineFacts);
  }

  private buildSystemPrompt(
    pass: { id: string; title: string; target_words: number },
    input: OrchestratorInput,
    register: RegisterBand,
  ): string {
    const { min, max } = resolveTargetWords(this.mode, register, pass.id);
    const rolesLine = input.subjectRoles && input.subjectRoles.length > 0
      ? `Subjects (roles): ${input.subjectRoles.map((r) => `${r.name}=${r.role}`).join(', ')}`
      : `Subjects: ${input.subjectNames.join(', ')}`;
    const relLine = input.relationshipContext
      ? `Relationship: type=${input.relationshipContext.type}; goal="${input.relationshipContext.mapping_goal}"; sensitivity=${input.relationshipContext.sensitivity_level}`
      : '';
    const langLine = input.language ? `Language: ${input.language}.` : '';
    return `You are writing pass "${pass.title}" (id: ${pass.id}) for the ${this.mode.frontmatter.mode} reading mode.
Register band: ${register}.
Target length: ~${pass.target_words} words (acceptable range ${min}-${max}).
${rolesLine}
${relLine}
${langLine}
${this.mode.sections['overlay-rules'] ?? ''}`;
  }

  private buildOverlaySummary(): string {
    const weights = Object.entries(this.mode.frontmatter.engine_overlay_weights)
      .map(([k, v]) => `${k}: ${v}`)
      .join(', ');
    return `Engine weights: ${weights}; Houses: ${this.mode.frontmatter.house_overlay.join(', ')}`;
  }
}
