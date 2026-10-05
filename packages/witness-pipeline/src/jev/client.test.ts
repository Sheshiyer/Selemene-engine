import { describe, it, expect, vi } from 'vitest';
import { jevSystemOne, createJevClient, JevError, choice, score, noul, JEV_ENDPOINT, JEV_MODEL, type JevResult } from './client.js';

const RESULT: JevResult = {
  model: 'jev-1.13.0',
  answers: {
    guardrail_clean: { type: 'noul', noul: 0.93 },
    fact_grounding: { type: 'score', score: 2.4, legend: { '0': 'none', '3': 'dense' }, probabilities: { '2': 0.5, '3': 0.4 }, confidence: 0.5 },
    register_fit: { type: 'choice', choice: 'l1_l3', probabilities: { l1_l3: 0.8, l4_l5: 0.2 }, confidence: 0.8 },
  },
  usage: { input_tokens: 300, output_tokens: 20 },
};

describe('jev client', () => {
  it('builders produce the System One shapes', () => {
    expect(choice('q', { a: 'A' })).toEqual({ type: 'choice', instructions: 'q', criteria: { a: 'A' } });
    expect(score('q', ['x', 'y'])).toEqual({ type: 'score', instructions: 'q', criteria: ['x', 'y'] });
    expect(noul()).toEqual({ type: 'noul' });
  });

  it('posts the request shape with bearer auth and returns the typed result', async () => {
    const fetchImpl = vi.fn(async () => Response.json(RESULT)) as unknown as typeof fetch;
    const out = await jevSystemOne({ state: { pass_id: 'opening' }, questions: { g: noul('clean?') } }, { apiKey: 'k', fetchImpl });
    const [url, init] = (fetchImpl as any).mock.calls[0];
    expect(url).toBe(JEV_ENDPOINT);
    expect((init.headers as any).Authorization).toBe('Bearer k');
    const body = JSON.parse(init.body);
    expect(body.model).toBe(JEV_MODEL);
    expect(body.state.pass_id).toBe('opening');
    expect(body.questions.g.type).toBe('noul');
    expect(out.answers.guardrail_clean).toEqual({ type: 'noul', noul: 0.93 });
  });

  it('retries once on 429/5xx and throws JevError on other failures', async () => {
    const seq = [new Response('slow down', { status: 429 }), Response.json(RESULT)];
    const fetchImpl = vi.fn(async () => seq.shift()!) as unknown as typeof fetch;
    const out = await jevSystemOne({ state: {}, questions: {} }, { apiKey: 'k', fetchImpl });
    expect((fetchImpl as any).mock.calls.length).toBe(2);
    expect(out.model).toBe('jev-1.13.0');

    const bad = vi.fn(async () => new Response('nope', { status: 401 })) as unknown as typeof fetch;
    await expect(jevSystemOne({ state: {}, questions: {} }, { apiKey: 'k', fetchImpl: bad })).rejects.toBeInstanceOf(JevError);
  });

  it('createJevClient returns null without a key so callers degrade honestly', () => {
    expect(createJevClient(undefined)).toBeNull();
    expect(typeof createJevClient('k')).toBe('function');
  });
});

describe('parseTypesafeKey', () => {
  it('prefers TYPESAFE_API_KEY, accepts export/quoted forms, and falls back to the API_KEY under a Jev header only', async () => {
    const { parseTypesafeKey } = await import('./client.js');
    expect(parseTypesafeKey('export TYPESAFE_API_KEY="abc"\n')).toBe('abc');
    expect(parseTypesafeKey('####JEV-TYPESAFE-AI####\nAPI_KEY=jevkey\n\n####TEMPERANCE####\nAPI_KEY=other\n')).toBe('jevkey');
    expect(parseTypesafeKey('####TEMPERANCE####\nAPI_KEY=other\n')).toBeUndefined();
    expect(parseTypesafeKey('####JEV####\n# API_KEY=commented\n')).toBeUndefined();
  });
});
