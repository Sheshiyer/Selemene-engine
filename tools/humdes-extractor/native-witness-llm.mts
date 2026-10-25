/** Native Codex-agent mailbox. Request/response bytes are retained; no API key. */
import { mkdir, writeFile, readFile, access } from 'node:fs/promises';
import { join } from 'node:path';
import { randomUUID, createHash } from 'node:crypto';
import type { LlmCall } from '../../packages/witness-pipeline/src/orchestrator/integrated.js';
export interface NativeCallReceipt {
  id: string; lane: string; agent: string; role: string; requestHash: string;
  responseHash: string; elapsedMs: number; modelAttribution: string;
}
export function nativeWitnessLlm(queue: string, lane: 'generation' | 'audit', onReceipt: (r: NativeCallReceipt) => void): LlmCall {
  return async (system, user, options) => {
    const id = randomUUID();
    const directory = join(queue, lane);
    await mkdir(directory, { recursive: true });
    const request = { id, lane, createdAt: new Date().toISOString(), system, user, maxTokens: options.max_tokens };
    const bytes = JSON.stringify(request, null, 2);
    const started = Date.now();
    await writeFile(join(directory, `${id}.request.json`), bytes, { flag: 'wx' });
    const answer = join(directory, `${id}.response.json`);
    for (;;) {
      try {
        await access(answer);
        const responseBytes = await readFile(answer, 'utf8');
        const response = JSON.parse(responseBytes);
        if (response.requestId !== id || typeof response.content !== 'string' || !response.content.trim() || !response.agent) {
          throw new Error('Malformed native-agent response');
        }
        if (response.error) throw new Error(response.error);
        await writeFile(join(directory, `${id}.consumed-response.json`), responseBytes, { flag: 'wx' });
        onReceipt({ id, lane, agent: response.agent, role: response.role ?? lane,
          requestHash: createHash('sha256').update(bytes).digest('hex'),
          responseHash: createHash('sha256').update(response.content.trim()).digest('hex'),
          elapsedMs: Date.now() - started, modelAttribution: 'native Codex GPT agent; exact model identifier unavailable in agent response' });
        return response.content.trim();
      } catch (error) {
        if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error;
      }
      if (Date.now() - started > 1800000) throw new Error(`Native agent request timed out: ${id}`);
      await new Promise(r => setTimeout(r, 1000));
    }
  };
}
