// Selemene LLM Proxy Worker
// Stores provider API keys in KV, routes to the active provider.
// Provider chain (in order): Command Code → NVIDIA NIM → OpenRouter → OpenAI
//
// Endpoint: POST /v1/chat/completions  (OpenAI-compatible)

interface Env {
  LLM_SECRETS: KVNamespace;
  /**
   * Optional shared-secret gate for the chat endpoint (urania chat
   * onboarding, Phase 1 W2). When configured, requests must carry
   * `x-chat-key: <token>`; when unset the worker behaves exactly as before
   * (no auth check). Set via `wrangler secret put CHAT_PROXY_TOKEN`.
   */
  CHAT_PROXY_TOKEN?: string;
}

/**
 * Shared-secret gate, inert by default: when CHAT_PROXY_TOKEN is not
 * configured every request is authorized (pre-W2 behavior); when configured,
 * the `x-chat-key` header must match exactly. Exported for tests.
 */
export function isChatRequestAuthorized(request: Request, env: Env): boolean {
  const token = env.CHAT_PROXY_TOKEN?.trim();
  if (!token) return true;
  return request.headers.get('x-chat-key') === token;
}

interface ChatRequest {
  model?: string;
  messages: Array<{ role: string; content: string }>;
  max_tokens?: number;
  temperature?: number;
}

interface Provider {
  name: string;
  hostname: string;
  path: string;
  defaultModel: string;
  keyKvKey: string;
}

const PROVIDERS: Provider[] = [
  {
    name: 'command-code',
    hostname: 'api.openai.com',
    path: '/v1/chat/completions',
    defaultModel: 'gpt-4o-mini',
    keyKvKey: 'COMMANDCODE_API_KEY',
  },
  {
    name: 'nvidia',
    hostname: 'integrate.api.nvidia.com',
    path: '/v1/chat/completions',
    defaultModel: 'meta/llama-3.1-8b-instruct',
    keyKvKey: 'NVIDIA_API_KEY',
  },
  {
    name: 'openrouter',
    hostname: 'openrouter.ai',
    path: '/api/v1/chat/completions',
    defaultModel: 'anthropic/claude-sonnet-4',
    keyKvKey: 'OPENROUTER_API_KEY',
  },
  {
    name: 'openai',
    hostname: 'api.openai.com',
    path: '/v1/chat/completions',
    defaultModel: 'gpt-4o-mini',
    keyKvKey: 'OPENAI_API_KEY',
  },
];

async function callProvider(
  provider: Provider,
  apiKey: string,
  body: ChatRequest,
  signal: AbortSignal,
): Promise<Response> {
  const model = body.model || provider.defaultModel;
  const headers: Record<string, string> = {
    'Authorization': `Bearer ${apiKey}`,
    'Content-Type': 'application/json',
  };
  if (provider.name === 'openrouter') {
    headers['HTTP-Referer'] = 'https://tryambakam.space';
    headers['X-Title'] = 'Selemene L0 Witness';
  }

  const fetchBody = JSON.stringify({
    model,
    messages: body.messages,
    max_tokens: Math.min(body.max_tokens || 4096, 16384),
    temperature: body.temperature ?? 0.7,
  });

  return fetch(`https://${provider.hostname}${provider.path}`, {
    method: 'POST',
    headers,
    body: fetchBody,
    signal,
  });
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    if (request.method !== 'POST') {
      return new Response('POST only', { status: 405 });
    }

    // Shared-secret gate (W2) — checked before any KV/provider work so an
    // unauthorized request costs nothing and learns nothing. Inert when
    // CHAT_PROXY_TOKEN is unset.
    if (!isChatRequestAuthorized(request, env)) {
      return Response.json(
        { error: { message: 'unauthorized: missing or invalid x-chat-key header' } },
        { status: 401 },
      );
    }

    let body: ChatRequest;
    try {
      body = await request.json();
    } catch {
      return new Response('Invalid JSON', { status: 400 });
    }

    if (!body.messages?.length) {
      return new Response('messages required', { status: 400 });
    }

    // Try each provider in order
    const errors: string[] = [];
    for (const provider of PROVIDERS) {
      const apiKey = await env.LLM_SECRETS.get(provider.keyKvKey);
      if (!apiKey) {
        errors.push(`${provider.name}: no key in KV (${provider.keyKvKey})`);
        continue;
      }

      const controller = new AbortController();
      const timeout = setTimeout(() => controller.abort(), 120000);

      try {
        const start = Date.now();
        const res = await callProvider(provider, apiKey, body, controller.signal);

        if (res.ok) {
          const data = await res.json() as any;
          const latency = Date.now() - start;
          // Tag the response to identify which provider was used
          if (data.choices?.[0]) {
            data.provider = provider.name;
            data._latency_ms = latency;
          }
          return Response.json(data);
        }

        const errText = await res.text().catch(() => '');
        errors.push(`${provider.name}: HTTP ${res.status} ${errText.slice(0, 100)}`);
      } catch (e: any) {
        errors.push(`${provider.name}: ${e.message}`);
      } finally {
        clearTimeout(timeout);
      }
    }

    return Response.json(
      { error: { message: `All providers failed:\n${errors.map(e => `  - ${e}`).join('\n')}` } },
      { status: 502 },
    );
  },
};