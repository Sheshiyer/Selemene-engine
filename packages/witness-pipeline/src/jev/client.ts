// ─── TypeSafe Jev (System One) client ────────────────────────────────
// Ported from urania-137 functions/lib/jev-client.ts (deployed v0.7.1) with an
// injectable fetch for tests. Jev answers typed questions (choice / score / noul);
// it never writes prose. Retries 429/5xx once; other errors surface immediately.

export const JEV_ENDPOINT = 'https://api.typesafe.ai/v1/systemone';
export const JEV_MODEL = 'jev-latest';
const DEFAULT_TIMEOUT_MS = 8_000;

export interface ChoiceQuestion<T extends Record<string, string> = Record<string, string>> {
  type: 'choice';
  instructions: string;
  criteria: T;
}
export interface ScoreQuestion {
  type: 'score';
  instructions: string;
  criteria: string[];
}
export interface NoulQuestion {
  type: 'noul';
  instructions?: string;
  criteria?: { true?: string; false?: string };
}
export type Question = ChoiceQuestion | ScoreQuestion | NoulQuestion;

export function choice<T extends Record<string, string>>(instructions: string, criteria: T): ChoiceQuestion<T> {
  return { type: 'choice', instructions, criteria };
}
export function score(instructions: string, criteria: string[]): ScoreQuestion {
  return { type: 'score', instructions, criteria };
}
export function noul(instructions?: string, criteria?: { true?: string; false?: string }): NoulQuestion {
  return { type: 'noul', ...(instructions ? { instructions } : {}), ...(criteria ? { criteria } : {}) };
}

export interface ChoiceResponse { type: 'choice'; choice: string; probabilities: Record<string, number>; confidence: number }
export interface ScoreResponse { type: 'score'; score: number; legend: Record<string, string>; probabilities: Record<string, number>; confidence: number }
export interface NoulResponse { type: 'noul'; noul: number }
export type Answer = ChoiceResponse | ScoreResponse | NoulResponse;

export interface JevRequest { state: unknown; questions: Record<string, Question>; model?: string }
export interface JevResult { model: string; answers: Record<string, Answer>; usage?: { input_tokens: number; output_tokens: number } }

export type JevClient = (request: JevRequest) => Promise<JevResult>;

export interface JevClientOptions {
  apiKey: string;
  timeoutMs?: number;
  fetchImpl?: typeof fetch;
  endpoint?: string;
}

export class JevError extends Error {
  constructor(public readonly status: number, public readonly body: string) {
    super(`Jev API ${status}: ${body.slice(0, 200)}`);
    this.name = 'JevError';
  }
}

async function fetchWithTimeout(fetchImpl: typeof fetch, url: string, init: RequestInit, timeoutMs: number): Promise<Response> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  try {
    return await fetchImpl(url, { ...init, signal: controller.signal });
  } finally {
    clearTimeout(timer);
  }
}

export async function jevSystemOne(request: JevRequest, opts: JevClientOptions): Promise<JevResult> {
  const timeout = opts.timeoutMs ?? DEFAULT_TIMEOUT_MS;
  const fetchImpl = opts.fetchImpl ?? fetch;
  const endpoint = opts.endpoint ?? JEV_ENDPOINT;
  const body = JSON.stringify({ state: request.state, model: request.model ?? JEV_MODEL, questions: request.questions });
  const headers: Record<string, string> = { Authorization: `Bearer ${opts.apiKey}`, 'Content-Type': 'application/json' };

  let lastError: Error | null = null;
  for (let attempt = 0; attempt < 2; attempt++) {
    if (attempt > 0) await new Promise((r) => setTimeout(r, 500 * attempt));
    let res: Response;
    try {
      res = await fetchWithTimeout(fetchImpl, endpoint, { method: 'POST', headers, body }, timeout);
    } catch (err) {
      lastError = err instanceof Error ? err : new Error(String(err));
      continue;
    }
    if (res.status === 429 || res.status >= 500) {
      lastError = new JevError(res.status, await res.text());
      continue;
    }
    if (!res.ok) throw new JevError(res.status, await res.text());
    return (await res.json()) as JevResult;
  }
  throw lastError ?? new Error('Jev request failed');
}

/** Bound client from a TYPESAFE_API_KEY; null when the key is absent so callers degrade honestly. */
export function createJevClient(apiKey: string | undefined, opts: Omit<JevClientOptions, 'apiKey'> = {}): JevClient | null {
  if (!apiKey) return null;
  return (request) => jevSystemOne(request, { apiKey, ...opts });
}

/**
 * Load the TypeSafe key. Order: `TYPESAFE_API_KEY` in env; `TYPESAFE_API_KEY=` in
 * ~/.claude/.env; else the `API_KEY=` line that sits under a `#...JEV...` or
 * `#...TYPESAFE...` section header in that file (the operator's layout). Never logs the value.
 */
export async function loadTypesafeKey(): Promise<string | undefined> {
  if (process.env.TYPESAFE_API_KEY) return process.env.TYPESAFE_API_KEY;
  const { default: fs } = await import('node:fs');
  const { default: path } = await import('node:path');
  const { default: os } = await import('node:os');
  const envPath = path.join(os.homedir(), '.claude', '.env');
  if (!fs.existsSync(envPath)) return undefined;
  return parseTypesafeKey(fs.readFileSync(envPath, 'utf-8'));
}

/** Pure parser for the env-file layouts above; exported for tests. */
export function parseTypesafeKey(text: string): string | undefined {
  const direct = text.match(/^(?:export\s+)?TYPESAFE_API_KEY\s*=\s*["']?([^"'\s#]+)/m);
  if (direct) return direct[1];
  let inJevSection = false;
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (line.startsWith('#')) { inJevSection = /jev|typesafe/i.test(line); continue; }
    if (!line) continue;
    const m = line.match(/^(?:export\s+)?API_KEY\s*=\s*["']?([^"'\s#]+)/);
    if (m && inJevSection) return m[1];
  }
  return undefined;
}
