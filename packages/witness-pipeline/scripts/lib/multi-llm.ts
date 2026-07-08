// Multi-provider LLM adapter with tier-agnostic automatic fallback.
//
// Provider chain: Command Code → NVIDIA NIM → OpenRouter → OpenAI
// Each exposes an OpenAI-compatible /v1/chat/completions endpoint.
// Tier (L0-L5) only changes the number of NN calls and rubric depth,
// not the provider routing — the chain is identical across all tiers.
//
// Usage:
//   import { createLlmCall } from './lib/multi-llm.js';
//   const llm = createLlmCall();
//   const output = await llm(system, user, { max_tokens: 2048 });

import { readFileSync } from 'node:fs';
import { request as httpsRequest } from 'node:https';

// ─── Type ─────────────────────────────────────────────────────────────

type LlmCall = (system: string, user: string, options: { max_tokens: number }) => Promise<string>;

interface Provider {
  name: string;
  hostname: string;
  path: string;
  model: string;
  headers: (apiKey: string) => Record<string, string>;
}

interface LlmOptions {
  temperature?: number;
  timeout_ms?: number;
}

// ─── Key loading ──────────────────────────────────────────────────────

function loadKey(varName: string): string | undefined {
  if (process.env[varName]) return process.env[varName];
  // Try .claude/.env
  try {
    const home = process.env.HOME || '/Users/sheshnarayaniyer';
    const env = readFileSync(`${home}/.claude/.env`, 'utf8');
    const match = env.match(new RegExp(`${varName}=(.+)`));
    if (match) return match[1].trim();
  } catch {}
  return undefined;
}

function loadCodexOAuthToken(): string | undefined {
  try {
    const home = process.env.HOME || '/Users/sheshnarayaniyer';
    const auth = JSON.parse(readFileSync(`${home}/.codex/auth.json`, 'utf8'));
    return auth?.tokens?.access_token;
  } catch {}
  return undefined;
}

// ─── Provider catalog (in fallback order) ─────────────────────────────

const PROVIDERS: Provider[] = [
  {
    name: 'command-code',
    hostname: 'api.openai.com',
    path: '/v1/chat/completions',
    model: 'gpt-4o-mini',
    headers: (key) => ({ 'Authorization': `Bearer ${key}` }),
  },
  {
    name: 'nvidia',
    hostname: 'integrate.api.nvidia.com',
    path: '/v1/chat/completions',
    model: 'meta/llama-3.1-8b-instruct',
    headers: (key) => ({ 'Authorization': `Bearer ${key}` }),
  },
  {
    name: 'openrouter',
    hostname: 'openrouter.ai',
    path: '/api/v1/chat/completions',
    model: 'anthropic/claude-sonnet-4',
    headers: (key) => ({
      'Authorization': `Bearer ${key}`,
      'HTTP-Referer': 'https://tryambakam.space',
      'X-Title': 'Selemene L0 Witness',
    }),
  },
  {
    name: 'openai',
    hostname: 'api.openai.com',
    path: '/v1/chat/completions',
    model: 'gpt-4o-mini',
    headers: (key) => ({ 'Authorization': `Bearer ${key}` }),
  },
];

const PROVIDER_KEY_VARS: Record<string, string> = {
  'command-code': 'COMMANDCODE_API_KEY',
  nvidia: 'NVIDIA_API_KEY',
  openrouter: 'OPENROUTER_API_KEY',
  openai: 'OPENAI_API_KEY',
};

// ─── Core LLM call ────────────────────────────────────────────────────

