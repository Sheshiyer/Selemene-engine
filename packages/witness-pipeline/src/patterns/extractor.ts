import type { PassResult } from '../orchestrator/integrated.js';
import type { ReportLevel } from '../modes/types.js';
import type { ExtractedPattern } from './types.js';
import { scrubPrivateBirthData } from './privacy.js';

export interface ExtractReportPatternsInput {
  mode: string;
  reportLevel: ReportLevel;
  subjectNames: string[];
  passes: PassResult[];
  language?: string;
  relationship_type?: string;
}

function anonymize(text: string, subjectNames: string[]): string {
  let out = text;
  for (const name of subjectNames) {
    out = out.replace(new RegExp(name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'gi'), '[SUBJECT]');
  }
  return out;
}

function extractPatternText(output: string): string {
  const sentences = output.split(/[.!?]+/).map(s => s.trim()).filter(Boolean);
  const candidates = sentences.filter(s => s.length > 20);
  return candidates.length > 0 ? candidates[0] + '.' : output.slice(0, 120);
}

function inferKind(sectionId: string): ExtractedPattern['kind'] {
  if (/remed|practice/i.test(sectionId)) return 'remedy-pattern';
  if (/converg|map/i.test(sectionId)) return 'convergence';
  if (/tens|conflict|shadow/i.test(sectionId)) return 'tension';
  if (/guard|risk|caution/i.test(sectionId)) return 'guardrail-safe-framing';
  return 'section-structure';
}

function scoreRubric(rubric: PassResult['rubric']): number {
  const wc = rubric.word_count_fit === 'pass' ? 1 : rubric.word_count_fit === 'warn' ? 0.7 : 0.4;
  const df = rubric.deterministic_fact_gate === 'pass' ? 1 : rubric.deterministic_fact_gate === 'warn' ? 0.7 : 0.4;
  const il = rubric.integrated_layering_gate === 'pass' ? 1 : rubric.integrated_layering_gate === 'warn' ? 0.7 : 0.4;
  const gr = rubric.guardrail_gate === 'pass' ? 1 : 0;
  return Number(((wc + df + il + gr) / 4).toFixed(2));
}

function inferSystems(output: string): string[] {
  const systems: string[] = [];
  if (/\b(Vedic|Lagna|nakshatra|Saturn|Jupiter)\b/i.test(output)) systems.push('vedic');
  if (/\b(Human Design|gate|channel|Projector|Generator)\b/i.test(output)) systems.push('human-design');
  if (/\b(Gene Key|Evolution|Vocation|Pearl)\b/i.test(output)) systems.push('gene-keys');
  if (/\b(panchanga|tithi|transit)\b/i.test(output)) systems.push('panchanga');
  if (/\b(Vimshottari|mahadasha)\b/i.test(output)) systems.push('vimshottari');
  return systems.length > 0 ? systems : ['generic'];
}

export function extractReportPatterns(input: ExtractReportPatternsInput): ExtractedPattern[] {
  return input.passes
    .filter((p) => p.rubric.guardrail_gate === 'pass')
    .filter((p) => p.rubric.integrated_layering_gate === 'pass')
    .map((p) => {
      const { scrubbed, hadPrivate } = scrubPrivateBirthData(p.output);
      if (hadPrivate) {
        return null;
      }
      return {
        id: `${input.mode}:${p.id}:v1`,
        text: anonymize(extractPatternText(scrubbed), input.subjectNames),
        kind: inferKind(p.id),
        source_section_id: p.id,
        source_rubric_score: scoreRubric(p.rubric),
        metadata: {
          mode: input.mode,
          report_level: input.reportLevel,
          language: (input as any).language ?? 'en',
          relationship_type: (input as any).relationship_type,
          systems: inferSystems(scrubbed),
          source: 'post-report-extraction' as const,
          version: '1',
        },
      };
    })
    .filter((p): p is ExtractedPattern => p !== null)
    .filter((p) => p.text.length >= 40);
}
