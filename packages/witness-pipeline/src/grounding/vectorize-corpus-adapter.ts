// ─── Cloudflare Vectorize Corpus Adapter ─────────────────────────────────
// Read-only adapter for the witness-wisdom-corpus Vectorize index.
// Uses wrangler CLI via execFile (NO shell: true) — no embeddings are generated.
// Two retrieval modes:
//   1. Direct lookup:  wrangler vectorize get-vectors   (exact corpus IDs)
//   2. Semantic neigh: wrangler vectorize query --vector-id  (same embedding space)
//
// PRIVACY RULES (enforced here, not in callers):
//   - Subject names and birth data MUST NOT appear in arguments.
//   - Private subject entries (containing recognisable birth fields) are rejected.
//   - Generic placeholder excerpts from 58/64 HD-gates are rejected.
//   - Namespace IDs outside sw:* are rejected (e.g. report-pattern vectors).
//   - Vectors are NEVER retained in the receipt beyond the passage text + hash.

import { execFile as nodeExecFile } from 'node:child_process';
import { createHash } from 'node:crypto';
import { promisify } from 'node:util';
import type {
  VectorizeCorpusAdapterOptions,
  CommandRunner,
  CommandRunnerResult,
  CorpusPassage,
  CorpusReceipt,
  CorpusReceiptState,
  WranglerGetVectorsResponse,
  WranglerQueryResponse,
  CorpusPassageMetadata,
  WranglerVector,
  WranglerQueryMatch,
} from './types.js';

// ─── Constants ───────────────────────────────────────────────────────────────

const DEFAULT_INDEX = 'witness-wisdom-corpus';
const DEFAULT_TIMEOUT_MS = 15_000;
const CORPUS_NS_PREFIX = 'sw:';

// ─── Default wrangler execFile runner ────────────────────────────────────────

const execFileAsync = promisify(nodeExecFile);

function createDefaultRunner(): CommandRunner {
  return {
    async run(
      command: string,
      args: string[],
      options?: { timeoutMs?: number; env?: NodeJS.ProcessEnv },
    ): Promise<CommandRunnerResult> {
      try {
        const { stdout, stderr } = await execFileAsync(command, args, {
          timeout: options?.timeoutMs ?? DEFAULT_TIMEOUT_MS,
          maxBuffer: 8 * 1024 * 1024,
          shell: false,
          env: options?.env,
        });
        return { stdout: stdout ?? '', stderr: stderr ?? '', exitCode: 0 };
      } catch (err: unknown) {
        const e = err as NodeJS.ErrnoException & {
          stdout?: string;
          stderr?: string;
          code?: number | string;
          killed?: boolean;
        };
        const exitCode = typeof e.code === 'number' ? e.code : 1;
        return {
          stdout: e.stdout ?? '',
          stderr: e.stderr ?? (e.killed ? 'TIMEOUT' : e.message ?? 'unknown error'),
          exitCode,
        };
      }
    },
  };
}

// ─── JSON extraction — robust, handles wrangler's mixed stdout ──────────────

/**
 * Extract the first valid JSON object or array from wrangler stdout.
 * Wrangler may prepend progress/spinner lines before the JSON payload.
 */
