import { afterEach, describe, expect, it, vi } from 'vitest';
import { createOmniRouteLlmCall } from './multi-llm.js';

afterEach(() => { vi.unstubAllGlobals(); vi.unstubAllEnvs(); });

function response(events: unknown[]) {
  const bytes = new TextEncoder().encode(events.map(e => `data: ${JSON.stringify(e)}\n\n`).join('') + 'data: [DONE]\n\n');
  return new Response(new ReadableStream({ start(controller) {
    // Split across UTF-8 and SSE event boundaries to exercise the incremental decoder.
    for (let i = 0; i < bytes.length; i += 7) controller.enqueue(bytes.slice(i, i + 7));
    controller.close();
  }}), { status: 200 });
}

describe('reference combo streaming transport', () => {
  it('retains final prose and model usage while ignoring reasoning deltas', async () => {
    vi.stubEnv('OMNIROUTE_API_KEY', 'test-only');
    const fetcher = vi.fn(async () => response([
      { model: 'resolved-model', choices: [{ delta: { reasoning_content: 'private scratch', content: 'Précision ' } }] },
      { choices: [{ delta: { content: 'et réflexion.' }, finish_reason: 'stop' }], usage: { completion_tokens: 12 } },
    ]));
    vi.stubGlobal('fetch', fetcher);
    const receipt = vi.fn();
    const output = await createOmniRouteLlmCall('noesis-execute', { onReceipt: receipt })('system', 'user', { max_tokens: 4000 });
    expect(output).toBe('Précision et réflexion.');
    expect(receipt.mock.calls[0][0].returned_model).toBe('resolved-model');
    expect(JSON.parse((fetcher.mock.calls[0] as any)[1].body).stream).toBe(true);
  });
  it('rejects a stream ending without a clean final completion', async () => {
    vi.stubEnv('OMNIROUTE_API_KEY', 'test-only');
    vi.stubGlobal('fetch', vi.fn(async () => response([{ choices: [{ delta: { content: 'Partial' } }] }])));
    await expect(createOmniRouteLlmCall('noesis-execute')('s', 'u', { max_tokens: 100 })).rejects.toThrow('finish_reason=missing, content_chars=7, max_tokens=100');
  });
  it('reports token truncation without exposing partial content', async () => {
    vi.stubEnv('OMNIROUTE_API_KEY', 'test-only');
    const receipt = vi.fn();
    vi.stubGlobal('fetch', vi.fn(async () => response([{ model: 'audit-model', choices: [{ delta: { content: 'sensitive' }, finish_reason: 'length' }] }])));
    const error = await createOmniRouteLlmCall('noesis-verify', { onReceipt: receipt })('s', 'u', { max_tokens: 20000 }).catch(e => e);
    expect(error.message).toContain('model=audit-model, finish_reason=length, content_chars=9, max_tokens=16384');
    expect(error.message).not.toContain('sensitive');
    expect(receipt).not.toHaveBeenCalled();
  });
});
