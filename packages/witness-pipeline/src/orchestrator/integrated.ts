// ─── Integrated Reading Orchestrator ─────────────────────────────────
// Multi-pass reading orchestrator driven by a parsed mode document.

import type { ParsedModeDoc, RegisterBand, SelemeneEngineOutput } from '../index.js';
import { getPassTemplate, getTargetWordsForRegister, summarizeLessons } from '../modes/parser.js';
import { auditSectionOutput } from './rubric.js';
import { extractReportPatterns } from '../patterns/extractor.js';
import type { ExtractedPattern } from '../patterns/types.js';
import type { PatternVectorRetriever, RetrievedPattern } from '../patterns/retrieval.js';
import { renderRetrievedPatternsForPrompt } from '../patterns/retrieval.js';

export interface OrchestratorInput {
  subjectNames: string[];
  engineResultsBySubject: SelemeneEngineOutput[][];
  consciousnessLevel: number;
  retriever?: PatternVectorRetriever;
  retrievalQuery?: string;
  retrievalFilters?: import('../patterns/retrieval.js').RetrievalFilters;
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
}

export interface OrchestratorOutput {
  mode: string;
  subject_names: string[];
  register: RegisterBand;
  passes: PassResult[];
  assembled: string;
  patterns: ExtractedPattern[];
  retrieved_patterns?: RetrievedPattern[];
}

export interface LlmCall {
  (system: string, user: string, options: { max_tokens: number }): Promise<string>;
}

export interface OrchestratorOptions {
  mode: ParsedModeDoc;
  llm: LlmCall;
  retriever?: PatternVectorRetriever;
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

  constructor(opts: OrchestratorOptions) {
    this.mode = opts.mode;
    this.llm = opts.llm;
    this.retriever = opts.retriever;
  }

  async run(input: OrchestratorInput): Promise<OrchestratorOutput> {
    const register = resolveRegister(input.consciousnessLevel);
    const passOutputs: PassResult[] = [];
    let assembled = '';

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

    for (const pass of this.mode.frontmatter.pass_plan) {
      const prior = assembled.slice(-4000);
      const templateContent = getPassTemplate(this.mode, pass.id, register);
      const basePrompt = this.renderPassTemplate(templateContent, pass, input, prior, register);
      const prompt = retrievedBlock ? `${basePrompt}\n\n${retrievedBlock}` : basePrompt;
      const system = this.buildSystemPrompt(pass, input, register);
      const { max } = resolveTargetWords(this.mode, register, pass.id);
      const started = Date.now();
      const output = await this.llm(system, prompt, { max_tokens: Math.round(max * 2) });
      const latencyMs = Date.now() - started;
      const model = pass.model ?? 'tier-default';
      const rubric = auditSectionOutput({
        sectionId: pass.id,
        title: pass.title,
        targetWords: pass.target_words,
        output,
        modelRequested: model,
        modelUsed: model,
        latencyMs,
        engineResults: input.engineResultsBySubject[0] ?? [],
      });
      passOutputs.push({ id: pass.id, title: pass.title, output, rubric });
      assembled += `\n\n## ${pass.title}\n\n${output}`;
    }

    const patterns = extractReportPatterns({
      mode: this.mode.frontmatter.mode,
      reportLevel: (this.mode.frontmatter as any).report_level ?? 'L3',
      subjectNames: input.subjectNames,
      passes: passOutputs,
    });

    const out: OrchestratorOutput = {
      mode: this.mode.frontmatter.mode,
      subject_names: input.subjectNames,
      register,
      passes: passOutputs,
      assembled: assembled.trim(),
      patterns,
    };
    if (retrieved.length) out.retrieved_patterns = retrieved;
    return out;
  }

  private renderPassTemplate(
    template: string,
    pass: { id: string; title: string; target_words: number },
    input: OrchestratorInput,
    priorPass: string,
    register: RegisterBand,
  ): string {
    const overlaySummary = this.buildOverlaySummary();
    const bridgeMandates = this.mode.frontmatter.bridge_mandates.map((m) => `- ${m}`).join('\n');
    const lessonsSummary = summarizeLessons(this.mode.lessons, 5);

    return template
      .replace(/\{\{subject_names\}\}/g, input.subjectNames.join(', '))
      .replace(/\{\{prior_pass\}\}/g, priorPass)
      .replace(/\{\{overlay_summary\}\}/g, overlaySummary)
      .replace(/\{\{bridge_mandates\}\}/g, bridgeMandates)
      .replace(/\{\{lessons_summary\}\}/g, lessonsSummary)
      .replace(/\{\{register\}\}/g, register)
      .replace(/\{\{pass_id\}\}/g, pass.id)
      .replace(/\{\{target_words\}\}/g, String(pass.target_words));
  }

  private buildSystemPrompt(
    pass: { id: string; title: string; target_words: number },
    input: OrchestratorInput,
    register: RegisterBand,
  ): string {
    const { min, max } = resolveTargetWords(this.mode, register, pass.id);
    return `You are writing pass "${pass.title}" (id: ${pass.id}) for the ${this.mode.frontmatter.mode} reading mode.
Register band: ${register}.
Target length: ~${pass.target_words} words (acceptable range ${min}-${max}).
Subjects: ${input.subjectNames.join(', ')}.
${this.mode.sections['overlay-rules'] ?? ''}`;
  }

  private buildOverlaySummary(): string {
    const weights = Object.entries(this.mode.frontmatter.engine_overlay_weights)
      .map(([k, v]) => `${k}: ${v}`)
      .join(', ');
    return `Engine weights: ${weights}; Houses: ${this.mode.frontmatter.house_overlay.join(', ')}`;
  }
}
