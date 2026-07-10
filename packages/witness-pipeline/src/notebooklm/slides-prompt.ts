// packages/witness-pipeline/src/notebooklm/slides-prompt.ts
import type { OrchestratorOutput } from '../orchestrator/integrated.js';

export function generateSlidesPrompt(
  output: OrchestratorOutput,
  opts?: { language?: string }
): string {
  const language = opts?.language ?? 'en';
  const header = output.relationship_header
    ? output.relationship_header
    : 'Solo Reading — non-predictive pattern witness';

  const guard = 'Facts only. No prediction. No diagnosis. Use only observable patterns and engine data.';

  const slides = [
    `# NotebookLM Slides Prompt`,
    ``,
    `Language: ${language}`,
    `Mode: ${output.mode}`,
    `Register: ${output.register}`,
    ``,
    `## Relationship / Framing (MANDATORY — copy verbatim into every slide)`,
    header,
    guard,
    ``,
    `## Slide Outline (8-12 slides recommended)`,
    `1. Title: ${header}`,
    `2. Subjects & Context`,
    `3-8. One slide per major pass or pattern cluster (cite pass id)`,
    `9. Synthesis (non-predictive)`,
    `10. Reflection questions`,
    ``,
    `Source assembled length (chars): ${output.assembled.length}`,
    `Pass count: ${output.passes.length}`,
  ];

  return slides.join('\n');
}
