// Tests for the W2 shared-secret gate on the chat endpoint.
// Runs on Node's built-in test runner (node --test) — no new dependencies.
// Node >= 22.18 strips TypeScript types natively; the worker imports only
// type-only Cloudflare bindings, so it loads cleanly outside workerd.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import worker, { isChatRequestAuthorized } from './index.ts';

interface TestEnv {
  LLM_SECRETS: { get(key: string): Promise<string | null> };
  CHAT_PROXY_TOKEN?: string;
}

const chatReq = (headers: Record<string, string> = {}) =>
  new Request('https://llm-proxy.test/v1/chat/completions', {
    method: 'POST',
    headers: { 'content-type': 'application/json', ...headers },
    body: JSON.stringify({ messages: [{ role: 'user', content: 'hi' }] }),
  });

test('inert by default: no CHAT_PROXY_TOKEN → authorized without any header', () => {
  const env = { LLM_SECRETS: { get: async () => null } } as TestEnv as never;
  assert.equal(isChatRequestAuthorized(chatReq(), env), true);
});

test('configured token: missing/wrong header rejected, exact match accepted', () => {
  const env = { LLM_SECRETS: { get: async () => null }, CHAT_PROXY_TOKEN: 's3cret' } as TestEnv as never;
  assert.equal(isChatRequestAuthorized(chatReq(), env), false);
  assert.equal(isChatRequestAuthorized(chatReq({ 'x-chat-key': 'wrong' }), env), false);
  assert.equal(isChatRequestAuthorized(chatReq({ 'x-chat-key': 's3cret' }), env), true);
});

test('fetch: 401 before any KV/provider work when the gate rejects', async () => {
  let kvTouched = false;
  const env = {
    LLM_SECRETS: {
      get: async () => {
        kvTouched = true;
        return null;
      },
    },
    CHAT_PROXY_TOKEN: 's3cret',
  } as TestEnv as never;

  const res = await worker.fetch(chatReq(), env);
  assert.equal(res.status, 401);
  assert.equal(kvTouched, false, 'KV must not be touched on an unauthorized request');
  const body = (await res.json()) as { error?: { message?: string } };
  assert.match(body.error?.message ?? '', /x-chat-key/);
});

test('fetch: authorized requests route to providers exactly as before', async () => {
  const env = {
    LLM_SECRETS: { get: async (key: string) => (key === 'COMMANDCODE_API_KEY' ? 'provider-key' : null) },
    CHAT_PROXY_TOKEN: 's3cret',
  } as TestEnv as never;

  const originalFetch = globalThis.fetch;
  let providerAuth = '';
  globalThis.fetch = (async (_url: unknown, init?: { headers?: Record<string, string> }) => {
    providerAuth = init?.headers?.['Authorization'] ?? '';
    return Response.json({ choices: [{ message: { content: 'ok' } }] });
  }) as typeof fetch;
  try {
    const res = await worker.fetch(chatReq({ 'x-chat-key': 's3cret' }), env);
    assert.equal(res.status, 200);
    const body = (await res.json()) as { provider?: string };
    assert.equal(body.provider, 'command-code'); // provider routing untouched
    assert.equal(providerAuth, 'Bearer provider-key');
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test('fetch: unconfigured token keeps the pre-W2 open behavior end to end', async () => {
  const env = {
    LLM_SECRETS: { get: async (key: string) => (key === 'COMMANDCODE_API_KEY' ? 'provider-key' : null) },
  } as TestEnv as never;

  const originalFetch = globalThis.fetch;
  globalThis.fetch = (async () =>
    Response.json({ choices: [{ message: { content: 'ok' } }] })) as typeof fetch;
  try {
    const res = await worker.fetch(chatReq(), env); // no x-chat-key at all
    assert.equal(res.status, 200);
  } finally {
    globalThis.fetch = originalFetch;
  }
});