function extractJson(raw: string): unknown {
  // Try full output first
  const trimmed = raw.trim();
  if (!trimmed) return null;

  // Walk lines from end, accumulate a suffix, attempt parse
  const lines = trimmed.split('\n');
  for (let start = 0; start < lines.length; start++) {
    const candidate = lines.slice(start).join('\n').trim();
    if (!candidate.startsWith('{') && !candidate.startsWith('[')) continue;
    try {
      return JSON.parse(candidate);
    } catch {
      // keep scanning
    }
  }

  // Last-resort: find first { or [ and try from there
  const braceIdx = trimmed.search(/[{[]/);
  if (braceIdx !== -1) {
    try {
      return JSON.parse(trimmed.slice(braceIdx));
    } catch {
      // fall through
    }
  }

  return null;
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

function textHash(text: string): string {
  return createHash('sha256').update(text, 'utf8').digest('hex');
}

function idNamespace(id: string): string {
  const parts = id.split(':');
  return parts.length >= 2 ? `${parts[0]}:${parts[1]}` : parts[0];
}

function isAllowedId(id: string): boolean {
  return id.startsWith(CORPUS_NS_PREFIX);
}

function isGenericPlaceholder(id: string, text: string): boolean {
  if (/Authentic Human Design gate[\s\S]*representing specific life themes and energy patterns/i.test(text)) return true;
  // Observed live across different Gene Keys. These describe the frequency
  // category, not the requested key, and cannot ground a personal reading.
  const genericGeneKeys = [
    'The shadow frequency represents the unconscious pattern that creates limitation and suffering in this area of life.',
    'The gift frequency expresses the balanced state of consciousness that serves the collective good.',
    'The siddhi frequency embodies the highest potential of human consciousness in this archetypal pattern.',
    'Transforming unconscious patterns into conscious service',
  ];
  if (id.startsWith('sw:gk:') && genericGeneKeys.some(value => value.toLowerCase() === text.trim().replace(/\s+/g, ' ').toLowerCase())) return true;
  // Reject very short text that looks like a placeholder stub
  if (text.length < 20 && /^(placeholder|stub|todo|n\/a|tbd)$/i.test(text.trim())) return true;
  return false;
}

function isPrivateSubjectEntry(metadata: Record<string, unknown>): boolean {
  const haystack = JSON.stringify(metadata).toLowerCase();
  const privatePatterns = [
    /\b(19|20)\d{2}[-/]\d{1,2}[-/]\d{1,2}\b/,
    /\b\d{1,2}[-/]\d{1,2}[-/](19|20)\d{2}\b/,
    /\b([01]?[0-9]|2[0-3]):[0-5][0-9]\b/,
    /\b-?\d{1,3}\.\d{3,}/,
    /\b(birthplace|born.on|born.at|birth_date|birth_time)\b/i,
  ];
  return privatePatterns.some((r) => r.test(haystack));
}

function buildPassage(
  vector: WranglerVector | WranglerQueryMatch,
  strategy: 'direct' | 'semantic',
  queryAnchorId?: string,
): CorpusPassage | null {
  const id = vector.id;
  if (!isAllowedId(id)) return null;

  const meta = (vector.metadata ?? {}) as Partial<CorpusPassageMetadata> & Record<string, unknown>;
  if (isPrivateSubjectEntry(meta)) return null;

  const rawText: string = typeof meta.text === 'string' ? meta.text : '';
  if (!rawText.trim()) return null;
  const displayText = rawText;

  if (isGenericPlaceholder(id, rawText)) return null;

  const score = 'score' in vector ? (vector as WranglerQueryMatch).score : undefined;

  return {
    id,
    text: displayText,
    textHash: textHash(displayText),
    idNamespace: idNamespace(id),
    system: typeof meta.system === 'string' ? meta.system : 'unknown',
    category: typeof meta.category === 'string' ? meta.category : 'unknown',
    field: typeof meta.field === 'string' ? meta.field : 'unknown',
    score,
    strategy,
    queryAnchorId: strategy === 'semantic' ? queryAnchorId : undefined,
  };
}

function classifyStderr(stderr: string, exitCode: number): CorpusReceiptState {
  if (exitCode === 0) return 'success';
  const lower = stderr.toLowerCase();
  if (lower.includes('authentication error') || lower.includes('code: 10000') || lower.includes('unauthorized') || lower.includes('401') || lower.includes('forbidden') || lower.includes('403')) {
    return 'auth_error';
  }
  if (lower.includes('timeout') || lower.includes('etimedout') || lower.includes('timed out')) {
    return 'timeout';
  }
  return 'failure';
}

// ─── Adapter class ────────────────────────────────────────────────────────────

export class VectorizeCorpusAdapter {
  private readonly accountId: string;
  private readonly indexName: string;
  private readonly timeoutMs: number;
  private readonly runner: CommandRunner;

  constructor(opts: VectorizeCorpusAdapterOptions) {
    this.accountId = opts.accountId;
    this.indexName = opts.indexName ?? DEFAULT_INDEX;
    this.timeoutMs = opts.timeoutMs ?? DEFAULT_TIMEOUT_MS;
    this.runner = opts.commandRunner ?? createDefaultRunner();
  }

  private async runRead(args: string[]) {
    const commandAttempts: Array<{ exitCode: number; state: CorpusReceiptState }> = [];
    for (let attempt = 0; ; attempt++) {
      const result = await this.runner.run('wrangler', args, { timeoutMs: this.timeoutMs, env: { ...process.env, CLOUDFLARE_ACCOUNT_ID: this.accountId } });
      const state = classifyStderr(result.stderr, result.exitCode);
      commandAttempts.push({ exitCode: result.exitCode, state });
      // One bounded retry covers transient OAuth-refresh and read timeouts. The
      // account and credentials are never changed, and both attempts are recorded.
      if (attempt === 0 && ['auth_error', 'timeout'].includes(state)) continue;
      return { ...result, commandAttempts };
    }
  }

  // ── Direct exact-ID lookup via wrangler vectorize get-vectors ──────────────

  async getVectors(ids: string[]): Promise<CorpusReceipt> {
    const start = Date.now();
    // Vectorize get_by_ids permits at most 20 IDs per request.
    if (ids.length > 20) {
      const passages: CorpusPassage[] = [];
      const commandAttempts: NonNullable<CorpusReceipt['commandAttempts']> = [];
      for (let i = 0; i < ids.length; i += 20) {
        const batch = await this.getVectors(ids.slice(i, i + 20));
        commandAttempts.push(...(batch.commandAttempts ?? []));
        if (!['success', 'empty'].includes(batch.state)) {
          return { ...batch, commandAttempts, requestedIds: ids, passages: [], durationMs: Date.now() - start };
        }
        passages.push(...batch.passages);
      }
      return { state: passages.length ? 'success' : 'empty', requestedIds: ids,
        commandAttempts,
        passages: [...new Map(passages.map(p => [p.id, p])).values()], durationMs: Date.now() - start };
    }

    // Validate all IDs before touching the CLI
    const validIds = ids.filter(isAllowedId);
    if (validIds.length === 0) {
      return {
        state: 'empty',
        passages: [],
        requestedIds: ids,
        durationMs: Date.now() - start,
        reason: 'no valid corpus IDs after namespace filter',
      };
    }

    const args = [
      'vectorize',
      'get-vectors',
      this.indexName,
      '--ids',
      ...validIds,
    ];

    const result = await this.runRead(args);

    if (result.exitCode !== 0) {
      const state = classifyStderr(result.stderr, result.exitCode);
      return {
        state,
        commandAttempts: result.commandAttempts,
        passages: [],
        requestedIds: ids,
        durationMs: Date.now() - start,
        reason: result.stderr.trim().slice(0, 300),
        exitCode: result.exitCode,
      };
    }

    const parsed = extractJson(result.stdout);
    if (parsed === null) {
      return {
        state: 'malformed',
        passages: [],
        requestedIds: ids,
        durationMs: Date.now() - start,
        reason: `could not parse JSON from wrangler stdout (${result.stdout.length} chars)`,
      };
    }

    const response = parsed as WranglerGetVectorsResponse;
    const rawVectors: (WranglerVector | undefined)[] =
      response.result?.vectors ??
      response.vectors ??
      (Array.isArray(parsed) ? (parsed as WranglerVector[]) : undefined as any);
    if (!Array.isArray(rawVectors)) return { state: 'malformed', passages: [], requestedIds: ids, durationMs: Date.now() - start, reason: 'Missing vector array in Wrangler response' };

    const passages: CorpusPassage[] = [];
    for (const v of rawVectors) {
      if (!v || typeof v.id !== 'string' || !validIds.includes(v.id)) continue;
      const p = buildPassage(v, 'direct');
      if (p) passages.push(p);
    }

    return {
      state: passages.length > 0 ? 'success' : 'empty',
      commandAttempts: result.commandAttempts,
      passages,
      requestedIds: ids,
      durationMs: Date.now() - start,
    };
  }

  // ── Semantic neighbors via wrangler vectorize query --vector-id ────────────
  // Uses a stored vector's own embedding as query — no new embedding call.

  async queryByVectorId(
    anchorId: string,
    topK = 5,
  ): Promise<CorpusReceipt> {
    const start = Date.now();

    if (!isAllowedId(anchorId)) {
      return {
        state: 'empty',
        passages: [],
        requestedIds: [anchorId],
        durationMs: Date.now() - start,
        reason: `anchor ID not in corpus namespace: ${anchorId}`,
      };
    }

    const args = [
      'vectorize',
      'query',
      this.indexName,
      '--vector-id',
      anchorId,
      '--top-k',
      String(topK),
      '--return-metadata',
      'all',
    ];

    const result = await this.runRead(args);

    if (result.exitCode !== 0) {
      const state = classifyStderr(result.stderr, result.exitCode);
      return {
        state,
        commandAttempts: result.commandAttempts,
        passages: [],
        requestedIds: [anchorId],
        durationMs: Date.now() - start,
        reason: result.stderr.trim().slice(0, 300),
        exitCode: result.exitCode,
      };
    }

    const parsed = extractJson(result.stdout);
    if (parsed === null) {
      return {
        state: 'malformed',
        passages: [],
        requestedIds: [anchorId],
        durationMs: Date.now() - start,
        reason: `could not parse JSON from wrangler stdout`,
      };
    }

    const response = parsed as WranglerQueryResponse;
    const rawMatches =
      response.result?.matches ??
      response.matches ??
      (Array.isArray(parsed) ? (parsed as WranglerQueryMatch[]) : undefined);
    if (!Array.isArray(rawMatches)) return { state: 'malformed', passages: [], requestedIds: [anchorId], durationMs: Date.now() - start, reason: 'Missing matches array in Wrangler response' };

    const passages: CorpusPassage[] = [];
    for (const m of rawMatches) {
      if (!m || typeof m.id !== 'string') continue;
      const p = buildPassage(m, 'semantic', anchorId);
      if (p) passages.push(p);
    }

    return {
      state: passages.length > 0 ? 'success' : 'empty',
      commandAttempts: result.commandAttempts,
      passages,
      requestedIds: [anchorId],
      durationMs: Date.now() - start,
    };
  }
}

// ─── Factory ─────────────────────────────────────────────────────────────────

export function createVectorizeCorpusAdapter(
  opts: VectorizeCorpusAdapterOptions,
): VectorizeCorpusAdapter {
  return new VectorizeCorpusAdapter(opts);
}
