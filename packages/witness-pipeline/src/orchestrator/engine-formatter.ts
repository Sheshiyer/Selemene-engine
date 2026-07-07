import type { SelemeneEngineOutput } from '../index.js';

export function formatEngineResultsForPrompt(
  engineResults: SelemeneEngineOutput[],
): string {
  const blocks = engineResults.map((engine) => {
    const result = engine.result ?? {};
    return `### ${engine.engine_id}\n\n${jsonBlock(result)}`;
  });
  return `## Deterministic Engine Results\n\n${blocks.join('\n\n')}`;
}

function jsonBlock(value: unknown): string {
  return '```json\n' + JSON.stringify(value, null, 2) + '\n```';
}