async function callProvider(
  provider: Provider,
  apiKey: string,
  model: string,
  system: string,
  user: string,
  maxTokens: number,
  temperature: number,
  timeoutMs: number,
): Promise<string> {
  const body = JSON.stringify({
    model,
    messages: [
      { role: 'system', content: system },
      { role: 'user', content: user },
    ],
    max_tokens: Math.min(maxTokens, 16384),
    temperature,
  });

  return new Promise((resolve, reject) => {
    const req = httpsRequest(
      {
        hostname: provider.hostname,
        port: 443,
        path: provider.path,
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...provider.headers(apiKey),
        },
        timeout: timeoutMs,
      },
      (res) => {
        let data = '';
        res.on('data', (chunk: string) => (data += chunk));
        res.on('end', () => {
          if (res.statusCode !== 200) {
            const preview = data.slice(0, 200).replace(/\n/g, ' ');
            reject(new Error(`HTTP ${res.statusCode}: ${preview}`));
            return;
          }
          try {
            const parsed = JSON.parse(data);
            if (parsed.error) {
              reject(new Error(`API error: ${parsed.error.message || parsed.error.code}`));
              return;
            }
            const content = parsed.choices?.[0]?.message?.content;
            const reasoning = parsed.choices?.[0]?.message?.reasoning_content;
            const final = (content || reasoning || '').trim();
            if (!final) {
              reject(new Error('Empty response content'));
              return;
            }
            resolve(final);
          } catch (e) {
            reject(e);
          }
        });
        res.on('error', (e: Error) => reject(e));
      },
    );
    req.on('error', (e: Error) => reject(e));
    req.on('timeout', () => { req.destroy(); reject(new Error('timeout')); });
    req.write(body);
    req.end();
  });
}

export function createLlmCall(opts: LlmOptions = {}): LlmCall {
  const temperature = opts.temperature ?? 0.7;
  const timeoutMs = opts.timeout_ms ?? 120000;

  return async (system: string, user: string, callOpts: { max_tokens: number }): Promise<string> => {
    const maxTokens = callOpts.max_tokens || 4096;
    const errors: string[] = [];

    for (const provider of PROVIDERS) {
      let apiKey: string | undefined;
      if (provider.name === 'command-code') {
        // Command Code: try direct API key first, then Codex OAuth token
        apiKey = loadKey('COMMANDCODE_API_KEY') || loadCodexOAuthToken();
        if (!apiKey) {
          errors.push('command-code: no key (COMMANDCODE_API_KEY or Codex OAuth)');
          continue;
        }
      } else {
        const keyVar = PROVIDER_KEY_VARS[provider.name];
        apiKey = loadKey(keyVar);
        if (!apiKey) {
          errors.push(`${provider.name}: no key (${keyVar})`);
          continue;
        }
      }

      const start = Date.now();
      try {
        const result = await callProvider(
          provider, apiKey, provider.model, system, user, maxTokens, temperature, timeoutMs,
        );
        const latency = Date.now() - start;
        if (process.env.DEBUG_LLM) {
          process.stderr.write(`[llm] ${provider.name}/${provider.model} ${latency}ms ${result.length}chars\n`);
        }
        return result;
      } catch (e: any) {
        const msg = `${provider.name}: ${e.message} (${Date.now() - start}ms)`;
        errors.push(msg);
        if (process.env.DEBUG_LLM) {
          process.stderr.write(`[llm] FAIL ${msg}\n`);
        }
      }
    }

    throw new Error(`All LLM providers failed:\n${errors.map(e => `  - ${e}`).join('\n')}`);
  };
}

// ─── Probe utility ────────────────────────────────────────────────────

export async function probeProviders(): Promise<string[]> {
  const results: string[] = [];
  for (const provider of PROVIDERS) {
    const keyVar = PROVIDER_KEY_VARS[provider.name];
    const apiKey = loadKey(keyVar);
    if (!apiKey) { results.push(`${provider.name}: no key`); continue; }
    try {
      const start = Date.now();
      const out = await callProvider(
        provider, apiKey, provider.model,
        'Reply with exactly one word.',
        'What is the capital of France?',
        16, 0.0, 15000,
      );
      results.push(`${provider.name}/${provider.model}: OK (${Date.now()-start}ms) → "${out}"`);
    } catch (e: any) {
      results.push(`${provider.name}/${provider.model}: ${e.message}`);
    }
  }
  return results;
}