// packages/witness-pipeline/src/notebooklm/slides-prompt.ts
import type { OrchestratorOutput } from '../orchestrator/integrated.js';

export function generateSlidesPrompt(
  output: OrchestratorOutput,
  opts?: { language?: string; bridgeMandates?: string[] }
): string {
  const language = opts?.language ?? 'en';
  const header = output.relationship_header || 'Solo Reading — non-predictive pattern witness';
  const guard = 'Facts only. No prediction. No diagnosis. Use only observable patterns and engine data.';
  const mandates = (opts?.bridgeMandates || []).map(m => `- ${m}`).join('\n');

  const passLines = output.passes.length > 0
    ? output.passes.map((p, i) => `${i + 3}. ${p.title || p.id} (cite pass ${p.id})`)
    : ['3-8. One slide per major pass or pattern cluster (cite pass id)'];

  const slides = [
    `# NotebookLM Slides Prompt`,
    ``,
    `Language: ${language}`,
    `Mode: ${output.mode}`,
    `Register: ${output.register}`,
    ``,
    `## Relationship / Framing (MANDATORY — copy verbatim)`,
    header,
    guard,
    mandates ? `## Mode Mandates\n${mandates}` : '',
    ``,
    `## Slide Outline (8-12 slides recommended)`,
    `1. Title: ${header}`,
    `2. Subjects & Context`,
    ...passLines,
    `${output.passes.length + 3}. Synthesis (non-predictive)`,
    `${output.passes.length + 4}. Reflection questions`,
    ``,
    `Source assembled length (chars): ${output.assembled.length}`,
    `Pass count: ${output.passes.length}`,
  ].filter(Boolean);

  return slides.join('\n');
}
