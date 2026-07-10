// packages/witness-pipeline/src/notebooklm/slides-prompt.ts
import type { OrchestratorOutput } from '../orchestrator/integrated.js';

export function generateSlidesPrompt(
  output: OrchestratorOutput,
  opts?: { language?: string }
): string {
  const language = opts?.language ?? 'en';
  return `Language: ${language}\nMode: ${output.mode}\nSlides prompt placeholder`;
}
