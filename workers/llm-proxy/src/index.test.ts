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

test('command-code default: Provider API endpoint + deepseek/deepseek-v4-pro', async () => {
  const env = {
    LLM_SECRETS: { get: async (key: string) => (key === 'COMMANDCODE_API_KEY' ? 'provider-key' : null) },
  } as TestEnv as never;

  const originalFetch = globalThis.fetch;
  let calledUrl = '';
  let sentBody: { model?: string; tools?: unknown[]; tool_choice?: unknown } = {};
  globalThis.fetch = (async (url: unknown, init?: { body?: string }) => {
    calledUrl = String(url);
    sentBody = JSON.parse(init?.body ?? '{}');
    return Response.json({ choices: [{ message: { content: 'ok' } }] });
  }) as typeof fetch;
  try {
    const req = new Request('https://llm-proxy.test/v1/chat/completions', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        messages: [{ role: 'user', content: 'hi' }],
        tools: [{ type: 'function', function: { name: 'record_intake', parameters: {} } }],
      }),
    });
    const res = await worker.fetch(req, env);
    assert.equal(res.status, 200);
    assert.equal(calledUrl, 'https://api.commandcode.ai/provider/v1/chat/completions');
    assert.equal(sentBody.model, 'deepseek/deepseek-v4-pro'); // omitted model → provider default
    assert.equal(Array.isArray(sentBody.tools), true, 'tools must be forwarded');
    assert.equal(sentBody.tool_choice, 'auto');
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test('nebius fallback: Nebius AI Studio endpoint + Llama-3.3-70B default model', async () => {
  const env = {
    // No COMMANDCODE key → chain falls through to nebius.
    LLM_SECRETS: { get: async (key: string) => (key === 'NEBIUS_API_KEY' ? 'nb-key' : null) },
  } as TestEnv as never;

  const originalFetch = globalThis.fetch;
  let calledUrl = '';
  let sentBody: { model?: string } = {};
  globalThis.fetch = (async (url: unknown, init?: { body?: string }) => {
    calledUrl = String(url);
    sentBody = JSON.parse(init?.body ?? '{}');
    return Response.json({ choices: [{ message: { content: 'ok' } }] });
  }) as typeof fetch;
  try {
    const res = await worker.fetch(chatReq(), env);
    assert.equal(res.status, 200);
    const body = (await res.json()) as { provider?: string };
    assert.equal(body.provider, 'nebius');
    assert.equal(calledUrl, 'https://api.studio.nebius.com/v1/chat/completions');
    assert.equal(sentBody.model, 'meta-llama/Llama-3.3-70B-Instruct');
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test('nvidia fallback: NIM endpoint + nemotron-super-49b default model', async () => {
  const env = {
    // No COMMANDCODE/NEBIUS key → chain falls through to nvidia.
    LLM_SECRETS: { get: async (key: string) => (key === 'NVIDIA_API_KEY' ? 'nv-key' : null) },
  } as TestEnv as never;

  const originalFetch = globalThis.fetch;
  let calledUrl = '';
  let sentBody: { model?: string } = {};
  globalThis.fetch = (async (url: unknown, init?: { body?: string }) => {
    calledUrl = String(url);
    sentBody = JSON.parse(init?.body ?? '{}');
    return Response.json({ choices: [{ message: { content: 'ok' } }] });
  }) as typeof fetch;
  try {
    const res = await worker.fetch(chatReq(), env);
    assert.equal(res.status, 200);
    const body = (await res.json()) as { provider?: string };
    assert.equal(body.provider, 'nvidia');
    assert.equal(calledUrl, 'https://integrate.api.nvidia.com/v1/chat/completions');
    assert.equal(sentBody.model, 'nvidia/llama-3.3-nemotron-super-49b-v1.5');
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test('kimi fallback (tertiary): Moonshot endpoint + kimi-k2.6 default model', async () => {
  const env = {
    // Only MOONSHOT key → chain falls through command-code/nebius/nvidia to kimi.
    LLM_SECRETS: { get: async (key: string) => (key === 'MOONSHOT_API_KEY' ? 'ms-key' : null) },
  } as TestEnv as never;

  const originalFetch = globalThis.fetch;
  let calledUrl = '';
  let sentBody: { model?: string } = {};
  globalThis.fetch = (async (url: unknown, init?: { body?: string }) => {
    calledUrl = String(url);
    sentBody = JSON.parse(init?.body ?? '{}');
    return Response.json({ choices: [{ message: { content: 'ok' } }] });
  }) as typeof fetch;
  try {
    const res = await worker.fetch(chatReq(), env);
    assert.equal(res.status, 200);
    const body = (await res.json()) as { provider?: string };
    assert.equal(body.provider, 'kimi');
    assert.equal(calledUrl, 'https://api.moonshot.ai/v1/chat/completions');
    assert.equal(sentBody.model, 'kimi-k2.6');
  } finally {
    globalThis.fetch = originalFetch;
  }
});
